use cgmath::{InnerSpace, Vector3};
use serde::{Deserialize, Serialize};
use strum::FromRepr;
use tethys::prelude::*;

use super::PartLoader;


#[derive(Clone, Copy, Debug,  Serialize, Deserialize)]
pub struct PanelLayout {
    pub vertices: [(i32, i32, i32); 3],
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Serialize, FromRepr, Deserialize)]
pub enum Panel {
    Metal,
}
impl Panel {
    pub(crate) fn get_object(&self, loader: PartLoader, layout: PanelLayout) -> Option<Object> {
        // Do not try to make an object for a panel that is actually a line
        if layout.vertices[0] == layout.vertices[1] || layout.vertices[0] == layout.vertices[2] || layout.vertices[1] == layout.vertices[2] {return None;}
        let material = loader.load_panel(*self);
        let v1 = Vector3::new(
            (layout.vertices[1].0 - layout.vertices[0].0) as f32,
            (layout.vertices[1].1 - layout.vertices[0].1) as f32,
            (layout.vertices[1].2 - layout.vertices[0].2) as f32
        );
        let v2 = Vector3::new(
            (layout.vertices[2].0 - layout.vertices[0].0) as f32,
            (layout.vertices[2].1 - layout.vertices[0].1) as f32,
            (layout.vertices[2].2 - layout.vertices[0].2) as f32
        );
        let normal = v1.cross(v2).normalize();
        let mut up = Vector3::new(0., 0., 1.);
        if up.dot(normal) > 0.99 {
            up =  Vector3::new(1., 0., 0.);
        }
        let tangent_x = normal.cross(up).normalize();
        let tangent_y = tangent_x.cross(normal);
        let vertices = layout.vertices.iter().map(|v| {
            let offset = Vector3::new(
                (v.0 - layout.vertices[0].0) as f32,
                (v.1 - layout.vertices[0].1) as f32, 
                (v.2 - layout.vertices[0].2) as f32
            );
            TexVertex {
                position: [v.0 as f32, v.1 as f32, v.2 as f32],
                tex_coords: [offset.dot(tangent_x), offset.dot(tangent_y)],// TODO
                normal: [normal.x, normal.y, normal.z],
            }
        }).collect::<Vec<_>>();
        let indices = [0, 1, 2, 0, 2, 1];
        let model = Model::from_vertices_and_material(&loader.graphics, &vertices, &indices, material);
        Some(Object::zeroed::<ObjectUniform>(&loader.graphics, model))
    }
}