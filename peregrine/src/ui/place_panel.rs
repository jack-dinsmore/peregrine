use tethys::prelude::*;
use crate::ship::{get_corner, PanelModel, SaveShipInterior};

const MAX_DISTANCE: f64 = 10.;
use crate::ship::{Panel, PanelLayout, PartLoader, ShipInterior};
use crate::util::Save;

pub struct PlacePanelState {
    interior: ShipInterior,
    panel_layout: PanelLayout,
    display: bool,
    place_coords: Vec<(i32, i32, i32)>,// The coordinate on interior that should go where the mouse is
    cursor_pos: (i32, i32, i32),
}
impl PlacePanelState {
    pub fn new(loader: PartLoader, panel_model: PanelModel) -> Self {
        let rigid_body = RigidBody::default();
        let panel = Panel {
            vertices: [(0, 0, 0); 3],
            panel_model,
        };
        let layout = PanelLayout {
            
        };
        let save = SaveShipInterior {
            parts: Vec::new(),
            part_layouts: Vec::new(),
            panels: vec![panel.clone()],
            panel_layouts: vec![layout.clone()],
            rigid_body,
        };
        Self {
            interior:  save.build(loader),
            display: false,
            place_coords: Vec::new(),
            panel_layout: layout,
            cursor_pos: (0, 0, 0),
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
        self.cursor_pos = corner;
        if self.place_coords.len() <= 1 {return;} // There's nothing to show

        // Show the panel
        let last_index = self.place_coords.len()-1;
        if self.place_coords[last_index] != corner {
            self.interior.panels[0].vertices[2] = corner; // Change the panel vertices
            // TODO recalculate the object
        }
        self.interior.rigid_body.pos = closest_ship.rigid_body.pos;
        self.interior.rigid_body.orientation = closest_ship.rigid_body.orientation;
        self.interior.update_graphics();
        self.display = true;
    }
    
    /// Add the panel vertex. If fewer than three vertices have been selected so far, this will not place the panel.
    pub(crate) fn place(&mut self, loader: PartLoader, ship: &mut ShipInterior) {
        let panel = &mut self.interior.panels[0];
        if self.place_coords.len() == 0 {
            panel.vertices[0] = self.cursor_pos;
        } else if self.place_coords.len() == 1 {
            panel.vertices[1] = self.cursor_pos;
        } else {
            panel.vertices[2] = self.cursor_pos;
            ship.add_panel(loader, panel.clone(), self.panel_layout);
            panel.vertices[0] = panel.vertices[1]; 
            panel.vertices[1] = panel.vertices[2]; 
        }
        self.place_coords.push(self.cursor_pos);
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