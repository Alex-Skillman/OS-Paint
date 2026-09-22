use macroquad::prelude::*;
use crate::canvas::{self, CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::stroke::Stroke;

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

// Draws the fixed canvas's edges (in screen space, given the current pan/zoom)
// so panning or zooming to the edge of the drawing surface is visually obvious.
pub fn draw_canvas_border(pan_x: f32, pan_y: f32, zoom: f32) {
    let (x, y) = canvas::canvas_to_screen(0.0, 0.0, pan_x, pan_y, zoom);
    draw_rectangle_lines(x, y, CANVAS_WIDTH * zoom, CANVAS_HEIGHT * zoom, 2.0, Color::new(1.0, 1.0, 1.0, 0.25));
}
