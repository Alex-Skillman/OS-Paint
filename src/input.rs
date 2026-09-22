use crate::network::DrawPacket;
use crate::network::{ErasePacket, NetworkPacket, StrokeErasePacket, send_packet};
use crate::stroke::{self, Stroke, Tool};
use macroquad::color::WHITE;
use macroquad::input::MouseButton;
use macroquad::input::{is_mouse_button_down, is_mouse_button_pressed, mouse_position, mouse_wheel, is_key_down};
use macroquad::input::{KeyCode::Up, KeyCode::Down};
use matchbox_socket::WebRtcSocket;

pub fn handle_tool(
    strokes: &mut Vec<Stroke>,
    socket: &mut WebRtcSocket,
    radius: u16,
    eraser_size: u16,
    current_tool: Tool,
) -> bool {
    match current_tool {
        Tool::Pen => pen_drawing(strokes, socket, radius),
        Tool::Eraser => erasing(strokes, socket, eraser_size),
        Tool::StrokeEraser => stroke_erase(strokes, socket, eraser_size),
    }
}

fn pen_drawing(strokes: &mut Vec<Stroke>, socket: &mut WebRtcSocket, radius: u16) -> bool {
    let (mouse_x, mouse_y) = mouse_position();

    if is_mouse_button_pressed(MouseButton::Left) {
        // If the button is pressed then push a new Stroke to the vector, string
        strokes.push(Stroke {
            size: radius,
            color: WHITE,
            layer: 1,
            coordinates: vec![(mouse_x as u16, mouse_y as u16)],
        });
        let packet = DrawPacket {
            point: (mouse_x as u16, mouse_y as u16),
            is_new_stroke: true,
            size: Some(radius),
            color: Some(WHITE.into()),
            layer: Some(1),
        };
        send_packet(socket, &NetworkPacket::Draw(packet));
        return true;
    } else if is_mouse_button_down(MouseButton::Left) {
        if let Some(current_stroke) = strokes.last_mut() {
            // Push new coordinates when the mouse buttne is held down
            current_stroke
                .coordinates
                .push((mouse_x as u16, mouse_y as u16))
        }

        let packet = DrawPacket {
            point: (mouse_x as u16, mouse_y as u16),
            is_new_stroke: false,
            size: None,
            color: None,
            layer: None,
        };
        send_packet(socket, &NetworkPacket::Draw(packet));
        return true;
    }

    false
}

fn erasing(strokes: &mut Vec<Stroke>, socket: &mut WebRtcSocket, eraser_size: u16) -> bool {
    if !is_mouse_button_down(MouseButton::Left) {
        return false;
    }

    let (eraser_x, eraser_y) = mouse_position();
    let point = (eraser_x as u16, eraser_y as u16);

    if erase_at(strokes, point, eraser_size) {
        let packet = ErasePacket {
            point,
            size: eraser_size,
        };
        send_packet(socket, &NetworkPacket::Erase(packet));
        return true;
    }

    false
}

pub fn erase_at(strokes: &mut Vec<Stroke>, point: (u16, u16), eraser_size: u16) -> bool {
    let (eraser_x, eraser_y) = (point.0 as f32, point.1 as f32);
    let mut new_strokes: Vec<Stroke> = Vec::new();
    let mut erased = false;

    for stroke in strokes.iter_mut() {
        let mut loop_accum: usize = 0;
        while loop_accum < stroke.coordinates.len() {
            let (x, y): (u16, u16) = stroke.coordinates[loop_accum];

            let dx = eraser_x - x as f32;
            let dy = eraser_y - y as f32;
            let dist = ((dx * dx) + (dy * dy)).sqrt();

            let threshold = (stroke.size + eraser_size) as f32;

            if dist < threshold {
                erased = true;
                let remaining = stroke.coordinates.split_off(loop_accum);
                if remaining.len() > 1 {
                    new_strokes.push(Stroke {
                        size: stroke.size,
                        color: stroke.color,
                        layer: stroke.layer,
                        coordinates: remaining[1..].to_vec(),
                    });
                }
                break;
            } else {
                loop_accum += 1;
            }
        }
    }

    strokes.append(&mut new_strokes);
    strokes.retain(|s| s.coordinates.len() > 1);
    erased
}


pub fn change_tool_size(tool: Tool, pen_size: &mut u16, eraser_size: &mut u16) {
    if tool == Tool::Pen {
        if is_key_down(Up) {
            *pen_size += 1;
        }
        if is_key_down(Down) {
            if *pen_size > 1 {
            *pen_size -= 1;
            }
        }
    }
    if tool == Tool::Eraser || tool == Tool::StrokeEraser {
        if is_key_down(Up) {
            *eraser_size += 1;
        }
        if is_key_down(Down) {
            if *eraser_size > 1 {
            *eraser_size -= 1;
            }
        }
    }
}

fn stroke_erase(strokes: &mut Vec<Stroke>, socket: &mut WebRtcSocket, eraser_size: u16) -> bool {
    if !is_mouse_button_down(MouseButton::Left) {
        return false;
    }

    let (eraser_x, eraser_y) = mouse_position();
    let point = (eraser_x as u16, eraser_y as u16);

    if stroke_erase_at(strokes, point, eraser_size) {
        let packet = StrokeErasePacket {
            point,
            size: eraser_size,
        };
        send_packet(socket, &NetworkPacket::StrokeErase(packet));
        return true;
    }

    false
}

// Removes entire strokes that pass within `eraser_size` of `point`, rather than
// trimming just the coordinates that are in range like the point eraser does.
pub fn stroke_erase_at(strokes: &mut Vec<Stroke>, point: (u16, u16), eraser_size: u16) -> bool {
    let (eraser_x, eraser_y) = (point.0 as f32, point.1 as f32);
    let before = strokes.len();

    strokes.retain(|stroke| {
        !stroke.coordinates.iter().any(|&(x, y)| {
            let dx = eraser_x - x as f32;
            let dy = eraser_y - y as f32;
            let dist = ((dx * dx) + (dy * dy)).sqrt();
            let threshold = (stroke.size + eraser_size) as f32;
            dist < threshold
        })
    });

    strokes.len() != before
}