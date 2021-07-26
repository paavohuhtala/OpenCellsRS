mod hexagon;
mod input;
mod level;
mod render;
mod update;

use cgmath::{Matrix4, Ortho, Vector2};
use glutin::{
    self,
    dpi::LogicalSize,
    event::{Event, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use input::{HexKind, InputAction, InputState};

use luminance_glutin::{self, GlutinSurface};
use render::Renderer;
use update::{update_state, GameState};

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

    let mut projection = get_projection_matrix(1600.0, 900.0);
    let scale = 64.0;

    let mut renderer = Renderer::new(&mut surface);

    let offset: Vector2<f32> =
        Vector2::new(flat_hex_width(scale) * 2.0, flat_hex_height(scale) * 1.5);

    let mut input_state = InputState::default();
    let mut game_state = GameState::new();

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
                    input_state.mouse_position =
                        Vector2::new(position.x as f32, position.y as f32) - offset;
                }
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
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
                    Some(VirtualKeyCode::F2) => {
                        input_state.action_queue.push(InputAction::RingDebug);
                    }
                    _ => {}
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                update_state(&mut game_state, &mut input_state);
                surface.ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                renderer.render(&game_state, &mut surface, scale, &projection, offset);
            }
            _ => (),
        }
    });
}
