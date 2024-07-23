use std::sync::Arc;

use wgpu::util::DeviceExt;
use crate::graphics::{shader::ShaderBinding, primitives::TexVertex};

use super::Graphics;

pub struct LoadedObj {
    pub meshes: Vec<LoadMesh>,
    pub materials: Vec<LoadMaterial>,
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

pub struct LoadMesh {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub indices: Vec<u32>,
    pub material_id: usize,
}

pub struct LoadMaterial {
    pub name: String,
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub normal_texture: Vec<u8>,
    pub diffuse_texture: Vec<u8>,
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
                    mesh.texcoords[2*i + 1],
                ],
            })
        }
        
        let vertex_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("TexVertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
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
        let diffuse_image = image::load_from_memory(&material.diffuse_texture).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = graphics.device.create_texture(
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
                label: Some("diffuse_texture"),
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
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
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
                    }
                ],
                label: Some("model_bind_group"),
            }
        );

        Self {
            bind_group,
        }
    }
}