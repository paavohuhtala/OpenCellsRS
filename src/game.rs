use cgmath::{Vector2, Zero};

use crate::{
    hexagon::{
        axial_to_cube, cube_to_axial, flat_hex_height, flat_hex_width, nearest_edge_hex,
        pixel_to_flat_hex, spiral_ring, CUBE_DIRECTIONS,
    },
    input::{HexKind, InputAction, InputState},
    level::{CellState, Hex, Level},
};

pub struct GameState {
    pub level: Level,

    pub scale: f32,
    pub offset: Vector2<f32>,

    pub nearest_edge: Vector2<f32>,
    pub cursor_hex_position: Vector2<i32>,
}

impl GameState {
    pub fn new() -> Self {
        let scale = 48.0;

        let offset = Vector2::new(flat_hex_width(scale) * 2.0, flat_hex_height(scale) * 1.5);

        GameState {
            level: Level::new(),
            scale,
            offset,
            cursor_hex_position: Vector2::zero(),
            nearest_edge: Vector2::zero(),
        }
    }
}

fn handle_input(state: &mut GameState, input_state: &mut InputState) {
    input_state.mouse_position = input_state.absolute_mouse_position - state.offset;

    state.cursor_hex_position = pixel_to_flat_hex(input_state.mouse_position, state.scale);

    let nearest_edge = nearest_edge_hex(input_state.mouse_position, state.scale);
    state.nearest_edge = nearest_edge;

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
            InputAction::ToggleRevealed => {
                let cell = state.level.cells.get_mut(&state.cursor_hex_position);
                if let Some(cell) = cell {
                    cell.start_revealed = !cell.start_revealed;
                }
            }
        }
    }

    if invalidated {
        calculate_hints(state);
    }
}

pub fn update(state: &mut GameState, input_state: &mut InputState) {
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
