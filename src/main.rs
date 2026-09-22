mod canvas;
mod input;
mod menu;
mod network;
mod render;
mod stroke;
mod ui;

use crate::input::{change_tool_size, handle_pan, handle_tool};
use crate::menu::{update_menu, MenuAction, MenuState};
use crate::network::{handle_incoming, peer_state, send_canvas_snapshot};
use crate::render::{draw_canvas_border, render_stroke};
use crate::ui::{draw_size_slider, draw_toolbar};
use macroquad::prelude::*;
use matchbox_socket::PeerId;
use matchbox_socket::WebRtcSocket;
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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

// An active lobby connection: the socket, its background message-loop task,
// and the room code it was opened with (so the menu can display it again).
struct NetworkSession {
    socket: WebRtcSocket,
    task: tokio::task::JoinHandle<Result<(), matchbox_socket::Error>>,
    room_code: String,
}

// A random 6-digit code, used as the room id appended to the signaling
// server's URL so a "lobby" is just everyone who connected with the same code.
fn generate_room_code() -> String {
    format!("{:06}", macroquad::rand::gen_range(0u32, 1_000_000u32))
}

fn connect_to_room(signaling_base: &str, code: &str) -> NetworkSession {
    let room_url = format!("{}/{}", signaling_base.trim_end_matches('/'), code);
    println!("Connecting to signaling server: {room_url}");
    let (socket, loop_fut) = WebRtcSocket::new_reliable(room_url);
    let task = tokio::spawn(loop_fut);
    NetworkSession {
        socket,
        task,
        room_code: code.to_string(),
    }
}

#[macroquad::main("OS-Paint")]
async fn main() {
    // Create a tokio runtime for the WebRTC signaling/message-loop background task
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    // quad_rand starts from a fixed seed every run, so it must be seeded from
    // real time or every host would generate the same "random" room codes.
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    macroquad::rand::srand(seed);

    let signaling_server =
        env::var("OS_PAINT_SIGNALING_SERVER").unwrap_or("wss://matchbox-8uwy.onrender.com".to_string());

    // No lobby is joined at startup: the app boots straight into single-person
    // drawing, with networking only started once the player hosts or joins.
    let mut network: Option<NetworkSession> = None;

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

    // View offset/zoom into the fixed-size canvas; the window is a scrollable,
    // zoomable viewport onto it. Starts centered over the canvas at 100%.
    let mut zoom: f32 = 1.0;
    let mut pan_x: f32 = ((canvas::CANVAS_WIDTH - screen_width()) / 2.0).max(0.0);
    let mut pan_y: f32 = ((canvas::CANVAS_HEIGHT - screen_height()) / 2.0).max(0.0);
    let mut pan_drag_origin: Option<(f32, f32)> = None;

    // Escape opens this pause menu; it offers Host/Join while in single-person
    // mode, or Leave Lobby while connected.
    let mut menu_open = false;
    let mut menu_state = MenuState::new();

    loop {
        if let Some(net) = network.as_mut() {
            // Prints if a peer connects or disonnects
            peer_state(&mut net.socket, &strokes, canvas_revision);

            // Read incoming messages from peers
            handle_incoming(
                &mut net.socket,
                &mut strokes,
                &mut peer_current_stroke,
                &mut canvas_revision,
                &mut local_stroke_idx,
            );

            if last_snapshot_sent.elapsed() >= snapshot_interval {
                send_canvas_snapshot(&mut net.socket, &strokes, canvas_revision);
                last_snapshot_sent = Instant::now();
            }
        }

        // Whether the menu was open at the start of this frame. Used instead of
        // `menu_open` directly below so that pressing Escape to open the menu
        // doesn't also feed that same key-press into the menu's own Escape
        // check further down and immediately close it again in the same frame.
        let menu_was_open = menu_open;

        if !menu_was_open {
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

            if is_key_pressed(KeyCode::Escape) {
                menu_open = true;
                menu_state = MenuState::new();
            }

            // Pans the view via right-click drag, and zooms via the mouse wheel
            handle_pan(&mut pan_x, &mut pan_y, &mut zoom, &mut pan_drag_origin);

            // Gets the user input to draw
            if handle_tool(
                &mut strokes,
                network.as_mut().map(|net| &mut net.socket),
                radius,
                eraser_size,
                current_tool,
                &mut local_stroke_idx,
                &mut peer_current_stroke,
                pan_x,
                pan_y,
                zoom,
            ) {
                canvas_revision += 1;
            }

            // Gets user tool size change
            change_tool_size(current_tool, &mut radius, &mut eraser_size);
        }

        // Draws the canvas edges so panning/zooming to the boundary is visible
        draw_canvas_border(pan_x, pan_y, zoom);

        // Draws the strokes onto the frame
        render_stroke(&mut strokes, pan_x, pan_y, zoom);

        // Draws the tool selection bar on top of the canvas
        let menu_button_clicked = draw_toolbar(&mut current_tool);
        if menu_button_clicked && !menu_was_open {
            menu_open = true;
            menu_state = MenuState::new();
        }

        // Draws the brush/eraser size slider on the right edge of the screen
        draw_size_slider(current_tool, &mut radius, &mut eraser_size, &mut slider_dragging);

        if menu_was_open {
            let room_code = network.as_ref().map(|net| net.room_code.as_str());
            match update_menu(&mut menu_state, network.is_some(), room_code) {
                MenuAction::None => {}
                MenuAction::Close => menu_open = false,
                MenuAction::Host => {
                    let code = generate_room_code();
                    network = Some(connect_to_room(&signaling_server, &code));
                    last_snapshot_sent = Instant::now();
                    menu_state = MenuState::new();
                }
                MenuAction::Join(code) => {
                    // Joining shows the room's own canvas rather than mixing in
                    // whatever was doodled locally before connecting.
                    strokes.clear();
                    canvas_revision = 0;
                    peer_current_stroke.clear();
                    local_stroke_idx = None;
                    network = Some(connect_to_room(&signaling_server, &code));
                    last_snapshot_sent = Instant::now();
                    menu_open = false;
                }
                MenuAction::Leave => {
                    if let Some(net) = network.take() {
                        net.task.abort();
                    }
                    peer_current_stroke.clear();
                    local_stroke_idx = None;
                    menu_state = MenuState::new();
                }
            }
        }

        next_frame().await;
        std::thread::sleep(Duration::from_millis(16));
    }
}
