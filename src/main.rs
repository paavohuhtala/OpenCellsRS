mod game;
mod hexagon;
mod input;
mod level;
mod render;

use cgmath::Vector2;
use glutin::{
    self,
    dpi::LogicalSize,
    event::{Event, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};
use input::{HexKind, InputAction, InputState};

use game::{update, GameState};
use luminance_glutin::{self, GlutinSurface};
use render::Renderer;

fn handle_window_event(
    event: WindowEvent,
    input_state: &mut InputState,
    surface: &mut GlutinSurface,
    renderer: &mut Renderer,
) -> Option<ControlFlow> {
    match event {
        WindowEvent::Resized(physical_size) => {
            surface.ctx.resize(physical_size);
            renderer.update_resolution(physical_size.width, physical_size.height);
            None
        }
        WindowEvent::CloseRequested => Some(ControlFlow::Exit),
        WindowEvent::CursorMoved { position, .. } => {
            input_state.absolute_mouse_position =
                Vector2::new(position.x as f32, position.y as f32);
            None
        }
        WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
            Some(VirtualKeyCode::Key1) => {
                input_state.action_queue.push(InputAction::ClearHex);
                None
            }
            Some(VirtualKeyCode::Key2) => {
                input_state
                    .action_queue
                    .push(InputAction::PlaceHex(HexKind::Empty));
                None
            }
            Some(VirtualKeyCode::Key3) => {
                input_state
                    .action_queue
                    .push(InputAction::PlaceHex(HexKind::Marked));
                None
            }
            Some(VirtualKeyCode::F2) => {
                input_state.action_queue.push(InputAction::RingDebug);
                None
            }
            _ => None,
        },
        _ => None,
    }
}

fn main() {
    let window_builder = WindowBuilder::new()
        .with_title("OpenCells")
        .with_inner_size(LogicalSize::new(1600.0, 900.0));
    let (mut surface, event_loop) = GlutinSurface::new_gl33(window_builder, 8).unwrap();

    let mut renderer = Renderer::new(&mut surface);

    let mut input_state = InputState::default();
    let mut game_state = GameState::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(StartCause::Init) => *control_flow = ControlFlow::Wait,
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => {
                if let Some(cf) =
                    handle_window_event(event, &mut input_state, &mut surface, &mut renderer)
                {
                    *control_flow = cf;
                }
            }
            Event::MainEventsCleared => {
                update(&mut game_state, &mut input_state);
                surface.ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                renderer.render(&game_state, &mut surface);
            }
            _ => (),
        }
    });
}
