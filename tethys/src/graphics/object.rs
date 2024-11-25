use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;

use super::{camera::Camera, model::Model, shader::ShaderBinding, Graphics};

pub struct Object {
    pub(crate) model: Model,
    pub position: Vector3<f64>,
    pub orientation: Quaternion<f64>,
    pub(crate) model_buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
}

pub enum ObjectHandle<'a> {
    Ref(&'a Object),
    Own(Object),
}
impl<'a> ObjectHandle<'a> {
    pub fn as_ref(&'a self) -> &'a Object {
        match self {
            ObjectHandle::Ref(object) => object,
            ObjectHandle::Own(object) => &object,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    world: [[f32; 4]; 4],
    rot: [[f32; 4]; 4],
}

impl Object {
    pub fn new(graphics: &Graphics, model: Model, position: Vector3<f64>, orientation: Quaternion<f64>) -> Self {
        let uniform = ModelUniform {
            world: Matrix4::identity().into(),
            rot: Matrix4::identity().into(),
        };

        let model_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Model Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ShaderBinding::Model.get_bind_group_layout(graphics),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_buffer.as_entire_binding(),
                }
            ],
            label: Some("object_bind_group"),
        });
        
        Self {
            model,
            position,
            orientation,
            model_buffer,
            bind_group,
        }
    }

    fn get_uniform(&self, camera: &Camera) -> ModelUniform {
        let difference = self.position - camera.position;
        let rot = Matrix4::from(Quaternion::new(
            self.orientation.s as f32,
            self.orientation.v.x as f32,
            self.orientation.v.y as f32,
            self.orientation.v.z as f32,
        ));
        let world = Matrix4::from_translation(
            Vector3::new(
                difference.x as f32,
                difference.y as f32,
                difference.z as f32,
            )
        ) * rot;
        ModelUniform {
            rot: rot.into(),
            world: world.into(),
        }
    }

    pub(crate) fn update(&self, graphics: &Graphics, camera: &Camera) {
        let uniform = self.get_uniform(camera);
        graphics.queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&[uniform]))
    }
}