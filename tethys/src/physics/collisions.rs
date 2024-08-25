use core::f64;
use std::{ops::AddAssign, vec::IntoIter};

use cgmath::{InnerSpace, Rotation, Vector3};
use log::warn;

use crate::util::{unreacho, BinaryTree};
use super::RigidBody;

const EPSILON: f64 = 1e-5;

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
    fn reorient(&self, from: Option<&RigidBody>) -> Self {
        Self {
            depths: self.depths.iter().map(|v| reorient_rot(*v, from, None)).collect(),
            positions: self.positions.iter().map(|v| reorient(*v, from, None)).collect(),
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

#[derive(Clone)]
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
                dbg!(min_dist, p.x, p.y, p.z, self.dimensions.x-p.x, self.dimensions.y-p.y, self.dimensions.z-p.z);
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
            let p2 = (dx, a * dx + b);
            let p3 = (-b/a, 0.);
            let p4 = ((dy-b)/a, dy);
            let inside1 = -EPSILON <= p1.1 && p1.1 <= dy+EPSILON;
            let inside2 = -EPSILON <= p2.1 && p2.1 <= dy+EPSILON;
            let inside3 = -EPSILON <= p3.0 && p3.0 <= dx+EPSILON;
            let inside4 = -EPSILON <= p4.0 && p4.0 <= dx+EPSILON;
            if !inside1 && !inside2 && !inside3 && !inside4 {
                // No collision
                return CollisionReport::none();
            }

            // There was a collision. Get the opposing corner
            let near_corner = if inside1 && inside2 {
                if p1.1 > p2.1 {
                    if p1.1 + p2.1 > dy {
                        (dx, dy)
                    } else {
                        (0., 0.)
                    }
                } else {
                    if p1.1 + p2.1 > dy {
                        (dx, 0.)
                    } else {
                        (0., dy)
                    }
                }
            } else if inside1 && inside3 {
                (0., 0.)
            } else if inside1 && inside4 {
                (0., dy)
            } else if inside2 && inside3 {
                (dx, 0.)
            } else if inside2 && inside4 {
                (dx, dy)
            } else if inside3 && inside4 {
                if p3.0 > p4.0 {
                    if p3.0 + p4.0 > dx {
                        (dx, dy)
                    } else {
                        (0., 0.)
                    }
                } else {
                    if p3.0 + p4.0 > dx {
                        (dx, 0.)
                    } else {
                        (0., dy)
                    }
                }
            } else {
                warn!("Invalid collision");
                warn!("{}, {}", dx, dy);
                warn!("{:?}, {:?}, {:?}, {:?}", p1, p2, p3, p4);
                warn!("{}, {}, {}, {}", inside1, inside2, inside3, inside4);
                return CollisionReport::none();
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
    
    /// Check for collisions between a box and a line. The origin p and direction v are in the box frame
    fn check_ray(&self, p: Vector3<f64>, v: Vector3<f64>) -> CollisionReport {
        let mut closest_report = CollisionReport::none();
        let mut min_alpha = f64::INFINITY;
        // Close, normal=x
        let alpha = (self.corner.x - p.x) / v.x;
        let x = p + v * alpha;
        if self.corner.y < x.y && x.y < self.corner.y + self.dimensions.y {
            if self.corner.z < x.z && x.z < self.corner.z + self.dimensions.z {
                // It's in
                if alpha < min_alpha {
                    min_alpha = alpha;
                    closest_report = CollisionReport::new(v * alpha, x);
                }
            }
        }
        // Far, normal=x
        let alpha = (self.corner.x + self.dimensions.x - p.x) / v.x;
        let x = p + v * alpha;
        if self.corner.y < x.y && x.y < self.corner.y + self.dimensions.y {
            if self.corner.z < x.z && x.z < self.corner.z + self.dimensions.z {
                // It's in
                if alpha < min_alpha {
                    min_alpha = alpha;
                    closest_report = CollisionReport::new(v * alpha, x);
                }
            }
        }
        // Close, normal=y
        let alpha = (self.corner.y - p.y) / v.y;
        let x = p + v * alpha;
        if self.corner.x < x.x && x.x < self.corner.x + self.dimensions.x {
            if self.corner.z < x.z && x.z < self.corner.z + self.dimensions.z {
                // It's in
                if alpha < min_alpha {
                    min_alpha = alpha;
                    closest_report = CollisionReport::new(v * alpha, x);
                }
            }
        }
        // Far, normal=y
        let alpha = (self.corner.y + self.dimensions.y - p.y) / v.y;
        let x = p + v * alpha;
        if self.corner.x < x.x && x.x < self.corner.x + self.dimensions.x {
            if self.corner.z < x.z && x.z < self.corner.z + self.dimensions.z {
                // It's in
                if alpha < min_alpha {
                    min_alpha = alpha;
                    closest_report = CollisionReport::new(v * alpha, x);
                }
            }
        }
        // Close, normal=z
        let alpha = (self.corner.z - p.z) / v.z;
        let x = p + v * alpha;
        if self.corner.y < x.y && x.y < self.corner.y + self.dimensions.y {
            if self.corner.x < x.x && x.x < self.corner.x + self.dimensions.x {
                // It's in
                if alpha < min_alpha {
                    min_alpha = alpha;
                    closest_report = CollisionReport::new(v * alpha, x);
                }
            }
        }
        // Far, normal=z
        let alpha = (self.corner.z + self.dimensions.z - p.z) / v.z;
        let x = p + v * alpha;
        if self.corner.y < x.y && x.y < self.corner.y + self.dimensions.y {
            if self.corner.x < x.x && x.x < self.corner.x + self.dimensions.x {
                // It's in
                if alpha < min_alpha {
                    closest_report = CollisionReport::new(v * alpha, x);
                }
            }
        }
        closest_report
    }
    
    /// Check for collisions between two boxes
    fn check_box(&self, rigid_body: Option<&RigidBody>, o: &CollisionBox, o_rigid_body: Option<&RigidBody>) -> CollisionReport {
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
                reorient_rot(v, rigid_body, o_rigid_body)
            );
            if report > deepest_report {
                deepest_report = report.reorient(o_rigid_body);
            }
        }
        for (p, v) in o.get_lines() {
            let report = self.check_line(
                reorient(p, o_rigid_body, rigid_body),
                reorient_rot(v, o_rigid_body, rigid_body)
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
    
    // Returns the smallest box containing all these (assumed to be co-aligned) boxes. Also returns a bool which is true if the returned box is completely full, which assumes that the composing boxes are not intersecting.
    fn superbox(boxes: &[&CollisionBox]) -> (CollisionBox, bool) {
        if boxes.len() == 0 {panic!("Cannot compute the superbox of just no objects")};
        let mut superbox = boxes[0].clone();
        let mut volume = 0.;
        for x in boxes {
            volume += x.volume();
            let delta_x = superbox.corner.x - x.corner.x;
            let delta_y = superbox.corner.y - x.corner.y;
            let delta_z = superbox.corner.z - x.corner.z;
            if delta_x > 0. {
                superbox.corner.x -= delta_x;
                superbox.dimensions.x += delta_x;
            }
            if delta_y > 0. {
                superbox.corner.y -= delta_y;
                superbox.dimensions.y += delta_y;
            }
            if delta_z > 0. {
                superbox.corner.z -= delta_z;
                superbox.dimensions.z += delta_z;
            }
            let delta_x = x.corner.x + x.dimensions.x - superbox.corner.x - superbox.dimensions.x;
            let delta_y = x.corner.y + x.dimensions.y - superbox.corner.y - superbox.dimensions.y;
            let delta_z = x.corner.z + x.dimensions.z - superbox.corner.z - superbox.dimensions.z;
            if delta_x > 0. {
                superbox.dimensions.x += delta_x;
            }
            if delta_y > 0. {
                superbox.dimensions.y += delta_y;
            }
            if delta_z > 0. {
                superbox.dimensions.z += delta_z;
            }
        }
        let is_full = (superbox.volume() - volume).abs() < 1e-5;
        (superbox, is_full)
    }
    
    fn subdivide<'a>(boxes: &[&'a CollisionBox]) -> (Vec<&'a CollisionBox>, Vec<&'a CollisionBox>) {
        if boxes.len() < 2 {
            panic!("Cannot subdivide fewer than two boxes");
        }
        if boxes.len() == 2 {
            return (vec![boxes[0]], vec![boxes[1]]);
        }
        let mut mean = Vector3::new(0., 0., 0.);
        let mut mean2 = Vector3::new(0., 0., 0.);
        for x in boxes {
            let center = x.corner + x.dimensions/2.;
            mean += center;
            mean2 += Vector3::new(center.x.powi(2), center.y.powi(2), center.z.powi(2));
        }
        mean /= boxes.len() as f64;
        mean2 /= boxes.len() as f64;
        let std = mean2 - Vector3::new(mean.x.powi(2), mean.y.powi(2), mean.z.powi(2));
        let dimension = if std.x > std.y && std.x > std.z {
            0
        } else if std.y > std.x && std.y > std.z {
            1
        } else {
            2
        };

        let mut left = Vec::new();
        let mut right = Vec::new();
        for x in boxes {
            let center = x.corner + x.dimensions/2.;
            let use_left = match dimension {
                0 => center.x > mean.x,
                1 => center.y > mean.y,
                2 => center.z > mean.z,
                _ => unreachable!()
            };
            if use_left {
                left.push(*x);
            } else {
                right.push(*x);
            }
        }
        (left, right)
    }
    
    pub fn volume(&self) -> f64 {
        self.dimensions.x * self.dimensions.y * self.dimensions.z
    }
}

pub enum Collider {
    Point {p: Vector3<f64>},
    Line {p: Vector3<f64>, v: Vector3<f64>},
    Ray {p: Vector3<f64>, v: Vector3<f64>},
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
                let v = reorient_rot(*v, b.rigid_body, a.rigid_body);
                check_tree(t, |x| x.check_line(p, v))
            },
            (Collider::Line { p, v }, Collider::BoxTree(t)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                let v = reorient_rot(*v, a.rigid_body, b.rigid_body);
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
                let v = reorient_rot(*v, a.rigid_body, b.rigid_body);
                x.check_line(p, v)
            }
            (Collider::Box(x), Collider::Line { p, v }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                let v = reorient_rot(*v, b.rigid_body, a.rigid_body);
                x.check_line(p, v)
            },
            (Collider::Box(x1), Collider::Box(x2)) => {
                x1.check_box(a.rigid_body, x2, b.rigid_body)
            },
            (Collider::BoxTree(t1), Collider::BoxTree(t2)) => {
                //OPTIMIZE
                check_tree(t1, |x1| check_tree(t2, |x2| x1.check_box(a.rigid_body, x2, b.rigid_body)))
            },
            (Collider::Point { .. }, Collider::Ray { .. }) |
            (Collider::Line { .. }, Collider::Ray { .. }) |
            (Collider::Ray { .. }, Collider::Point { .. }) |
            (Collider::Ray { .. }, Collider::Line { .. })|
            (Collider::Ray { .. }, Collider::Ray { .. }) => CollisionReport::none(),
            (Collider::Ray { p, v }, Collider::Box(x)) |
            (Collider::Box(x), Collider::Ray { p, v }) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                let v = reorient_rot(*v, a.rigid_body, b.rigid_body);
                x.check_ray(p, v)
            },
            (Collider::Ray { p, v }, Collider::BoxTree(t)) |
            (Collider::BoxTree(t), Collider::Ray { p, v }) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                let v = reorient_rot(*v, a.rigid_body, b.rigid_body);
                check_tree(t, |x| x.check_ray(p, v))
            },
        }
    }

    /// Create a collider tree from a list of collider boxes
    pub fn make_tree(boxes: Vec<CollisionBox>) -> Self {
        let box_pointer = boxes.iter().collect::<Vec<_>>();
        let (superbox, is_full) = CollisionBox::superbox(&box_pointer);
        let tree = BinaryTree::new(superbox);
        if is_full {return Self::BoxTree(tree);}

        
        let tree = tree.root_mut(move |root| {
            let box_pointer = boxes.iter().collect::<Vec<_>>();
            let mut node_queue = vec![(
                root,
                box_pointer,
            )];

            // Sort the boxes into groups
            while let Some((node, boxes)) = node_queue.pop() {
                let (left_array, right_array) = CollisionBox::subdivide(&boxes);
                let (left_box, left_full) = CollisionBox::superbox(&left_array);
                let (right_box, right_full) = CollisionBox::superbox(&right_array);
                node.insert_left(left_box);
                node.insert_right(right_box);
                if !left_full {
                    node_queue.push((unreacho(node.left()), left_array));
                }
                if !right_full {
                    node_queue.push((unreacho(node.right()), right_array));
                }
            }
        });

        Self::BoxTree(tree)
    }
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

fn reorient(v: Vector3<f64>, from: Option<&RigidBody>, to: Option<&RigidBody>) -> Vector3<f64> {
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

fn reorient_rot(v: Vector3<f64>, from: Option<&RigidBody>, to: Option<&RigidBody>) -> Vector3<f64> {
    match (from, to) {
        (Some(from), Some(to)) => to.orientation.invert().rotate_vector(
            from.orientation.rotate_vector(v)
        ),
        (None, Some(to)) => to.orientation.invert().rotate_vector(v),
        (Some(from), None) => from.orientation.rotate_vector(v),
        (None, None) => v,
    }
}