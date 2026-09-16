use macroquad::prelude::*;
use crate::stroke::Stroke;

pub fn render_stroke (strokes: &mut Vec<Stroke>) {
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
}