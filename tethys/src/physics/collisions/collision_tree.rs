use cgmath::Vector3;

use crate::util::{unreacho, BinaryTree};

use super::{collision_box::BoxCollider, Collider, CollisionReport};

impl Collider {
    /// Create a collider tree from a list of collider boxes
    pub fn make_tree(boxes: Vec<BoxCollider>) -> Self {
        let box_pointer = boxes.iter().collect::<Vec<_>>();
        let (superbox, is_full) = BoxCollider::superbox(&box_pointer);
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
                let (left_array, right_array) = BoxCollider::subdivide(&boxes);
                let (left_box, left_full) = BoxCollider::superbox(&left_array);
                let (right_box, right_full) = BoxCollider::superbox(&right_array);
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

impl BoxCollider {
    // Returns the smallest box containing all these (assumed to be co-aligned) boxes. Also returns a bool which is true if the returned box is completely full, which assumes that the composing boxes are not intersecting.
    pub(crate) fn superbox(boxes: &[&BoxCollider]) -> (BoxCollider, bool) {
        if boxes.len() == 0 {panic!("Cannot compute the superbox of zero objects")};
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
    
    pub(crate) fn subdivide<'a>(boxes: &[&'a BoxCollider]) -> (Vec<&'a BoxCollider>, Vec<&'a BoxCollider>) {
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
}

pub(crate) fn check_tree(t: &BinaryTree<BoxCollider>, check_function: impl Fn(&BoxCollider)->CollisionReport) -> CollisionReport {
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