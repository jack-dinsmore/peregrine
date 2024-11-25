
use cgmath::{Rotation, Vector3};
use strum::FromRepr;
use tethys::prelude::*;

use super::PartLayout;

const MODEL_CAPACITY: usize = 64;

pub(super) struct ObjectInfo {
    pub object: Object,
    pub layout: PartLayout,
    pub part_index: usize,
}
impl ObjectInfo {
    fn new(graphics: &Graphics, model: Model, layout: PartLayout, part_index: usize) -> Self {
        let (position, orientation) = layout.as_physical();
        let object = Object::new(graphics, model, position, orientation);
        Self {
            object,
            layout,
            part_index
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
    /// List all the blocks within a part and their 
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
    pub(super) fn get_objects(&self, part_loader: PartLoader, layout: PartLayout, index: usize) -> Vec<ObjectInfo> {
        let mut output = Vec::new();

        let mut default = |part_model: PartModel| {
            // Load the single model for a given part
            let model = part_loader.load(part_model);
            output.append(&mut self.get_blocks(layout).into_iter().map(|b| {
                ObjectInfo::new(part_loader.graphics, model.clone(), b, index)
            }).collect::<Vec<_>>());
        };

        match self {
            Self::Tank { length } => {
                let cap = part_loader.load(PartModel::TankCap);
                let body = part_loader.load(PartModel::TankBody);
                output.append(&mut self.get_blocks(layout).into_iter().enumerate().map(|(i, layout)| {
                    if i == 0 {
                        ObjectInfo::new(part_loader.graphics, cap.clone(), layout, index)
                    } else if i == *length as usize-1 {
                        ObjectInfo::new(part_loader.graphics, cap.clone(), layout, index)
                    } else {
                        ObjectInfo::new(part_loader.graphics, body.clone(), layout, index)
                    }
                }).collect::<Vec<_>>());
            },
            Self::FuelCell =>  default(PartModel::FuelCell),
            Self::Box { .. } => default(PartModel::Box),
        }
        output
    }

    pub(super) fn get_dimensions(&self, orientation: u8) -> (u32, u32, u32) {
        let dimensions = match self {
            Part::Tank { length } => (1, 1, *length),
            Part::Box { length, width, height } => (*length, *width ,*height),
            Part::FuelCell => (1,1,2),
        };
        match orientation {
            0|2 => (dimensions.0, dimensions.1, dimensions.2),
            1|3 => (dimensions.1, dimensions.0, dimensions.2),
            4|6 => (dimensions.0, dimensions.2, dimensions.1),
            5|7 => (dimensions.2, dimensions.0, dimensions.1),
            8|10 => (dimensions.2, dimensions.1, dimensions.0),
            9|11 => (dimensions.1, dimensions.2, dimensions.0),
            12|14 => (dimensions.0, dimensions.1, dimensions.2),
            13|15 => (dimensions.1, dimensions.0, dimensions.2),
            _ => unreachable!()
        }
    }
    
    pub(crate) fn get_collider(&self, layout: &PartLayout) -> CollisionBox {
        let dimensions = self.get_dimensions(layout.orientation);
        let dimensions = Vector3::new(dimensions.0 as f64, dimensions.1 as f64, dimensions.2 as f64);
        let (offset, quat) = layout.as_physical();
        let offset = offset - dimensions / 2.;
        CollisionBox::new(offset, quat.rotate_vector(dimensions))
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, FromRepr)]
pub enum PartModel {
    TankCap = 0,
    TankBody = 1,
    Box = 2,
    FuelCell = 3,
}


// Stores the ship part models and provides a seamless interface to load them
pub struct PartData {
    container: ModelContainer<MODEL_CAPACITY>,
}
impl PartData {
    pub fn new() -> Self {
        Self {
            container: ModelContainer::new()
        }
    }

    pub fn get_loader<'a>(&'a self, graphics: &'a Graphics) -> PartLoader<'a> {
        PartLoader {
            loader: self.container.loader(|index| {
                let part = PartModel::from_repr(index).unwrap();
                let loaded_obj = match part {
                    PartModel::TankCap => include_obj!("tank-cap"),
                    PartModel::TankBody => include_obj!("tank-body"),
                    PartModel::Box => include_obj!("box"),
                    PartModel::FuelCell => include_obj!("fuel-cell"),
                };
                Model::new(graphics, loaded_obj)
            }),
            graphics,
        }
    }
}

#[derive(Clone)]
pub struct PartLoader<'a> {
    loader: ModelLoader<'a, MODEL_CAPACITY>,
    graphics: &'a Graphics<'a>,
}

impl<'a> PartLoader<'a> {
    pub fn load(&self, part: PartModel) -> Model {
        self.loader.borrow(part as usize)
    }
}