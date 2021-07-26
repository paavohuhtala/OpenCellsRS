use cgmath::{num_traits::Signed, Vector2, Vector3};
use luminance::{context::GraphicsContext, tess::Interleaved};
use luminance_front::{tess::Tess, Backend};

use crate::render::{HexVertex, HexVertexIndex, VertexEdginess, VertexPosition};

pub type Axial = Vector2<i32>;
pub type AxialF = Vector2<f32>;
pub type Cube = Vector3<i32>;
pub type CubeF = Vector3<f32>;

pub fn create_hexagon_mesh_border<C>(
    context: &mut C,
) -> Tess<HexVertex, HexVertexIndex, (), Interleaved>
where
    C: GraphicsContext<Backend = Backend>,
{
    let mut verts = Vec::with_capacity(7);

    verts.push(HexVertex {
        position: VertexPosition::new(Vector2::new(0.0, 0.0).into()),
        edginess: VertexEdginess::new(0.0),
    });

    for i in 0..6 {
        let outer_position = hex_corner(Vector2::new(0.0, 0.0), 0.95, i);
        let vert = HexVertex {
            position: VertexPosition::new(outer_position.into()),
            edginess: VertexEdginess::new(1.0),
        };
        verts.push(vert);
    }

    #[rustfmt::skip]
    let indices: &[HexVertexIndex] = &[
        0, 1, 2, 3, 4, 5, 6, 1
    ];

    context
        .new_tess()
        .set_vertices(verts)
        .set_indices(indices)
        .set_mode(luminance::tess::Mode::TriangleFan)
        .build()
        .unwrap()
}

// https://www.redblobgames.com/grids/hexagons/

fn hex_corner(center: Vector2<f32>, size: f32, i: usize) -> Vector2<f32> {
    assert!(i <= 5, "i must be between 0 and 6");

    let angle_deg = 60.0 * i as f32;
    let angle_rad = angle_deg.to_radians();

    center + Vector2::new(size * angle_rad.cos(), size * angle_rad.sin())
}

pub fn cube_to_axial<N: Signed + Copy>(cube: Vector3<N>) -> Vector2<N> {
    Vector2::new(cube.x, cube.z)
}

pub fn axial_to_cube<N: Signed + Copy>(hex: Vector2<N>) -> Vector3<N> {
    let x = hex.x;
    let z = hex.y;
    let y = x * (-N::one()) - z;
    Vector3::new(x, y, z)
}

pub fn cube_round(f_cube: CubeF) -> Cube {
    let mut rx = f_cube.x.round();
    let mut ry = f_cube.y.round();
    let mut rz = f_cube.z.round();

    let x_diff = (rx - f_cube.x).abs();
    let y_diff = (ry - f_cube.y).abs();
    let z_diff = (rz - f_cube.z).abs();

    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff > z_diff {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }

    Vector3::new(rx as i32, ry as i32, rz as i32)
}

pub fn hex_round(f_hex: AxialF) -> Axial {
    cube_to_axial(cube_round(axial_to_cube(f_hex)))
}

pub fn pixel_to_flat_hex(point: Vector2<f32>, size: f32) -> Vector2<i32> {
    let x = (2.0 / 3.0 * point.x) / size;
    let y = (-1.0 / 3.0 * point.x + 3.0f32.sqrt() / 3.0 * point.y) / size;

    hex_round(Vector2::new(x, y))
}

pub fn flat_hex_to_pixel(point: Vector2<i32>, size: f32) -> Vector2<f32> {
    let x = size * (3.0 / 2.0 * point.x as f32);
    let y = size * (3f32.sqrt() / 2.0 * (point.x as f32) + 3f32.sqrt() * (point.y as f32));
    Vector2::new(x, y)
}

pub fn flat_hex_height(size: f32) -> f32 {
    3f32.sqrt() * size
}

pub fn flat_hex_width(size: f32) -> f32 {
    size * 2.0
}

pub const CUBE_DIRECTIONS: [Cube; 6] = [
    Cube::new(1, -1, 0),
    Cube::new(1, 0, -1),
    Cube::new(0, 1, -1),
    Cube::new(-1, 1, 0),
    Cube::new(-1, 0, 1),
    Cube::new(0, -1, 1),
];

fn cube_neighbor(cube: Cube, direction: usize) -> Cube {
    cube + CUBE_DIRECTIONS[direction]
}

pub fn cube_ring(center: Cube, radius: u32, results: &mut Vec<Cube>) {
    let mut cube = center + CUBE_DIRECTIONS[4] * (radius as i32);

    for i in 0..6 {
        for _ in 0..radius {
            results.push(cube);
            cube = cube_neighbor(cube, i);
        }
    }
}

pub fn spiral_ring(center: Cube, radius: u32, results: &mut Vec<Cube>) {
    for i in 1..=radius {
        cube_ring(center, i, results);
    }
}
