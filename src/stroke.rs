use macroquad::color::Color;

#[derive(PartialEq, Clone, Copy)]
pub enum Tool {
    Pen,
    Eraser,
}
pub struct Stroke {
    pub size: u16,
    pub color: Color,
    pub layer: i8, //Unused for now but may add later
    pub coordinates: Vec<(u16, u16)>,
}