mod hexagon;
mod render;

use std::collections::HashSet;

use cgmath::{Matrix4, Ortho, Vector2, Vector3, Zero};
use glutin::{
    self,
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, StartCause, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use luminance::{context::GraphicsContext, pipeline::PipelineState, render_state::RenderState};
use luminance_glutin::{self, GlutinSurface};
use rand::Rng;
use render::{HexInterface, HexVertexSemantics};

use crate::hexagon::{create_hexagon_mesh_border, flat_hex_to_pixel, pixel_to_flat_hex};

const VS: &'static str = include_str!("hex-vs.glsl");
const FS: &'static str = include_str!("hex-fs.glsl");

fn get_projection_matrix(width: f32, height: f32) -> Matrix4<f32> {
    Matrix4::from(Ortho {
        left: 0.0,
        right: width,
        bottom: height,
        top: 0.0,
        near: -0.01,
        far: 100.0,
    })
}

fn main() {
    let window_builder = WindowBuilder::new()
        .with_title("OpenCells")
        .with_inner_size(LogicalSize::new(1600.0, 900.0));
    let (mut surface, event_loop) = GlutinSurface::new_gl33(window_builder, 1).unwrap();

    let mut program = surface
        .new_shader_program::<HexVertexSemantics, (), HexInterface>()
        .from_strings(VS, None, None, FS)
        .unwrap()
        .ignore_warnings();

    let mesh = create_hexagon_mesh_border(&mut surface);

    let mut projection = get_projection_matrix(1600.0, 900.0);
    let scale = 64.0;

    let colors = rand::random::<[[f32; 3]; 32]>();

    let offset: Vector2<f32> = Vector2::new(160.0, 160.0);

    let hexes_len = rand::thread_rng().gen_range(0..32);

    let mut positions = (0..hexes_len)
        .map(|_| {
            let x = rand::thread_rng().gen_range(-1..8);
            let y = rand::thread_rng().gen_range(-1..8);
            Vector2::new(x, y)
        })
        .collect::<HashSet<_>>();

    let mut active_hex = Vector2::zero();

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
                    active_hex = hex;
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                    ..
                } => {
                    if positions.contains(&active_hex) {
                        positions.remove(&active_hex);
                    } else {
                        positions.insert(active_hex);
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                surface.ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let back_buffer = surface.back_buffer().unwrap();

                let render = surface
                    .new_pipeline_gate()
                    .pipeline(
                        &back_buffer,
                        &PipelineState::default().set_clear_color([0.1, 0.1, 0.1, 1.0]),
                        |_, mut shd_gate| {
                            shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
                                for (i, position) in positions.iter().enumerate() {
                                    let offset = Vector3::new(offset.x, offset.y, 0.0);
                                    let relative_position = flat_hex_to_pixel(*position, scale);

                                    let translation = Matrix4::from_translation(
                                        offset + relative_position.extend(0.0),
                                    );

                                    let scale = Matrix4::from_scale(scale);

                                    let view = projection * translation * scale;
                                    iface.set(&uni.view, view.into());

                                    let color = Vector3::from(colors[i % colors.len()])
                                        * (if active_hex == *position { 1.5 } else { 1.0 });

                                    iface.set(&uni.model_color, color.into());

                                    rdr_gate
                                        .render(&RenderState::default(), |mut tess_gate| {
                                            tess_gate.render(&mesh)
                                        })
                                        .map_err(|_: &'static str| ())
                                        .unwrap();
                                }

                                Ok(())
                            })
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
