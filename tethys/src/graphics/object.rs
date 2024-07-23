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

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl ModelUniform {
    fn new(matrix: Matrix4<f32>) -> Self {
        Self {
            view_proj: matrix.into()
        }
    }
}


impl Object {
    pub fn new(graphics: &Graphics, model: Model, position: Vector3<f64>, orientation: Quaternion<f64>) -> Self {
        let uniform = ModelUniform::new(Matrix4::identity());

        let model_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
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
            label: Some("camera_bind_group"),
        });
        
        Self {
            model,
            position,
            orientation,
            model_buffer,
            bind_group,
        }
    }

    pub(crate) fn get_world(&self, camera: &Camera) -> Matrix4<f32> {
        let difference = self.position - camera.position;
        let quat = Quaternion::new(
            self.orientation.s as f32,
            self.orientation.v.x as f32,
            self.orientation.v.y as f32,
            self.orientation.v.z as f32,
        );
        Matrix4::from(quat) *
        Matrix4::from_translation(
            Vector3::new(
                difference.x as f32,
                difference.y as f32,
                difference.z as f32,
            )
        )
    }

    pub(crate) fn update(&self, graphics: &Graphics, camera: &Camera) {
        let uniform = ModelUniform::new(self.get_world(camera));
        graphics.queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&[uniform]))
    }
}