mod stroke;
mod network;
mod input;
mod render;

use macroquad::prelude::*;
use stroke::Stroke;
use std::{string, time::Duration};
use crate::input::stroke_drawing;
use crate::network::tokio_runtime;
use crate::render::render_stroke;

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
    tokio_runtime();

    // Background task that drives the message loop
    tokio::spawn(loop_fut);

    // This initalizes the vector of balls drawn
    let mut strokes: Vec<Stroke> = Vec::new();

    // Hardcoded start radius for balls
    let radius: u16 = 10;

    loop {
        // Detect peers joining/leaving
        for (peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => println!("Peer Joined"),
                PeerState::Disconnected => println!("Peer Left"),
            }
        }

        // Read incoming messages from peers
        for (peer, packet) in socket.channel_mut(0).receive() {
            println!("From {peer:?}: {:?}", String::from_utf8_lossy(&packet))
        }

        // Send a message to all peers
        let peers: Vec<_> = socket.connected_peers().collect();
        for peer in peers {
            socket.channel_mut(0).send(b"Hello".to_vec().into_boxed_slice(), peer);
        }

        // Draw the current fps on the screen
        draw_fps();

        // Gets the user input to draw
        stroke_drawing(&mut strokes, radius);

        // Draws the strokes onto the screen
        render_stroke(&mut strokes);

        next_frame().await;
        std::thread::sleep(Duration::from_millis(16));
    }
}