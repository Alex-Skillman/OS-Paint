use macroquad::input::MouseButton;
use macroquad::prelude::*;
use macroquad::color::Color;
use matchbox_socket::{WebRtcSocket, PeerState};
use std::{string, time::Duration};

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

struct Stroke {
    size: u16,
    color: Color,
    layer: i8, //Unused for now but may add later
    coordinates: Vec<(u16, u16)>,
}

#[macroquad::main("OS-Paint")]
async fn main() {
    //This inits the address to a local signalling server
    // THIS WILL ONLY WORK LOCALLY RIGHT NOW
    let(mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://localhost:3536/my_room");

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


        // Get the mouse position each frame
        let (mouse_x, mouse_y) = mouse_position();

        // Draw the current fps on the screen
        draw_fps();

        if is_mouse_button_pressed(MouseButton::Left) {
            // If the button is pressed then push a new Stroke to the vector, string
            strokes.push(Stroke{
                size: radius,
                color: YELLOW,
                layer: 1,
                coordinates: vec![(mouse_x as u16, mouse_y as u16)]
            });
        } else if is_mouse_button_down(MouseButton::Left) {
            if let Some(current_stroke) = strokes.last_mut() {
                // Push new coordinates when the mouse buttne is held down
                current_stroke.coordinates.push((mouse_x as u16, mouse_y as u16))
            }
        }

        for drawn_stroke in strokes.iter() {

            let coords = &drawn_stroke.coordinates;

            for i in 0..coords.len() {
                // Find the x and y of each dot in the stroke for each coordinate
                let(x,y) = coords[i];
                // Draw the dot for each coordinate in the stroke
                draw_circle(x as f32, y as f32, drawn_stroke.size as f32, drawn_stroke.color);

                if i > 0 {
                    let (prev_x, prev_y) = coords[i-1];
                    // Draw the connecting line
                    draw_line(
                        prev_x as f32, prev_y as f32,
                        x as f32, y as f32,
                        drawn_stroke.size as f32 * 2.0, // Muliply by to match the radius of the circle
                        drawn_stroke.color,
                    );
                }
            }
        }

        println!("{}", strokes.len());


        next_frame().await;
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}