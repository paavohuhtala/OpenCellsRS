use std::{cell::Cell, collections::HashMap};

type MemoryCell<T> = Cell<T>;

use cgmath::{Vector2, Vector3};

use crate::hexagon::Axial;

#[derive(Debug)]
pub enum Hex {
    Empty { show_neighbor_count: bool },
    Marked { show_around: bool },
}

impl Hex {
    pub fn get_color(&self, revealed: bool) -> Vector3<f32> {
        match self {
            _ if !revealed => Vector3::new(0.960, 0.505, 0.058),
            Hex::Empty { .. } => Vector3::new(0.368, 0.368, 0.368),
            Hex::Marked { .. } => Vector3::new(0.058, 0.533, 0.960),
        }
    }
}

#[derive(Debug)]
pub struct CellState {
    pub hex: Hex,
    pub start_revealed: bool,
    revealed: MemoryCell<bool>,
    marked_neighbors: MemoryCell<usize>,
}

impl CellState {
    pub fn new(hex: Hex) -> Self {
        CellState {
            hex,
            start_revealed: false,
            revealed: MemoryCell::new(false),
            marked_neighbors: MemoryCell::new(0),
        }
    }

    pub fn neighbors(&self) -> usize {
        self.marked_neighbors.get()
    }

    pub fn neighbors_str(&self) -> &'static str {
        let neighbors = self.neighbors();

        match neighbors {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            _ => unreachable!("A cell can have up to 6 neighbors."),
        }
    }

    pub fn is_revealed(&self) -> bool {
        self.revealed.get()
    }

    pub fn update_neighbors(&self, neighbors: usize) {
        self.marked_neighbors.set(neighbors)
    }

    pub fn reveal(&self) {
        self.revealed.set(true)
    }
}

pub struct Level {
    pub cells: HashMap<Vector2<i32>, CellState>,
}

impl Level {
    pub fn new() -> Level {
        let cells = HashMap::new();

        Level { cells }
    }

    pub fn is_marked(&self, axial: Axial) -> bool {
        self.cells
            .get(&axial)
            .map(|cell| match &cell.hex {
                Hex::Marked { .. } => true,
                _ => false,
            })
            .unwrap_or(false)
    }
}
