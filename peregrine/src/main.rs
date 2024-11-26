use log::info;
use tethys::prelude::*;
use cgmath::{Quaternion, Vector3};
use clap::Parser;

mod dev;
mod ship;
mod ui;
mod util;

use ship::{Part, PartData, PartLayout, ShipInterior};
use ui::{FpsCounter, PlacePartState, UiMode};

struct Peregrine<'a> {
    shader_3d: Shader,
    shader_2d: Shader,
    shader_placement: Shader,
    camera: Camera,
    graphics: Graphics<'a>,
    part_data: PartData,
    exit: bool,
    ui_mode: UiMode,
    fps_counter: FpsCounter,
    
    ship: Option<ShipInterior>,
}

impl<'a> App for Peregrine<'a> {
    fn new(graphics: Graphics) -> impl App {
        std::env::set_var("RUST_LOG", "warn");
        env_logger::init();
        let shader_3d = Shader::new::<TexVertex>(&graphics, include_str!("shaders/shader_3d.wgsl"), &[
            ShaderBinding::Camera,
            ShaderBinding::Model,
            ShaderBinding::NoisyTexture,
        ]);
        let shader_placement = Shader::new::<TexVertex>(&graphics, include_str!("shaders/shader_placement.wgsl"), &[
            ShaderBinding::Camera,
            ShaderBinding::Model,
        ]);
        let shader_2d = Shader::new::<ScreenVertex>(&graphics, include_str!("shaders/shader_2d.wgsl"), &[
            ShaderBinding::Texture,
        ]);
        let camera = Camera::new(&graphics, Vector3::new(-2., 0., 0.), 1.57, 0., 0.1, 10., 1.5);
        let part_data = PartData::new();
    
        let ui_mode = UiMode::Flying;
    
        Peregrine {
            exit: false,
            shader_3d,
            shader_2d,
            ship: None,
            camera,
            ui_mode,
            fps_counter: FpsCounter::new(),
            graphics,
            part_data,
            shader_placement,
        }
    }

    fn initialize(&mut self) {
        let parts = vec![
            Part::Tank {length: 3},
            Part::Box { length: 1, width: 1, height: 1},
            // , Part::FuelCell
        ];
        let layout = vec![
            PartLayout { x: 0, y: 0, z: 0, orientation: 0 },
            PartLayout { x: 1, y: 0, z: 0, orientation: 0 },
        ];
        let rigid_body = RigidBody {
            angvel: Quaternion::new(0., 0., 0., 0.0),
            // orientation: Quaternion::new(0., 0., 0., 1.),
            ..Default::default()
        };

        let part_loader = self.part_data.get_loader(&self.graphics);
        let ship = ShipInterior::new(part_loader.clone(), parts, layout, rigid_body);
        self.ui_mode = UiMode::PlacePart(PlacePartState::new(part_loader, Part::Tank { length: 3 }));
        self.ship = Some(ship);
    }

    fn tick(&mut self, key_state: &KeyState, delta_t: f64) {
        info!("FPS: {}", self.fps_counter.get_fps());
        if let Some(ship) = &mut self.ship {
            ship.update(delta_t);
        }

        match &mut self.ui_mode {
            UiMode::PlacePart(place_part_state) => {
                if let Some(ship) = &mut self.ship {
                    place_part_state.update(&self.camera, ship);
                    ship.initialize_placement(self.part_data.get_loader(&self.graphics)); // TODO move to whenever this UI element is created
                }
            },
            UiMode::PlacePanel(place_panel_state) => {
                if let Some(ship) = &mut self.ship {
                    place_panel_state.update(&self.camera, ship);
                    ship.initialize_placement(self.part_data.get_loader(&self.graphics)); // TODO move to whenever this UI element is created
                }
            },
            UiMode::Flying => (),
        };

        self.graphics.set_mouse_pos((self.graphics.size.0/2, self.graphics.size.1/2));
        if key_state.is_down(Key::Char('w')) {
            self.camera.position += 2. * delta_t * self.camera.get_forward();
        }

        if key_state.is_down(Key::Char('s')) {
            self.camera.position -= 2. * delta_t * self.camera.get_forward();
        }

        if key_state.is_down(Key::Char('a')) {
            self.camera.position += 2. * delta_t * self.camera.get_left();
        }

        if key_state.is_down(Key::Char('d')) {
            self.camera.position -= 2. * delta_t * self.camera.get_left();
        }

        if key_state.is_down(Key::Char('q')) {
            self.camera.position += 2. * delta_t * self.camera.get_up();
        }

        if key_state.is_down(Key::Char('e')) {
            self.camera.position -= 2. * delta_t * self.camera.get_up();
        }

        self.fps_counter.update();
    }

    fn exit_check(&self) -> bool {
        self.exit
    }

    fn close_requested(&mut self) {
        self.exit = true;
    }

    fn key_down(&mut self, key: Key) {
        match key {
            Key::Escape => self.exit = true,
            Key::Char('0') => self.ui_mode = UiMode::Flying,
            // Key::Char('1') => self.ui_mode = UiMode::Placement(PlacePartState::new(&self.part_loader, Part::Box { length: 1, width: 1, height: 1 })),
            // Key::Char('2') => self.ui_mode = UiMode::Placement(PlacePartState::new(&self.part_loader, Part::Tank { length: 3 })),
            _ => (),
        }
    }

    fn mouse_down(&mut self, _mouse: &Mouse) {
        match &self.ui_mode {
            UiMode::PlacePart(place_part_state) => {
                let part_loader = self.part_data.get_loader(&self.graphics);
                if let Some(ship) = &mut self.ship {
                    place_part_state.place(part_loader, ship)
                }
            },
            UiMode::PlacePanel(place_panel_state) => {
                let part_loader = self.part_data.get_loader(&self.graphics);
                if let Some(ship) = &mut self.ship {
                    place_panel_state.place(part_loader, ship)
                }
            },
            UiMode::Flying => (),
        }
    }

    fn mouse_motion(&mut self, pos: (f64, f64)) {
        let dx = (pos.0 - self.graphics.size.0 as f64 / 2.) / 300.;
        let dy = (pos.1 - self.graphics.size.1 as f64 / 2.) / 300.;
        self.camera.phi += -dx as f32;
        self.camera.theta += dy as f32;
        self.camera.theta = self.camera.theta.clamp(0., std::f32::consts::PI);
    }
    
    fn render<'c, 'b: 'c> (&'b self, mut render_pass: RenderPass<'c>) {
        // 3D
        render_pass.set_camera(&self.camera);
        render_pass.set_shader(&self.shader_3d);
        if let Some(ship) = &self.ship {
            render_pass.render(ship.objects());
        }

        self.ui_mode.render(&mut render_pass);
        if self.ui_mode.is_placement() {
            render_pass.set_shader(&self.shader_placement);
            if let Some(ship) = &self.ship {
                render_pass.render(ship.get_placement_objects());
            }
        }

        // 2D
        render_pass.set_shader(&self.shader_2d);
    }
    
    fn get_graphics(&self) -> &Graphics {
        &self.graphics
    }

    fn resize(&mut self, new_size: (u32, u32)) {
        self.graphics.resize(new_size);
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    normal: bool,
}

fn main() {
    let args = Args::parse();
    if args.normal {
        dev::normal::fourier_save_bumpmap();
    } else {
        tethys::main::<Peregrine>();
    }
}