use tethys::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Part {
    Null = 0,
    TankCap = 1,
    TankBody = 2,
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

    pub(super) fn get_part(&mut self, part: Part) -> Model {
        if let None = self.parts[part as usize] {
            let loaded_obj = match part {
                Part::Null => unimplemented!(),
                Part::TankCap => load_obj!("assets/parts/tank-cap.obj"),
                Part::TankBody => load_obj!("assets/parts/tank-body.obj"),
            };
            self.parts[part as usize] = Some(Model::new(self.graphics, loaded_obj));
        }
        self.parts[part as usize].as_ref().unwrap().clone()
    }
}