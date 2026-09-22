use crate::stroke::Tool;
use macroquad::prelude::*;

// Height in pixels of the toolbar strip along the top of the window.
pub const TOOLBAR_HEIGHT: f32 = 36.0;

const BAR_RADIUS: f32 = 10.0;
const BUTTON_RADIUS: f32 = 8.0;
const BUTTON_WIDTH: f32 = 34.0;
const BUTTON_GAP: f32 = 6.0;
const SIDE_PADDING: f32 = 8.0;

const BG: Color = Color::new(0.10, 0.10, 0.11, 0.96);
const IDLE_ICON: Color = Color::new(0.62, 0.62, 0.66, 1.0);
const HOVER_BG: Color = Color::new(0.28, 0.28, 0.30, 1.0);
const HOVER_ICON: Color = Color::new(0.88, 0.88, 0.90, 1.0);
const SELECTED_BG: Color = Color::new(0.30, 0.55, 0.98, 1.0);
const SELECTED_ICON: Color = WHITE;
const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.25);

const TOOLS: [Tool; 3] = [Tool::Pen, Tool::Eraser, Tool::StrokeEraser];

// Width in pixels of the size-slider panel docked to the left edge.
pub const SLIDER_PANEL_WIDTH: f32 = 44.0;

const SLIDER_MARGIN: f32 = 18.0;
const SLIDER_LABEL_HEIGHT: f32 = 26.0;
const TRACK_WIDTH: f32 = 4.0;
const HANDLE_RADIUS: f32 = 9.0;
const HANDLE_HIT_RADIUS: f32 = 16.0;
const MIN_SIZE: u16 = 1;
const PEN_MAX_SIZE: u16 = 60;
const ERASER_MAX_SIZE: u16 = 120;
const TRACK_IDLE: Color = Color::new(1.0, 1.0, 1.0, 0.12);

// Draws the toolbar and handles clicks on it, updating `current_tool` when a
// button is pressed. Returns true the frame the menu button (leftmost) is
// clicked, so the caller can open the pause menu without needing Escape.
// Call once per frame.
pub fn draw_toolbar(current_tool: &mut Tool) -> bool {
    let screen_w = screen_width();

    draw_rounded_rect_bottom(0.0, 0.0, screen_w, TOOLBAR_HEIGHT, BAR_RADIUS, BG);
    draw_rounded_rect_bottom(0.0, TOOLBAR_HEIGHT, screen_w, 2.0, 0.0, SHADOW);

    let (mouse_x, mouse_y) = mouse_position();
    let clicked = is_mouse_button_pressed(MouseButton::Left);

    let button_height = TOOLBAR_HEIGHT - 12.0;
    let button_y = (TOOLBAR_HEIGHT - button_height) / 2.0;
    let mut x = SIDE_PADDING;

    let menu_rect = Rect::new(x, button_y, BUTTON_WIDTH, button_height);
    let menu_hovered = menu_rect.contains(Vec2::new(mouse_x, mouse_y));
    if menu_hovered {
        draw_rounded_rect(menu_rect.x, menu_rect.y, menu_rect.w, menu_rect.h, BUTTON_RADIUS, HOVER_BG);
    }
    draw_menu_icon(
        menu_rect.x + menu_rect.w / 2.0,
        menu_rect.y + menu_rect.h / 2.0,
        button_height * 0.62,
        if menu_hovered { HOVER_ICON } else { IDLE_ICON },
    );
    let menu_clicked = menu_hovered && clicked;

    x += BUTTON_WIDTH + BUTTON_GAP * 2.0;

    for &tool in TOOLS.iter() {
        let rect = Rect::new(x, button_y, BUTTON_WIDTH, button_height);

        let is_selected = *current_tool == tool;
        let hovered = rect.contains(Vec2::new(mouse_x, mouse_y));

        let (bg_color, icon_color) = if is_selected {
            (Some(SELECTED_BG), SELECTED_ICON)
        } else if hovered {
            (Some(HOVER_BG), HOVER_ICON)
        } else {
            (None, IDLE_ICON)
        };

        if let Some(bg) = bg_color {
            draw_rounded_rect(rect.x, rect.y, rect.w, rect.h, BUTTON_RADIUS, bg);
        }

        let cx = rect.x + rect.w / 2.0;
        let cy = rect.y + rect.h / 2.0;
        let icon_size = button_height * 0.62;

        match tool {
            Tool::Pen => draw_pen_icon(cx, cy, icon_size, icon_color),
            Tool::Eraser => draw_eraser_icon(cx, cy, icon_size, icon_color),
            Tool::StrokeEraser => draw_stroke_eraser_icon(cx, cy, icon_size, icon_color),
        }

        if hovered && clicked {
            *current_tool = tool;
        }

        x += BUTTON_WIDTH + BUTTON_GAP;
    }

    menu_clicked
}

// Draws the vertical brush/eraser size slider docked to the left edge of
// the screen and handles dragging it. `dragging` persists across frames so
// a drag continues even if the mouse slips past the handle's hit radius.
pub fn draw_size_slider(current_tool: Tool, pen_size: &mut u16, eraser_size: &mut u16, dragging: &mut bool) {
    let screen_h = screen_height();

    let max_size = if current_tool == Tool::Pen {
        PEN_MAX_SIZE
    } else {
        ERASER_MAX_SIZE
    };
    let size_ref = if current_tool == Tool::Pen {
        pen_size
    } else {
        eraser_size
    };

    let panel_h: f32 = 260.0;
    let preview_area = SLIDER_LABEL_HEIGHT;
    let label_area = SLIDER_LABEL_HEIGHT;

    let panel_x = 0.0;
    let panel_y = ((screen_h - panel_h) / 2.0).max(TOOLBAR_HEIGHT + SLIDER_MARGIN);
    let track_top = panel_y + preview_area + SLIDER_MARGIN;
    let track_bottom = panel_y + panel_h - label_area - SLIDER_MARGIN;
    let track_x = panel_x + SLIDER_PANEL_WIDTH / 2.0;

    // Live preview of the current size, scaled to fit the panel.
    let t = ((*size_ref as f32 - MIN_SIZE as f32) / (max_size - MIN_SIZE) as f32).clamp(0.0, 1.0);
    let preview_radius = 3.0 + t * (SLIDER_PANEL_WIDTH / 2.0 - 7.0);
    draw_circle(track_x, panel_y + preview_area / 2.0, preview_radius, HOVER_ICON);

    // Idle track and filled portion below the handle.
    let handle_y = track_bottom - t * (track_bottom - track_top);
    draw_line(track_x, track_top, track_x, track_bottom, TRACK_WIDTH, TRACK_IDLE);
    draw_line(track_x, handle_y, track_x, track_bottom, TRACK_WIDTH, SELECTED_BG);

    let (mouse_x, mouse_y) = mouse_position();
    let clicked = is_mouse_button_pressed(MouseButton::Left);
    let held = is_mouse_button_down(MouseButton::Left);

    let within_handle = ((mouse_x - track_x).powi(2) + (mouse_y - handle_y).powi(2)).sqrt() <= HANDLE_HIT_RADIUS;
    let within_track = mouse_x >= panel_x
        && mouse_x <= panel_x + SLIDER_PANEL_WIDTH
        && mouse_y >= track_top
        && mouse_y <= track_bottom;

    if clicked && (within_handle || within_track) {
        *dragging = true;
    }
    if !held {
        *dragging = false;
    }

    if *dragging {
        let clamped_y = mouse_y.clamp(track_top, track_bottom);
        let new_t = (track_bottom - clamped_y) / (track_bottom - track_top);
        let new_value = (MIN_SIZE as f32 + new_t * (max_size - MIN_SIZE) as f32).round() as u16;
        *size_ref = new_value.clamp(MIN_SIZE, max_size);
    }

    let handle_color = if *dragging || within_handle { SELECTED_ICON } else { HOVER_ICON };
    draw_circle(track_x, handle_y, HANDLE_RADIUS, handle_color);

    let label = format!("{}", *size_ref);
    let dims = measure_text(&label, None, 14, 1.0);
    draw_text(
        &label,
        track_x - dims.width / 2.0,
        panel_y + panel_h - label_area / 2.0 + dims.height / 2.0 - 2.0,
        14.0,
        IDLE_ICON,
    );
}

// A hamburger icon (three stacked bars) for the menu button.
fn draw_menu_icon(cx: f32, cy: f32, size: f32, color: Color) {
    let half = size / 2.0;
    let thickness = size * 0.16;

    for i in -1..=1 {
        let y = cy + i as f32 * half * 0.7;
        draw_line(cx - half, y, cx + half, y, thickness, color);
    }
}

// A pen mid-stroke: two dots joined by a line, matching how the canvas
// itself renders a stroke.
fn draw_pen_icon(cx: f32, cy: f32, size: f32, color: Color) {
    let half = size / 2.0;
    let (x1, y1) = (cx - half * 0.6, cy + half * 0.6);
    let (x2, y2) = (cx + half * 0.6, cy - half * 0.6);

    draw_line(x1, y1, x2, y2, size * 0.16, color);
    draw_circle(x1, y1, size * 0.11, color);
    draw_circle(x2, y2, size * 0.14, color);
}

// A solid eraser block.
fn draw_eraser_icon(cx: f32, cy: f32, size: f32, color: Color) {
    let w = size * 0.82;
    let h = size * 0.56;
    draw_rounded_rect(cx - w / 2.0, cy - h / 2.0, w, h, size * 0.14, color);
}

// A drawn stroke with a line struck through it, signaling "delete the
// whole stroke" rather than just a point.
fn draw_stroke_eraser_icon(cx: f32, cy: f32, size: f32, color: Color) {
    let half = size / 2.0;
    let thickness = size * 0.15;

    let p1 = (cx - half * 0.7, cy + half * 0.35);
    let p2 = (cx - half * 0.05, cy - half * 0.45);
    let p3 = (cx + half * 0.7, cy + half * 0.15);

    draw_line(p1.0, p1.1, p2.0, p2.1, thickness, color);
    draw_line(p2.0, p2.1, p3.0, p3.1, thickness, color);
    draw_line(cx - half * 0.8, cy - half * 0.65, cx + half * 0.8, cy + half * 0.65, thickness, color);
}

// Fills a rectangle with all four corners rounded.
fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);

    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);

    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

// Fills a rectangle whose top edge stays flush (for a bar sitting on the
// screen edge) but whose bottom corners are rounded.
fn draw_rounded_rect_bottom(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);

    if r <= 0.0 {
        draw_rectangle(x, y, w, h, color);
        return;
    }

    draw_rectangle(x, y, w, h - r, color);
    draw_rectangle(x + r, y + h - r, w - 2.0 * r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}
