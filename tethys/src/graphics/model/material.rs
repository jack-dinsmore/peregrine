use anyhow::Result;
use wgpu::util::DeviceExt;

use crate::prelude::ShaderBinding;
use super::super::Graphics;
use super::container::{Loader, Container, MaybeInstanced};
use super::loading::LoadMaterial;

pub type Material = MaybeInstanced<wgpu::BindGroup>;
pub type MaterialContainer<const CAPACITY: usize> = Container<CAPACITY, wgpu::BindGroup>;
pub type MaterialLoader<'a, const CAPACITY: usize> = Loader<'a, CAPACITY, wgpu::BindGroup>;


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

impl Material {
    pub fn new(graphics: &Graphics, material: &LoadMaterial) -> Self {
        let mut entries = Vec::new();

        // Load diffuse texture
        let diffuse_info = if !material.diffuse_texture.is_empty() {
            Some(make_texture(
                graphics, &material.diffuse_texture,
                wgpu::AddressMode::MirrorRepeat
            ).unwrap())
        } else {
            None
        };
        if let Some((diffuse_texture_view, diffuse_sampler)) = &diffuse_info {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            });
        }
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
        let bind_group_layout = match (&diffuse_info, &normal_info) {
            (Some(_), Some(_)) => &ShaderBinding::NoisyTexture.get_bind_group_layout(graphics),
            (Some(_), None) => &ShaderBinding::Texture.get_bind_group_layout(graphics),
            (None, Some(_)) => panic!("This material has a normal texture but no normal texture"),
            (None, None) => &ShaderBinding::Model.get_bind_group_layout(graphics),
        };
        let bind_group = graphics.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                entries: &entries,
                label: Some("material_bind_group"),
            }
        );

        Self::Singleton(bind_group)
    }
}

pub(super) fn make_texture(graphics: &Graphics, texture: &[u8], address_mode: wgpu::AddressMode) -> Result<(wgpu::TextureView, wgpu::Sampler)> {
    let image = image::load_from_memory(&texture)?;
    let rgba = image.to_rgba8();

    use image::GenericImageView;
    let dimensions = image.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let texture = graphics.device.create_texture(
        &wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB, so we need to reflect that here.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("texture"),
            // This is the same as with the SurfaceConfig. It
            // specifies what texture formats can be used to
            // create TextureViews for this texture. The base
            // texture format (Rgba8UnormSrgb in this case) is
            // always supported. Note that using a different
            // texture format is not supported on the WebGL2
            // backend.
            view_formats: &[],
        }
    );

    graphics.queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        // The actual pixel data
        &rgba,
        // The layout of the texture
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_size,
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    Ok((texture_view, sampler))
}