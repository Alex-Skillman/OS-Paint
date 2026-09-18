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
