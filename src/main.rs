mod input;
mod network;
mod render;
mod stroke;

use crate::input::handle_tool;
use crate::network::{handle_incoming, peer_state, send_canvas_snapshot};
use crate::render::render_stroke;
use macroquad::prelude::*;
use matchbox_socket::PeerId;
use matchbox_socket::WebRtcSocket;
use std::collections::HashMap;
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

    // THIS WILL ONLY WORK LOCALLY RIGHT NOW
    let (mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://localhost:3536/my_room");
    // Background task that drives the message loop
    tokio::spawn(loop_fut);

    // This initalizes the vector of balls drawn
    let mut strokes: Vec<Stroke> = Vec::new();

    // Hardcoded start radius for balls
    let radius: u16 = 10;

    // Initalize a variable for the current tool
    let mut current_tool: Tool = Tool::Pen;

    // Find the last tool used
    let mut peer_current_stroke: HashMap<PeerId, usize> = HashMap::new();

    // TEMP: Eraser hardcoded size
    let eraser_size: u16 = 25;

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
                'p' => current_tool = Tool::Pen,
                'e' => current_tool = Tool::Eraser,
                _ => {}
            }
        }

        // Gets the user input to draw
        if handle_tool(&mut strokes, &mut socket, radius, eraser_size, current_tool) {
            canvas_revision += 1;
        }

        // Draws the strokes onto the screen
        render_stroke(&mut strokes);

        next_frame().await;
        std::thread::sleep(Duration::from_millis(16));
    }
}
