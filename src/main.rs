mod input;
mod network;
mod render;
mod stroke;
mod ui;

use crate::input::{change_tool_size, handle_tool};
use crate::network::{handle_incoming, peer_state, send_canvas_snapshot};
use crate::render::render_stroke;
use crate::ui::{draw_size_slider, draw_toolbar};
use macroquad::prelude::*;
use matchbox_socket::PeerId;
use matchbox_socket::WebRtcSocket;
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use stroke::{Stroke, Tool};

fn window_conf() -> Conf {
    Conf {
        window_title: "OS-Paint".to_owned(),
        window_width: 800,
        // 0 turns off V-Sync, forcing the highest FPS possible
        platform: miniquad::conf::Platform {
            swap_interval: Some(0),
            ..Default::default()
        },
        window_height: 600,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main("OS-Paint")]
async fn main() {
    // Creates a tokio runtime
    // Function only works for local host and does not connect to server

    // Create a tokio runtime inside the loop
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let signaling_server =
        env::var("OS_PAINT_SIGNALING_SERVER").unwrap_or("wss://matchbox-8uwy.onrender.com/".to_string());
    println!("Connecting to signaling server: {signaling_server}");

    let (mut socket, loop_fut) = WebRtcSocket::new_reliable(&signaling_server);
    // Background task that drives the message loop
    tokio::spawn(loop_fut);

    // This initalizes the vector of strokes drawn
    let mut strokes: Vec<Stroke> = Vec::new();

    // Start radius for stroke
    let mut radius: u16 = 5;

    // Initalize a variable for the current tool
    let mut current_tool: Tool = Tool::Pen;

    // Find the last tool used
    let mut peer_current_stroke: HashMap<PeerId, usize> = HashMap::new();

    // Tracks the index of the local player's in-progress stroke in `strokes`,
    // since incoming peer strokes can be appended to the same vector mid-frame.
    let mut local_stroke_idx: Option<usize> = None;

    // Inital eraser size
    let mut eraser_size: u16 = 25;

    // Whether the size slider handle is currently being dragged
    let mut slider_dragging: bool = false;

    let snapshot_interval = Duration::from_secs(30);
    let mut last_snapshot_sent = Instant::now();
    let mut canvas_revision: u64 = 0;

    loop {
        // Prints if a peer connects or disonnects
        peer_state(&mut socket, &strokes, canvas_revision);

        // // Read incoming messages from peers
        handle_incoming(
            &mut socket,
            &mut strokes,
            &mut peer_current_stroke,
            &mut canvas_revision,
            &mut local_stroke_idx,
        );

        if last_snapshot_sent.elapsed() >= snapshot_interval {
            send_canvas_snapshot(&mut socket, &strokes, canvas_revision);
            last_snapshot_sent = Instant::now();
        }

        // Draw the current fps on the screen
        draw_fps();

        // Find the last keypress
        let last_key_press = get_char_pressed();

        // Match last keypress to a tool
        if let Some(key) = last_key_press {
            match key {
                'd' => current_tool = Tool::Pen,
                'e' => current_tool = Tool::Eraser,
                'r' => current_tool = Tool::StrokeEraser,
                _ => {}
            }
        }

        // Gets the user input to draw
        if handle_tool(
            &mut strokes,
            &mut socket,
            radius,
            eraser_size,
            current_tool,
            &mut local_stroke_idx,
            &mut peer_current_stroke,
        ) {
            canvas_revision += 1;
        }

        // Gets user tool size change
        change_tool_size(current_tool, &mut radius, &mut eraser_size);

        // Draws the strokes onto the frame
        render_stroke(&mut strokes);

        // Draws the tool selection bar on top of the canvas
        draw_toolbar(&mut current_tool);

        // Draws the brush/eraser size slider on the right edge of the screen
        draw_size_slider(current_tool, &mut radius, &mut eraser_size, &mut slider_dragging);

        next_frame().await;
        std::thread::sleep(Duration::from_millis(16));
    }
}
