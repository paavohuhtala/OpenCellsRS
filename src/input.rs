use cgmath::{Vector2, Zero};

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
    pub absolute_mouse_position: Vector2<f32>,
    pub mouse_position: Vector2<f32>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            action_queue: Vec::new(),
            absolute_mouse_position: Vector2::zero(),
            mouse_position: Vector2::zero(),
        }
    }
}
