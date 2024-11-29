use cgmath::{InnerSpace, Quaternion, Vector3};
use serde::{Deserialize, Serialize};
use strum::FromRepr;
use tethys::prelude::*;

use super::PartLoader;


#[repr(usize)]
#[derive(Copy, Clone, Debug, FromRepr, Serialize, Deserialize)]
pub enum PanelModel {
    Metal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Panel {
    pub vertices: [(i32, i32, i32); 3],
    pub panel_model: PanelModel,
}
impl Panel {
    pub(crate) fn get_object(&self, loader: PartLoader, layout: PanelLayout) -> Option<Object> {
        // Do not try to make an object for a panel that is actually a line
        if self.vertices[0] == self.vertices[1] || self.vertices[0] == self.vertices[2] || self.vertices[1] == self.vertices[2] {return None;}
        let material = loader.load_panel(self.panel_model);
        let v1 = Vector3::new(
            (self.vertices[1].0 - self.vertices[0].0) as f32,
            (self.vertices[1].1 - self.vertices[0].1) as f32,
            (self.vertices[1].2 - self.vertices[0].2) as f32
        );
        let v2 = Vector3::new(
            (self.vertices[2].0 - self.vertices[0].0) as f32,
            (self.vertices[2].1 - self.vertices[0].1) as f32,
            (self.vertices[2].2 - self.vertices[0].2) as f32
        );
        let normal = v1.cross(v2).normalize();
        let mut up = Vector3::new(0., 0., 1.);
        if up.dot(normal) > 0.99 {
            up =  Vector3::new(1., 0., 0.);
        }
        let tangent_x = normal.cross(up).normalize();
        let tangent_y = tangent_x.cross(normal);
        let vertices = self.vertices.iter().map(|v| {
            let offset = Vector3::new(
                (v.0 - self.vertices[0].0) as f32,
                (v.1 - self.vertices[0].1) as f32, 
                (v.2 - self.vertices[0].2) as f32
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

#[derive(Clone, Copy, Debug,  Serialize, Deserialize)]
pub struct PanelLayout {

}