use cgmath::{InnerSpace, Quaternion, Rotation, Vector3};

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

pub struct CollisionBox {
    orientation: Quaternion<f64>, 
    dimensions: Vector3<f64>
}
impl CollisionBox {
    /// Make a new collision box
    pub fn new() -> Self {
        unimplemented!()
    }

    /// Check for collisions between a box and a point. The point is in the box frame
    fn check_point(&self, p: Vector3<f64>) -> Option<Vector3<f64>> {
        unimplemented!()
    }
    
    /// Check for collisions between a box and a line. The origin p and direction v are in the box frame
    fn check_line(&self, p: Vector3<f64>, v: Vector3<f64>) -> Option<Vector3<f64>> {
        unimplemented!()
    }
    
    /// Check for collisions between two boxes
    fn check_box(&self, rigid_body: &RigidBody, o: &CollisionBox, rigid_body_p: &RigidBody) -> Option<Vector3<f64>> {
        unimplemented!()
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
    pub fn check_intersection(a: ColliderPackage, b: ColliderPackage) -> Option<Vector3<f64>> {
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
                check_tree(t, |x| x.check_box(a.rigid_body, x, b.rigid_body))
            },
            (Collider::Box(x), Collider::BoxTree(t)) => {
                check_tree(t, |x| x.check_box(b.rigid_body, x, a.rigid_body))
            },
            (Collider::Point { .. }, Collider::Point { .. }) => None,
            (Collider::Point { .. }, Collider::Line { .. }) |
            (Collider::Line { .. }, Collider::Point { .. }) => None,
            (Collider::Point { p }, Collider::Box(x)) => {
                let p = reorient(*p, a.rigid_body, b.rigid_body);
                x.check_point(p)
            }
            (Collider::Box(x), Collider::Point { p }) => {
                let p = reorient(*p, b.rigid_body, a.rigid_body);
                x.check_point(p)
            },
            (Collider::Line { .. }, Collider::Line { .. }) => None,
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
                unimplemented!()
                // This should be a custom function to downselect both trees at the same time
            },
        }
    }

    /// Create a collider tree from a list of collider boxes
    pub fn make_tree(boxes: Vec<CollisionBox>) -> Self {
        let mut tree = BinaryTree::new();

        unimplemented!();

        Self::BoxTree(tree)
    }
}

fn check_tree(t: &BinaryTree<CollisionBox>, check_function: impl Fn(&CollisionBox)->Option<Vector3<f64>>) -> Option<Vector3<f64>> {
    let mut node_queue = vec![t.root()];
    let mut best_collision: Option<Vector3<f64>> = None;
    while let Some(node) = node_queue.pop() {
        match check_function(&node) {
            Some(v) => {
                match best_collision {
                    Some(b) => {
                        if b.magnitude2() < v.magnitude2() {
                            // This one collides but not as well as the best
                            continue
                        } else {
                            // This one is the best
                            best_collision = Some(v)
                        }
                    },
                    None => {
                        // This one is the first
                        best_collision = Some(v)
                    }
                }
            },
            // This one doesn't collide at all
            None => continue,
        };
        if let Some(n) = node.left() { node_queue.push(n);}
        if let Some(n) = node.right() { node_queue.push(n);}
    }
    best_collision
}

fn reorient(v: Vector3<f64>, from: &RigidBody, to: &RigidBody) -> Vector3<f64> {
    to.orientation.invert().rotate_vector(
        from.orientation.rotate_vector(v)
        + from.pos - to.pos
    )
}