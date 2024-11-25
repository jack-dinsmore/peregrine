use wgpu::util::DeviceExt;

use crate::prelude::ShaderBinding;
use super::super::Graphics;
use super::loading::LoadMaterial;
use super::texture::make_texture;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MateriallUniform {
    light_info: [f32; 4],
}
impl MateriallUniform {
    fn new(diffuse: f32, specular: f32, shininess: f32) -> Self {
        Self {
            light_info: [diffuse, specular, shininess, 0.]
        }
    }
}

pub(crate) struct Material {
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub(super) fn new(graphics: &Graphics, material: &LoadMaterial) -> Self {
        let mut entries = Vec::new();

        // Load diffuse texture
        let (diffuse_texture_view, diffuse_sampler) = make_texture(
            graphics, &material.diffuse_texture,
            wgpu::AddressMode::ClampToEdge
        ).unwrap();
        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
        });
        entries.push(wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
        });
        
        // Load normal texture
        let normal_info = if !material.normal_texture.is_empty() {
            Some(make_texture(
                graphics, &material.normal_texture,
                wgpu::AddressMode::Repeat,
            ).unwrap())
        } else {
            None
        };
        if let Some((normal_texture_view, normal_sampler)) = &normal_info {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_texture_view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&normal_sampler),
            });
        }

        // Load model buffer
        let uniform = MateriallUniform::new(1., 1., material.shininess);
        let material_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Material Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        entries.push(wgpu::BindGroupEntry {
            binding: entries.len() as u32,
            resource: material_buffer.as_entire_binding(),
        });

        // Create bind group
        let bind_group_layout = match normal_info {
            Some(_) => &ShaderBinding::NoisyTexture.get_bind_group_layout(graphics),
            None => &ShaderBinding::Texture.get_bind_group_layout(graphics),
        };
        let bind_group = graphics.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                entries: &entries,
                label: Some("material_bind_group"),
            }
        );

        Self {
            bind_group,
        }
    }
}