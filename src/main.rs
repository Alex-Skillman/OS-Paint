mod canvas;
mod input;
mod menu;
mod network;
mod render;
mod stroke;
mod ui;

use crate::input::{change_tool_size, handle_pan, handle_tool, over_ui};
use crate::menu::{update_menu, MenuAction, MenuState};
use crate::network::{
    handle_incoming, peer_color, peer_label, peer_state, send_canvas_snapshot, send_cursor, PeerCursor,
};
use crate::render::{draw_canvas_border, render_peer_cursors, render_stroke};
use crate::ui::{color_wheel_panel_rect, draw_color_wheel, draw_size_slider, draw_toolbar, DEFAULT_PALETTE};
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
    // The local player's own id, once the signaling server assigns one; used
    // to include "you" in the lobby's participant list.
    local_peer_id: Option<PeerId>,
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
        local_peer_id: None,
    }
}

// Caps how many colors picked from the wheel accumulate in the palette,
// beyond the permanent defaults.
const MAX_CUSTOM_PALETTE_COLORS: usize = 3;

// Adds a freshly picked color to the palette (skipping it if already present),
// dropping the oldest non-default entry once the custom colors overflow.
fn remember_color(palette: &mut Vec<Color>, color: Color) {
    if palette.contains(&color) {
        return;
    }
    palette.push(color);
    if palette.len() > DEFAULT_PALETTE.len() + MAX_CUSTOM_PALETTE_COLORS {
        palette.remove(DEFAULT_PALETTE.len());
    }
}

// How many undo steps are kept around.
const MAX_UNDO_STEPS: usize = 50;

// Records the canvas state just before a new discrete drawing action (a whole
// pen stroke, or an erase/stroke-erase drag) starts, so Undo can restore it.
// Starting a new action clears the redo stack, matching standard undo/redo
// semantics (you can't redo past a fresh edit).
fn push_undo_snapshot(undo_stack: &mut Vec<Vec<Stroke>>, redo_stack: &mut Vec<Vec<Stroke>>, strokes: &[Stroke]) {
    undo_stack.push(strokes.to_vec());
    if undo_stack.len() > MAX_UNDO_STEPS {
        undo_stack.remove(0);
    }
    redo_stack.clear();
}

// Restores the previous canvas snapshot (if any) and, when connected to a
// lobby, immediately broadcasts the result so peers converge to it instead of
// waiting for the periodic snapshot.
fn apply_undo(
    undo_stack: &mut Vec<Vec<Stroke>>,
    redo_stack: &mut Vec<Vec<Stroke>>,
    strokes: &mut Vec<Stroke>,
    canvas_revision: &mut u64,
    network: &mut Option<NetworkSession>,
) {
    if let Some(previous) = undo_stack.pop() {
        redo_stack.push(std::mem::replace(strokes, previous));
        *canvas_revision += 1;
        if let Some(net) = network.as_mut() {
            send_canvas_snapshot(&mut net.socket, strokes, *canvas_revision);
        }
    }
}

// The reverse of `apply_undo`: reapplies a snapshot that was undone.
fn apply_redo(
    undo_stack: &mut Vec<Vec<Stroke>>,
    redo_stack: &mut Vec<Vec<Stroke>>,
    strokes: &mut Vec<Stroke>,
    canvas_revision: &mut u64,
    network: &mut Option<NetworkSession>,
) {
    if let Some(next) = redo_stack.pop() {
        undo_stack.push(std::mem::replace(strokes, next));
        *canvas_revision += 1;
        if let Some(net) = network.as_mut() {
            send_canvas_snapshot(&mut net.socket, strokes, *canvas_revision);
        }
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

    // Shown to others next to our cursor and in the lobby list; editable from
    // the pause menu.
    let mut player_name: String = format!("Player{:04}", macroquad::rand::gen_range(0u32, 10_000u32));

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

    // Pen color, changed via the color wheel dropped down from the toolbar swatch,
    // or picked directly from the palette (defaults + previously used colors).
    let mut current_color: Color = WHITE;
    let mut color_wheel_open = false;
    let mut color_wheel_picking = false;
    let mut palette: Vec<Color> = DEFAULT_PALETTE.to_vec();

    // Undo/redo history, as full canvas snapshots taken before each discrete
    // drawing action (see `push_undo_snapshot`).
    let mut undo_stack: Vec<Vec<Stroke>> = Vec::new();
    let mut redo_stack: Vec<Vec<Stroke>> = Vec::new();

    // Find the last tool used
    let mut peer_current_stroke: HashMap<PeerId, usize> = HashMap::new();

    // Last-known canvas-space cursor position for each connected peer, so
    // everyone's pointer can be drawn on the shared canvas.
    let mut peer_cursors: HashMap<PeerId, PeerCursor> = HashMap::new();

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
            net.local_peer_id = net.socket.id();

            // Prints if a peer connects or disonnects
            peer_state(&mut net.socket, &strokes, canvas_revision, &mut peer_cursors);

            // Read incoming messages from peers
            handle_incoming(
                &mut net.socket,
                &mut strokes,
                &mut peer_current_stroke,
                &mut canvas_revision,
                &mut local_stroke_idx,
                &mut peer_cursors,
            );

            if last_snapshot_sent.elapsed() >= snapshot_interval {
                send_canvas_snapshot(&mut net.socket, &strokes, canvas_revision);
                last_snapshot_sent = Instant::now();
            }
        }

        // If the color wheel is open and the player clicks to start drawing
        // (anywhere that isn't the wheel itself or other UI), dismiss the
        // wheel right away instead of silently eating the click, so drawing
        // isn't blocked by having picked a color a moment ago.
        if color_wheel_open && is_mouse_button_pressed(MouseButton::Left) {
            let (click_x, click_y) = mouse_position();
            let over_wheel = color_wheel_panel_rect().contains(Vec2::new(click_x, click_y));
            if !over_wheel && !over_ui(click_x, click_y) {
                color_wheel_open = false;
            }
        }

        // Whether the menu/color wheel were open at the start of this frame
        // (after the dismissal check above). Used instead of
        // `menu_open`/`color_wheel_open` directly below so that pressing
        // Escape to open the menu doesn't also feed that same key-press into
        // a check further down and immediately undo itself in the same frame.
        let menu_was_open = menu_open;
        let wheel_was_open = color_wheel_open;

        if !menu_was_open && !wheel_was_open {
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

            let ctrl_held = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
            if ctrl_held && is_key_pressed(KeyCode::Z) {
                apply_undo(&mut undo_stack, &mut redo_stack, &mut strokes, &mut canvas_revision, &mut network);
                local_stroke_idx = None;
                peer_current_stroke.clear();
            }
            if ctrl_held && is_key_pressed(KeyCode::Y) {
                apply_redo(&mut undo_stack, &mut redo_stack, &mut strokes, &mut canvas_revision, &mut network);
                local_stroke_idx = None;
                peer_current_stroke.clear();
            }

            // Pans the view via right-click drag, and zooms via the mouse wheel
            handle_pan(&mut pan_x, &mut pan_y, &mut zoom, &mut pan_drag_origin);

            // A left click on the canvas starts a new discrete drawing action
            // (a whole pen stroke, or an erase/stroke-erase drag) — snapshot
            // the canvas now so Undo can restore exactly this point.
            let (raw_mouse_x, raw_mouse_y) = mouse_position();
            if is_mouse_button_pressed(MouseButton::Left) && !over_ui(raw_mouse_x, raw_mouse_y) {
                push_undo_snapshot(&mut undo_stack, &mut redo_stack, &strokes);
            }

            // Broadcast our own cursor position to the lobby so peers can draw
            // it on their canvas, skipping while we're over the toolbar/slider.
            if let Some(net) = network.as_mut() {
                if !over_ui(raw_mouse_x, raw_mouse_y) {
                    let (canvas_x, canvas_y) =
                        canvas::screen_to_canvas(raw_mouse_x, raw_mouse_y, pan_x, pan_y, zoom);
                    send_cursor(&mut net.socket, (canvas_x as u16, canvas_y as u16), current_color, &player_name);
                }
            }

            // Gets the user input to draw
            if handle_tool(
                &mut strokes,
                network.as_mut().map(|net| &mut net.socket),
                radius,
                eraser_size,
                current_tool,
                current_color,
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

        // Draws each connected peer's cursor on top of the canvas
        render_peer_cursors(&peer_cursors, pan_x, pan_y, zoom);

        // Draws the tool selection bar on top of the canvas
        let toolbar_click = draw_toolbar(
            &mut current_tool,
            &mut current_color,
            &palette,
            !undo_stack.is_empty(),
            !redo_stack.is_empty(),
        );
        if toolbar_click.menu && !menu_was_open && !wheel_was_open {
            menu_open = true;
            menu_state = MenuState::new();
        }
        if toolbar_click.color_swatch {
            color_wheel_open = !color_wheel_open;
        }
        if toolbar_click.undo {
            apply_undo(&mut undo_stack, &mut redo_stack, &mut strokes, &mut canvas_revision, &mut network);
            local_stroke_idx = None;
            peer_current_stroke.clear();
        }
        if toolbar_click.redo {
            apply_redo(&mut undo_stack, &mut redo_stack, &mut strokes, &mut canvas_revision, &mut network);
            local_stroke_idx = None;
            peer_current_stroke.clear();
        }

        // Draws the brush/eraser size slider on the right edge of the screen
        draw_size_slider(current_tool, &mut radius, &mut eraser_size, &mut slider_dragging);

        if wheel_was_open {
            if draw_color_wheel(&mut current_color, &mut color_wheel_open, &mut color_wheel_picking) {
                remember_color(&mut palette, current_color);
            }
        }

        if menu_was_open {
            let room_code = network.as_ref().map(|net| net.room_code.as_str());
            // Each connected peer's live name/color (from their last cursor
            // update, falling back to a placeholder name and their stable hash
            // color until they move their mouse onto the canvas at least
            // once), plus our own name and current color.
            let lobby_members: Vec<(String, Color)> = match network.as_ref() {
                Some(net) => {
                    let mut members: Vec<(String, Color)> = Vec::new();
                    if net.local_peer_id.is_some() {
                        members.push((player_name.clone(), current_color));
                    }
                    members.extend(net.socket.connected_peers().map(|id| match peer_cursors.get(&id) {
                        Some(cursor) => (cursor.name.clone(), cursor.color),
                        None => (peer_label(id), peer_color(id)),
                    }));
                    members
                }
                None => Vec::new(),
            };
            match update_menu(&mut menu_state, network.is_some(), room_code, &player_name, &lobby_members) {
                MenuAction::None => {}
                MenuAction::Close => menu_open = false,
                MenuAction::Host => {
                    let code = generate_room_code();
                    network = Some(connect_to_room(&signaling_server, &code));
                    last_snapshot_sent = Instant::now();
                    undo_stack.clear();
                    redo_stack.clear();
                    menu_state = MenuState::new();
                }
                MenuAction::Join(code) => {
                    // Joining shows the room's own canvas rather than mixing in
                    // whatever was doodled locally before connecting.
                    strokes.clear();
                    canvas_revision = 0;
                    peer_current_stroke.clear();
                    local_stroke_idx = None;
                    peer_cursors.clear();
                    network = Some(connect_to_room(&signaling_server, &code));
                    last_snapshot_sent = Instant::now();
                    undo_stack.clear();
                    redo_stack.clear();
                    menu_open = false;
                }
                MenuAction::Leave => {
                    if let Some(net) = network.take() {
                        net.task.abort();
                    }
                    peer_current_stroke.clear();
                    local_stroke_idx = None;
                    peer_cursors.clear();
                    undo_stack.clear();
                    redo_stack.clear();
                    menu_state = MenuState::new();
                }
                MenuAction::SetName(name) => {
                    player_name = name;
                    menu_state = MenuState::new();
                }
            }
        }

        next_frame().await;
        std::thread::sleep(Duration::from_millis(16));
    }
}
