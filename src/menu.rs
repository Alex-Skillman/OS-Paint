use macroquad::prelude::*;

// Sub-screen shown inside the pause menu.
pub enum MenuScreen {
    Root,
    JoinInput(String),
}

pub struct MenuState {
    pub screen: MenuScreen,
}

impl MenuState {
    pub fn new() -> Self {
        MenuState { screen: MenuScreen::Root }
    }
}

// What the caller (main.rs) should do in response to this frame's menu input.
// Menu.rs never touches networking directly, since it doesn't own the socket.
pub enum MenuAction {
    None,
    Close,
    Host,
    Join(String),
    Leave,
}

const PANEL_WIDTH: f32 = 320.0;
const BUTTON_HEIGHT: f32 = 40.0;
const BUTTON_GAP: f32 = 14.0;

// Draws the pause menu (dimmed overlay on top of the canvas) and handles its
// input for one frame. `in_lobby`/`room_code` switch it between offering
// "Host"/"Join" (single-person mode) and "Leave Lobby" (connected).
pub fn update_menu(menu: &mut MenuState, in_lobby: bool, room_code: Option<&str>) -> MenuAction {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.6));

    let cx = screen_width() / 2.0;

    match &mut menu.screen {
        MenuScreen::Root => {
            if is_key_pressed(KeyCode::Escape) {
                return MenuAction::Close;
            }

            let mut y = screen_height() / 2.0 - 140.0;
            draw_text_centered("Menu", cx, y, 32.0, WHITE);
            y += 60.0;

            if in_lobby {
                if let Some(code) = room_code {
                    draw_text_centered(&format!("Room code: {code}"), cx, y, 22.0, LIGHTGRAY);
                    y += 40.0;
                }
                if button(cx - PANEL_WIDTH / 2.0, y, "Leave Lobby") {
                    return MenuAction::Leave;
                }
                y += BUTTON_HEIGHT + BUTTON_GAP;
            } else {
                if button(cx - PANEL_WIDTH / 2.0, y, "Host a Lobby") {
                    return MenuAction::Host;
                }
                y += BUTTON_HEIGHT + BUTTON_GAP;

                if button(cx - PANEL_WIDTH / 2.0, y, "Join a Lobby") {
                    menu.screen = MenuScreen::JoinInput(String::new());
                    return MenuAction::None;
                }
                y += BUTTON_HEIGHT + BUTTON_GAP;
            }

            if button(cx - PANEL_WIDTH / 2.0, y, "Resume") {
                return MenuAction::Close;
            }

            MenuAction::None
        }
        MenuScreen::JoinInput(digits) => {
            if is_key_pressed(KeyCode::Escape) {
                menu.screen = MenuScreen::Root;
                return MenuAction::None;
            }

            if let Some(c) = get_char_pressed() {
                if c.is_ascii_digit() && digits.len() < 6 {
                    digits.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                digits.pop();
            }

            let mut y = screen_height() / 2.0 - 100.0;
            draw_text_centered("Enter 6-digit room code", cx, y, 26.0, WHITE);
            y += 60.0;

            let shown = if digits.is_empty() { "______".to_string() } else { digits.clone() };
            draw_text_centered(&shown, cx, y, 40.0, WHITE);
            y += 60.0;

            let ready = digits.len() == 6;
            let joined_by_enter = ready && is_key_pressed(KeyCode::Enter);
            let joined_by_click = button(cx - PANEL_WIDTH / 2.0, y, "Join") && ready;
            if joined_by_enter || joined_by_click {
                return MenuAction::Join(digits.clone());
            }
            y += BUTTON_HEIGHT + BUTTON_GAP;

            if button(cx - PANEL_WIDTH / 2.0, y, "Back") {
                menu.screen = MenuScreen::Root;
            }

            MenuAction::None
        }
    }
}

fn draw_text_centered(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, cx - dims.width / 2.0, y, size, color);
}

// A full-width menu button at (x, y); returns true the frame it's clicked.
fn button(x: f32, y: f32, label: &str) -> bool {
    let rect = Rect::new(x, y, PANEL_WIDTH, BUTTON_HEIGHT);
    let (mx, my) = mouse_position();
    let hovered = rect.contains(Vec2::new(mx, my));

    let bg = if hovered {
        Color::new(0.30, 0.55, 0.98, 1.0)
    } else {
        Color::new(1.0, 1.0, 1.0, 0.08)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_text_centered(label, rect.x + rect.w / 2.0, rect.y + rect.h / 2.0 + 7.0, 20.0, WHITE);

    hovered && is_mouse_button_pressed(MouseButton::Left)
}
