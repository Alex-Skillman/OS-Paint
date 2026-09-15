use macroquad::input::MouseButton;
use macroquad::prelude::*;
use macroquad::rand::*;
use macroquad::color::Color;
use macroquad::input::KeyCode::C;
use macroquad::input::MouseButton::Left;
use macroquad::input::MouseButton::Right;

// add in matchbox_signaling
// Used for p2p between clients

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
    size: u16,
    x: u16,
    y: u16,
    color: Color,
    layer: i8, //Unused for now but may add later
}

#[macroquad::main("OS-Paint")]
async fn main() {
    // This initalizes the vector of balls drawn
    let mut figures: Vec<figure> = Vec::new();

    // Hardcoded start radius for balls
    let radius: u16 = 12;

    loop {
        // Get the mouse position each frame
        let (mouse_x, mouse_y) = mouse_position();

        // Draw the current fps on the screen
        draw_fps();

        if is_mouse_button_down(MouseButton::Left) {
            figures.push(figure{
                size: radius,
                x: mouse_x as u16,
                y: mouse_y as u16,
                color: YELLOW,
                layer: 1,
        })
        }
        
        // This loop goes through all figures in figure and draws them to the frame buffer
        for figure in figures.iter() {
            draw_circle(figure.x as f32, figure.y as f32, figure.size as f32, YELLOW)
        }

        println!("{}", figures.len());


        next_frame().await; 
    }
}