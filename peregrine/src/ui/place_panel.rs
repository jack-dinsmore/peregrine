use tethys::prelude::*;
use crate::ship::get_corner;

const MAX_DISTANCE: f64 = 10.;
use crate::ship::{Panel, PanelLayout, PartLoader, ShipInterior};

pub struct PlacePanelState {
    interior: ShipInterior,
    panel: Panel,
    panel_layout: PanelLayout,
    display: bool,
    place_coords: Vec<(i32, i32, i32)>,// The coordinate on interior that should go where the mouse is
}
impl PlacePanelState {
    pub fn new(loader: PartLoader, panel: Panel) -> Self {
        let rigid_body = RigidBody::default();
        let layout = PanelLayout {
            
        };
        Self {
            interior:  ShipInterior::new(loader, Vec::new(), Vec::new(), vec![panel.clone()], vec![layout], rigid_body),
            display: false,
            place_coords: Vec::new(),
            panel,
            panel_layout: layout,
        }
    }

    pub(crate) fn update(&mut self, camera: &Camera, closest_ship: &mut ShipInterior) {
        self.display = false;

        // Get the intersection of the mouse pointer with the body
        let line = LineCollider::segment(camera.position, camera.get_forward::<f64>() * MAX_DISTANCE);
        let corner = get_corner(closest_ship.collider_package(), line);
        let corner = match corner {
            Some(c) => c,
            None => return
        };

        // Check to see if the panel can be placed
        if self.place_coords.len() == 1 {
            if !closest_ship.is_new_line_allowed([self.place_coords[0], corner]) { return; }
        } else if self.place_coords.len() >= 2 {
            let last = self.place_coords.len() - 1;
            if !closest_ship.is_new_panel_allowed([self.place_coords[last-1], self.place_coords[last], corner]) { return; }
        }

        // Show the panel
        self.place_coords.push(corner);
        self.interior.rigid_body.orientation = closest_ship.rigid_body.orientation;
        self.interior.update_graphics();
        self.display = true;
    }
    
    pub(crate) fn place(&mut self, loader: PartLoader, ship: &mut ShipInterior) {
        self.panel.vertices[0] = self.place_coords[0];
        self.panel.vertices[1] = self.place_coords[1];
        self.panel.vertices[2] = self.place_coords[2];
        ship.add_panel(loader, self.panel.clone(), self.panel_layout);
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