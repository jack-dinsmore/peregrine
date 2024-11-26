use std::vec::IntoIter;

use cgmath::Vector3;

use crate::physics::RigidBody;

use super::{reorient, LineCollider, CollisionReport};


#[derive(Clone)]
pub struct BoxCollider {
    /// Position of the box
    pub(crate) corner: Vector3<f64>,
    pub(crate) dimensions: Vector3<f64>
}
impl BoxCollider {
    /// Make a new collision box
    pub fn new(corner: Vector3<f64>, dimensions: Vector3<f64>) -> Self {
        Self {
            corner,
            dimensions,
        }
    }

    /// Check for collisions between a box and a point. The point is in the box frame
    pub(crate) fn check_point(&self, p: Vector3<f64>) -> CollisionReport {
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
    pub(crate) fn check_line(&self, line: LineCollider) -> CollisionReport {
        let p = line.p;
        let v = line.v;
        let mut closest_report = CollisionReport::none();
        let mut min_alpha = f64::INFINITY;

        // x
        for mut alpha in [
            (self.corner.x - p.x) / v.x, // Close
            (self.corner.x + self.dimensions.x - p.x) / v.x// Far
        ] {
            if let Some(start) = line.start { alpha = alpha.max(start); } 
            if let Some(stop) = line.stop { alpha = alpha.min(stop); } 
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
        }
        // y
        for mut alpha in [
            (self.corner.y - p.y) / v.y, // Close
            (self.corner.y + self.dimensions.y - p.y) / v.y// Far
        ] {
            if let Some(start) = line.start { alpha = alpha.max(start); } 
            if let Some(stop) = line.stop { alpha = alpha.min(stop); } 
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
        }
        // z
        for mut alpha in [
            (self.corner.z - p.z) / v.z, // Close
            (self.corner.z + self.dimensions.z - p.z) / v.z// Far
        ] {
            if let Some(start) = line.start { alpha = alpha.max(start); } 
            if let Some(stop) = line.stop { alpha = alpha.min(stop); } 
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
        }
        closest_report
    }
    
    /// Check for collisions between two boxes
    pub(crate) fn check_box(&self, rigid_body: Option<&RigidBody>, o: &BoxCollider, o_rigid_body: Option<&RigidBody>) -> CollisionReport {
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
        for line in self.get_lines() {
            let report = o.check_line(line.reorient(rigid_body, o_rigid_body));
            if report > deepest_report {
                deepest_report = report.reorient(o_rigid_body);
            }
        }
        for line in o.get_lines() {
            let report = self.check_line(line.reorient(o_rigid_body, rigid_body));
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

    fn get_lines(&self) -> IntoIter<LineCollider> {
        let opposite_corner = self.corner + Vector3::new(self.dimensions.x, self.dimensions.y, self.dimensions.z);
        vec![
            LineCollider::segment(
                self.corner,
                Vector3::new(self.dimensions.x, 0., 0.)
            ),
            LineCollider::segment(
                self.corner,
                Vector3::new(0., self.dimensions.y, 0.)
            ),
            LineCollider::segment(
                self.corner,
                Vector3::new(0., 0., self.dimensions.z)
            ),
            LineCollider::segment(
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
            ),
            LineCollider::segment(
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
                self.corner + Vector3::new(0., 0., self.dimensions.z),
            ),
            LineCollider::segment(
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
            ),
            LineCollider::segment(
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
                self.corner + Vector3::new(0., 0., self.dimensions.z),
            ),
            LineCollider::segment(
                self.corner + Vector3::new(0., 0., self.dimensions.z),
                self.corner + Vector3::new(0., self.dimensions.y, 0.),
            ),
            LineCollider::segment(
                self.corner + Vector3::new(0., 0., self.dimensions.z),
                self.corner + Vector3::new(self.dimensions.x, 0., 0.),
            ),
            LineCollider::segment(
                opposite_corner,
                Vector3::new(-self.dimensions.x, 0., 0.)
            ),
            LineCollider::segment(
                opposite_corner,
                Vector3::new(0., -self.dimensions.y, 0.)
            ),
            LineCollider::segment(
                opposite_corner,
                Vector3::new(0., 0., -self.dimensions.z)
            ),
        ].into_iter()
    }
    
    pub fn volume(&self) -> f64 {
        self.dimensions.x * self.dimensions.y * self.dimensions.z
    }
}