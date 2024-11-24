use std::ops::Add;

use cgmath::{Quaternion, Rotation, Vector3};
use parts::{ObjectInfo, PartModel};
use tethys::{physics::collisions::{ColliderPackage, GridCollider}, prelude::*};

mod parts;
mod grid;
pub mod orientation;

pub use parts::{Part, PartData, PartLoader};
use grid::*;

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
    collider: Collider,
    objects: Vec<ObjectInfo>,
    placement_objects: Option<Vec<Object>>,
    pub rigid_body: RigidBody,
}

impl ShipInterior {
    pub fn new(part_loader: PartLoader, parts: Vec<Part>, layouts: Vec<PartLayout>, rigid_body: RigidBody) -> Self {
        let mut objects = Vec::with_capacity(parts.len());
        let mut grid = GridCollider::new();
        for (i, (part, layout)) in parts.iter().zip(&layouts).enumerate() {
            objects.append(&mut part.get_objects(part_loader.clone(), *layout, i));
            update_grid(&mut grid, part, *layout, i);
        }
        Self {
            parts,
            layouts,
            objects,
            collider: Collider::Grid(grid),
            rigid_body,
            placement_objects: None,
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
            for ((x, y, z), part_number) in self.collider.get_grid_collider().unwrap().indexed_iter() {
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
            if self.collider.get_grid_collider().unwrap().get_entry(block.x, block.y, block.z) != -1 {
                return false;
            }
        }
        true
    }
    
    pub(crate) fn add_part(&mut self, part_loader: PartLoader, part: Part, layout: PartLayout) {
        let part_index = self.parts.len();
        self.parts.push(part);
        self.layouts.push(layout);
        update_grid(self.collider.get_grid_collider_mut().unwrap(), &part, layout, part_index);
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
    }
}