use cgmath::{InnerSpace, Quaternion, Vector3};
use tethys::prelude::*;

use crate::{ship::{Part, PartLayout, PartLoader, ShipInterior}, util::vector_cast};

const MIN_DISTANCE: f32 = 0.5;
const MAX_DISTANCE: f32 = 5.;

pub struct PlacementState {
    interior: ShipInterior,
    pub desire_distance: f32,
    distance: f32,
    display: bool,
}

impl PlacementState {
    pub fn new(graphics: &Graphics, part: Part) -> Self {
        let mut part_loader = PartLoader::new(graphics);
        let rigid_body = RigidBody::default();
        let layout = PartLayout { x: 0, y: 0, z: 0, orientation: 0 };
        Self {
            interior:  ShipInterior::new(&mut part_loader, vec![part], vec![layout], rigid_body),
            desire_distance: 1.,
            distance: 1.,
            display: true,
        }
    }

    pub fn rotate(&mut self, axis: Vector3<f32>) {
        self.interior.rigid_body.orientation = self.interior.rigid_body.orientation *
        if axis.x.abs() > axis.y.abs() && axis.x.abs() > axis.z.abs() {
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
    }

    pub fn update(&mut self, camera: &Camera, closest_ship: &ShipInterior) {
        self.desire_distance = self.desire_distance.clamp(MIN_DISTANCE, MAX_DISTANCE);
        let look = camera.get_forward();
        
        // Find distance
        self.distance = self.desire_distance;
        self.display = true;
        let mut iterations = 0;
        self.interior.rigid_body.pos = vector_cast(vector_cast(camera.position) + look * self.desire_distance);
        while let Some(depth) = ShipInterior::check_intersection(closest_ship, &self.interior) {
            // Move the object closer
            let projection = look / vector_cast(depth).dot(look);
            self.distance -= projection.magnitude();
            self.interior.rigid_body.pos = vector_cast(vector_cast(camera.position) + look * self.desire_distance);

            // TODO snap to grid and allow placement

            // Complete the loop
            iterations += 1;
            if iterations >= 10 || self.distance < MIN_DISTANCE {
                self.display = false;
                break;
            }
        };

        self.interior.update_graphics();
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