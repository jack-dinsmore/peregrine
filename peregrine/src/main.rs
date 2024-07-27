use tethys::prelude::*;
use cgmath::{Quaternion, Vector3};
use clap::Parser;

mod util;

struct Peregrine {
    shader: Shader,
    camera: Camera,
    object: Object,
    exit: bool,
    size: (u32, u32)
}

impl App for Peregrine {
    fn new(graphics: &Graphics) -> Self {
        let shader = Shader::new::<TexVertex>(graphics, include_str!("../shaders/shader.wgsl"), &[
            ShaderBinding::Camera,
            ShaderBinding::Model,
            ShaderBinding::Texture,
        ]);
        let camera = Camera::new(&graphics, Vector3::new(0., 0., 0.), 1.57, 0., 0.1, 10., 1.5);
        let model = Model::new(graphics, load_obj!("assets/parts/tank.obj"));
        let object = Object::new(graphics, model.clone(), Vector3::new(0., 0., 0.), Quaternion::new(1., 0., 0., 0.));
        let size = graphics.get_size();
        Self {
            exit: false,
            shader,
            object,
            camera,
            size,
        }
    }

    fn tick(&mut self, graphics: &Graphics, key_state: &KeyState) {
        graphics.set_mouse_pos((self.size.0/2, self.size.1/2));
        if key_state.is_down(Key::Char('w')) {
            self.camera.position += 0.01 * self.camera.get_forward();
        }

        if key_state.is_down(Key::Char('s')) {
            self.camera.position -= 0.01 * self.camera.get_forward();
        }

        if key_state.is_down(Key::Char('a')) {
            self.camera.position += 0.01 * self.camera.get_left();
        }

        if key_state.is_down(Key::Char('d')) {
            self.camera.position -= 0.01 * self.camera.get_left();
        }

        if key_state.is_down(Key::Char('q')) {
            self.camera.position += 0.01 * self.camera.get_up();
        }

        if key_state.is_down(Key::Char('e')) {
            self.camera.position -= 0.01 * self.camera.get_up();
        }
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

    fn mouse_motion(&mut self, pos: (f64, f64)) {
        let dx = (pos.0 - self.size.0 as f64 / 2.) / 300.;
        let dy = (pos.1 - self.size.1 as f64 / 2.) / 300.;
        self.camera.phi += -dx as f32;
        self.camera.theta += dy as f32;
        self.camera.theta = self.camera.theta.clamp(0., std::f32::consts::PI);
    }
    
    fn render(&self, render_pass: RenderPass) {
        render_pass
            .set_camera(&self.camera)
            .set_shader(&self.shader)
            .render(&[&self.object])
        ;
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
        util::normal::save_bumpmap();
    } else {
        tethys::main::<Peregrine>();
    }
}