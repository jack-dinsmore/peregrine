use std::sync::Arc;
use anyhow::Result;
use wgpu::util::DeviceExt;

use crate::graphics::{shader::ShaderBinding, primitives::TexVertex};
use super::Graphics;

pub use super::model_loading::{LoadMaterial, LoadMesh, LoadedObj};

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

pub(crate) struct Mesh {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
    pub(crate) material_id: usize,
}

pub(crate) struct Material {
    pub bind_group: wgpu::BindGroup,
}

#[derive(Clone)]
pub struct Model {
    pub(crate) model_data: Arc<(Vec<Mesh>, Vec<Material>)>,
    identifier: usize,
}

impl Model {
    pub fn new(graphics: &Graphics, obj: LoadedObj) -> Self {

        let mut meshes = Vec::with_capacity(obj.meshes.len());
        for load_mesh in &obj.meshes {
            meshes.push(Mesh::new(graphics, load_mesh));
        }

        let mut materials = Vec::with_capacity(obj.materials.len());
        for load_material in &obj.materials {
            materials.push(Material::new(graphics, load_material));
        }

        let model_data = Arc::new((meshes, materials));

        let identifier = Arc::as_ptr(&model_data) as usize;
        Self {
            model_data,
            identifier
        }
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for Model {}

impl PartialOrd for Model {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.identifier.partial_cmp(&other.identifier)
    }
}

impl Ord for Model {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.identifier.cmp(&other.identifier)
    }
}

impl Mesh {
    fn new(graphics: &Graphics, mesh: &LoadMesh) -> Self {
        let mut vertices = Vec::with_capacity(mesh.positions.len());
        let n_triangles = mesh.positions.len()/3;

        let rescale = {
            let l_phys = (
                (mesh.positions[0] - mesh.positions[3]).powi(2)
                + (mesh.positions[1] - mesh.positions[4]).powi(2)
                + (mesh.positions[2] - mesh.positions[5]).powi(2)
            ).sqrt();
            let l_tex = (
                (mesh.texcoords[0] - mesh.texcoords[2]).powi(2)
                + (mesh.texcoords[1] - mesh.texcoords[3]).powi(2)
            ).sqrt();
            l_phys / l_tex
        };

        for i in 0..n_triangles {
            vertices.push(TexVertex {
                position: [
                    mesh.positions[3*i + 0],
                    mesh.positions[3*i + 1],
                    mesh.positions[3*i + 2],
                ],
                normal: [
                    mesh.normals[3*i + 0],
                    mesh.normals[3*i + 1],
                    mesh.normals[3*i + 2],
                ],
                tex_coords: [
                    mesh.texcoords[2*i + 0],
                    1.-mesh.texcoords[2*i + 1],
                ],
                normal_coords: [
                    mesh.texcoords[2*i + 0] * rescale,
                    (1.-mesh.texcoords[2*i + 1]) * rescale,
                ]
            })
        }
        
        let vertex_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("TexVertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let indices = mesh.indices.iter().map(|i| *i as u16).collect::<Vec<_>>();
        let index_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
            material_id: mesh.material_id,
        }
    }
}

impl Material {
    fn new(graphics: &Graphics, material: &LoadMaterial) -> Self {
        let (diffuse_texture_view, diffuse_sampler) = make_texture(
            graphics, &material.diffuse_texture,
            wgpu::AddressMode::ClampToEdge
        ).unwrap();
        let (normal_texture_view, normal_sampler) = make_texture(
            graphics, &material.normal_texture,
            wgpu::AddressMode::Repeat,
        ).unwrap();

        let uniform = MateriallUniform::new(1., 1., material.shininess);

        let material_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Material Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        let bind_group = graphics.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &ShaderBinding::Texture.get_bind_group_layout(graphics),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&normal_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&normal_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: material_buffer.as_entire_binding(),
                    }
                ],
                label: Some("material_bind_group"),
            }
        );

        Self {
            bind_group,
        }
    }
}

fn make_texture(graphics: &Graphics, texture: &[u8], address_mode: wgpu::AddressMode) -> Result<(wgpu::TextureView, wgpu::Sampler)> {
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