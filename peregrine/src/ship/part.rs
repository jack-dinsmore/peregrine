
use std::ops::Add;

use cgmath::{Quaternion, Vector3};
use strum::FromRepr;
use tethys::prelude::*;

use super::{orientation, panel::PanelModel};

pub const MODEL_CAPACITY: usize = 64;
pub const MATERIAL_CAPACITY: usize = 64;

pub(super) struct Block {
    pub object: Object,
    pub layout: PartLayout,
}
impl Block {
    fn new(graphics: &Graphics, model: Model, layout: PartLayout) -> Self {
        let (position, orientation) = layout.as_physical();
        let object = Object::new(graphics, model, position, orientation);
        Self {
            object,
            layout,
        }
    }
}

/// The physical position of an entire part, or the blocks within a part
#[derive(Clone, Copy, Debug)]
pub struct PartLayout {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub orientation: u8,
}
impl PartLayout {
    pub fn as_physical(&self) -> (Vector3<f64>, Quaternion<f64>) {
        (
            Vector3::new(self.x as f64, self.y as f64, self.z as f64),
            orientation::to_quat(self.orientation)
        )
    }
}
impl Add for PartLayout {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            orientation: orientation::compose(self.orientation, rhs.orientation)
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Part {
    Tank { length: u32 },
    Box { length: u32, width: u32, height: u32 },
    FuelCell,
}

impl Part {
    /// List all the blocks within a part
    pub(super) fn get_blocks(&self, layout: PartLayout) -> Vec<PartLayout> {
        let mut output = Vec::new();
        match self {
            Self::Tank { length } => {
                let z0 = -(*length as i32) / 2;
                output.push(PartLayout {
                    x: 0,
                    y: 0,
                    z: z0,
                    orientation: 12
                } + layout);

                for i in 0..(*length as i32-2) {
                    output.push(PartLayout {
                        x: 0,
                        y: 0,
                        z: z0 + i+1,
                        orientation: 0
                    } + layout);
                }

                output.push(PartLayout {
                    x: 0,
                    y: 0,
                    z: z0 + *length as i32 - 1,
                    orientation: 0
                } + layout);
            },
            Self::FuelCell => {
                output.push(PartLayout {
                    x: 0,
                    y: 0,
                    z: 0,
                    orientation: 0
                } + layout);
            }
            Self::Box { length, width, height } => {
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
                            } + layout);
                        }
                    }
                }
            }
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
            Self::Box { .. } => default(PartModel::Box),
        }
        output
    }
    
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
        let (min_x, min_y, min_z) = orientation::rotate_integer(layout.orientation, min_x, min_y, min_z);
        let (max_x,max_y,max_z) = orientation::rotate_integer(layout.orientation, max_x,max_y,max_z);
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
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, FromRepr)]
pub enum PartModel {
    Placement,
    TankCap,
    TankBody,
    Box,
    FuelCell,
}


// Stores the ship part models and provides a seamless interface to load them
pub struct PartData {
    model_container: ModelContainer<MODEL_CAPACITY>,
    material_container: MaterialContainer<MATERIAL_CAPACITY>,
}
impl PartData {
    pub fn new() -> Self {
        Self {
            model_container: ModelContainer::new(),
            material_container: MaterialContainer::new()
        }
    }

    pub fn get_loader<'a>(&'a self, graphics: &'a Graphics) -> PartLoader<'a> {
        PartLoader {
            model_loader: self.model_container.loader(|index| {
                let part = PartModel::from_repr(index).unwrap();
                let loaded_obj = match part {
                    PartModel::Placement => include_model!("placement"),
                    PartModel::TankCap => include_model!("tank-cap"),
                    PartModel::TankBody => include_model!("tank-body"),
                    PartModel::Box => include_model!("box"),
                    PartModel::FuelCell => include_model!("fuel-cell"),
                };
                Model::new(graphics, loaded_obj)
            }),
            material_loader: self.material_container.loader(|index| {
                let panel = PanelModel::from_repr(index).unwrap();
                let loaded_material = match panel {
                    PanelModel::Metal => include_material!("metal")
                };
                Material::new(graphics, &loaded_material)
            }),
            graphics,
        }
    }
}

#[derive(Clone)]
pub struct PartLoader<'a> {
    material_loader: MaterialLoader<'a, MATERIAL_CAPACITY>,
    model_loader: ModelLoader<'a, MODEL_CAPACITY>,
    pub graphics: &'a Graphics<'a>,
}

impl<'a> PartLoader<'a> {
    pub fn load_part(&self, part: PartModel) -> Model {
        self.model_loader.borrow(part as usize)
    }
    pub fn load_panel(&self, panel: PanelModel) -> Material {
        self.material_loader.borrow(panel as usize)
    }
}