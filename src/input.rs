use cgmath::Vector2;

#[derive(Debug)]
pub enum HexKind {
    Empty,
    Marked,
}

#[derive(Debug)]
pub enum InputAction {
    ClearHex,
    PlaceHex(HexKind),
    RingDebug,
}

pub struct InputState {
    pub action_queue: Vec<InputAction>,
    pub mouse_position: Vector2<f32>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            action_queue: Vec::new(),
            mouse_position: Vector2::new(0.0, 0.0),
        }
    }
}
