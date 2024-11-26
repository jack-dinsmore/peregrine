use cgmath::{Quaternion, Vector3};
use strum::FromRepr;
use tethys::prelude::*;

use super::PartLoader;


#[repr(usize)]
#[derive(Copy, Clone, Debug, FromRepr)]
pub enum PanelModel {
    Metal,
}

#[derive(Clone, Debug)]
pub struct Panel {
    pub vertices: [(i32, i32, i32); 3],
    panel_model: PanelModel,
}
impl Panel {
    pub(crate) fn get_object(&self, loader: PartLoader, layout: PanelLayout) -> Object {
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
        let normal = v1.cross(v2);
        let vertices = self.vertices.iter().map(|v| 
            TexVertex {
                position: [v.0 as f32, v.1 as f32, v.2 as f32],
                tex_coords: [0., 0.],// TODO
                normal: [normal.x, normal.y, normal.z],
            }
        ).collect::<Vec<_>>();
        let indices = [0, 1, 2, 0, 2, 1];
        let model = Model::from_vertices(&loader.graphics, &vertices, &indices, material);
        Object::new(&loader.graphics, model, Vector3::new(0., 0., 0.), Quaternion::new(1., 0., 0., 0.))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PanelLayout {

}