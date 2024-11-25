use std::ops::Add;

use cgmath::{Quaternion, Rotation, Vector3};
use parts::{ObjectInfo, PartModel};
use tethys::{physics::collisions::ColliderPackage, prelude::*};

mod parts;
mod part_grid;
pub use part_grid::PartGrid;
pub use parts::{Part, PartData, PartLoader};

/// The physical position of an entire part, or the blocks within a part
#[derive(Clone, Copy, Debug)]
pub struct PartLayout {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub orientation: u8,
}
impl PartLayout {
    fn as_physical(&self) -> (Vector3<f64>, Quaternion<f64>) {
        (
            Vector3::new(self.x as f64, self.y as f64, self.z as f64),
            orientation::to_quat(self.orientation)
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
            orientation: orientation::compose(self.orientation, rhs.orientation)
        }
    }
}


/// Contains the data of a single ship, including its internal components, its hull model, its 
/// physics data, and its simulated properties
pub struct ShipInterior {
    parts: Vec<Part>,
    layouts: Vec<PartLayout>,
    grid: PartGrid, // Grid of cells that point to the part index of the part that's there
    collider: Collider,
    objects: Vec<ObjectInfo>,
    placement_objects: Option<Vec<Object>>,
    pub rigid_body: RigidBody,
}

impl ShipInterior {
    pub fn new(part_loader: PartLoader, parts: Vec<Part>, layouts: Vec<PartLayout>, rigid_body: RigidBody) -> Self {
        let mut objects = Vec::with_capacity(parts.len());
        let mut grid = PartGrid::new();
        let mut boxes = Vec::with_capacity(parts.len());
        for (i, (part, layout)) in parts.iter().zip(&layouts).enumerate() {
            objects.append(&mut part.get_objects(part_loader.clone(), *layout, i));
            boxes.push(part.get_collider(*layout));
            grid.update(part, *layout, i);
        }
        Self {
            parts,
            layouts,
            objects,
            grid,
            rigid_body,
            placement_objects: None,
            collider: Collider::make_tree(boxes),
        }
    }

    /// Update all the objects within the ship according to the physics component
    pub fn update(&mut self, delta_t: f64) {
        self.rigid_body.update(delta_t);
        self.update_graphics();
    }

    /// Update all the objects within the ship according to the physics component
    pub fn update_graphics(&mut self) {
        // TODO Remove this
        for object_info in &mut self.objects {
            let (position, orientation) = object_info.layout.as_physical();
            object_info.object.position = self.rigid_body.pos + self.rigid_body.orientation.rotate_vector(position);
            object_info.object.orientation = self.rigid_body.orientation * orientation;
        }
    }
    
    pub fn objects(&self) -> Vec<ObjectHandle> {
        self.objects.iter().map(|o| ObjectHandle::Ref(&o.object)).collect::<Vec<_>>()
    }
    
    /// Get the list of objects to be painted with the ``placing`` texture
    pub fn get_placement_objects(&self) -> Vec<ObjectHandle> {
        self.placement_objects.as_ref().unwrap().iter().map(|o| ObjectHandle::Ref(&o)).collect::<Vec<_>>()
    }

    /// Get the ship ready for placing parts
    pub fn initialize_placement(&mut self, part_loader: PartLoader) {
        if self.placement_objects.is_none() {
            let mut objects = Vec::new();
            let placement_model = part_loader.load(PartModel::Placement);
            let orientation = Quaternion::new(1., 0., 0., 0.,);
            for ((x, y, z), part_number) in self.grid.indexed_iter() {
                if part_number == -1 {continue;}
                let pos = Vector3::new(x as f64, y as f64,z as f64);
                objects.push(Object::new(part_loader.graphics, placement_model.clone(), pos, orientation));
            }
            self.placement_objects = Some(objects);
        }
    }

    pub(crate) fn collider_package(&self) -> ColliderPackage {
        (&self.collider, &self.rigid_body).into()
    }
    
    pub(crate) fn is_new_part_allowed(&self, part: Part, layout: PartLayout) -> bool {
        for block in part.get_blocks(layout) {
            if self.grid.get_entry(block.x, block.y, block.z) != -1 {
                return false;
            }
        }
        true
    }
    
    pub(crate) fn add_part(&mut self, part_loader: PartLoader, part: Part, layout: PartLayout) {
        let part_index = self.parts.len();
        self.parts.push(part);
        self.layouts.push(layout);
        self.grid.update(&part, layout, part_index);
        let mut objects = part.get_objects(part_loader.clone(), layout, part_index);
        self.objects.append(&mut objects);
        if let Some(objects) = &mut self.placement_objects {
            let placement_model = part_loader.load(PartModel::Placement);
            let orientation = Quaternion::new(1., 0., 0., 0.,);
            for block in part.get_blocks(layout) {
                let pos = Vector3::new(block.x as f64, block.y as f64, block.z as f64);
                objects.push(Object::new(part_loader.graphics, placement_model.clone(), pos, orientation));
            }
        }
        
        // Update collider
        let mut boxes = Vec::with_capacity(self.parts.len());
        for (part, layout) in self.parts.iter().zip(&self.layouts) {
            boxes.push(part.get_collider(*layout));
        }
        self.collider = Collider::make_tree(boxes);
    }
}

/// Helps manage the orientation of a part
pub mod orientation {
    // OPTIMIZE
    use core::f64;

    use cgmath::{Deg, InnerSpace, Quaternion, Rotation, Rotation3, Vector3};
    const RZ0: Quaternion<f64> = Quaternion::new(1., 0., 0., 0.);
    const RZ1: Quaternion<f64> = Quaternion::new(0., 0., 0., 1.);
    const RZ2: Quaternion<f64> = Quaternion::new(-1., 0., 0., 0.);
    const RZ3: Quaternion<f64> = Quaternion::new(0., 0., 0., -1.);

    pub fn from_quat(q: Quaternion<f64>) -> u8 {
        let possible_quats = [
            RZ0,
            RZ1,
            RZ2,
            RZ3,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ0,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ1,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ2,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ3,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ0,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ1,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ2,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ3,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ0,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ1,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ2,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ3,
        ];
        let mut best_index = 0;
        let mut best_mag2 = f64::INFINITY;
        for (i, r) in possible_quats.iter().enumerate() {
            let mag2 = (q - r).magnitude2();
            if mag2 < best_mag2 {
                best_index = i as u8;
                best_mag2 = mag2;
            }
        }
        best_index
    }

    pub fn to_quat(orientation: u8) -> Quaternion<f64> {
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

    pub fn compose(a: u8, b: u8) -> u8 {
        from_quat(to_quat(a) * to_quat(b))
    }

    pub fn rotate_by_quat(a: u8, q: Quaternion<f64>) -> u8 {
        from_quat(to_quat(a) * q)
    }
    
    /// Rotates a set of integers around the origin
    pub(crate) fn rotate_integer(orientation: u8, x: i32, y: i32, z: i32) -> (i32, i32, i32) {
        let quat = to_quat(orientation);
        let vec = Vector3::new(x as f64, y as f64, z as f64);
        let point = quat.rotate_vector(vec);
        (
            point.x.round() as i32,
            point.y.round() as i32,
            point.z.round() as i32,
        )
    }
}