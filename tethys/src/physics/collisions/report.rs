use std::ops::AddAssign;

use cgmath::{InnerSpace, Vector3};

use crate::physics::RigidBody;

use super::{reorient, reorient_rot, Collider};


#[derive(Clone, Copy)]
pub struct ColliderPackage<'a> {
    pub collider: &'a Collider,
    pub rigid_body: Option<&'a RigidBody>,
}
impl<'a> From<(&'a Collider, &'a RigidBody)> for ColliderPackage<'a> {
    fn from((collider, rigid_body): (&'a Collider, &'a RigidBody)) -> Self {
        Self {
            collider,
            rigid_body: Some(rigid_body),
        }
    }
}
impl<'a> From<(&'a RigidBody, &'a Collider)> for ColliderPackage<'a> {
    fn from((rigid_body, collider): (&'a RigidBody, &'a Collider)) -> Self {
        Self {
            collider,
            rigid_body: Some(rigid_body),
        }
    }
}

impl<'a> From<&'a Collider> for ColliderPackage<'a> {
    fn from(collider: &'a Collider) -> Self {
        Self {
            collider,
            rigid_body: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollisionReport {
    /// Vector that points from the surface of the shape to the collision point
    pub depths: Vec<Vector3<f64>>,
    pub positions: Vec<Vector3<f64>>,
    /// Value of the grid if collided with a grid
    pub index: Vec<isize>,
}

impl CollisionReport {
    pub fn none() -> Self {
        Self {
            depths: Vec::new(),
            positions: Vec::new(),
            index: Vec::new(),
        }
    }
    pub fn new(depth: Vector3<f64>, position: Vector3<f64>) -> Self {
        Self {
            depths: vec![depth],
            positions: vec![position],
            index: Vec::new(),
        }
    }
    /// Rotate from a given body to inertial
    pub(crate) fn reorient(&self, from: Option<&RigidBody>) -> Self {
        Self {
            depths: self.depths.iter().map(|v| reorient_rot(*v, from, None)).collect(),
            positions: self.positions.iter().map(|v| reorient(*v, from, None)).collect(),
            index: Vec::new(),
        }
    }
    /// Rotate into the frame of a given body
    pub fn orient(&self, to: &RigidBody) -> Self {
        Self {
            depths: self.depths.iter().map(|v| reorient_rot(*v, None, Some(to))).collect(),
            positions: self.positions.iter().map(|v| reorient(*v, None, Some(to))).collect(),
            index: Vec::new(),
        }
    }
    
    pub fn collision(&self) -> bool {
        !self.positions.is_empty()
    }

    fn deepest_mag2(&self) -> f64 {
        self.positions.iter().fold(0., |accum, v| v.magnitude2().max(accum))
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

impl AddAssign for CollisionReport {
    fn add_assign(&mut self, mut rhs: Self) {
        self.depths.append(&mut rhs.depths);
        self.positions.append(&mut rhs.positions);
        self.index.append(&mut rhs.index);
    }
}
impl PartialEq for CollisionReport {
    fn eq(&self, other: &Self) -> bool {
        self.deepest_mag2().eq(&other.deepest_mag2())
    }
}
impl PartialOrd for CollisionReport {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deepest_mag2().partial_cmp(&other.deepest_mag2())
    }
}