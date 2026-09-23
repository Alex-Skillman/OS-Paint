use macroquad::prelude::*;

// Sub-screen shown inside the pause menu.
pub enum MenuScreen {
    Root,
    JoinInput(String),
    NameInput(String),
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
    SetName(String),
}

const PANEL_WIDTH: f32 = 320.0;
const BUTTON_HEIGHT: f32 = 40.0;
const BUTTON_GAP: f32 = 14.0;
const IDLE_BG: Color = Color::new(1.0, 1.0, 1.0, 0.08);

// Draws the pause menu (dimmed overlay on top of the canvas) and handles its
// input for one frame. `in_lobby`/`room_code` switch it between offering
// "Host"/"Join" (single-person mode) and "Leave Lobby" (connected).
// `player_name` is shown (and editable) so the local player can set the name
// shown to others. `lobby_members` (the local player's name/color plus each
// connected peer's) is shown as a row of colored, named circles so everyone
// can see who's in the room, and what they're about to draw with, at a glance.
pub fn update_menu(
    menu: &mut MenuState,
    in_lobby: bool,
    room_code: Option<&str>,
    player_name: &str,
    lobby_members: &[(String, Color)],
) -> MenuAction {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.6));

    let cx = screen_width() / 2.0;

    match &mut menu.screen {
        MenuScreen::Root => {
            if is_key_pressed(KeyCode::Escape) {
                return MenuAction::Close;
            }

            let mut y = screen_height() / 2.0 - if in_lobby { 240.0 } else { 170.0 };
            draw_text_centered("Menu", cx, y, 32.0, WHITE);
            y += 50.0;

            if in_lobby {
                draw_participant_row(lobby_members, cx, y);
                y += 100.0;
            }

            if button(cx - PANEL_WIDTH / 2.0, y, &format!("Name: {player_name}")) {
                menu.screen = MenuScreen::NameInput(player_name.to_string());
                return MenuAction::None;
            }
            y += BUTTON_HEIGHT + BUTTON_GAP;

            if in_lobby {
                if let Some(code) = room_code {
                    info_row(cx - PANEL_WIDTH / 2.0, y, &format!("Room code: {code}"));
                    y += BUTTON_HEIGHT + BUTTON_GAP;
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
        MenuScreen::NameInput(name) => {
            if is_key_pressed(KeyCode::Escape) {
                menu.screen = MenuScreen::Root;
                return MenuAction::None;
            }

            if let Some(c) = get_char_pressed() {
                if (c.is_ascii_alphanumeric() || c == ' ') && name.len() < 16 {
                    name.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                name.pop();
            }

            let mut y = screen_height() / 2.0 - 100.0;
            draw_text_centered("Enter your name", cx, y, 26.0, WHITE);
            y += 60.0;

            let shown = if name.is_empty() { "_".to_string() } else { name.clone() };
            draw_text_centered(&shown, cx, y, 32.0, WHITE);
            y += 60.0;

            let ready = !name.trim().is_empty();
            let saved_by_enter = ready && is_key_pressed(KeyCode::Enter);
            let saved_by_click = button(cx - PANEL_WIDTH / 2.0, y, "Save") && ready;
            if saved_by_enter || saved_by_click {
                return MenuAction::SetName(name.trim().to_string());
            }
            y += BUTTON_HEIGHT + BUTTON_GAP;

            if button(cx - PANEL_WIDTH / 2.0, y, "Back") {
                menu.screen = MenuScreen::Root;
            }

            MenuAction::None
        }
    }
}

const PARTICIPANT_RADIUS: f32 = 14.0;
const PARTICIPANT_GAP: f32 = 22.0;
const PARTICIPANT_LABEL_SIZE: f32 = 16.0;
const PARTICIPANT_COUNT_SIZE: f32 = 18.0;

// Draws one colored, named circle per lobby member (the local player plus
// every connected peer), each in that member's currently-selected color,
// with a count underneath. Each member gets a column at least as wide as
// their own name label, so longer names push their neighbors aside instead
// of overlapping them.
fn draw_participant_row(members: &[(String, Color)], cx: f32, y: f32) {
    if members.is_empty() {
        return;
    }

    let col_widths: Vec<f32> = members
        .iter()
        .map(|(name, _)| {
            measure_text(name, None, PARTICIPANT_LABEL_SIZE as u16, 1.0)
                .width
                .max(PARTICIPANT_RADIUS * 2.0)
        })
        .collect();
    let row_width: f32 =
        col_widths.iter().sum::<f32>() + PARTICIPANT_GAP * (members.len().saturating_sub(1)) as f32;

    let mut x = cx - row_width / 2.0;
    for ((name, color), &col_w) in members.iter().zip(col_widths.iter()) {
        let center_x = x + col_w / 2.0;
        draw_circle(center_x, y, PARTICIPANT_RADIUS, *color);
        draw_text_centered(name, center_x, y + PARTICIPANT_RADIUS + 22.0, PARTICIPANT_LABEL_SIZE, LIGHTGRAY);
        x += col_w + PARTICIPANT_GAP;
    }

    let count_label = format!("{} in lobby", members.len());
    draw_text_centered(&count_label, cx, y + PARTICIPANT_RADIUS + 48.0, PARTICIPANT_COUNT_SIZE, LIGHTGRAY);
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

    let bg = if hovered { Color::new(0.30, 0.55, 0.98, 1.0) } else { IDLE_BG };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_text_centered(label, rect.x + rect.w / 2.0, rect.y + rect.h / 2.0 + 7.0, 20.0, WHITE);

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

// A full-width, non-interactive row with the same background as `button`,
// for read-only info (e.g. the room code) so it lines up visually with the
// buttons around it instead of floating as bare text.
fn info_row(x: f32, y: f32, label: &str) {
    let rect = Rect::new(x, y, PANEL_WIDTH, BUTTON_HEIGHT);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, IDLE_BG);
    draw_text_centered(label, rect.x + rect.w / 2.0, rect.y + rect.h / 2.0 + 7.0, 20.0, LIGHTGRAY);
}
