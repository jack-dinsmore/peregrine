use std::ops::Mul;
use cgmath::{Quaternion, Vector3};
use serde::{Deserialize, Serialize};
use strum::FromRepr;
use tethys::prelude::*;

use super::part_loader::PartLoader;
use super::orientation;

pub(super) struct Block {
    pub object: Object,
    pub layout: PartLayout,
}
impl Block {
    fn new(graphics: &Graphics, model: Model, layout: PartLayout) -> Self {
        let object = Object::zeroed::<ObjectUniform>(graphics, model);
        Self {
            object,
            layout,
        }
    }
}

/// The physical position of an entire part, or the blocks within a part
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PartLayout {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub orientation: u8,
}
impl PartLayout {
    pub fn as_physical(&self) -> (Vector3<f64>, Quaternion<f64>) {
        (
            Vector3::new(self.x as f64 + 0.5, self.y as f64 + 0.5, self.z as f64 + 0.5),
            orientation::to_quat(self.orientation)
        )
    }
}
impl Mul for PartLayout {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let (new_x, new_y, new_z) = orientation::rotate_integer(rhs.orientation, self.x, self.y, self.z);
        Self {
            x: new_x + rhs.x,
            y: new_y + rhs.y,
            z: new_z + rhs.z,
            orientation: orientation::compose(rhs.orientation, self.orientation)
        }
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, FromRepr)]
pub enum PartModel {
    TankCap,
    TankBody,
    Scaffold,
    Thruster,
    FuelCell,
    Battery,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Part {
    Tank { length: u32 },
    Scaffold { length: u32, width: u32, height: u32 },
    Thruster,
    FuelCell,
    Battery,
}

impl Part {
    /// List all the blocks within a part
    pub fn get_blocks(&self, layout: PartLayout) -> Vec<PartLayout> {
        let mut output = Vec::new();
        let mut default = || {
            output.push(PartLayout {
                x: 0,
                y: 0,
                z: 0,
                orientation: 0
            } * layout);
        };
        match self {
            Self::Tank { length } => {
                let z0 = -(*length as i32) / 2;
                output.push(PartLayout {
                    x: 0,
                    y: 0,
                    z: z0,
                    orientation: 12
                } * layout);

                for i in 0..(*length as i32-2) {
                    output.push(PartLayout {
                        x: 0,
                        y: 0,
                        z: z0 + i+1,
                        orientation: 0
                    } * layout);
                }

                output.push(PartLayout {
                    x: 0,
                    y: 0,
                    z: z0 + *length as i32 - 1,
                    orientation: 0
                } * layout);
            },
            Self::Scaffold { length, width, height } => {
                let x0 = (*length as i32) / -2;
                let y0 = (*width as i32) / -2;
                let z0 = (*height as i32) / -2;
                for i in 0..*length {
                    for j in 0..*width {
                        for k in 0..*height {
                            output.push(PartLayout {
                                x: x0 + i as i32,
                                y: y0 + j as i32,
                                z: z0 + k as i32,
                                orientation: 0
                            } * layout);
                        }
                    }
                }
            }
            Part::Thruster => default(),
            Self::FuelCell => default(),
            Self::Battery => default(),
        }
        output
    }

    /// Gets all the object infos for a part.
    pub(super) fn get_objects(&self, part_loader: PartLoader, layout: PartLayout) -> Vec<Block> {
        let mut output = Vec::new();

        let mut default = |part_model: PartModel| {
            // Load the single model for a given part
            let model = part_loader.load_part(part_model);
            output.append(&mut self.get_blocks(layout).into_iter().map(|b| {
                Block::new(part_loader.graphics, model.clone(), b)
            }).collect::<Vec<_>>());
        };

        match self {
            Self::Tank { length } => {
                let cap = part_loader.load_part(PartModel::TankCap);
                let body = part_loader.load_part(PartModel::TankBody);
                output.append(&mut self.get_blocks(layout).into_iter().enumerate().map(|(i, layout)| {
                    if i == 0 {
                        Block::new(part_loader.graphics, cap.clone(), layout)
                    } else if i == *length as usize-1 {
                        Block::new(part_loader.graphics, cap.clone(), layout)
                    } else {
                        Block::new(part_loader.graphics, body.clone(), layout)
                    }
                }).collect::<Vec<_>>());
            },
            Self::FuelCell =>  default(PartModel::FuelCell),
            Self::Thruster =>  default(PartModel::Thruster),
            Self::Battery =>  default(PartModel::Battery),
            Self::Scaffold { .. } => default(PartModel::Scaffold),
        }
        output
    }
    
    /// Returns the coordinates of the minimum and maximum corners
    pub(crate) fn get_bbox(&self, layout: PartLayout) -> (PartLayout, PartLayout) {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut min_z = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut max_z = i32::MIN;
        for block in self.get_blocks(layout) {
            min_x = min_x.min(block.x);
            min_y = min_y.min(block.y);
            min_z = min_z.min(block.z);
            max_x = max_x.max(block.x);
            max_y = max_y.max(block.y);
            max_z = max_z.max(block.z);
        }
        (
            PartLayout {
                x: min_x,
                y: min_y,
                z: min_z,
                orientation: 0,
            },
            PartLayout {
                x: max_x,
                y: max_y,
                z: max_z,
                orientation: 0,
            }
        )
    }

    /// Return power drawn in Watts (a negative number indicates draw, while a positive number is power generation). Returns None if object does not participate in the power system
    pub fn typical_power_draw(&self) -> Option<f64> {
        match self {
            Part::Tank { .. } => None,
            Part::Scaffold { .. } => None,
            Part::Thruster => Some(-10.),
            Part::FuelCell => Some(50.),
            Part::Battery => Some(50.),
        }
    }
}