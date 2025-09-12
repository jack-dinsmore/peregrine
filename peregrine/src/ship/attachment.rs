use serde::{Deserialize, Serialize};
use cgmath::{Quaternion, Vector3};
use strum::FromRepr;
use tethys::prelude::*;

use super::orientation;
use super::part_loader::PartLoader;

#[repr(usize)]
#[derive(Clone, Copy, Debug, FromRepr)]
pub enum AttachmentModel {
    HydrogenHub,
    CircuitHub,
}

/// The physical position of an entire part, or the blocks within a part
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AttachmentLayout {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation: u8,
}

impl AttachmentLayout {
    pub fn as_physical(&self) -> (Vector3<f64>, Quaternion<f64>) {
        (
            Vector3::new(self.x as f64, self.y as f64, self.z as f64),
            orientation::to_quat(self.orientation)
        )
    }

    pub fn from_normal(pos: Vector3<f32>, normal: Vector3<f32>) -> Self {
        dbg!(normal);
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            orientation: 0, // TODO
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Attachment {
    HydrogenHub,
    CircuitHub,
}

impl Attachment {
    pub(super) fn get_objects(&self, part_loader: PartLoader, layout: AttachmentLayout) -> Object {

        let model_type = match self {
            Attachment::HydrogenHub => AttachmentModel::HydrogenHub,
            Attachment::CircuitHub => AttachmentModel::CircuitHub,
        };

        let model = part_loader.load_attachment(model_type);

        // TODO Also consider making the to-be-placed part a new part of the ship, with a weird alpha. Not a new ship entirely. That way, attachments can be added. Maybe do a wire mesh instead of transparency? That could look cool.
        // If there is no cell that can fit the block, simply don't show it.
        Object::zeroed::<ObjectUniform>(&part_loader.graphics, model)
    }
}