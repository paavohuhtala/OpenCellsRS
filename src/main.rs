mod hexagon;
mod input;
mod level;
mod render;

use cgmath::{Matrix4, Ortho, Vector2};
use glutin::{
    self,
    dpi::LogicalSize,
    event::{Event, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use input::{handle_input_system, HexKind, InputAction, InputState};
use legion::{Resources, Schedule, World};
use level::Level;

use luminance_glutin::{self, GlutinSurface};
use render::Renderer;

use crate::hexagon::{flat_hex_height, flat_hex_width};

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

    let level = Level::new();

    let mut world = World::default();
    let mut resources = Resources::default();
    resources.insert(level);
    resources.insert(InputState::default());

    let mut schedule = Schedule::builder()
        .add_system(handle_input_system())
        .build();

    let mut projection = get_projection_matrix(1600.0, 900.0);
    let scale = 64.0;

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
                    let mut input_state = resources.get_mut::<InputState>().unwrap();
                    input_state.mouse_position =
                        Vector2::new(position.x as f32, position.y as f32) - offset;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    let mut input_state = resources.get_mut::<InputState>().unwrap();

                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Key1) => {
                            input_state.action_queue.push(InputAction::ClearHex);
                        }
                        Some(VirtualKeyCode::Key2) => {
                            input_state
                                .action_queue
                                .push(InputAction::PlaceHex(HexKind::Empty));
                        }
                        Some(VirtualKeyCode::Key3) => {
                            input_state
                                .action_queue
                                .push(InputAction::PlaceHex(HexKind::Marked));
                        }
                        _ => {}
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                surface.ctx.window().request_redraw();
                schedule.execute(&mut world, &mut resources);
            }
            Event::RedrawRequested(_) => {
                renderer.render(&mut resources, &mut surface, scale, &projection, offset);
            }
            _ => (),
        }
    });
}
