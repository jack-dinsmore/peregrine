use cgmath::{InnerSpace, Quaternion, Rotation, Vector3};
use tethys::prelude::*;

use crate::ship::{orientation::{self, from_quat}, Part, PartLayout, PartLoader, ShipInterior};

const MAX_DISTANCE: f32 = 5.;

pub struct PlacementState {
    interior: ShipInterior,
    part: Part,
    display: bool,
    place_coord: Vector3<f64>,// The coordinate on interior that should go where the mouse is
    part_orientation: u8, // Orientation the part would have
    layout: Option<PartLayout>,
}

impl PlacementState {
    pub fn new(graphics: &Graphics, part: Part) -> Self {
        let mut part_loader = PartLoader::new(graphics);
        let rigid_body = RigidBody::default();
        let layout = PartLayout { x: 0, y: 0, z: 0, orientation: 0 };
        Self {
            part_orientation: from_quat(rigid_body.orientation),
            interior:  ShipInterior::new(&mut part_loader, vec![part.clone()], vec![layout], rigid_body),
            display: false,
            place_coord: Vector3::new(0., 0., 0.),
            part,
            layout: None,
        }
    }

    pub fn rotate(&mut self, axis: Vector3<f32>) {
        let reorient = if axis.x.abs() > axis.y.abs() && axis.x.abs() > axis.z.abs() {
            if axis.x > 0. {
                // Rotate around +x
                Quaternion::new(0., 1., 0., 0.)
            } else {
                // Rotate around -x
                Quaternion::new(0., 1., 0., 0.)
            }
        } else if axis.y.abs() > axis.x.abs() && axis.y.abs() > axis.z.abs() {
            if axis.y > 0. {
                // Rotate around +y
                Quaternion::new(0., 0., 1., 0.)
            } else {
                // Rotate around -y
                Quaternion::new(0., 0., -1., 0.)
            }
        } else {
            if axis.z > 0. {
                // Rotate around +z
                Quaternion::new(0., 0., 0., 1.)
            } else {
                // Rotate around -z
                Quaternion::new(0., 0., 0., -1.)
            }
        };
        self.place_coord = reorient.rotate_vector(self.place_coord);
        self.part_orientation = orientation::rotate_by_quat(self.part_orientation, reorient);
    }

    pub fn update(&mut self, camera: &Camera, closest_ship: &ShipInterior) {
        self.display = false;
        
        // Get the intersection of the mouse pointer with the body
        let line = Collider::Ray{p: camera.position, v: camera.get_forward()};
        let result = Collider::check_intersection(closest_ship.collider_package(), (&line).into());
        if !result.collision() { return; }

        // Make sure it isn't too far away
        let dist = (result.positions[0] - camera.position).magnitude() as f32;
        if dist > MAX_DISTANCE { return; }

        // Check to see if the part can be placed
        let mut pos_in_grid = closest_ship.rigid_body.to_local(result.positions[0] - camera.get_forward().normalize() * 0.1) - self.place_coord;
        pos_in_grid.x = pos_in_grid.x.round();
        pos_in_grid.y = pos_in_grid.y.round();
        pos_in_grid.z = pos_in_grid.z.round();
        let layout = PartLayout {
            x: pos_in_grid.x as i32,
            y: pos_in_grid.y as i32,
            z: pos_in_grid.z as i32,
            orientation: self.part_orientation,
        };
        if !closest_ship.is_new_part_allowed(self.part, layout) { return; }

        // Show the part
        self.layout = Some(layout);
        self.interior.rigid_body.orientation = closest_ship.rigid_body.orientation * orientation::to_quat(self.part_orientation);
        self.interior.rigid_body.pos = closest_ship.rigid_body.to_global(pos_in_grid);
        self.interior.update_graphics();
        self.display = true;
    }

    pub fn place(&self, graphics: &Graphics, closest_ship: &mut ShipInterior) {
        if let Some(layout) = self.layout {
            closest_ship.add_part(graphics, self.part, layout);
        }
    }
    
    pub fn object(&self) -> Vec<&Object> {
        if self.display {
            self.interior.objects()
        }
        else {
            Vec::new()
        }
    }
}