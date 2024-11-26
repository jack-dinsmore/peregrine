use wgpu::util::DeviceExt;

use crate::prelude::TexVertex;

use super::super::Graphics;
use super::loading::LoadMesh;

pub struct Mesh {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
    pub(crate) material_id: usize,
}

impl Mesh {
    pub(super) fn new(graphics: &Graphics, mesh: &LoadMesh) -> Self {
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