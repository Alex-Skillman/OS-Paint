mod stroke;
mod network;
mod input;
mod render;

use macroquad::prelude::*;
use stroke::{Stroke, Tool};
use std::thread::current;
use std::time::Duration;
use crate::{input::stroke_drawing, network::peer_state};
use crate::render::render_stroke;
use crate::network::handle_incoming;
use std::collections::HashMap;
use matchbox_socket::PeerId;
use matchbox_socket::{WebRtcSocket, PeerState};

fn window_conf() -> Conf {
    Conf {
        window_title: "OS-Paint".to_owned(),
        window_width: 800,
        // 0 turns off V-Sync, forcing the highest FPS possible
        platform: miniquad::conf::Platform {swap_interval: Some(0), ..Default::default()},
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
    let(mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://localhost:3536/my_room");
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

    // Find the last key pressed
    let mut last_key_press: char = '\0'; 
    loop {
        // Prints if a peer connects or disonnects
        peer_state(&mut socket);

        // // Read incoming messages from peers
        handle_incoming(&mut socket, &mut strokes, &mut peer_current_stroke);

        // Draw the current fps on the screen
        draw_fps();

        // Find the last keypress
        let last_key_press = get_char_pressed();

        // Match last keypress to a tool
        match last_key_press {
            Some('p') => current_tool = Tool::Pen,
            Some('e') => current_tool = Tool::Eraser,
            _ => current_tool = Tool::Pen,
        }

        // Gets the user input to draw
        stroke_drawing(&mut strokes, &mut socket, radius, current_tool);

        // Draws the strokes onto the screen
        render_stroke(&mut strokes);

        next_frame().await;
        std::thread::sleep(Duration::from_millis(16));
    }
}