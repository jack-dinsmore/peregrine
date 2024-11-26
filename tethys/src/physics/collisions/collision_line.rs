use cgmath::Vector3;

use crate::physics::RigidBody;

use super::{reorient, reorient_rot};



pub struct LineCollider {
    pub p: Vector3<f64>,
    pub v: Vector3<f64>,
    pub start: Option<f64>,
    pub stop: Option<f64>
}
impl LineCollider {
    pub fn segment(p: Vector3<f64>, v: Vector3<f64>) -> Self {
        Self {
            p,
            v,
            start: Some(0.),
            stop: Some(1.),
        }
    }

    pub(crate) fn reorient(&self, from: Option<&RigidBody>, to: Option<&RigidBody>) -> Self {
        Self {
            p: reorient(self.p, from, to),
            v: reorient_rot(self.v, from, to),
            start: self.start,
            stop: self.stop,
        }
    }

}