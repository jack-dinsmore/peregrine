pub mod graphics;
pub mod physics;
pub mod io;
pub mod util;

pub mod prelude {
    pub use crate::App;
    pub use crate::io::key::{Key, KeyState};
    pub use crate::io::mouse::Mouse;
    pub use crate::graphics::{Graphics, RenderPass};
    pub use crate::graphics::model::{Model, ModelContainer, ModelLoader, Material, MaterialContainer, MaterialLoader, LoadModel, LoadMaterial};
    pub use crate::graphics::shader::{Shader, ShaderBuilder, ShaderBinding};
    pub use crate::graphics::camera::Camera;
    pub use crate::graphics::object::{Object, ObjectHandle};
    pub use crate::graphics::primitives::*;
    pub use crate::physics::RigidBody;
    pub use crate::physics::collisions::{Collider, BoxCollider, LineCollider, GridCollider};
    pub use crate::include_model;
    pub use crate::include_material;
}

use std::time::Instant;

use graphics::{Graphics, RenderPass};
use io::{key::KeyState, mouse::Mouse};
use winit::{dpi::LogicalSize, event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::EventLoop, window::WindowBuilder};


pub use io::key::Key;

pub trait App {
    fn new<'a>(graphics: Graphics<'a>) -> impl App;
    fn tick(&mut self, key_state: &KeyState, delta_t: f64);
    fn render<'c, 'b: 'c>(&'b self, render_pass: RenderPass<'c>);
    fn exit_check(&self) -> bool;
    fn get_graphics(&self) -> &Graphics;
    fn resize(&mut self, new_size: (u32, u32));
    
    fn initialize(&mut self) {}
    fn key_up(&mut self, _key: Key) {}
    fn key_down(&mut self, _key: Key) {}
    fn mouse_down(&mut self, _mouse: &Mouse) {}
    fn mouse_motion(&mut self, _pos: (f64, f64)) {}
    fn close_requested(&mut self) {}
}

async fn main_internal<T: App>() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize{ width: 1280, height: 960})
        .build(&event_loop).unwrap();
    let graphics = graphics::Graphics::new(&window).await;
    let mut app = T::new(graphics);
    let mut surface_configured = false;
    let mut key_state = KeyState::new();
    let mut mouse = Mouse::new();
    let mut time = Instant::now();
    app.initialize();

    event_loop.run(move |event, control_flow| {
        let delta_t = time.elapsed().as_micros() as f64/ 1e6;
        time = Instant::now();
        app.tick(&key_state, delta_t);
        let window = app.get_graphics().window();
        
        match event {
            Event::WindowEvent { ref event, window_id, } if window_id == window.id() => match event {
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
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button,
                    ..
                } => {
                    mouse.update(*button);
                    app.mouse_down(&mouse)
                },
                WindowEvent::Resized(physical_size) => {
                    surface_configured = true;
                    app.resize((physical_size.width, physical_size.height));
                },
                WindowEvent::RedrawRequested => {
                    window.request_redraw();

                    if !surface_configured {
                        return;
                    }

                    match app.get_graphics().render(|render_pass| app.render(render_pass)) {
                        Ok(_) => (),
                        // Reconfigure the surface if lost
                        // Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => app.get_graphics().resize(),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
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