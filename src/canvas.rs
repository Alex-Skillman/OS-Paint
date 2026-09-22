use macroquad::window::{screen_height, screen_width};

// The canvas is a fixed size regardless of window size, so every peer shares
// the exact same drawing surface no matter how their window is sized. The
// window acts as a scrollable, zoomable viewport onto it (see `pan_x`/`pan_y`/
// `zoom` in main.rs).
pub const CANVAS_WIDTH: f32 = 3200.0;
pub const CANVAS_HEIGHT: f32 = 2400.0;

pub const MIN_ZOOM: f32 = 0.2;
pub const MAX_ZOOM: f32 = 4.0;

// Keeps the zoom level within sane bounds.
pub fn clamp_zoom(zoom: f32) -> f32 {
    zoom.clamp(MIN_ZOOM, MAX_ZOOM)
}

// Keeps the viewport's pan offset from scrolling past the canvas edges. Called
// every frame (including after a window resize or zoom change) so the view
// snaps back in bounds instead of showing empty space beyond the canvas. If
// the viewport is zoomed out far enough to show the whole canvas on an axis,
// that axis is centered instead of pinned to the top/left edge.
pub fn clamp_pan(pan_x: f32, pan_y: f32, zoom: f32) -> (f32, f32) {
    let visible_w = screen_width() / zoom;
    let visible_h = screen_height() / zoom;

    let x = if visible_w >= CANVAS_WIDTH {
        (CANVAS_WIDTH - visible_w) / 2.0
    } else {
        pan_x.clamp(0.0, CANVAS_WIDTH - visible_w)
    };
    let y = if visible_h >= CANVAS_HEIGHT {
        (CANVAS_HEIGHT - visible_h) / 2.0
    } else {
        pan_y.clamp(0.0, CANVAS_HEIGHT - visible_h)
    };
    (x, y)
}

// Converts a screen-space point (e.g. from `mouse_position()`) into
// canvas-space, clamped to the canvas bounds so nothing can be drawn "off
// canvas" even if the window is larger than the canvas itself.
pub fn screen_to_canvas(x: f32, y: f32, pan_x: f32, pan_y: f32, zoom: f32) -> (f32, f32) {
    ((x / zoom + pan_x).clamp(0.0, CANVAS_WIDTH), (y / zoom + pan_y).clamp(0.0, CANVAS_HEIGHT))
}

// Converts a canvas-space point into screen-space for drawing.
pub fn canvas_to_screen(x: f32, y: f32, pan_x: f32, pan_y: f32, zoom: f32) -> (f32, f32) {
    ((x - pan_x) * zoom, (y - pan_y) * zoom)
}
