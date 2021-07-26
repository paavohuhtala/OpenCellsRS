use cgmath::{Vector2, Zero};

use crate::{
    hexagon::{axial_to_cube, cube_to_axial, pixel_to_flat_hex, spiral_ring, CUBE_DIRECTIONS},
    input::{HexKind, InputAction, InputState},
    level::{CellState, Hex, Level},
};

pub struct GameState {
    pub level: Level,
    pub cursor_hex_position: Vector2<i32>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            level: Level::new(),
            cursor_hex_position: Vector2::zero(),
        }
    }
}

fn handle_input(state: &mut GameState, input_state: &mut InputState) {
    // TODO: make scale configurable
    state.cursor_hex_position = pixel_to_flat_hex(input_state.mouse_position, 64.0);

    let mut invalidated = false;

    for action in input_state.action_queue.drain(..) {
        match action {
            InputAction::PlaceHex(kind) => {
                state.level.cells.insert(
                    state.cursor_hex_position,
                    match kind {
                        HexKind::Empty => CellState::new(Hex::Empty {
                            show_neighbor_count: true,
                        }),
                        HexKind::Marked => CellState::new(Hex::Marked { show_around: false }),
                    },
                );

                invalidated = true;
            }
            InputAction::ClearHex => {
                state.level.cells.remove(&state.cursor_hex_position);

                invalidated = true;
            }
            InputAction::RingDebug => {
                let mut coords = Vec::new();
                spiral_ring(axial_to_cube(state.cursor_hex_position), 1, &mut coords);

                for c in coords.drain(..) {
                    let coord = cube_to_axial(c);
                    state.level.cells.insert(
                        coord,
                        CellState::new(Hex::Empty {
                            show_neighbor_count: false,
                        }),
                    );
                }

                invalidated = true;
            }
        }
    }

    if invalidated {
        calculate_hints(state);
    }
}

pub fn update_state(state: &mut GameState, input_state: &mut InputState) {
    handle_input(state, input_state);
}

pub fn calculate_hints(state: &mut GameState) {
    for (axial, cell) in state.level.cells.iter() {
        match &cell.hex {
            Hex::Empty {
                show_neighbor_count: true,
            } => {
                let cube = axial_to_cube(*axial);

                let mut neighbor_count = 0;

                for direction in &CUBE_DIRECTIONS {
                    let neighbor_cube = cube + direction;
                    let neighbor_axial = cube_to_axial(neighbor_cube);

                    if state.level.is_marked(neighbor_axial) {
                        neighbor_count += 1;
                    }
                }

                cell.update_neighbors(neighbor_count);
            }
            _ => {}
        }
    }
}
