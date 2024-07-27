use tethys::prelude::*;
use cgmath::{Quaternion, Vector3};

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
            self.camera.position.x += 0.01;
        }

        if key_state.is_down(Key::Char('s')) {
            self.camera.position.x -= 0.01;
        }

        if key_state.is_down(Key::Char('a')) {
            self.camera.position.y += 0.01;
        }

        if key_state.is_down(Key::Char('d')) {
            self.camera.position.y -= 0.01;
        }

        if key_state.is_down(Key::Char('q')) {
            self.camera.position.z += 0.01;
        }

        if key_state.is_down(Key::Char('e')) {
            self.camera.position.z -= 0.01;
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
            Key::Char(_) => (),
        }
    }

    fn mouse_motion(&mut self, pos: (f64, f64)) {
        let dx = (pos.0 - self.size.0 as f64 / 2.) / 300.;
        let dy = (pos.1 - self.size.1 as f64 / 2.) / 300.;
        self.camera.phi += -dx as f32;
        self.camera.theta += dy as f32;
    }
    
    fn render(&self, render_pass: RenderPass) {
        render_pass
            .set_camera(&self.camera)
            .set_shader(&self.shader)
            .render(&[&self.object])
        ;
    }
}

fn main() {
    tethys::main::<Peregrine>();
}