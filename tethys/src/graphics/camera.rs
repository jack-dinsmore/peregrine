use cgmath::{Matrix4, Point3, Rad, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;

use super::{shader::ShaderBinding, Graphics};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new(matrix: Matrix4<f32>) -> Self {
        Self {
            view_proj: matrix.into()
        }
    }
}

pub struct Camera {
    pub position: Vector3<f64>,
    pub theta: f32,
    pub phi: f32,
    pub fovy: f32,
    aspect: f32,
    znear: f32,
    zfar: f32,
    camera_buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(graphics: &Graphics, position: Vector3<f64>, theta: f32, phi: f32, znear: f32, zfar: f32, fovy: f32) -> Self {
        let aspect = graphics.size.height as f32 / graphics.size.width as f32;
        let uniform = CameraUniform::new(Matrix4::identity());

        let camera_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ShaderBinding::Camera.get_bind_group_layout(graphics),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        Self {
            position,
            theta,
            phi,
            znear,
            zfar,
            fovy: fovy,
            aspect,
            camera_buffer,
            bind_group,
        }
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            Point3::new(0., 0., 0.),
            Vector3::new(self.phi.cos() * self.theta.sin(), self.phi.sin() * self.theta.sin(), self.theta.cos()),
            Vector3::new(0., 0., 1.),
        )
    }

    pub fn get_proj(&self) -> Matrix4<f32> {
        cgmath::perspective(Rad(self.fovy), self.aspect, self.znear, self.zfar)
    }

    pub fn get_view_proj(&self) -> Matrix4<f32> {
        self.get_proj() * self.get_view()
    }

    pub fn update(&self, graphics: &Graphics) {
        let uniform = CameraUniform::new(self.get_view_proj());
        graphics.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]))
    }
}