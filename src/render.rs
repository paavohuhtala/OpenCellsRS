use luminance::shader::Uniform;
use luminance_derive::{Semantics, UniformInterface, Vertex};

#[derive(Copy, Clone, Debug, Semantics)]
pub enum HexVertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "color", repr = "f32", wrapper = "VertexEdginess")]
    Edginess,
}

#[derive(Vertex, Clone, Copy)]
#[vertex(sem = "HexVertexSemantics")]
pub struct HexVertex {
    #[allow(dead_code)]
    pub(crate) position: VertexPosition,

    #[allow(dead_code)]
    #[vertex(normalized = "true")]
    pub(crate) edginess: VertexEdginess,
}

pub type HexVertexIndex = u16;

#[derive(UniformInterface, Debug)]
pub struct HexInterface {
    pub(crate) view: Uniform<[[f32; 4]; 4]>,
    pub(crate) model_color: Uniform<[f32; 3]>,
}
