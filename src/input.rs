use cgmath::Vector2;
use legion::system;

use crate::{
    hexagon::pixel_to_flat_hex,
    level::{Hex, Level},
};

#[derive(Debug)]
pub enum HexKind {
    Empty,
    Marked,
}

#[derive(Debug)]
pub enum InputAction {
    ClearHex,
    PlaceHex(HexKind),
}

pub struct InputState {
    pub action_queue: Vec<InputAction>,
    pub mouse_position: Vector2<f32>,
    pub hex_position: Vector2<i32>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            action_queue: Vec::new(),
            mouse_position: Vector2::new(0.0, 0.0),
            hex_position: Vector2::new(0, 0),
        }
    }
}

#[system]
pub fn handle_input(#[resource] input: &mut InputState, #[resource] level: &mut Level) {
    // TODO: make scale configurable
    input.hex_position = pixel_to_flat_hex(input.mouse_position, 64.0);

    for action in input.action_queue.drain(..) {
        match action {
            InputAction::PlaceHex(kind) => {
                level.hexes.insert(
                    input.hex_position,
                    match kind {
                        HexKind::Empty => Hex::Empty {
                            show_neighbor_count: true,
                        },
                        HexKind::Marked => Hex::Marked { show_around: false },
                    },
                );
            }
            InputAction::ClearHex => {
                level.hexes.remove(&input.hex_position);
            }
        }
    }
}
