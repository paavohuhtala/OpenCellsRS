use cgmath::{Matrix4, Ortho, Vector3};
use luminance::{
    context::GraphicsContext, pipeline::PipelineState, render_state::RenderState, shader::Uniform,
    tess::Interleaved,
};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::{shader::Program, tess::Tess, Backend};
use luminance_glutin::GlutinSurface;
use luminance_glyph::{
    ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text,
    VerticalAlign,
};

use crate::{
    game::GameState,
    hexagon::{create_hexagon_mesh_border, flat_hex_height, flat_hex_to_pixel},
    level::Hex,
};

#[derive(Copy, Clone, Debug, Semantics)]
pub enum HexVertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "HexVertexPosition")]
    Position,
    #[sem(name = "edginess", repr = "f32", wrapper = "HexVertexEdginess")]
    Edginess,
}

#[derive(Vertex, Clone, Copy)]
#[vertex(sem = "HexVertexSemantics")]
pub struct HexVertex {
    #[allow(dead_code)]
    pub(crate) position: HexVertexPosition,

    #[allow(dead_code)]
    #[vertex(normalized = "true")]
    pub(crate) edginess: HexVertexEdginess,
}

#[derive(UniformInterface, Debug)]
pub struct HexInterface {
    pub(crate) view: Uniform<[[f32; 4]; 4]>,
    pub(crate) model_color: Uniform<[f32; 3]>,
}

const HEX_VS: &'static str = include_str!("shaders/hex-vs.glsl");
const HEX_FS: &'static str = include_str!("shaders/hex-fs.glsl");

#[derive(Copy, Clone, Debug, Semantics)]
pub enum BasicVertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "BasicVertexPosition")]
    Position,
}

#[derive(Vertex, Clone, Copy)]
#[vertex(sem = "BasicVertexSemantics")]
pub struct BasicVertex {
    #[allow(dead_code)]
    pub(crate) position: BasicVertexPosition,
}

#[derive(UniformInterface, Debug)]
pub struct BasicVertexInterface {
    pub(crate) view: Uniform<[[f32; 4]; 4]>,
    pub(crate) model_color: Uniform<[f32; 3]>,
}

const BASIC_VS: &'static str = include_str!("shaders/basic-2d-vs.glsl");
const BASIC_FS: &'static str = include_str!("shaders/basic-2d-fs.glsl");

pub type SmallVertexIndex = u16;

pub struct Renderer {
    hex_program: Program<HexVertexSemantics, (), HexInterface>,
    bordered_hex_mesh: Tess<HexVertex, SmallVertexIndex, (), Interleaved>,

    basic_program: Program<BasicVertexSemantics, (), BasicVertexInterface>,

    glyph_brush: GlyphBrush<Backend>,

    projection_matrix: Matrix4<f32>,
}

impl Renderer {
    pub fn new(surface: &mut GlutinSurface) -> Renderer {
        let hex_program = surface
            .new_shader_program::<HexVertexSemantics, (), HexInterface>()
            .from_strings(HEX_VS, None, None, HEX_FS)
            .unwrap()
            .ignore_warnings();
        let bordered_hex_mesh = create_hexagon_mesh_border(surface);

        let basic_program = surface
            .new_shader_program::<BasicVertexSemantics, (), BasicVertexInterface>()
            .from_strings(BASIC_VS, None, None, BASIC_FS)
            .unwrap()
            .ignore_warnings();

        let glyph_brush = GlyphBrushBuilder::using_font(
            FontArc::try_from_slice(include_bytes!("../assets/fonts/Aileron-Regular.otf")).unwrap(),
        )
        .build(surface);

        let [width, height] = surface.size();

        Renderer {
            hex_program,
            bordered_hex_mesh,

            basic_program,

            glyph_brush,
            projection_matrix: Self::get_projection_matrix(width, height),
        }
    }

    fn get_projection_matrix(width: u32, height: u32) -> Matrix4<f32> {
        Matrix4::from(Ortho {
            left: 0.0,
            right: width as f32,
            bottom: height as f32,
            top: 0.0,
            near: -2.0,
            far: 100.0,
        })
    }

    pub fn update_resolution(&mut self, width: u32, height: u32) {
        self.projection_matrix = Self::get_projection_matrix(width, height);
    }

    pub fn queue_text(&mut self, section: Section) {
        self.glyph_brush.queue(section);
    }

    pub fn render(&mut self, state: &GameState, surface: &mut GlutinSurface) {
        let level = &state.level;
        let hex_under_cursor = state.cursor_hex_position;
        let camera_offset = state.offset;
        let scale = state.scale;

        let [viewport_width, viewport_height] = surface.size();

        let back_buffer = surface.back_buffer().unwrap();

        self.queue_text(
            Section::default()
                .add_text(
                    Text::new("Mistakes: 0")
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(48f32)
                        .with_z(-1.0),
                )
                .with_screen_position((viewport_width as f32 - 250.0, 100.0)),
        );

        self.render_diagonal_hover(state, camera_offset);

        for (pos, cell) in &level.cells {
            match &cell.hex {
                Hex::Empty {
                    show_neighbor_count: true,
                } => {
                    self.queue_text(
                        Section::default()
                            .add_text(
                                Text::new(cell.neighbors_str())
                                    .with_color([1.0, 1.0, 1.0, 1.0])
                                    .with_scale(state.scale / 2.0)
                                    .with_z(-1.0),
                            )
                            .with_layout(
                                Layout::default_single_line()
                                    .h_align(HorizontalAlign::Center)
                                    .v_align(VerticalAlign::Center),
                            )
                            .with_screen_position(
                                flat_hex_to_pixel(pos.clone(), scale) + camera_offset,
                            ),
                    );
                }
                _ => {}
            }
        }

        self.glyph_brush.process_queued(surface);

        let hex_program = &mut self.hex_program;
        let hex_mesh = &self.bordered_hex_mesh;

        let basic_program = &mut self.basic_program;

        let glyph_brush = &mut self.glyph_brush;
        let projection = &self.projection_matrix;

        let render = surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color([0.1, 0.1, 0.1, 1.0]),
                |mut pipeline, mut shd_gate| {
                    shd_gate
                        .shade(hex_program, |mut iface, uni, mut rdr_gate| {
                            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                                for (position, cell) in &level.cells {
                                    let view = get_hex_view_matrix(
                                        camera_offset,
                                        *position,
                                        scale,
                                        projection,
                                    );
                                    iface.set(&uni.view, view.into());

                                    let color = cell.hex.get_color(true)
                                        * (if hex_under_cursor == *position {
                                            1.5
                                        } else {
                                            1.0
                                        });

                                    iface.set(&uni.model_color, color.into());

                                    tess_gate
                                        .render(hex_mesh)
                                        .map_err(|_e: &'static str| ())
                                        .unwrap();
                                }

                                Ok(())
                            })
                        })
                        .map_err(|_: &'static str| ())
                        .unwrap();

                    shd_gate
                        .shade(basic_program, |mut iface, uni, mut rdr_gate| {
                            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                                for (&hex_position, cell) in
                                    level.cells.iter().filter(|cell| cell.1.start_revealed)
                                {
                                    if cell.start_revealed {
                                        let view = get_hex_revealed_indicator_matrix(
                                            camera_offset,
                                            hex_position,
                                            scale,
                                            projection,
                                        );

                                        iface.set(&uni.view, view.into());
                                        iface.set(&uni.model_color, [1.0, 1.0, 1.0]);

                                        tess_gate
                                            .render(hex_mesh)
                                            .map_err(|_e: &'static str| ())
                                            .unwrap();
                                    }
                                }

                                Ok(())
                            })
                        })
                        .map_err(|_: &'static str| ())
                        .unwrap();

                    glyph_brush
                        .draw_queued(
                            &mut pipeline,
                            &mut shd_gate,
                            viewport_width,
                            viewport_height,
                        )
                        .expect("failed to render glyphs");

                    Ok(())
                },
            )
            .assume();
        if render.is_ok() {
            surface.swap_buffers();
        }
    }

    fn render_diagonal_hover(&mut self, state: &GameState, offset: cgmath::Vector2<f32>) {
        self.queue_text(
            Section::default()
                .add_text(
                    Text::new("=")
                        .with_color([0.0, 0.0, 1.0, 1.0])
                        .with_scale(48f32)
                        .with_z(-1.0),
                )
                .with_layout(
                    Layout::default_single_line()
                        .h_align(HorizontalAlign::Center)
                        .v_align(VerticalAlign::Center),
                )
                .with_screen_position(state.nearest_edge + offset),
        );
    }
}

fn get_hex_revealed_indicator_matrix(
    camera_offset: cgmath::Vector2<f32>,
    hex_position: cgmath::Vector2<i32>,
    scale: f32,
    projection: &Matrix4<f32>,
) -> Matrix4<f32> {
    let offset = camera_offset.extend(0.0);
    let relative_hex_position = flat_hex_to_pixel(hex_position, scale);
    let symbol_offset = flat_hex_height(scale) / 2.0 - 16.0;
    let translation = Matrix4::from_translation(
        offset + relative_hex_position.extend(0.0) + Vector3::new(0.0, symbol_offset, 1.0),
    );
    let scale = Matrix4::from_scale(4.0);
    let view = projection * translation * scale;
    view
}

fn get_hex_view_matrix(
    offset: cgmath::Vector2<f32>,
    hex_position: cgmath::Vector2<i32>,
    scale: f32,
    projection: &Matrix4<f32>,
) -> Matrix4<f32> {
    let offset = Vector3::new(offset.x, offset.y, 0.0);
    let relative_position = flat_hex_to_pixel(hex_position, scale);
    let translation = Matrix4::from_translation(offset + relative_position.extend(0.0));
    let scale = Matrix4::from_scale(scale);
    let view = projection * translation * scale;
    view
}
