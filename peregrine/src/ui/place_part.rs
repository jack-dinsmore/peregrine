use cgmath::Vector3;
use tethys::prelude::*;

use crate::{ship::{orientation, Part, PartLayout, PartLoader, SaveShipInterior, ShipInterior}, ui::place_tools::{grid_shrink, PlacementTools, PLACEMENT_REACH}, util::Save};

pub struct PlacePartState {
    // Part information
    tools: PlacementTools,
}



impl PlacePartState {
    pub fn new(part_loader: PartLoader, part: Part, ship: &ShipInterior) -> Self {
        // Initialize the new part
        let layout = PartLayout { x: 0, y: 0, z: 0, orientation: 0 };
        let save = SaveShipInterior {
            parts: vec![part.clone()],
            part_layouts: vec![layout.clone()],
            ..Default::default()
        };

        Self {
            tools: PlacementTools::new(part_loader.clone(), save.build(part_loader), ship),
        }
    }

    pub fn get_placed_layout(&self) -> Option<PartLayout> {
        let pos = match self.tools.ship_location {
            Some(loc) => loc,
            None => return None,
        };
        let layout = self.tools.interior.part_layouts[0];
        let layout_pos = orientation::rotate_integer(layout.orientation, layout.x, layout.y, layout.z);
        Some(PartLayout {
            x: pos.x.round() as i32 - layout_pos.0,
            y: pos.y.round() as i32 - layout_pos.1,
            z: pos.z.round() as i32 - layout_pos.2,
            orientation: orientation::compose(self.tools.roll, layout.orientation),
        })
    }
    
    pub fn rotate(&mut self, axis: Vector3<f64>) {
        self.tools.rotate(axis)
    }

    pub fn update(&mut self, graphics: &Graphics, camera: &Camera, ship: &ShipInterior) {
        let forward = camera.get_forward::<f64>();
        let line = Collider::Line(
            LineCollider::segment(camera.position, forward * PLACEMENT_REACH)
        );
        let result = Collider::check_intersection(ship.collider_package(), (&line).into());
        let pos_in_grid = if result.collision() { 
            // Check to see if the part can be placed
            let mut pos_in_grid = ship.rigid_body.to_local(result.positions[0] - forward * 0.001);
            pos_in_grid = grid_shrink(pos_in_grid, forward);
            
            // Temporarily add the position to the ship to see if it's allowed
            self.tools.ship_location = Some(pos_in_grid);
            if ship.is_new_part_allowed(self.tools.interior.parts[0], self.get_placed_layout().unwrap()) { 
                Some(pos_in_grid)
            } else {
                None
            }
        } else {
            None
        };

        self.tools.update(graphics, camera, ship, pos_in_grid);
    }
    
    pub fn place(&mut self, part_loader: PartLoader, ship: &mut ShipInterior) {
        // Add the part
        let part = self.tools.interior.parts[0];
        if let Some(layout) = self.get_placed_layout() {
            ship.add_part(part_loader.clone(), part, layout);
            self.tools.add_block(part_loader, part.get_blocks(layout).len());
        }
    }

    pub fn get_objects(&self) -> Vec<ObjectHandle<'_>> {
        self.tools.get_placement_objects()
    }
}