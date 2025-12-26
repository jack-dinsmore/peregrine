use cgmath::{Quaternion, Vector3};
use tethys::prelude::*;

use crate::ship::{orientation, PartLoader, ShipInterior};

pub const PLACEMENT_REACH: f64 = 5.;

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
const PLACEMENT_INDICES: [u16; 3] = [
    0, 1, 2,//, 1, 3, 3, 2, 2, 0,
    // 4, 5, 5, 7, 7, 6, 6, 4,
    // 0, 4, 1, 5, 2, 6, 3, 7,
];

/// Find the grid cell which contains this point
pub fn grid_shrink(mut vec: Vector3<f64>, forward: Vector3<f64>) -> Vector3<f64> {
    if forward.x > 0. {
        vec.x = vec.x.floor();
    } else {
        vec.x = vec.x.ceil();
    }
    if forward.y > 0. {
        vec.y = vec.y.floor();
    } else {
        vec.y = vec.y.ceil();
    }
    if forward.z > 0. {
        vec.z = vec.z.floor();
    } else {
        vec.z = vec.z.ceil();
    }
    vec
}

pub struct PlacementTools {
    display: bool,
    pub interior: ShipInterior,
    pub roll: u8,
    pub ship_location: Option<Vector3<f64>>,

    placement_model: Model,
    placement_objects: Vec<Object>,
}
impl PlacementTools {
    pub fn new(part_loader: PartLoader, interior: ShipInterior, ship: &ShipInterior) -> Self {
        let placement_model = Model::from_vertices(&part_loader.graphics, &PLACEMENT_VERTICES, &PLACEMENT_INDICES);
        let mut placement_objects = Vec::new();
        for (_, part_number) in ship.collider.get_grid_collider().unwrap().indexed_iter() {
            if part_number == -1 {continue;}
            placement_objects.push(Object::zeroed::<ObjectUniform>(&part_loader.graphics, placement_model.clone()));
        }

        Self {
            display: false,
            interior,
            roll: 0,
            ship_location: None,
            placement_model,
            placement_objects,
        }
    }

    /// Rotate the ship
    pub fn rotate(&mut self, axis: Vector3<f64>) {
        let reorient = if axis.x.abs() > axis.y.abs() && axis.x.abs() > axis.z.abs() {
            Quaternion::new(0., axis.x.signum(), 0., 0.)
        } else if axis.y.abs() > axis.x.abs() && axis.y.abs() > axis.z.abs() {
            Quaternion::new(0., 0., axis.y.signum(), 0.)
        } else {
            Quaternion::new(0., 0., 0., axis.z.signum())
        };
        self.roll = orientation::rotate_by_quat(self.roll, reorient);
    }

    /// Update the placement ship position and update placement objects to follow the ship
    pub fn update(&mut self, graphics: &Graphics, camera: &Camera, ship: &ShipInterior, pos_in_grid: Option<Vector3<f64>>) {
        // Show the part
        match pos_in_grid {
            Some(pos) => {
                self.interior.rigid_body.orientation = ship.rigid_body.orientation * orientation::to_quat(self.roll);
                self.interior.rigid_body.pos = ship.rigid_body.to_global(pos);
                self.display = true;
            },
            None => {
                self.display = false;
            },
        }

        self.interior.update_graphics(graphics, camera);
        let mut i = 0;
        let orientation = Quaternion::new(1., 0., 0., 0.,);
        for ((x, y, z), part_number) in ship.collider.get_grid_collider().unwrap().indexed_iter() {
            if part_number < 0 {continue;}
            let pos = Vector3::new(x as f64 + 0.5, y as f64 + 0.5,z as f64 + 0.5);
            self.placement_objects[i].update(graphics, ObjectUniform::new(camera, pos, orientation));
            i += 1;
        }
    }

    /// Get all renderable objects
    pub fn get_placement_objects(&self) -> Vec<ObjectHandle<'_>> {
        let mut placement_objects = self.placement_objects.iter().map(|o| ObjectHandle::Ref(&o)).collect::<Vec<_>>();
        if self.display {
            placement_objects.append(&mut self.interior.objects());
        }
        placement_objects
    }

    /// Add a block to the ship
    pub fn add_block(&mut self, part_loader: PartLoader, n_blocks: usize) {
        for _ in 0..n_blocks {
            self.placement_objects.push(Object::zeroed::<ObjectUniform>(part_loader.graphics, self.placement_model.clone()));
        }
    }
}