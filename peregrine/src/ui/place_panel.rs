use cgmath::{Quaternion, Vector3};
use tethys::prelude::*;
use crate::ship::{get_corner, PanelModel, SaveShipInterior};

const MAX_DISTANCE: f64 = 10.;
use crate::ship::{Panel, PanelLayout, PartLoader, ShipInterior};
use crate::util::Save;

pub struct PlacePanelState {
    // Panel information
    interior: ShipInterior,
    panel_layout: PanelLayout,
    display: bool,
    place_coords: Vec<(i32, i32, i32)>,// The coordinate on interior that should go where the mouse is
    cursor_pos: (i32, i32, i32),

    // Placement information
    placement_model: Model,
    placement_objects: Vec<Object>,
}
impl PlacePanelState {
    pub fn new(loader: PartLoader, panel_model: PanelModel, ship: &ShipInterior) -> Self {
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

        // Initialize the placement blocks
        const PLACEMENT_VERTICES: [PointVertex ; 8] = [
            PointVertex { position: [0.5, 0.5, 0.5] },
            PointVertex { position: [-0.5, 0.5, 0.5] },
            PointVertex { position: [0.5, -0.5, 0.5] },
            PointVertex { position: [-0.5, -0.5, 0.5] },
            PointVertex { position: [0.5, 0.5, -0.5] },
            PointVertex { position: [-0.5, 0.5, -0.5] },
            PointVertex { position: [0.5, -0.5, -0.5] },
            PointVertex { position: [-0.5, -0.5, -0.5] },
        ];
        const PLACEMENT_INDICES: [u16; 24] = [
            0, 1, 1, 3, 3, 2, 2, 0,
            4, 5, 5, 7, 7, 6, 6, 4,
            0, 4, 1, 5, 2, 6, 3, 7,
        ];
        let placement_model = Model::from_vertices(&loader.graphics, &PLACEMENT_VERTICES, &PLACEMENT_INDICES);
        let mut placement_objects = Vec::new();
        for (_, part_number) in ship.collider.get_grid_collider().unwrap().indexed_iter() {
            if part_number == -1 {continue;}
            placement_objects.push(Object::zeroed::<ObjectUniform>(loader.graphics, placement_model.clone()));
        }

        Self {
            interior:  save.build(loader),
            display: false,
            place_coords: Vec::new(),
            panel_layout: layout,
            cursor_pos: (0, 0, 0),
            placement_model,
            placement_objects,
        }
    }

    pub(crate) fn update(&mut self, graphics: &Graphics, camera: &Camera, closest_ship: &ShipInterior) {
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
        self.display = true;
        self.update_graphics(graphics, camera, closest_ship);
    }

    pub fn update_graphics(&self, graphics: &Graphics, camera: &Camera, ship: &ShipInterior) {
        self.interior.update_graphics(graphics, camera);

        let mut i = 0;
        let orientation = Quaternion::new(1., 0., 0., 0.,);
        for ((x, y, z), part_number) in ship.collider.get_grid_collider().unwrap().indexed_iter() {
            if part_number == -1 {continue;}
            let pos = Vector3::new(x as f64 + 0.5, y as f64 + 0.5,z as f64 + 0.5);
            self.placement_objects[i].update(graphics, ObjectUniform::new(camera, pos, orientation));
            i += 1;
        }
    }

    pub fn get_placement_objects(&self) -> Vec<ObjectHandle> {
        self.placement_objects.iter().map(|o| ObjectHandle::Ref(&o)).collect::<Vec<_>>()
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