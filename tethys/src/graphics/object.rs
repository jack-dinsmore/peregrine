use wgpu::util::DeviceExt;

use crate::prelude::Uniform;

use super::{model::Model, shader::ShaderBinding, Graphics};

pub struct Object {
    pub(crate) model: Model,
    pub(crate) object_buffer: wgpu::Buffer,
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

impl Object {
    pub fn new<U: Uniform>(graphics: &Graphics, model: Model, uniform: U) -> Self {
        let object_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Object Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ShaderBinding::Object.get_bind_group_layout(graphics),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: object_buffer.as_entire_binding(),
                }
            ],
            label: Some("object_bind_group"),
        });
        
        Self {
            model,
            object_buffer,
            bind_group,
        }
    }

    pub fn zeroed<U: Uniform>(graphics: &Graphics, model: Model) -> Self {
        let object_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Object Buffer"),
                contents: bytemuck::cast_slice(&[U::zeroed()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ShaderBinding::Object.get_bind_group_layout(graphics),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: object_buffer.as_entire_binding(),
                }
            ],
            label: Some("object_bind_group"),
        });
        
        Self {
            model,
            object_buffer,
            bind_group,
        }
    }

    // fn get_uniform(&self, camera: &Camera) -> ModelUniform {
    //     let difference = self.position - camera.position;
    //     let rot = Matrix4::from(Quaternion::new(
    //         self.orientation.s as f32,
    //         self.orientation.v.x as f32,
    //         self.orientation.v.y as f32,
    //         self.orientation.v.z as f32,
    //     ));
    //     let world = Matrix4::from_translation(
    //         Vector3::new(
    //             difference.x as f32,
    //             difference.y as f32,
    //             difference.z as f32,
    //         )
    //     ) * rot;
    //     ModelUniform {
    //         rot: rot.into(),
    //         world: world.into(),
    //     }d
    // }

    pub fn update<U: Uniform>(&self, graphics: &Graphics, uniform: U) {
        graphics.queue.write_buffer(&self.object_buffer, 0, bytemuck::cast_slice(&[uniform]))
    }
}