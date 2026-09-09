use macroquad::input::MouseButton;
use macroquad::prelude::*;
use macroquad::rand::*;
use macroquad::color::Color;
use macroquad::input::KeyCode::C;
use macroquad::input::MouseButton::Left;
use macroquad::input::MouseButton::Right;

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

struct figure {
    size: f32,
    x: f32,
    y: f32,
    color: Color,
    layer: i8, //Unused for now but may add later
}

#[macroquad::main("OS-Paint")]
async fn main() {
    // This initalizes the vector of balls drawn
    let mut figures: Vec<figure> = Vec::new();

    // Hardcoded start radius for balls
    let radius: f32 = 12.0;

    loop {
        // Get the mouse position each frame
        let (mouse_x, mouse_y) = mouse_position();

        // Draw the current fps on the screen
        draw_fps();

        if is_mouse_button_down(MouseButton::Left) {
            figures.push(figure{
                size: radius,
                x: mouse_x,
                y: mouse_y,
                color: YELLOW,
                layer: 1,
        })
        }
        
        // This loop goes through all figures in figure and draws them to the frame buffer
        for figure in figures.iter_mut() {
            draw_circle(figure.x, figure.y, figure.size, YELLOW)
        }

        next_frame().await; 
    }
}