use std::{ops::AddAssign, vec::IntoIter};

use cgmath::{InnerSpace, Rotation, Vector3};

use crate::util::BinaryTree;
use super::RigidBody;

#[derive(Clone, Copy)]
pub struct ColliderPackage<'a> {
    pub collider: &'a Collider,
    pub rigid_body: &'a RigidBody,
}
impl<'a> From<(&'a Collider, &'a RigidBody)> for ColliderPackage<'a> {
    fn from((collider, rigid_body): (&'a Collider, &'a RigidBody)) -> Self {
        Self {
            collider,
            rigid_body,
        }
    }
}
impl<'a> From<(&'a RigidBody, &'a Collider)> for ColliderPackage<'a> {
    fn from((rigid_body, collider): (&'a RigidBody, &'a Collider)) -> Self {
        Self {
            collider,
            rigid_body,
        }
    }
}

pub struct CollisionReport {
    pub depths: Vec<Vector3<f64>>,
    pub positions: Vec<Vector3<f64>>,
}

impl CollisionReport {
    pub fn none() -> Self {
        Self {
            depths: Vec::new(),
            positions: Vec::new(),
        }
    }
    pub fn new(depth: Vector3<f64>, position: Vector3<f64>) -> Self {
        Self {
            depths: vec![depth],
            positions: vec![position],
        }
    }
    fn reorient(&self, from: &RigidBody) -> Self {
        Self {
            depths: self.depths.iter().map(|v| orient_global(*v, from)).collect(),
            positions: self.positions.iter().map(|v| orient_global(*v, from)).collect(),
        }
    }
    
    pub fn collision(&self) -> bool {
        !self.positions.is_empty()
    }

    fn deepest_mag2(&self) -> f64 {
        self.positions.iter().fold(0., |accum, v| v.magnitude2().max(accum))
    }
}

impl AddAssign for CollisionReport {
    fn add_assign(&mut self, mut rhs: Self) {
        self.depths.append(&mut rhs.depths);
        self.positions.append(&mut rhs.positions);
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

pub struct CollisionBox {
    /// Position of the box
    corner: Vector3<f64>,
    dimensions: Vector3<f64>
}
impl CollisionBox {
    /// Make a new collision box
    pub fn new(corner: Vector3<f64>, dimensions: Vector3<f64>) -> Self {
        Self {
            corner,
            dimensions,
        }
    }

    /// Check for collisions between a box and a point. The point is in the box frame
    fn check_point(&self, p: Vector3<f64>) -> CollisionReport {
        if 0. < p.x && p.x < self.dimensions.x && 0. < p.y && p.y < self.dimensions.y && 0. < p.z && p.z < self.dimensions.z {
            let min_dist = p.x.max(p.y.max(p.z.max((self.dimensions.x - p.x).max((self.dimensions.y - p.y).max(self.dimensions.z - p.z)))));
            let depth = if min_dist == p.x {
                Vector3::new(-min_dist, 0., 0.)
            } else if min_dist == p.y {
                Vector3::new(0., -min_dist, 0.)
            } else if min_dist == p.z {
                Vector3::new(0., 0., -min_dist)
            } else if min_dist == self.dimensions.x - p.x {
                Vector3::new(min_dist, 0., 0.)
            } else if min_dist == self.dimensions.y - p.y {
                Vector3::new(0., min_dist, 0.)
            } else if min_dist == self.dimensions.z - p.z {
                Vector3::new(0., 0., min_dist)
            } else {
                unreachable!()
            };
            CollisionReport::new(depth, p + depth/2.)
        } else {
            CollisionReport::none()
        }
    }
    
    /// Check for collisions between a box and a line. The origin p and direction v are in the box frame
    fn check_line(&self, p: Vector3<f64>, v: Vector3<f64>) -> CollisionReport {
        let mut deepest_report = CollisionReport::none();
        for (px, py, vx, vy, dx, dy, n) in [
            (p.x, p.y, v.x, v.y, self.dimensions.x, self.dimensions.y, Vector3::new(0., 0., 1.)),
            (p.x, p.z, v.x, v.z, self.dimensions.x, self.dimensions.z, Vector3::new(0., -1., 0.)),
            (p.y, p.z, v.y, v.z, self.dimensions.y, self.dimensions.z, Vector3::new(1., 0., 0.)),
        ] {
            let a = vy / vx;
            let b = py - px * a;
            let p1 = (0., b);
            let p2 = (1., a + b);
            let p3 = (-b/a, 0.);
            let p4 = ((1.-b)/a, 0.);
            let inside1 = 0. < p1.1 && p1.1 < dy;
            let inside2 = 0. < p2.1 && p2.1 < dy;
            let inside3 = 0. < p3.0 && p3.0 < dx;
            let inside4 = 0. < p4.0 && p4.0 < dx;
            if !inside1 && !inside2 && !inside3 && !inside4 {
                // No collision
                return CollisionReport::none();
            }

            // There was a collision. Get the opposing corner
            let near_corner = if inside1 && inside2 {
                if p1.1 > p2.1 {
                    if p1.1 + p2.1 > 1. {
                        (1., 1.)
                    } else {
                        (0., 0.)
                    }
                } else {
                    if p1.1 + p2.1 > 1. {
                        (1., 0.)
                    } else {
                        (1., 0.)
                    }
                }
            } else if inside1 && inside3 {
                (0., 0.)
            } else if inside1 && inside4 {
                (0., 1.)
            } else if inside2 && inside3 {
                (1., 0.)
            } else if inside2 && inside4 {
                (1., 1.)
            } else if inside3 && inside4 {
                if p3.0 > p4.0 {
                    if p3.0 + p4.0 > 1. {
                        (1., 1.)
                    } else {
                        (0., 0.)
                    }
                } else {
                    if p3.0 + p4.0 > 1. {
                        (1., 0.)
                    } else {
                        (0., 1.)
                    }
                }
            } else {
                unreachable!()
            };

            let alpha = (vx * (near_corner.0 - px) + vy * (near_corner.1 - py)) / (vx*vx + vy*vy);
            let dist = ((near_corner.0 - px - alpha * vx).powi(2) + (near_corner.1 - py - alpha * vy).powi(2)).sqrt();

            // Get distance to this corner
            let normal = v.cross(n).normalize() * dist;
            let this_report = CollisionReport::new(normal, p + v * alpha + normal/2.);
            if this_report > deepest_report {
                deepest_report = this_report;
            }
        }
        deepest_report
    }
    
    /// Check for collisions between two boxes
    fn check_box(&self, rigid_body: &RigidBody, o: &CollisionBox, o_rigid_body: &RigidBody) -> CollisionReport {
        let mut deepest_report = CollisionReport::none();

        // Check points
        for point in self.get_points() {
            let report = o.check_point(reorient(point, rigid_body, o_rigid_body));
            if report > deepest_report {
                deepest_report = report.reorient(o_rigid_body);
            }
        }
        for point in o.get_points() {
            let report = self.check_point(reorient(point, o_rigid_body, rigid_body));
            if report > deepest_report {
                deepest_report = report.reorient(rigid_body);
            }
        }

        // Check lines
        for (p, v) in self.get_lines() {
            let report = o.check_line(
                reorient(p, rigid_body, o_rigid_body),
                reorient(v, rigid_body, o_rigid_body)
            );
            if report > deepest_report {
                deepest_report = report.reorient(o_rigid_body);
            }
        }
        for (p, v) in o.get_lines() {
            let report = self.check_line(
                reorient(p, o_rigid_body, rigid_body),
                reorient(v, o_rigid_body, rigid_body)
            );
            if report > deepest_report {
                deepest_report = report.reorient(rigid_body);
            }
        }

        deepest_report
    }

    fn get_points(&self) -> IntoIter<Vector3<f64>> {
        vec![
            self.corner,
            self.corner + Vector3::new(self.dimensions.x, 0., 0.),
            self.corner + Vector3::new(0., self.dimensions.y, 0.),
            self.corner + Vector3::new(0., 0., self.dimensions.z),
            self.corner + Vector3::new(self.dimensions.x, self.dimensions.y, 0.),
            self.corner + Vector3::new(self.dimensions.x, 0., self.dimensions.z),
            self.corner + Vector3::new(0., self.dimensions.y, self.dimensions.z),
            self.corner + Vector3::new(self.dimensions.x, self.dimensions.y, self.dimensions.z),
        ].into_iter()
    }

    fn get_lines(&self) -> IntoIter<(Vector3<f64>, Vector3<f64>)> {
        let opposite_corner = self.corner + Vector3::new(self.dimensions.x, self.dimensions.y, self.dimensions.z);
        vec![
            (
                self.corner,
                Vector3::new(self.dimensions.x, 0., 0.)
            ),
            (
                self.corner,
                Vector3::new(0., self.dimensions.y, 0.)
            ),
            (
                self.corner,
                Vector3::new(0., 0., self.dimensions.z)
            ),
            (
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
            ),
            (
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
                self.corner + Vector3::new(0., 0., self.dimensions.z),
            ),
            (
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
            ),
            (
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
                self.corner + Vector3::new(0., 0., self.dimensions.z),
            ),
            (
                self.corner + Vector3::new(0., 0., self.dimensions.z),
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
            ),
            (
                self.corner + Vector3::new(0., 0., self.dimensions.z),
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
            ),
            (
                opposite_corner,
                Vector3::new(-self.dimensions.x, 0., 0.)
            ),
            (
                opposite_corner,
                Vector3::new(0., -self.dimensions.y, 0.)
            ),
            (
                opposite_corner,
                Vector3::new(0., 0., -self.dimensions.z)
            ),
        ].into_iter()
    }
}

pub enum Collider {
    Point {p: Vector3<f64>},
    Line {p: Vector3<f64>, v: Vector3<f64>},
    Box(CollisionBox),
    BoxTree(BinaryTree<CollisionBox>),
}

impl Collider {
    /// Check for intersections between two generic colliders
    pub fn check_intersection(a: ColliderPackage, b: ColliderPackage) -> CollisionReport {
        match (a.collider, b.collider) {
            (Collider::BoxTree(t), Collider::Point { p }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                check_tree(t, |x| x.check_point(p))
            },
            (Collider::Point { p }, Collider::BoxTree(t)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                check_tree(t, |x| x.check_point(p))
            },
            (Collider::BoxTree(t), Collider::Line { p, v }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                let v = reorient(*v, b.rigid_body, a.rigid_body);
                check_tree(t, |x| x.check_line(p, v))
            },
            (Collider::Line { p, v }, Collider::BoxTree(t)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                let v = reorient(*v, a.rigid_body, b.rigid_body);
                check_tree(t, |x| x.check_line(p, v))
            },
            (Collider::BoxTree(t), Collider::Box(x)) => {
                check_tree(t, |xx| xx.check_box(a.rigid_body, x, b.rigid_body))
            },
            (Collider::Box(x), Collider::BoxTree(t)) => {
                check_tree(t, |xx| xx.check_box(b.rigid_body, x, a.rigid_body))
            },
            (Collider::Point { .. }, Collider::Point { .. }) => CollisionReport::none(),
            (Collider::Point { .. }, Collider::Line { .. }) |
            (Collider::Line { .. }, Collider::Point { .. }) => CollisionReport::none(),
            (Collider::Point { p }, Collider::Box(x)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                x.check_point(p)
            }
            (Collider::Box(x), Collider::Point { p }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                x.check_point(p)
            },
            (Collider::Line { .. }, Collider::Line { .. }) => CollisionReport::none(),
            (Collider::Line { p, v }, Collider::Box(x)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                let v = reorient(*v, a.rigid_body, b.rigid_body);
                x.check_line(p, v)
            }
            (Collider::Box(x), Collider::Line { p, v }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                let v = reorient(*v, b.rigid_body, a.rigid_body);
                x.check_line(p, v)
            },
            (Collider::Box(x1), Collider::Box(x2)) => {
                x1.check_box(a.rigid_body, x2, b.rigid_body)
            },
            (Collider::BoxTree(t1), Collider::BoxTree(t2)) => {
                //TODO optimize
                check_tree(t1, |x1| check_tree(t2, |x2| x1.check_box(a.rigid_body, x2, b.rigid_body)))
            },
        }
    }

    /// Create a collider tree from a list of collider boxes
    pub fn make_tree(boxes: Vec<CollisionBox>) -> Self {
        let mut tree = BinaryTree::new();

        unimplemented!();

        merge_tree(&mut tree);

        Self::BoxTree(tree)
    }
}

/// Merge all boxes in a tree which represent an entire box
fn merge_tree(t: &mut BinaryTree<CollisionBox>) {
    unimplemented!()
}

fn check_tree(t: &BinaryTree<CollisionBox>, check_function: impl Fn(&CollisionBox)->CollisionReport) -> CollisionReport {
    let mut node_queue = vec![t.root()];
    let mut report = CollisionReport::none();
    while let Some(node) = node_queue.pop() {
        let this_report = check_function(&node);
        if this_report.collision() {
            report += this_report;
            if let Some(n) = node.left() { node_queue.push(n);}
            if let Some(n) = node.right() { node_queue.push(n);}
        }
    }
    report
}

fn reorient(v: Vector3<f64>, from: &RigidBody, to: &RigidBody) -> Vector3<f64> {
    to.orientation.invert().rotate_vector(
        from.orientation.rotate_vector(v)
        + from.pos - to.pos
    )
}

fn orient_global(v: Vector3<f64>, from: &RigidBody) -> Vector3<f64> {
    from.orientation.rotate_vector(v)
    + from.pos
}