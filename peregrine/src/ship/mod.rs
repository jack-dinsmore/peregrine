use std::ops::Add;

use cgmath::{Deg, Quaternion, Rotation, Rotation3, Vector3};
use parts::{compose_orientations, ObjectInfo};
use tethys::prelude::*;

mod parts;
pub use parts::{Part, PartLoader};

#[derive(Clone, Copy)]
pub struct PartLayout {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub orientation: u8,
}
impl PartLayout {
    fn as_physical(&self) -> (Vector3<f64>, Quaternion<f64>) {
        const RZ0: Quaternion<f64> = Quaternion::new(1., 0., 0., 0.);
        const RZ1: Quaternion<f64> = Quaternion::new(0., 0., 0., 1.);
        const RZ2: Quaternion<f64> = Quaternion::new(-1., 0., 0., 0.);
        const RZ3: Quaternion<f64> = Quaternion::new(0., 0., 0., -1.);
        (
            Vector3::new(self.x as f64, self.y as f64, self.z as f64),
            match self.orientation {
                0 => RZ0,
                1 => RZ1,
                2 => RZ2,
                3 => RZ3,

                4 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ0,
                5 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ1,
                6 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ2,
                7 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ3,

                8 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ0,
                9 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ1,
                10 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ2,
                11 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ3,

                12 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ0,
                13 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ1,
                14 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ2,
                15 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ3,
                _ => panic!("Orientation not supported"),
            }
        )
    }
}
impl Add for PartLayout {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            orientation: compose_orientations(self.orientation, rhs.orientation)
        }
    }
}


/// Contains the data of a single ship, including its internal components, its hull model, its 
/// physics data, and its simulated properties
pub struct Ship {
    parts: Vec<Part>,
    layout: Vec<PartLayout>,
    objects: Vec<ObjectInfo>,
    rigid_body: RigidBody,
}

impl Ship {
    pub fn new(part_loader: &mut PartLoader, parts: Vec<Part>, layout: Vec<PartLayout>, rigid_body: RigidBody) -> Self {
        let mut objects = Vec::new();
        for (i, (part, layout)) in parts.iter().zip(&layout).enumerate() {
            objects.append(&mut part.get_objects(part_loader, *layout, i));
        }
        Self {
            parts,
            layout,
            objects,
            rigid_body
        }
    }

    /// Update all the objects within the ship according to the physics component
    pub fn update(&mut self, delta_t: f64) {
        self.rigid_body.update(delta_t);
        self.update_graphics();
    }

    /// Update all the objects within the ship according to the physics component
    fn update_graphics(&mut self) {
        for object in &mut self.objects {
            let (position, orientation) = object.layout.as_physical();
            object.object.position = self.rigid_body.pos + self.rigid_body.orientation.rotate_vector(position);
            object.object.orientation = self.rigid_body.orientation * orientation;
        }
    }
    
    pub fn objects(&self) -> Vec<&Object> {
        self.objects.iter().map(|o| &o.object).collect::<Vec<_>>()
    }
}