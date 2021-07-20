mod hexagon;
mod level;
mod render;

use std::collections::HashMap;

use cgmath::{Matrix4, Ortho, Vector2, Zero};
use glutin::{
    self,
    dpi::LogicalSize,
    event::{Event, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use level::Hex;

use luminance_glutin::{self, GlutinSurface};
use luminance_glyph::{HorizontalAlign, Layout, Section, Text, VerticalAlign};
use render::Renderer;

use crate::hexagon::{flat_hex_height, flat_hex_to_pixel, flat_hex_width, pixel_to_flat_hex};

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

    let mut projection = get_projection_matrix(1600.0, 900.0);
    let scale = 64.0;

    let mut hexes = HashMap::new();
    let mut hex_under_cursor = Vector2::zero();

    let mut renderer = Renderer::new(&mut surface);

    let offset: Vector2<f32> =
        Vector2::new(flat_hex_width(scale) * 2.0, flat_hex_height(scale) * 1.5);

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
                for (pos, hex) in &hexes {
                    match hex {
                        Hex::Empty {
                            show_neighbor_count: true,
                        } => {
                            renderer.queue_text(
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

                renderer.render(
                    &mut surface,
                    scale,
                    &hexes,
                    &projection,
                    hex_under_cursor,
                    offset,
                );
            }
            _ => (),
        }
    });
}
