use wgpu::util::DeviceExt;

use crate::prelude::TexVertex;

use super::super::Graphics;
use super::loading::LoadMesh;

pub struct Mesh {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
    pub(crate) material_index: usize,
}

impl Mesh {
    pub(super) fn new(graphics: &Graphics, mesh: &LoadMesh) -> Self {
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
                    1.-mesh.texcoords[2*i + 1],
                ],
            })
        }

        let indices = mesh.indices.iter().map(|i| *i as u16).collect::<Vec<_>>();
        Self::from_vertices(graphics, &vertices, &indices, mesh.material_id)
    }

    pub(super) fn from_vertices(graphics: &Graphics, vertices: &[TexVertex], indices: &[u16], material_index: usize) -> Mesh {
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
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            material_index,
        }
    }
}