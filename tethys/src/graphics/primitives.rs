
use cgmath::{InnerSpace, Matrix4, Quaternion, Rad, Vector3};

use super::camera::Camera;

// =================================================
// UNIFORMS
// =================================================

pub trait Uniform: bytemuck::Zeroable + bytemuck::NoUninit + bytemuck::Pod + Clone + Copy + 'static {}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectUniform {
    world: [[f32; 4]; 4],
    rot: [[f32; 4]; 4],
}
impl ObjectUniform {
    pub fn new(camera: &Camera, pos: Vector3<f64>, orientation: Quaternion<f64>) -> Self {
        let rot = Matrix4::from(Quaternion::new(
            orientation.s as f32,
            orientation.v.x as f32,
            orientation.v.y as f32,
            orientation.v.z as f32,
        ));
        let world = Matrix4::from_translation((pos - camera.position).cast::<f32>().unwrap()) * rot;
        Self {
            rot: rot.into(),
            world: world.into(),
        }
    }
}
impl Uniform for ObjectUniform {}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SolidUniform {
    world: [[f32; 4]; 4],
    color: [f32; 4],
}
impl SolidUniform {
    pub fn line(camera: &Camera, start: Vector3<f64>, stop: Vector3<f64>, color: Vector3<f32>) -> Self {
        let delta = (stop - start).cast::<f32>().unwrap();
        let delta_mag = delta.magnitude();
        let psi = ((start + stop) / 2. - camera.position).cast::<f32>().unwrap().normalize().z.acos();
        let theta = (-delta.z / delta_mag).acos();
        let phi = f32::atan2(-delta.y, -delta.x);
        let world = Matrix4::from_translation(start.cast::<f32>().unwrap()) * Matrix4::from_angle_z(Rad(phi)) * Matrix4::from_angle_y(Rad(theta)) * Matrix4::from_angle_z(Rad(psi)) * Matrix4::from_nonuniform_scale(delta_mag, 1., 1.);

        Self {
            world: world.into(),
            color: [color.x, color.y, color.z, 1.]
        }
    }
    pub fn circle(camera: &Camera, pos: Vector3<f64>, color: Vector3<f32>) -> Self {
        let delta = (pos - camera.position).cast::<f32>().unwrap();
        let theta = (-delta.z / delta.magnitude()).acos();
        let phi = f32::atan2(-delta.y, -delta.x);
        let world = Matrix4::from_translation(delta) * Matrix4::from_angle_z(Rad(phi)) * Matrix4::from_angle_y(Rad(theta)) * Matrix4::from_scale(0.05);

        Self {
            world: world.into(),
            color: [color.x, color.y, color.z, 1.]
        }
    }
}
impl Uniform for SolidUniform {}

// =================================================
// VERTICES
// =================================================

pub trait Vertex: Sized + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub enum Primitive {
    /**
    This primitive, the default primitive, creates triangles with vertices indexed by an index buffer.
    */
    Triangle,
    /**
    This primitive creates lines with vertices indexed by an index buffer.
    */
    Line,
}
impl Primitive {
    pub(crate) fn to_topology(&self) -> wgpu::PrimitiveTopology {
        match self {
            Primitive::Triangle => wgpu::PrimitiveTopology::TriangleList,
            Primitive::Line => wgpu::PrimitiveTopology::LineList,
        }
    }
}


/**
 # `TexVertex`
 Vertex for a model with a texture and normal vectors. Load into the shader using
 ```
 struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    }
    ```
    */
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct TexVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

/**
 # `LineVertex`
 Vertex containing only a single point. This is especially useful for drawing non-triangle primitives, like lines and points. Load into the shader using
 ```
 struct VertexInput {
    @location(0) position: vec3<f32>,
    }
    ```
    */
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointVertex {
    pub position: [f32; 3],
}

/**
# `ScreenVertex`
Vertex containing a point in 2 space and texture coords. Useful for rendering to screen. Load into the shader using
```
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}
```
 */
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl Vertex for TexVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}

impl Vertex for ScreenVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}

impl Vertex for PointVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}