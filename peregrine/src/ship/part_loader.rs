use tethys::prelude::*;

use crate::ship::Panel;

use super::part::PartModel;
use super::attachment::AttachmentModel;

pub const MODEL_CAPACITY: usize = 64;
pub const MATERIAL_CAPACITY: usize = 64;
pub const ATTACHMENT_CAPACITY: usize = 64;

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
                    PartModel::TankCap => include_model!("tank-cap"),
                    PartModel::TankBody => include_model!("tank-body"),
                    PartModel::Scaffold => include_model!("scaffold"),
                    PartModel::FuelCell => include_model!("fuel-cell"),
                    PartModel::Thruster => include_model!("thruster"),
                    PartModel::Battery => include_model!("battery"),
                };
                Model::from_obj(graphics, loaded_obj)
            }),
            attachment_loader: self.model_container.loader(|index| {
                let part = AttachmentModel::from_repr(index).unwrap();
                let loaded_obj = match part {
                    AttachmentModel::HydrogenHub => include_model!("hub-fluid"),
                    AttachmentModel::CircuitHub => include_model!("hub-circuit"),
                };
                Model::from_obj(graphics, loaded_obj)
            }),
            material_loader: self.material_container.loader(|index| {
                let panel = Panel::from_repr(index).unwrap();
                let loaded_material = match panel {
                    Panel::Metal => include_material!("metal")
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
    attachment_loader: ModelLoader<'a, ATTACHMENT_CAPACITY>,
    pub graphics: &'a Graphics<'a>,
}

impl<'a> PartLoader<'a> {
    pub fn load_attachment(&self, attachment: AttachmentModel) -> Model {
        self.attachment_loader.borrow(attachment as usize)
    }
    pub fn load_part(&self, part: PartModel) -> Model {
        self.model_loader.borrow(part as usize)
    }
    pub fn load_panel(&self, panel: Panel) -> Material {
        self.material_loader.borrow(panel as usize)
    }
}