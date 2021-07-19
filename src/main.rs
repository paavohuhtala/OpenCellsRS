mod hexagon;
mod level;
mod render;

use std::collections::HashMap;

use cgmath::{Matrix4, Ortho, Vector2, Vector3, Zero};
use glutin::{
    self,
    dpi::LogicalSize,
    event::{Event, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use level::Hex;
use luminance_front::{
    context::GraphicsContext, pipeline::PipelineState, render_state::RenderState,
};
use luminance_glutin::{self, GlutinSurface};
use luminance_glyph::{
    ab_glyph::FontArc, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text, VerticalAlign,
};
use render::{HexInterface, HexVertexSemantics};

use crate::hexagon::{
    create_hexagon_mesh_border, flat_hex_height, flat_hex_to_pixel, flat_hex_width,
    pixel_to_flat_hex,
};

const VS: &'static str = include_str!("hex-vs.glsl");
const FS: &'static str = include_str!("hex-fs.glsl");

fn get_projection_matrix(width: f32, height: f32) -> Matrix4<f32> {
    Matrix4::from(Ortho {
        left: 0.0,
        right: width,
        bottom: height,
        top: 0.0,
        near: -2.0,
        far: 100.0,
    })
}

fn main() {
    let window_builder = WindowBuilder::new()
        .with_title("OpenCells")
        .with_inner_size(LogicalSize::new(1600.0, 900.0));
    let (mut surface, event_loop) = GlutinSurface::new_gl33(window_builder, 8).unwrap();

    let mut program = surface
        .new_shader_program::<HexVertexSemantics, (), HexInterface>()
        .from_strings(VS, None, None, FS)
        .unwrap()
        .ignore_warnings();

    let mesh = create_hexagon_mesh_border(&mut surface);

    let mut projection = get_projection_matrix(1600.0, 900.0);
    let scale = 64.0;

    let offset: Vector2<f32> =
        Vector2::new(flat_hex_width(scale) * 2.0, flat_hex_height(scale) * 1.5);

    let mut glyph_brush = GlyphBrushBuilder::using_font(
        FontArc::try_from_slice(include_bytes!("../assets/fonts/Aileron-Regular.otf")).unwrap(),
    )
    .build(&mut surface);

    let mut hexes = HashMap::new();
    let mut hex_under_cursor = Vector2::zero();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(StartCause::Init) => *control_flow = ControlFlow::Wait,
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    surface.ctx.resize(physical_size);
                    projection = get_projection_matrix(
                        physical_size.width as f32,
                        physical_size.height as f32,
                    );
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => {
                    let relative_pos = Vector2::new(position.x as f32, position.y as f32) - offset;
                    let hex = pixel_to_flat_hex(relative_pos, scale);
                    hex_under_cursor = hex;
                }
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Key1) => {
                        hexes.remove(&hex_under_cursor);
                    }
                    Some(VirtualKeyCode::Key2) => {
                        hexes.insert(
                            hex_under_cursor,
                            Hex::Empty {
                                show_neighbor_count: true,
                            },
                        );
                    }
                    Some(VirtualKeyCode::Key3) => {
                        hexes.insert(hex_under_cursor, Hex::Marked { show_around: false });
                    }
                    _ => {}
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                surface.ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let back_buffer = surface.back_buffer().unwrap();

                let [viewport_width, viewport_height] = surface.size();

                for (pos, hex) in &hexes {
                    match hex {
                        Hex::Empty {
                            show_neighbor_count: true,
                        } => {
                            glyph_brush.queue(
                                Section::default()
                                    .add_text(
                                        Text::new("1")
                                            .with_color([1.0, 1.0, 1.0, 1.0])
                                            .with_scale(32.0)
                                            .with_z(-1.0),
                                    )
                                    .with_layout(
                                        Layout::default_single_line()
                                            .h_align(HorizontalAlign::Center)
                                            .v_align(VerticalAlign::Center),
                                    )
                                    .with_screen_position(
                                        flat_hex_to_pixel(pos.clone(), scale) + offset,
                                    ),
                            );
                        }
                        _ => {}
                    }
                }

                glyph_brush.process_queued(&mut surface);

                let render = surface
                    .new_pipeline_gate()
                    .pipeline(
                        &back_buffer,
                        &PipelineState::default().set_clear_color([0.1, 0.1, 0.1, 1.0]),
                        |mut pipeline, mut shd_gate| {
                            let text_transform = projection;
                            let text_transform_16: &[f32; 16] = text_transform.as_ref();

                            shd_gate
                                .shade(&mut program, |mut iface, uni, mut rdr_gate| {
                                    rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                                        for (position, hex) in &hexes {
                                            let offset = Vector3::new(offset.x, offset.y, 0.0);
                                            let relative_position =
                                                flat_hex_to_pixel(*position, scale);

                                            let translation = Matrix4::from_translation(
                                                offset + relative_position.extend(0.0),
                                            );

                                            let scale = Matrix4::from_scale(scale);

                                            let view = projection * translation * scale;
                                            iface.set(&uni.view, view.into());

                                            let color = hex.get_color(true)
                                                * (if hex_under_cursor == *position {
                                                    1.5
                                                } else {
                                                    1.0
                                                });

                                            iface.set(&uni.model_color, color.into());

                                            tess_gate
                                                .render(&mesh)
                                                .map_err(|_e: &'static str| ())
                                                .unwrap();
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
            _ => (),
        }
    });
}
