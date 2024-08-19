use tethys::prelude::*;

use super::PartLayout;

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

#[derive(Clone, Copy, Debug)]
pub enum Part {
    Tank { length: u32 },
    FuelCell,
}

impl Part {
    pub(super) fn get_blocks(&self, layout: PartLayout) -> Vec<PartLayout> {
        let mut output = Vec::new();
        match self {
            Self::Tank { length } => {
                output.push(PartLayout {
                    x: 0,
                    y: 0,
                    z: 0,
                    orientation: 12
                } + layout);

                for i in 0..(*length as i32-2) {
                    output.push(PartLayout {
                        x: 0,
                        y: 0,
                        z: i+1,
                        orientation: 0
                    } + layout);
                }

                output.push(PartLayout {
                    x: 0,
                    y: 0,
                    z: *length as i32 - 1,
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
        }
        output
    }

    pub(super) fn get_objects(&self, part_loader: &mut PartLoader, layout: PartLayout, index: usize) -> Vec<ObjectInfo> {
        let mut output = Vec::new();
        match self {
            Self::Tank { length } => {
                let cap = part_loader.get_part_model(PartModel::TankCap).clone();
                let body = part_loader.get_part_model(PartModel::TankBody).clone();
                output.append(&mut self.get_blocks(layout).into_iter().enumerate().map(|(i, b)| {
                    if i == 0 {
                        ObjectInfo::new(part_loader.graphics, cap.clone(), b, index)
                    } else if i == *length as usize-1 {
                        ObjectInfo::new(part_loader.graphics, body.clone(), b, index)
                    } else {
                        ObjectInfo::new(part_loader.graphics, cap.clone(), b, index)
                    }
                }).collect::<Vec<_>>());
            },
            Self::FuelCell => {
                let model = part_loader.get_part_model(PartModel::FuelCell).clone();
                output.append(&mut self.get_blocks(layout).into_iter().map(|b| {
                        ObjectInfo::new(part_loader.graphics, model.clone(), b, index)
                }).collect::<Vec<_>>());
            }
        }
        output
    }

    pub(super) fn get_dimensions(&self, orientation: u8) -> (u32, u32, u32) {
        let dimensions = match self {
            Part::Tank { length } => (1, 1, *length),
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
}

#[derive(Clone, Copy, Debug)]
pub enum PartModel {
    TankCap = 0,
    TankBody = 1,
    FuelCell = 2,
}

pub struct PartLoader<'a> {
    pub(super) graphics: &'a Graphics<'a>,
    parts: [Option<Model>; 256],
}

impl<'a> PartLoader<'a> {
    pub fn new(graphics: &'a Graphics) -> Self {
        Self {
            parts: [ const {None}; 256],
            graphics,
        }
    }

    pub fn get_part_model(&mut self, part: PartModel) -> Model {
        if let None = self.parts[part as usize] {
            let loaded_obj = match part {
                PartModel::TankCap => include_obj!("tank-cap"),
                PartModel::TankBody => include_obj!("tank-body"),
                PartModel::FuelCell => include_obj!("fuel-cell"),
            };
            self.parts[part as usize] = Some(Model::new(self.graphics, loaded_obj));
        }
        self.parts[part as usize].as_ref().unwrap().clone()
    }
}
