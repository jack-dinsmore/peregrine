use core::f64;
use anyhow::{anyhow, Result};

use cgmath::{Rotation, Vector3};
use collision_tree::check_tree;

use crate::util::BinaryTree;
use super::RigidBody;

mod report;
mod collision_box;
mod collision_line;
mod collision_grid;
mod collision_tree;

pub use report::{ColliderPackage, CollisionReport};
pub use collision_box::BoxCollider;
pub use collision_line::LineCollider;
pub use collision_grid::GridCollider;



pub enum Collider {
    Point {p: Vector3<f64>},
    Line(LineCollider),
    Grid(GridCollider),
    Box(BoxCollider),
    BoxTree(BinaryTree<BoxCollider>),
}

impl Collider {
    pub fn segment(p: Vector3<f64>, v: Vector3<f64>) -> Self {
        Self::Line(LineCollider::segment(p, v))
    }
    /// Check for intersections between two generic colliders
    pub fn check_intersection(a: ColliderPackage, b: ColliderPackage) -> CollisionReport {
        match (a.collider, b.collider) {
            // ==================================== POINT ====================================
            (Collider::Point { p }, Collider::Box(x)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                x.check_point(p).reorient(b.rigid_body)
            }
            (Collider::Box(x), Collider::Point { p }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                x.check_point(p).reorient(a.rigid_body)
            },

            (Collider::Grid(grid), Collider::Point { p }) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                grid.check_point(p).reorient(a.rigid_body)
            },
            (Collider::Point { p }, Collider::Grid(grid)) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                grid.check_point(p).reorient(b.rigid_body)
            },

            (Collider::BoxTree(t), Collider::Point { p }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                check_tree(t, |x| x.check_point(p)).reorient(a.rigid_body)
            },
            (Collider::Point { p }, Collider::BoxTree(t)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                check_tree(t, |x| x.check_point(p)).reorient(b.rigid_body)
            },

            // ==================================== LINE ====================================
            (Collider::Line(line), Collider::Box(x)) => {
                x.check_line(line.reorient(a.rigid_body, b.rigid_body)).reorient(b.rigid_body)
            }
            (Collider::Box(x), Collider::Line(line)) => {
                x.check_line(line.reorient(b.rigid_body, a.rigid_body)).reorient(a.rigid_body)
            },
            
            (Collider::Line(line), Collider::Grid(grid)) => {
                grid.check_line(line.reorient(a.rigid_body, b.rigid_body)).reorient(b.rigid_body)
            },
            (Collider::Grid(grid), Collider::Line(line)) => {
                grid.check_line(line.reorient(b.rigid_body, a.rigid_body)).reorient(a.rigid_body)
            },

            (Collider::BoxTree(t), Collider::Line(line)) => {
                check_tree(t, |x| x.check_line(line.reorient(b.rigid_body, a.rigid_body))).reorient(a.rigid_body)
            },
            (Collider::Line(line), Collider::BoxTree(t)) => {
                check_tree(t, |x| x.check_line(line.reorient(a.rigid_body, b.rigid_body))).reorient(b.rigid_body)
            },
            
            // ==================================== BOX ====================================
            (Collider::Box(x1), Collider::Box(x2)) => {
                x1.check_box(a.rigid_body, x2, b.rigid_body).reorient(a.rigid_body)
            },

            (Collider::Grid(_), Collider::Box(_)) => unimplemented!(),
            (Collider::Box(_), Collider::Grid(_)) => unimplemented!(),

            (Collider::BoxTree(t), Collider::Box(x)) => {
                check_tree(t, |xx| xx.check_box(a.rigid_body, x, b.rigid_body)).reorient(a.rigid_body)
            },
            (Collider::Box(x), Collider::BoxTree(t)) => {
                check_tree(t, |xx| xx.check_box(b.rigid_body, x, a.rigid_body)).reorient(b.rigid_body)
            },
            
            // ==================================== GRID ====================================
            (Collider::Grid(_), Collider::Grid(_)) => unimplemented!(),

            (Collider::Grid(_), Collider::BoxTree(_)) => unimplemented!(),
            (Collider::BoxTree(_), Collider::Grid(_)) => unimplemented!(),

            // ==================================== TREE ====================================
            (Collider::BoxTree(t1), Collider::BoxTree(t2)) => {
                //OPTIMIZE
                check_tree(t1, |x1| check_tree(t2, |x2| x1.check_box(a.rigid_body, x2, b.rigid_body))).reorient(a.rigid_body)
            },

            // ==================================== NULL ====================================
            (Collider::Point { .. }, Collider::Point { .. }) |
            (Collider::Point { .. }, Collider::Line { .. }) |
            (Collider::Line { .. }, Collider::Point { .. }) |
            (Collider::Line { .. }, Collider::Line { .. }) => CollisionReport::none(),
        }
    }
    
    pub fn get_grid_collider_mut(&mut self) -> Result<&mut GridCollider> {
        match self {
            Self::Grid(c) => Ok(c),
            _ => Err(anyhow!("The collider was not a grid collider")),
        }
    }
    
    pub fn get_grid_collider(&self) -> Result<&GridCollider> {
        match self {
            Self::Grid(c) => Ok(c),
            _ => Err(anyhow!("The collider was not a grid collider")),
        }
    }
}

pub(crate) fn reorient(v: Vector3<f64>, from: Option<&RigidBody>, to: Option<&RigidBody>) -> Vector3<f64> {
    match (from, to) {
        (Some(from), Some(to)) => to.orientation.invert().rotate_vector(
            from.orientation.rotate_vector(v)
            + from.pos - to.pos
        ),
        (None, Some(to)) => to.orientation.invert().rotate_vector(v - to.pos),
        (Some(from), None) => from.orientation.rotate_vector(v) + from.pos,
        (None, None) => v,
    }
}

pub(crate) fn reorient_rot(v: Vector3<f64>, from: Option<&RigidBody>, to: Option<&RigidBody>) -> Vector3<f64> {
    match (from, to) {
        (Some(from), Some(to)) => to.orientation.invert().rotate_vector(
            from.orientation.rotate_vector(v)
        ),
        (None, Some(to)) => to.orientation.invert().rotate_vector(v),
        (Some(from), None) => from.orientation.rotate_vector(v),
        (None, None) => v,
    }
}