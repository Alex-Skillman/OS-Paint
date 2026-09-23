use macroquad::prelude::*;
use crate::canvas::{self, CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::network::PeerCursor;
use crate::stroke::Stroke;
use matchbox_socket::PeerId;
use std::collections::HashMap;

pub fn render_stroke (strokes: &mut Vec<Stroke>, pan_x: f32, pan_y: f32, zoom: f32) {
    for drawn_stroke in strokes.iter() {

            let coords = &drawn_stroke.coordinates;
            let size = drawn_stroke.size as f32 * zoom;

            for i in 0..coords.len() {
                // Find the x and y of each dot in the stroke for each coordinate
                let(x,y) = coords[i];
                let (sx, sy) = canvas::canvas_to_screen(x as f32, y as f32, pan_x, pan_y, zoom);
                // Draw the dot for each coordinate in the stroke
                draw_circle(sx, sy, size, drawn_stroke.color);

                if i > 0 {
                    let (prev_x, prev_y) = coords[i-1];
                    let (psx, psy) = canvas::canvas_to_screen(prev_x as f32, prev_y as f32, pan_x, pan_y, zoom);
                    // Draw the connecting line
                    draw_line(
                        psx, psy,
                        sx, sy,
                        size * 2.0, // Muliply by to match the radius of the circle
                        drawn_stroke.color,
                    );
                }
            }
        }
}

// Draws a small ring-and-dot cursor for each peer's last-known position, in
// the color they currently have selected, with their name labeled alongside
// it, so everyone can see where the rest of the lobby is pointing, who's
// pointing it, and what they're about to draw with.
pub fn render_peer_cursors(peer_cursors: &HashMap<PeerId, PeerCursor>, pan_x: f32, pan_y: f32, zoom: f32) {
    for cursor in peer_cursors.values() {
        let (x, y) = cursor.point;
        let (sx, sy) = canvas::canvas_to_screen(x as f32, y as f32, pan_x, pan_y, zoom);
        draw_circle_lines(sx, sy, 7.0, 2.0, cursor.color);
        draw_circle(sx, sy, 2.5, cursor.color);

        let label_size = 14.0;
        let dims = measure_text(&cursor.name, None, label_size as u16, 1.0);
        let label_x = sx + 12.0;
        let label_y = sy - 10.0;
        draw_rectangle(
            label_x - 4.0,
            label_y - dims.height,
            dims.width + 8.0,
            dims.height + 6.0,
            Color::new(0.0, 0.0, 0.0, 0.55),
        );
        draw_text(&cursor.name, label_x, label_y, label_size, cursor.color);
    }
}

// Draws the fixed canvas's edges (in screen space, given the current pan/zoom)
// so panning or zooming to the edge of the drawing surface is visually
// obvious. The border is drawn in whichever of black/white contrasts with
// `background`, so it stays visible whichever one is chosen.
pub fn draw_canvas_border(pan_x: f32, pan_y: f32, zoom: f32, background: Color) {
    let (x, y) = canvas::canvas_to_screen(0.0, 0.0, pan_x, pan_y, zoom);
    let luminance = 0.299 * background.r + 0.587 * background.g + 0.114 * background.b;
    let line_color = if luminance > 0.5 {
        Color::new(0.0, 0.0, 0.0, 0.25)
    } else {
        Color::new(1.0, 1.0, 1.0, 0.25)
    };
    draw_rectangle_lines(x, y, CANVAS_WIDTH * zoom, CANVAS_HEIGHT * zoom, 2.0, line_color);
}
