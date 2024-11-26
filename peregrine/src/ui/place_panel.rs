use tethys::prelude::*;

use crate::ship::{Part, PartLoader, ShipInterior};

pub struct PlacePanelState {
    
}
impl PlacePanelState {
    pub fn new(part_loader: PartLoader, part: Part) -> Self {
        unimplemented!()
    }

    pub(crate) fn update(&self, camera: &Camera, ship: &mut ShipInterior) {
        unimplemented!()
    }
    
    pub(crate) fn place(&self, part_loader: crate::ship::PartLoader<'_>, ship: &mut ShipInterior) {
        unimplemented!()
    }
    
    pub fn object(&self) -> Vec<ObjectHandle> {
        unimplemented!()
    }
}