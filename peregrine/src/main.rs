use tethys::prelude::*;
use cgmath::{Quaternion, Vector3};

struct Peregrine {
    shader: Shader,
    camera: Camera,
    object: Object,
    exit: bool,
}

impl App for Peregrine {
    fn new(graphics: &Graphics) -> Self {
        let shader = Shader::new::<TexVertex>(graphics, include_str!("../shaders/shader.wgsl"), &[
            ShaderBinding::Camera,
            ShaderBinding::Model,
        ]);
        let camera = Camera::new(&graphics, Vector3::new(0., 0., 0.), 1.57, 0., 0.1, 10., 1.);
        let model = Model::new(graphics, &shader, load_obj!("assets/parts/tank.obj"));
        let object = Object::new(model.clone(), Vector3::new(0., 0., 0.), Quaternion::new(1., 0., 0., 0.));
        Self {
            exit: false,
            shader,
            object,
            camera,
        }
    }

    fn tick(&mut self) {
        self.camera.update();
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
    
    fn render(&self, render_pass: RenderPass) {
        render_pass
            .set_camera(&self.camera)
            .set_shader(&self.shader)
            .render(&[self.object.clone()])
        ;
    }
}

fn main() {
    tethys::main::<Peregrine>();
}