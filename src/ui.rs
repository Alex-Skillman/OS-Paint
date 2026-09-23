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
const DISABLED_ICON: Color = Color::new(0.62, 0.62, 0.66, 0.35);

const TOOLS: [Tool; 3] = [Tool::Pen, Tool::Eraser, Tool::StrokeEraser];

// The palette always starts with these; colors picked from the wheel are
// appended after them (see `remember_color` in main.rs).
pub const DEFAULT_PALETTE: [Color; 4] = [BLACK, WHITE, RED, BLUE];

const PALETTE_SWATCH_RADIUS: f32 = 9.0;
const PALETTE_GAP: f32 = 8.0;
const PALETTE_RING: Color = Color::new(1.0, 1.0, 1.0, 0.8);

const WHEEL_RADIUS: f32 = 70.0;
const WHEEL_SEGMENTS: usize = 48;
const WHEEL_MARGIN: f32 = 26.0;
// Gap between the toolbar's bottom edge and the wheel's panel, so the panel
// (which shares the toolbar's background color) reads as a separate floating
// box instead of fusing seamlessly into the toolbar with no visible seam.
const WHEEL_PANEL_TOP_GAP: f32 = 10.0;

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

// Which toolbar buttons were clicked this frame.
pub struct ToolbarClick {
    pub menu: bool,
    pub color_swatch: bool,
    pub undo: bool,
    pub redo: bool,
}

// Draws the toolbar and handles clicks on it, updating `current_tool` when a
// tool button is pressed and `current_color` when a palette swatch is
// clicked. `can_undo`/`can_redo` dim the undo/redo buttons and suppress their
// clicks when there's nothing to do. Call once per frame.
pub fn draw_toolbar(
    current_tool: &mut Tool,
    current_color: &mut Color,
    palette: &[Color],
    can_undo: bool,
    can_redo: bool,
) -> ToolbarClick {
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

    // Row of quick-pick swatches (defaults + colors picked from the wheel),
    // centered in the middle of the bar.
    let palette_width = palette.len() as f32 * (PALETTE_SWATCH_RADIUS * 2.0)
        + (palette.len().saturating_sub(1)) as f32 * PALETTE_GAP;
    let mut px = screen_w / 2.0 - palette_width / 2.0 + PALETTE_SWATCH_RADIUS;
    let py = TOOLBAR_HEIGHT / 2.0;

    for &swatch_color in palette.iter() {
        let hovered = (Vec2::new(mouse_x, mouse_y) - Vec2::new(px, py)).length() <= PALETTE_SWATCH_RADIUS + 3.0;
        let is_selected = swatch_color == *current_color;

        if is_selected || hovered {
            draw_circle(px, py, PALETTE_SWATCH_RADIUS + 3.0, PALETTE_RING);
        }
        draw_circle(px, py, PALETTE_SWATCH_RADIUS, swatch_color);

        if hovered && clicked {
            *current_color = swatch_color;
        }

        px += PALETTE_SWATCH_RADIUS * 2.0 + PALETTE_GAP;
    }

    // Docked to the top-right corner: undo, redo, then the color swatch.
    let swatch_rect = Rect::new(screen_w - SIDE_PADDING - BUTTON_WIDTH, button_y, BUTTON_WIDTH, button_height);
    let redo_rect = Rect::new(swatch_rect.x - BUTTON_GAP * 2.0 - BUTTON_WIDTH, button_y, BUTTON_WIDTH, button_height);
    let undo_rect = Rect::new(redo_rect.x - BUTTON_GAP - BUTTON_WIDTH, button_y, BUTTON_WIDTH, button_height);

    let undo_hovered = can_undo && undo_rect.contains(Vec2::new(mouse_x, mouse_y));
    if undo_hovered {
        draw_rounded_rect(undo_rect.x, undo_rect.y, undo_rect.w, undo_rect.h, BUTTON_RADIUS, HOVER_BG);
    }
    draw_undo_redo_icon(
        undo_rect.x + undo_rect.w / 2.0,
        undo_rect.y + undo_rect.h / 2.0,
        button_height * 0.62,
        if !can_undo {
            DISABLED_ICON
        } else if undo_hovered {
            HOVER_ICON
        } else {
            IDLE_ICON
        },
        false,
    );
    let undo_clicked = undo_hovered && clicked;

    let redo_hovered = can_redo && redo_rect.contains(Vec2::new(mouse_x, mouse_y));
    if redo_hovered {
        draw_rounded_rect(redo_rect.x, redo_rect.y, redo_rect.w, redo_rect.h, BUTTON_RADIUS, HOVER_BG);
    }
    draw_undo_redo_icon(
        redo_rect.x + redo_rect.w / 2.0,
        redo_rect.y + redo_rect.h / 2.0,
        button_height * 0.62,
        if !can_redo {
            DISABLED_ICON
        } else if redo_hovered {
            HOVER_ICON
        } else {
            IDLE_ICON
        },
        true,
    );
    let redo_clicked = redo_hovered && clicked;

    let swatch_hovered = swatch_rect.contains(Vec2::new(mouse_x, mouse_y));
    if swatch_hovered {
        draw_rounded_rect(swatch_rect.x, swatch_rect.y, swatch_rect.w, swatch_rect.h, BUTTON_RADIUS, HOVER_BG);
    }
    draw_circle(
        swatch_rect.x + swatch_rect.w / 2.0,
        swatch_rect.y + swatch_rect.h / 2.0,
        button_height * 0.3,
        *current_color,
    );
    let color_swatch_clicked = swatch_hovered && clicked;

    ToolbarClick {
        menu: menu_clicked,
        color_swatch: color_swatch_clicked,
        undo: undo_clicked,
        redo: redo_clicked,
    }
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

// The color wheel panel's bounds in screen space, dropped down from the
// top-right color swatch button. Exposed so callers can tell whether a click
// landed on the wheel itself (e.g. to decide whether to dismiss it).
pub fn color_wheel_panel_rect() -> Rect {
    let panel_w = WHEEL_RADIUS * 2.0 + WHEEL_MARGIN * 2.0;
    let panel_h = WHEEL_RADIUS * 2.0 + WHEEL_MARGIN * 2.0;
    let panel_x = screen_width() - SIDE_PADDING - panel_w;
    let panel_y = TOOLBAR_HEIGHT + WHEEL_PANEL_TOP_GAP;
    Rect::new(panel_x, panel_y, panel_w, panel_h)
}

// Draws a hue/saturation color wheel panel dropped down from the toolbar's
// color swatch button, and handles picking a color from it. Value is fixed
// at 1.0 (full brightness) so the wheel alone covers hue + saturation.
// `open` is cleared when Escape is pressed while the wheel is showing.
// `picking` persists across frames to track an in-progress drag; returns true
// the frame a pick is finalized (mouse released after dragging inside the
// wheel), so the caller can add the result to the color palette.
pub fn draw_color_wheel(current_color: &mut Color, open: &mut bool, picking: &mut bool) -> bool {
    if is_key_pressed(KeyCode::Escape) {
        *open = false;
        return false;
    }

    let panel = color_wheel_panel_rect();
    let cx = panel.x + panel.w / 2.0;
    let cy = panel.y + panel.h / 2.0;
    draw_rounded_rect(panel.x, panel.y, panel.w, panel.h, BAR_RADIUS, BG);

    // Triangle fan from a white center (saturation 0) to a ring of fully
    // saturated hues; the GPU interpolates the vertex colors across each
    // wedge, giving a smooth wheel from just ~50 flat-colored triangles.
    let mut vertices = Vec::with_capacity(WHEEL_SEGMENTS + 2);
    let mut indices = Vec::with_capacity(WHEEL_SEGMENTS * 3);
    vertices.push(Vertex::new(cx, cy, 0.0, 0.0, 0.0, WHITE));
    for i in 0..=WHEEL_SEGMENTS {
        let t = i as f32 / WHEEL_SEGMENTS as f32;
        let angle = t * std::f32::consts::TAU;
        let x = cx + angle.cos() * WHEEL_RADIUS;
        let y = cy + angle.sin() * WHEEL_RADIUS;
        vertices.push(Vertex::new(x, y, 0.0, 0.0, 0.0, hsv_to_rgb(t * 360.0, 1.0, 1.0)));
    }
    for i in 0..WHEEL_SEGMENTS as u16 {
        indices.extend_from_slice(&[0, i + 1, i + 2]);
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });

    let (mouse_x, mouse_y) = mouse_position();
    let dx = mouse_x - cx;
    let dy = mouse_y - cy;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist <= WHEEL_RADIUS && is_mouse_button_down(MouseButton::Left) {
        let hue = dy.atan2(dx).to_degrees().rem_euclid(360.0);
        let saturation = (dist / WHEEL_RADIUS).min(1.0);
        *current_color = hsv_to_rgb(hue, saturation, 1.0);
        *picking = true;
        false
    } else if *picking {
        *picking = false;
        true
    } else {
        false
    }
}

// Converts hue (degrees, 0-360), saturation and value (both 0-1) to an RGB color.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r, g, b) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    Color::new(r + m, g + m, b + m, 1.0)
}

// A curved arrow icon (an arc with a triangular arrowhead) used for both the
// undo and redo buttons. `flipped` mirrors it horizontally so the arrowhead
// points the other way, distinguishing redo from undo.
fn draw_undo_redo_icon(cx: f32, cy: f32, size: f32, color: Color, flipped: bool) {
    let radius = size * 0.34;
    let thickness = size * 0.15;
    let mirror = if flipped { -1.0 } else { 1.0 };

    // A ~280-degree arc leaves a gap near the top where the arrowhead sits.
    let rotation = if flipped { 220.0 } else { -40.0 };
    draw_arc(cx, cy, 24, radius, rotation, thickness, 280.0, color);

    let head_size = size * 0.28;
    let tip = Vec2::new(cx + mirror * radius * 0.86, cy - radius * 0.5);
    draw_triangle(
        tip + vec2(0.0, -head_size * 0.55),
        tip + vec2(mirror * head_size * 0.95, head_size * 0.15),
        tip + vec2(-mirror * head_size * 0.15, head_size * 0.7),
        color,
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
