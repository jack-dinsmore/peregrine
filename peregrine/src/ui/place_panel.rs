use cgmath::Vector3;
use tethys::prelude::*;

use crate::ship::{Panel, PanelLayout, PartLoader, ShipInterior};

pub struct PlacePanelState {
    interior: ShipInterior,
    panel: Panel,
    display: bool,
    place_coords: Vec<Vector3<f64>>,// The coordinate on interior that should go where the mouse is
}
impl PlacePanelState {
    pub fn new(loader: PartLoader, panel: Panel) -> Self {
        let rigid_body = RigidBody::default();
        let layout = PanelLayout {
            
        };
        Self {
            interior:  ShipInterior::new(loader, Vec::new(), Vec::new(), vec![panel], vec![layout], rigid_body),
            display: false,
            place_coords: Vec::new(),
            panel,
        }
    }

    pub(crate) fn update(&self, camera: &Camera, ship: &mut ShipInterior) {
        unimplemented!()
    }
    
    pub(crate) fn place(&self, part_loader: crate::ship::PartLoader<'_>, ship: &mut ShipInterior) {
        unimplemented!()
    }
    
    pub fn object(&self) -> Vec<ObjectHandle> {
        if self.display {
            self.interior.objects()
        }
        else {
            Vec::new()
        }
    }
}