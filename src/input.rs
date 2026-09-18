use macroquad::input::MouseButton;
use macroquad::input::{is_mouse_button_pressed, is_mouse_button_down, mouse_position};
use macroquad::color::YELLOW;
use crate::network::send_packet;
use crate::stroke::{Stroke, Tool};
use crate::network::DrawPacket;
use matchbox_socket::WebRtcSocket;

pub fn handle_tool(strokes: &mut Vec<Stroke>, socket: &mut WebRtcSocket, radius: u16, current_tool: Tool) {
    match current_tool {
        Tool::Pen => pen_drawing(strokes, socket, radius, current_tool),
        Tool::Eraser => erasing(strokes),
        _ => panic!("Invalid tool. How did you manage that???"),
    }
}

fn pen_drawing(strokes: &mut Vec<Stroke>, socket: &mut WebRtcSocket, radius: u16, curent_tool: Tool) {
    let (mouse_x, mouse_y) = mouse_position();
    
    if is_mouse_button_pressed(MouseButton::Left) {
                // If the button is pressed then push a new Stroke to the vector, string
                strokes.push(Stroke{
                    size: radius,
                    color: YELLOW,
                    layer: 1,
                    coordinates: vec![(mouse_x as u16, mouse_y as u16)]
                });
                let packet = DrawPacket {
                    point: (mouse_x as u16, mouse_y as u16),
                    is_new_stroke: true,
                    size: Some(radius),
                    color: Some(YELLOW.into()),
                    layer: Some(1),
                };
                send_packet(socket, &packet);

            } else if is_mouse_button_down(MouseButton::Left) {
                if let Some(current_stroke) = strokes.last_mut() {
                    // Push new coordinates when the mouse buttne is held down
                    current_stroke.coordinates.push((mouse_x as u16, mouse_y as u16))
                }

                let packet = DrawPacket {
                    point: (mouse_x as u16, mouse_y as u16),
                    is_new_stroke: false,
                    size: None,
                    color: None,
                    layer: None,
                };
                send_packet(socket, &packet);
            }
    }

fn erasing (strokes: &mut Vec<Stroke>) {
    let mut new_strokes: Vec<Stroke> = Vec::new();

    for stroke in strokes.iter_mut() {
        let mut loop_accum: usize = 0;
        while loop_accum < stroke.coordinates.len() {
            if loop_accum == 0 {
                loop_accum += 1;
                continue;
            }

            let (last_x, last_y): (u16, u16) = stroke.coordinates[loop_accum - 1];
            let (x, y): (u16, u16) = stroke.coordinates[loop_accum];

            let dx = last_x as f32 - x as f32;
            let dy = last_y as f32 - y as f32;
            let dist = ((dx * dx) + (dy * dy)).sqrt();

            let threshold = (stroke.size + 25) as f32;

            if dist < threshold {
                // Split: everything from loop_accum onward becomes a new stroke
                let remaining = stroke.coordinates.split_off(loop_accum);
                if remaining.len() > 1 {
                    new_strokes.push(Stroke {
                        size: stroke.size,
                        color: stroke.color,
                        layer: stroke.layer,
                        coordinates: remaining[1..].to_vec(), // skip the erased point itself
                    });
                }
                break; // stop processing this stroke, it's been split
            } else {
                loop_accum += 1;
            }
        }
    }

    strokes.append(&mut new_strokes);
}