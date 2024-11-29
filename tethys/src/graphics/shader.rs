use std::marker::PhantomData;

use crate::graphics::primitives::Primitive;

use super::{primitives::Vertex, Graphics};

pub enum ShaderBinding {
    /** # Camera shader binding
    This binding loads the camera UBO at group 0, binding 0, which has type 
    ```
    struct CameraUniform {
        view_proj: mat4x4<f32>,
        light_pos: vec4<f32>,
    };
    ```
    and should be bound with 
    ```
    @group(0) @binding(0)
    var<uniform> camera: CameraUniform;
    ```
    */
    Camera,
    /** # Object shader binding
    This binding loads the model UBO at group 1, binding 0, which has type 
    ```
    struct ObjectUniform {
        world: mat4x4<f32>,
        rot_mat: mat4x4<f32>,
    };
    ```
    and should be bound with 
    ```
    @group(1) @binding(0)
    var<uniform> model: ObjectUniform;
    ```
    */
    Object,
    /** # Texture shader binding
    This binding loads the texture and its sampler for the fragment shader. Include them in the vertex shader using
    ```
    @group(2) @binding(0)
    var t_diffuse: texture_2d<f32>;
    @group(2) @binding(1)
    var s_diffuse: sampler;
    @group(2) @binding(2)
    var<uniform> material: MaterialUniform;
    ```
    where `MaterialUniform` stores light information
    ```
    struct MaterialUniform {
        light_info: vec4<f32>
    };
    ```
    */
    Texture,
    /** # Noisy Texture shader binding
    A noisy texture has two textures bound, nominally the first for the texture of the model and the second for a bump map,
    although it can be used for other purposes. Bind them like this:
    ```
    @group(2) @binding(0)
    var t_diffuse: texture_2d<f32>;
    @group(2) @binding(1)
    var s_diffuse: sampler;
    @group(2) @binding(2)
    var t_normal: texture_2d<f32>;
    @group(2) @binding(3)
    var s_normal: sampler;
    @group(2) @binding(4)
    var<uniform> material: MaterialUniform;
    ```
    where `MaterialUniform` stores light information
    ```
    struct MaterialUniform {
        light_info: vec4<f32>
    };
    ```
    */
    NoisyTexture,
}

impl ShaderBinding {
    pub(crate) fn get_bind_group_layout(&self, graphics: &Graphics) -> wgpu::BindGroupLayout {
        let entries = match self {
            ShaderBinding::Camera => vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            ShaderBinding::Object => vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            ShaderBinding::Texture => vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            ShaderBinding::NoisyTexture => vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        };

        graphics.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("texture_bind_group_layout"),
        })
    }
}

pub struct Shader {
    pub(crate) render_pipeline: wgpu::RenderPipeline,
}

/**
Remember to start your vertex shader with 
```
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {

}
```
and similarly for the fragment shader after properly defining your `VertexInput` and `VertexOutput`
structs. For example,
```
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) normal: vec3<f32>,
}```
 */
pub struct ShaderBuilder<'a, V: Vertex> {
    code: &'a str,
    bindings: &'a [ShaderBinding],
    primitive: Primitive,
    phantom_data: PhantomData<V>,
}
impl<'a, V: Vertex> ShaderBuilder<'a, V> {
    pub fn new(code: &'a str, bindings: &'a [ShaderBinding]) -> Self {
        Self {
            code,
            bindings,
            primitive: Primitive::Triangle,
            phantom_data: PhantomData::<V>,
        }
    }

    /// Sets the primitive type (e.g., triangles, lines, etc.) The type of primitive has no effect on the shader code.
    pub fn set_primitive(mut self, primitive: Primitive) -> Self {
        self.primitive = primitive;
        self
    }

    pub fn build(self, graphics: &Graphics) -> Shader {
        let shader = graphics.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(self.code.into()),
        });

        let bind_group_layouts = self.bindings.iter().map(|binding| binding.get_bind_group_layout(graphics)).collect::<Vec<_>>();
        let ptr_bind_group_layouts = bind_group_layouts.iter().map(|a| a).collect::<Vec<_>>();

        let render_pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &ptr_bind_group_layouts,
            push_constant_ranges: &[],
        });

        let render_pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    V::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: graphics.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.primitive.to_topology(),
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None,
            cache: None,
        });

        Shader {
            render_pipeline,
        }
    }
}