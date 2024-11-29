use cgmath::{InnerSpace, Quaternion, Rotation, Vector3};
use tethys::prelude::*;

use crate::{ship::{orientation::{self, from_quat}, Part, PartLayout, PartLoader, SaveShipInterior, ShipInterior}, util::Save};

const MAX_DISTANCE: f64 = 5.;

pub struct PlacePartState {
    interior: ShipInterior,
    part: Part,
    display: bool,
    place_coord: Vector3<f64>,// The coordinate on interior that should go where the mouse is
    part_orientation: u8, // Orientation the part would have
    layout: Option<PartLayout>,
}

impl PlacePartState {
    pub fn new(part_loader: PartLoader, part: Part) -> Self {
        let rigid_body = RigidBody::default();
        let layout = PartLayout { x: 0, y: 0, z: 0, orientation: 0 };
        let save = SaveShipInterior {
            parts: vec![part.clone()],
            part_layouts: vec![layout.clone()],
            panels: Vec::new(),
            panel_layouts: Vec::new(),
            rigid_body,
        };
        Self {
            part_orientation: from_quat(save.rigid_body.orientation),
            interior:  save.build(part_loader),
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
        let forward = camera.get_forward::<f64>().normalize();
        let line = Collider::Line(
            LineCollider::segment(camera.position, forward * MAX_DISTANCE)
        );
        let result = Collider::check_intersection(closest_ship.collider_package(), (&line).into());
        if !result.collision() { return; }

        // Check to see if the part can be placed
        let mut pos_in_grid = closest_ship.rigid_body.to_local(result.positions[0] - forward * 0.001) - self.place_coord;
        if forward.x > 0. {
            pos_in_grid.x = pos_in_grid.x.floor();
        } else {
            pos_in_grid.x = pos_in_grid.x.ceil();
        }
        if forward.y > 0. {
            pos_in_grid.y = pos_in_grid.y.floor();
        } else {
            pos_in_grid.y = pos_in_grid.y.ceil();
        }
        if forward.z > 0. {
            pos_in_grid.z = pos_in_grid.z.floor();
        } else {
            pos_in_grid.z = pos_in_grid.z.ceil();
        }
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

    pub fn place(&self, part_loader: PartLoader, closest_ship: &mut ShipInterior) {
        if let Some(layout) = self.layout {
            closest_ship.add_part(part_loader, self.part, layout);
        }
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