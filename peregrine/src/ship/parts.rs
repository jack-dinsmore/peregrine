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

    pub(super) fn get_part_model(&mut self, part: PartModel) -> Model {
        if let None = self.parts[part as usize] {
            let loaded_obj = match part {
                PartModel::TankCap => load_obj!("assets/parts/tank-cap.obj"),
                PartModel::TankBody => load_obj!("assets/parts/tank-body.obj"),
                PartModel::FuelCell => load_obj!("assets/parts/fuel-cell.obj"),
            };
            self.parts[part as usize] = Some(Model::new(self.graphics, loaded_obj));
        }
        self.parts[part as usize].as_ref().unwrap().clone()
    }
}

pub fn compose_orientations(a: u8, b: u8) -> u8 {
    match (a, b) {
        (0, 0) => 0,
        (0, 1) => 1,
        (0, 2) => 2,
        (0, 3) => 3,
        (0, 4) => 4,
        (0, 5) => 5,
        (0, 6) => 6,
        (0, 7) => 7,
        (0, 8) => 8,
        (0, 9) => 9,
        (0, 10) => 10,
        (0, 11) => 11,
        (0, 12) => 12,
        (0, 13) => 13,
        (0, 14) => 14,
        (0, 15) => 15,

        (1, 0) => 1,
        (1, 1) => 2,
        (1, 2) => 3,
        (1, 3) => 0,
        (1, 4) => 5,
        (1, 5) => 6,
        (1, 6) => 7,
        (1, 7) => 4,
        (1, 8) => 9,
        (1, 9) => 10,
        (1, 10) => 11,
        (1, 11) => 8,
        (1, 12) => 13,
        (1, 13) => 14,
        (1, 14) => 15,
        (1, 15) => 12,

        (2, 0) => 2,
        (2, 1) => 3,
        (2, 2) => 0,
        (2, 3) => 1,
        (2, 4) => 6,
        (2, 5) => 7,
        (2, 6) => 4,
        (2, 7) => 5,
        (2, 8) => 10,
        (2, 9) => 11,
        (2, 10) => 8,
        (2, 11) => 9,
        (2, 12) => 14,
        (2, 13) => 15,
        (2, 14) => 12,
        (2, 15) => 13,

        (3, 0) => 3,
        (3, 1) => 0,
        (3, 2) => 1,
        (3, 3) => 2,
        (3, 4) => 7,
        (3, 5) => 4,
        (3, 6) => 5,
        (3, 7) => 6,
        (3, 8) => 11,
        (3, 9) => 8,
        (3, 10) => 9,
        (3, 11) => 10,
        (3, 12) => 15,
        (3, 13) => 12,
        (3, 14) => 13,
        (3, 15) => 14,

        (4, 0) => 4,
        (4, 1) => 0,
        (4, 2) => 0,
        (4, 3) => 0,
        (4, 4) => 0,
        (4, 5) => 0,
        (4, 6) => 0,
        (4, 7) => 0,
        (4, 8) => 0,
        (4, 9) => 0,
        (4, 10) => 0,
        (4, 11) => 0,
        (4, 12) => 0,
        (4, 13) => 0,
        (4, 14) => 0,
        (4, 15) => 0,

        (5, 0) => 5,
        (5, 1) => 0,
        (5, 2) => 0,
        (5, 3) => 0,
        (5, 4) => 0,
        (5, 5) => 0,
        (5, 6) => 0,
        (5, 7) => 0,
        (5, 8) => 0,
        (5, 9) => 0,
        (5, 10) => 0,
        (5, 11) => 0,
        (5, 12) => 0,
        (5, 13) => 0,
        (5, 14) => 0,
        (5, 15) => 0,

        (6, 0) => 6,
        (6, 1) => 0,
        (6, 2) => 0,
        (6, 3) => 0,
        (6, 4) => 0,
        (6, 5) => 0,
        (6, 6) => 0,
        (6, 7) => 0,
        (6, 8) => 0,
        (6, 9) => 0,
        (6, 10) => 0,
        (6, 11) => 0,
        (6, 12) => 0,
        (6, 13) => 0,
        (6, 14) => 0,
        (6, 15) => 0,

        (7, 0) => 7,
        (7, 1) => 0,
        (7, 2) => 0,
        (7, 3) => 0,
        (7, 4) => 0,
        (7, 5) => 0,
        (7, 6) => 0,
        (7, 7) => 0,
        (7, 8) => 0,
        (7, 9) => 0,
        (7, 10) => 0,
        (7, 11) => 0,
        (7, 12) => 0,
        (7, 13) => 0,
        (7, 14) => 0,
        (7, 15) => 0,

        (8, 0) => 8,
        (8, 1) => 0,
        (8, 2) => 0,
        (8, 3) => 0,
        (8, 4) => 0,
        (8, 5) => 0,
        (8, 6) => 0,
        (8, 7) => 0,
        (8, 8) => 0,
        (8, 9) => 0,
        (8, 10) => 0,
        (8, 11) => 0,
        (8, 12) => 0,
        (8, 13) => 0,
        (8, 14) => 0,
        (8, 15) => 0,

        (9, 0) => 9,
        (9, 1) => 0,
        (9, 2) => 0,
        (9, 3) => 0,
        (9, 4) => 0,
        (9, 5) => 0,
        (9, 6) => 0,
        (9, 7) => 0,
        (9, 8) => 0,
        (9, 9) => 0,
        (9, 10) => 0,
        (9, 11) => 0,
        (9, 12) => 0,
        (9, 13) => 0,
        (9, 14) => 0,
        (9, 15) => 0,

        (10, 0) => 10,
        (10, 1) => 0,
        (10, 2) => 0,
        (10, 3) => 0,
        (10, 4) => 0,
        (10, 5) => 0,
        (10, 6) => 0,
        (10, 7) => 0,
        (10, 8) => 0,
        (10, 9) => 0,
        (10, 10) => 0,
        (10, 11) => 0,
        (10, 12) => 0,
        (10, 13) => 0,
        (10, 14) => 0,
        (10, 15) => 0,

        (11, 0) => 11,
        (11, 1) => 0,
        (11, 2) => 0,
        (11, 3) => 0,
        (11, 4) => 0,
        (11, 5) => 0,
        (11, 6) => 0,
        (11, 7) => 0,
        (11, 8) => 0,
        (11, 9) => 0,
        (11, 10) => 0,
        (11, 11) => 0,
        (11, 12) => 0,
        (11, 13) => 0,
        (11, 14) => 0,
        (11, 15) => 0,

        (12, 0) => 12,
        (12, 1) => 0,
        (12, 2) => 0,
        (12, 3) => 0,
        (12, 4) => 0,
        (12, 5) => 0,
        (12, 6) => 0,
        (12, 7) => 0,
        (12, 8) => 0,
        (12, 9) => 0,
        (12, 10) => 0,
        (12, 11) => 0,
        (12, 12) => 0,
        (12, 13) => 0,
        (12, 14) => 0,
        (12, 15) => 0,

        (13, 0) => 13,
        (13, 1) => 0,
        (13, 2) => 0,
        (13, 3) => 0,
        (13, 4) => 0,
        (13, 5) => 0,
        (13, 6) => 0,
        (13, 7) => 0,
        (13, 8) => 0,
        (13, 9) => 0,
        (13, 10) => 0,
        (13, 11) => 0,
        (13, 12) => 0,
        (13, 13) => 0,
        (13, 14) => 0,
        (13, 15) => 0,

        (14, 0) => 14,
        (14, 1) => 0,
        (14, 2) => 0,
        (14, 3) => 0,
        (14, 4) => 0,
        (14, 5) => 0,
        (14, 6) => 0,
        (14, 7) => 0,
        (14, 8) => 0,
        (14, 9) => 0,
        (14, 10) => 0,
        (14, 11) => 0,
        (14, 12) => 0,
        (14, 13) => 0,
        (14, 14) => 0,
        (14, 15) => 0,

        (15, 0) => 15,
        (15, 1) => 0,
        (15, 2) => 0,
        (15, 3) => 0,
        (15, 4) => 0,
        (15, 5) => 0,
        (15, 6) => 0,
        (15, 7) => 0,
        (15, 8) => 0,
        (15, 9) => 0,
        (15, 10) => 0,
        (15, 11) => 0,
        (15, 12) => 0,
        (15, 13) => 0,
        (15, 14) => 0,
        (15, 15) => 0,

        _ => unreachable!()
    }
}