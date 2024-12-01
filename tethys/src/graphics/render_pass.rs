use super::Graphics;
use super::camera::Camera;
use super::model::Material;
use super::object::ObjectHandle;
use super::shader::Shader;

pub struct RenderPass<'a> {
    graphics: &'a Graphics<'a>,
    render_pass: &'a mut Option<wgpu::RenderPass<'a>>,
    clear_depth_buffer: &'a dyn Fn(),
    camera: Option<&'a Camera>,
    objects: Vec<ObjectHandle<'a>>,
    global_material: bool,
}

impl<'a> RenderPass<'a> {
    pub(crate) fn new<'b: 'a>(graphics: &'a Graphics, render_pass: &'a mut Option<wgpu::RenderPass<'a>>, clear_depth_buffer: &'a dyn Fn()) -> Self {
        Self {
            graphics,
            render_pass,
            camera: None,
            objects: Vec::new(),
            global_material: false,
            clear_depth_buffer,
        }
    }

    fn render_models(&mut self) {
        for object in self.objects.drain(0..self.objects.len()) {
            let object = object.as_ref();
            let render_pass = self.render_pass.as_mut().unwrap();
            render_pass.set_bind_group(1, &object.bind_group, &[]);
            let model_data = &object.model.inner();
            for mesh in &model_data.0 {//TODO rearrange orderm instances
                if !self.global_material && model_data.1.len() > mesh.material_index {
                    render_pass.set_bind_group(2, &model_data.1[mesh.material_index].inner(), &[]);
                }
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }
    }

    pub fn set_camera(&mut self, camera: &'a Camera) {
        camera.update(&self.graphics);
        self.camera = Some(camera);
    }

    pub fn set_shader(&mut self, shader: &'a Shader) {
        self.render_models();
        self.global_material = false;
        let render_pass = self.render_pass.as_mut().unwrap();
        render_pass.set_pipeline(&shader.render_pipeline);
        render_pass.set_bind_group(0, &self.camera.expect("You must set a camera").bind_group, &[]);
    }
    
    pub fn set_global_material(&mut self, material: &Material) {
        self.global_material = true;
        let render_pass = self.render_pass.as_mut().unwrap();
        render_pass.set_bind_group(2, &material.inner(), &[]);
    }

    pub fn render(&mut self, objects: Vec<ObjectHandle<'a>>) {
        for object in objects {
            let index = match self.objects.binary_search_by(|probe| probe.as_ref().model.identifier().cmp(&object.as_ref().model.identifier())) {
                Ok(i) => i,
                Err(i) => i,
            };
            self.objects.insert(index, object);
        }
    }
    
    pub fn clear_depth(&mut self) {
        self.render_models();
        (self.clear_depth_buffer)()
    }
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        self.render_models();
    }
}