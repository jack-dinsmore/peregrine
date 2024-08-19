pub mod graphics;
pub mod physics;
pub mod io;
pub mod util;

pub mod prelude {
    pub use crate::App;
    pub use crate::io::key::{Key, KeyState};
    pub use crate::graphics::{Graphics, RenderPass};
    pub use crate::graphics::model::{Model,  LoadedObj, LoadMaterial, LoadMesh};
    pub use crate::graphics::shader::{Shader, ShaderBinding};
    pub use crate::graphics::camera::Camera;
    pub use crate::graphics::object::Object;
    pub use crate::graphics::primitives::{Vertex, TexVertex};
    pub use crate::physics::RigidBody;
    pub use crate::physics::collisions::Collider;
    pub use crate::include_obj;
}

use std::time::Instant;

use graphics::{Graphics, RenderPass};
use io::key::KeyState;
use winit::{event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::EventLoop, window::WindowBuilder};


pub use io::key::Key;

pub trait App {
    fn new(graphics: &Graphics) -> Self;
    fn tick(&mut self, graphics: &Graphics, key_state: &KeyState, delta_t: f64);
    fn render(&self, render_pass: RenderPass);
    fn exit_check(&self) -> bool;
    
    fn key_up(&mut self, _key: Key) {}
    fn key_down(&mut self, _key: Key) {}
    fn mouse_motion(&mut self, _pos: (f64, f64)) {}
    fn close_requested(&mut self) {}
    fn resize(&mut self, _new_size: (u32, u32)) {}
}

async fn main_internal<T: App>() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    
    let mut graphics = graphics::Graphics::new(&window).await;
    let mut app = T::new(&graphics);
    let mut surface_configured = false;
    let mut key_state = KeyState::new();
    let mut time = Instant::now();

    event_loop.run(move |event, control_flow| {
        let delta_t = time.elapsed().as_micros() as f64/ 1e6;
        time = Instant::now();
        app.tick(&graphics, &key_state, delta_t);
        
        match event {
            Event::WindowEvent { ref event, window_id, } if window_id == graphics.window().id() => match event {
                WindowEvent::CloseRequested => app.close_requested(),
                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                            state: ElementState::Pressed,
                            physical_key,
                            ..
                        },
                    ..
                } => match Key::from_physical(*physical_key) {
                    Some(key) => {
                        key_state.set_down(key);
                        app.key_down(key)
                    },
                    None => (),
                },
                WindowEvent::CursorMoved { position, .. } => {
                    app.mouse_motion((position.x, position.y));
                }
                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                            state: ElementState::Released,
                            physical_key,
                            ..
                        },
                    ..
                } => match Key::from_physical(*physical_key) {
                    Some(key) => {
                        key_state.set_up(key);
                        app.key_up(key)
                    },
                    None => (),
                }
                WindowEvent::Resized(physical_size) => {
                    surface_configured = true;
                    app.resize((physical_size.width, physical_size.height));
                    graphics.resize(*physical_size);
                },
                WindowEvent::RedrawRequested => {
                    graphics.window().request_redraw();

                    if !surface_configured {
                        return;
                    }

                    match graphics.render(&app) {
                        Ok(_) => (),
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => graphics.resize(graphics.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                graphics.window().request_redraw();
            }
            _ => {}
        }

        if app.exit_check() {
            control_flow.exit()
        }
    }).unwrap()
}

pub fn main<T: App>() {
    pollster::block_on(main_internal::<T>());
}