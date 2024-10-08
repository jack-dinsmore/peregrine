use tethys::prelude::*;
use cgmath::{Quaternion, Vector3};
use clap::Parser;

mod dev;
mod ship;
mod ui;
mod util;

use ship::{Part, PartLayout, PartLoader, ShipInterior};
use ui::{FpsCounter, PlacementState, UiMode};



struct Peregrine {
    shader: Shader,
    camera: Camera,
    ship: ShipInterior,
    exit: bool,
    size: (u32, u32),
    ui_mode: UiMode,
    fps_counter: FpsCounter,
}

impl App for Peregrine {
    fn new(graphics: &Graphics) -> Self {
        std::env::set_var("RUST_LOG", "warn");
        env_logger::init();
        let shader = Shader::new::<TexVertex>(graphics, include_str!("shaders/shader.wgsl"), &[
            ShaderBinding::Camera,
            ShaderBinding::Model,
            ShaderBinding::Texture,
        ]);
        let camera = Camera::new(&graphics, Vector3::new(-2., 0., 0.), 1.57, 0., 0.1, 10., 1.5);
        let mut part_loader = PartLoader::new(graphics);
        let rigid_body = RigidBody {
            angvel: Quaternion::new(0., 0., 3.0, 0.0),
            ..Default::default()
        };
        let parts = vec![
            Part::Tank {length: 3}
            //, Part::FuelCell
        ];
        let layout = vec![
            PartLayout { x: 0, y: 0, z: 0, orientation: 0 },
            // PartLayout { x: 1, y: 0, z: 0, orientation: 0 },
        ];
        let ship = ShipInterior::new(&mut part_loader, parts, layout, rigid_body);
        let size = graphics.get_size();


        Self {
            exit: false,
            shader,
            ship,
            camera,
            size,
            ui_mode: UiMode::Placement(PlacementState::new(graphics, Part::Tank { length: 3 })),
            fps_counter: FpsCounter::new(),
        }
    }

    fn tick(&mut self, graphics: &Graphics, key_state: &KeyState, delta_t: f64) {
        self.ship.update(delta_t);

        match &mut self.ui_mode {
            UiMode::Placement(placement) => {
                placement.update(&self.camera, &self.ship);
            }
        };

        graphics.set_mouse_pos((self.size.0/2, self.size.1/2));
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
        // dbg!(self.fps_counter.get_fps());
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
            _ => (),
        }
    }

    fn mouse_down(&mut self, graphics: &Graphics, _mouse: &Mouse) {
        match &self.ui_mode {
            UiMode::Placement(placement) => {
                placement.place(graphics, &mut self.ship)
            },
        }
    }

    fn mouse_motion(&mut self, pos: (f64, f64)) {
        let dx = (pos.0 - self.size.0 as f64 / 2.) / 300.;
        let dy = (pos.1 - self.size.1 as f64 / 2.) / 300.;
        self.camera.phi += -dx as f32;
        self.camera.theta += dy as f32;
        self.camera.theta = self.camera.theta.clamp(0., std::f32::consts::PI);
    }
    
    fn render(&self, render_pass: RenderPass) {
        let render_pass = render_pass
            .set_camera(&self.camera)
            .set_shader(&self.shader)
            .render(&self.ship.objects())
        ;
        match &self.ui_mode {
            UiMode::Placement(placement) => {
                render_pass.render(&placement.object());
            },
        }
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