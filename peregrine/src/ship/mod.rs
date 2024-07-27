use cgmath::{Deg, Quaternion, Rotation, Rotation3, Vector3};
use tethys::prelude::*;

mod parts;
pub use parts::{Part, PartLoader};

pub struct PartArray {
    x: usize,
    y: usize,
    z: usize,
    parts: Vec<Part>,
    orientations: Vec<u8>,
}
impl PartArray {
    pub fn new() -> Self {
        Self {
            x: 1,
            y: 3,
            z: 1,
            parts: vec![Part::TankCap, Part::TankBody, Part::TankCap],
            orientations: vec![12, 0, 0],
        }
    }

    fn get_index(&self, index: (usize, usize, usize)) -> usize {
        index.0 + self.x * index.1 + self.x * self.y * index.2
    }

    pub fn iter<'a>(&'a self) -> PartArrayIterator<'a> {
        PartArrayIterator {
            pointer: self,
            index: 0
        }
    }

    pub fn get_part(&self, index: (usize, usize, usize)) -> Part {
        self.parts[self.get_index(index)]
    }

    pub fn get_orientation(&self, index: (usize, usize, usize)) -> u8 {
        self.orientations[self.get_index(index)]
    }
}

pub struct PartArrayIterator<'a> {
    pointer: &'a PartArray,
    index: usize,
}
impl<'a> Iterator for PartArrayIterator<'a> {
    type Item = (usize, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.pointer.parts.len() {return None;}
        let x = self.index % self.pointer.x;
        let y = (self.index / self.pointer.x) % self.pointer.y;
        let z = self.index / (self.pointer.x * self.pointer.y);
        self.index += 1;
        Some((x, y, z))
    }
}

/// Contains the data of a single ship, including its internal components, its hull model, its 
/// physics data, and its simulated properties
pub struct Ship {
    objects: Vec<Object>,
    part_array: PartArray,
    rigid_body: RigidBody,
}

impl Ship {
    pub fn new(part_loader: &mut PartLoader, part_array: PartArray, rigid_body: RigidBody) -> Self {
        let mut objects = Vec::new();
        for index in part_array.iter() {
            let part = part_array.get_part(index);
            let orientation = get_orientation_quat(part_array.get_orientation(index));
            
            let model = part_loader.get_part(part).clone();
            let object = Object::new(part_loader.graphics, model, Vector3::new(index.0 as f64, index.1 as f64, index.2 as f64), orientation);
            objects.push(object);
        }
        Self {
            objects,
            part_array,
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
        for index in self.part_array.iter() {
            let orientation = self.part_array.get_orientation(index);
            let object = &mut self.objects[self.part_array.get_index(index)];
            let local_vector = Vector3::new(index.0 as f64, index.1 as f64, index.2 as f64);
            object.position = self.rigid_body.pos + self.rigid_body.orientation.rotate_vector(local_vector);
            object.orientation = self.rigid_body.orientation * get_orientation_quat(orientation);
        }
    }
    
    pub fn objects(&self) -> Vec<&Object> {
        self.objects.iter().collect::<Vec<_>>()
    }
}

fn get_orientation_quat(orientation: u8) -> Quaternion<f64> {
    const RZ0: Quaternion<f64> = Quaternion::new(1., 0., 0., 0.);
    const RZ1: Quaternion<f64> = Quaternion::new(0., 0., 0., 1.);
    const RZ2: Quaternion<f64> = Quaternion::new(-1., 0., 0., 0.);
    const RZ3: Quaternion<f64> = Quaternion::new(0., 0., 0., -1.);
    match orientation {
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
}