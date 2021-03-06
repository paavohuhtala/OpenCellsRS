use cgmath::{num_traits::Signed, EuclideanSpace, MetricSpace, Point2, Vector2, Vector3, Zero};
use luminance::{context::GraphicsContext, tess::Interleaved, vertex::Vertex};
use luminance_front::{tess::Tess, Backend};

use crate::render::{
    BasicVertex, BasicVertexPosition, HexVertex, HexVertexEdginess, HexVertexPosition,
    SmallVertexIndex,
};

pub type Axial = Vector2<i32>;
pub type AxialF = Vector2<f32>;
pub type Cube = Vector3<i32>;
pub type CubeF = Vector3<f32>;

fn create_mesh<C, V: Vertex>(
    context: &mut C,
    create_vertex: impl Fn(Vector2<f32>, bool) -> V,
) -> Tess<V, SmallVertexIndex, (), Interleaved>
where
    V: Vertex,
    C: GraphicsContext<Backend = Backend>,
{
    let mut verts = Vec::with_capacity(7);

    verts.push(create_vertex(Vector2::zero(), true));

    for i in 0..6 {
        let outer_position = hex_corner(Vector2::new(0.0, 0.0), 0.95, i);
        let vert = create_vertex(outer_position.into(), false);
        verts.push(vert);
    }

    #[rustfmt::skip]
        let indices: &[SmallVertexIndex] = &[
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

pub fn create_hexagon_mesh_border<C>(
    context: &mut C,
) -> Tess<HexVertex, SmallVertexIndex, (), Interleaved>
where
    C: GraphicsContext<Backend = Backend>,
{
    create_mesh(context, |position, is_center| HexVertex {
        position: HexVertexPosition::new(position.into()),
        edginess: HexVertexEdginess::new(if is_center { 0.0 } else { 1.0 }),
    })
}

#[allow(dead_code)]
pub fn create_hexagon_mesh<C>(
    context: &mut C,
) -> Tess<BasicVertex, SmallVertexIndex, (), Interleaved>
where
    C: GraphicsContext<Backend = Backend>,
{
    create_mesh(context, |position, _is_center| BasicVertex {
        position: BasicVertexPosition::new(position.into()),
    })
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

pub fn pixel_to_flat_hex(point: Vector2<f32>, size: f32) -> Axial {
    hex_round(pixel_to_flat_hex_f(point, size))
}

pub fn pixel_to_flat_hex_f(point: Vector2<f32>, size: f32) -> AxialF {
    let x = (2.0 / 3.0 * point.x) / size;
    let y = (-1.0 / 3.0 * point.x + 3.0f32.sqrt() / 3.0 * point.y) / size;
    Vector2::new(x, y)
}

pub fn flat_hex_to_pixel(point: Axial, size: f32) -> Vector2<f32> {
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

#[allow(dead_code)]
pub const AXIAL_DIRECTION: [Axial; 6] = [
    Axial::new(1, 0),
    Axial::new(1, -1),
    Axial::new(0, -1),
    Axial::new(-1, 0),
    Axial::new(-1, 1),
    Axial::new(0, 1),
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

pub fn nearest_edge_hex(pixel_pos: Vector2<f32>, scale: f32) -> Vector2<f32> {
    let axial_f = pixel_to_flat_hex_f(pixel_pos, scale);
    let axial = hex_round(axial_f);
    let pixel_center = flat_hex_to_pixel(axial, scale);

    let cube_f = axial_to_cube(axial_f);
    let cube = axial_to_cube(axial);

    let mut nearest_neighbor = None;
    let mut smallest_distance = f32::MAX;

    for dir in &CUBE_DIRECTIONS {
        let neighbor_cube = cube + dir;
        let neighbor_cube_f = neighbor_cube.cast::<f32>().unwrap();

        let distance = cube_f.distance2(neighbor_cube_f);

        if distance < smallest_distance {
            nearest_neighbor = Some(neighbor_cube);
            smallest_distance = distance;
        }
    }

    let nearest = nearest_neighbor.unwrap();
    let nearest_axial = cube_to_axial(nearest);
    let nearest_pixel = flat_hex_to_pixel(nearest_axial, scale);

    Point2::from_vec(pixel_center)
        .midpoint(Point2::from_vec(nearest_pixel))
        .to_vec()
}
