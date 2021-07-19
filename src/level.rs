use std::collections::HashMap;

use cgmath::{Vector2, Vector3};

pub enum Hex {
    Empty { show_neighbor_count: bool },
    Marked { show_around: bool },
}

impl Hex {
    pub fn get_color(&self, revealed: bool) -> Vector3<f32> {
        match self {
            _ if !revealed => Vector3::new(0.368, 0.368, 0.368),
            Hex::Empty { .. } => Vector3::new(0.960, 0.505, 0.058),
            Hex::Marked { .. } => Vector3::new(0.058, 0.533, 0.960),
        }
    }
}

pub struct Level {
    pub hexes: HashMap<Vector2<i32>, Hex>,
}
