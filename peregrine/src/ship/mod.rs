use std::ops::Add;

use cgmath::{Deg, Quaternion, Rotation, Rotation3, Vector3};
use parts::{compose_orientations, ObjectInfo};
use tethys::{physics::collisions::CollisionBox, prelude::*};

mod parts;
pub use parts::{Part, PartLoader};

/// The physical position of a part
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

struct PartGrid {
    data: Vec<isize>,
    x: u32,
    y: u32,
    z: u32,
    cx: i32,// Where is the origin in these grid coordinates
    cy: i32,
    cz: i32
}
impl PartGrid {
    fn new() -> Self {
        Self {
            x: 0, y: 0, z: 0,
            cx: 0, cy: 0, cz: 0,
            data: Vec::new()
        }
    }

    fn with_capacity(x: u32, y: u32, z: u32) -> Self {
        Self {
            x, y, z,
            cx: 0, cy: 0, cz: 0,
            data: Vec::with_capacity((x * y * z) as usize)
        }
    }

    fn reshape(&mut self, x: u32, y: u32, z: u32, cx: i32, cy: i32, cz: i32) {
        let mut new_data = Vec::with_capacity((x*y*z) as usize);
        for (i, data) in self.data.iter().enumerate() {
            if *data < 0 {continue;}
            let xi = i as i32 % self.x as i32 - self.cx;
            let yi = (i as i32 / self.x as i32) % self.y as i32 - self.cy;
            let zi = i as i32 / (self.x * self.y) as i32 - self.cz;
            let new_index = ((xi+cx) as u32 + (yi+cy) as u32 * x + (zi+cz) as u32 * x * y) as usize;
            while new_data.len() <= new_index {
                new_data.push(-1);
            }
            new_data[new_index] = *data;
        }
        self.x = x;
        self.y = y;
        self.z = z;
        self.cx = cx;
        self.cy = cy;
        self.cz = cz;
        self.data = new_data;
    }

    fn get_index(&self, x: i32, y: i32, z: i32) -> usize {
        ((x+self.cx) as u32 + (y+self.cy) as u32 * self.x + (z+self.cz) as u32 * self.x * self.y) as usize
    }

    fn update(&mut self, layout: PartLayout, data: usize) {
        let underflow_x = 0.min(layout.x + self.cx);
        let underflow_y = 0.min(layout.y + self.cy);
        let underflow_z = 0.min(layout.z + self.cz);
        let overflow_x = (layout.x + 1).max(self.x as i32);
        let overflow_y = (layout.y + 1).max(self.y as i32);
        let overflow_z = (layout.z + 1).max(self.z as i32);
        if underflow_x < 0 || underflow_y < 0 || underflow_z < 0 || overflow_x as u32 >= self.x || overflow_y as u32 >= self.y || overflow_z as u32 >= self.z {
            self.reshape(
                (overflow_x - underflow_x) as u32,
                (overflow_y - underflow_y) as u32,
                (overflow_z - underflow_z) as u32,
                self.cx - underflow_x,
                self.cy - underflow_y,
                self.cz - underflow_z,
            );
        }
        let index = self.get_index(layout.x, layout.y, layout.z);
        while self.data.len() <= index {
            self.data.push(-1);
        }
        self.data[index] = data as isize;
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
    pub rigid_body: RigidBody,
}

impl ShipInterior {
    pub fn new(part_loader: &mut PartLoader, parts: Vec<Part>, layouts: Vec<PartLayout>, rigid_body: RigidBody) -> Self {
        let mut objects = Vec::with_capacity(parts.len());
        let mut grid = PartGrid::new();
        let mut boxes = Vec::with_capacity(parts.len());
        for (i, (part, layout)) in parts.iter().zip(&layouts).enumerate() {
            objects.append(&mut part.get_objects(part_loader, *layout, i));
            boxes.push(CollisionBox::new());
            for block in part.get_blocks(*layout) {
                grid.update(block, i);
            }
        }
        Self {
            parts,
            layouts,
            objects,
            grid,
            rigid_body,
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
        for object in &mut self.objects {
            let (position, orientation) = object.layout.as_physical();
            object.object.position = self.rigid_body.pos + self.rigid_body.orientation.rotate_vector(position);
            object.object.orientation = self.rigid_body.orientation * orientation;
        }
    }
    
    pub fn objects(&self) -> Vec<&Object> {
        self.objects.iter().map(|o| &o.object).collect::<Vec<_>>()
    }
    
    pub(crate) fn check_intersection(a: &ShipInterior, b: &ShipInterior) -> Option<Vector3<f64>> {
        Collider::check_intersection((&a.collider, &a.rigid_body).into(), (&b.collider, &b.rigid_body).into())
    }
}