use cgmath::{InnerSpace, Vector3};
use tethys::{physics::collisions::ColliderPackage, prelude::*};
use super::{Panel, Part, PartLayout};


/// Add a part to the grid, where data represents the index of the part
pub(super) fn add_part_to_grid(collider: &mut GridCollider, part: &Part, layout: PartLayout, data: usize) {
    let (ul, lr) = part.get_bbox(layout);
    let underflow_x = (ul.x + collider.cx).min(0);
    let underflow_y = (ul.y + collider.cy).min(0);
    let underflow_z = (ul.z + collider.cz).min(0);
    let overflow_x = (lr.x + collider.cx).max(collider.x as i32);
    let overflow_y = (lr.y + collider.cy).max(collider.y as i32);
    let overflow_z = (lr.z + collider.cz).max(collider.z as i32);
    if underflow_x < 0 || underflow_y < 0 || underflow_z < 0 || overflow_x as u32 >= collider.x || overflow_y as u32 >= collider.y || overflow_z as u32 >= collider.z {
        collider.reshape(
            (overflow_x - underflow_x) as u32+1,
            (overflow_y - underflow_y) as u32+1,
            (overflow_z - underflow_z) as u32+1,
            collider.cx - underflow_x,
            collider.cy - underflow_y,
            collider.cz - underflow_z,
        );
    } 
    for block in part.get_blocks(layout) {
        *collider.get_entry_mut(block.x, block.y, block.z).unwrap() = data as isize;
    }
}

/// Add a panel to the grid, where data represents the index of the panel
pub(super) fn add_panel_to_grid(grid: &mut GridCollider, panel: &Panel, index: isize) {
    for (x, y, z) in get_triangle_intersections(grid, panel.vertices) {
        *grid.get_entry_mut(x, y, z).unwrap() = index;
    }
}

/// Get the closest corner to the given ray 
pub fn get_corner(grid: ColliderPackage, line: LineCollider) -> Option<(i32, i32, i32)> {
    let line = Collider::Line(line);
    let result = Collider::check_intersection(grid, (&line).into()).orient(grid.rigid_body.unwrap());
    if result.collision() { 
        Some((
            result.positions[0].x.round() as i32,
            result.positions[0].y.round() as i32,
            result.positions[0].z.round() as i32,
        ))
    } else {
        None
    }
}

/// Returns a list of cell indices intersected by a panel. The cells are labeled by their lower corner.
pub fn get_triangle_intersections(grid: &GridCollider, vertices: [(i32, i32, i32); 3]) -> Vec<(i32, i32, i32)> {
    let mut output = Vec::new();

    let min_x = i32::min(i32::min(vertices[0].0, vertices[1].0), i32::min(vertices[2].0, grid.x as i32 - grid.cx));
    let min_y = i32::min(i32::min(vertices[0].1, vertices[1].1), i32::min(vertices[2].1, grid.y as i32 - grid.cy));
    let min_z = i32::min(i32::min(vertices[0].2, vertices[1].2), i32::min(vertices[2].2, grid.z as i32 - grid.cz));
    let max_x = i32::max(i32::max(vertices[0].0, vertices[1].0), i32::max(vertices[2].0, -grid.cx));
    let max_y = i32::max(i32::max(vertices[0].1, vertices[1].1), i32::max(vertices[2].1, -grid.cy));
    let max_z = i32::max(i32::max(vertices[0].2, vertices[1].2), i32::max(vertices[2].2, -grid.cz));
    if min_x == grid.x as i32 - grid.cx || min_y == grid.y as i32 - grid.cy || min_z == grid.z as i32 - grid.cz
        || max_x == -grid.cx || max_y == -grid.cy || max_z == -grid.cz {
        // The bounding boxes do not overlap
        return output;
    }

    // Get some properties of the triangle
    let c0 = Vector3::new(vertices[0].0 as f64 , vertices[0].1 as f64, vertices[0].2 as f64); // One vertex
    let v1 = Vector3::new( // One edge
        (vertices[1].0 - vertices[0].0) as f64,
        (vertices[1].1 - vertices[0].1) as f64,
        (vertices[1].2 - vertices[0].2) as f64
    );
    let v2 = Vector3::new( // Another edge
        (vertices[2].0 - vertices[0].0) as f64,
        (vertices[2].1 - vertices[0].1) as f64,
        (vertices[2].2 - vertices[0].2) as f64
    );
    let n = v1.cross(v2); // Normal
    let d = n.dot(c0);
    let dvx = ((max_z - min_z + 1) * (max_y - min_y + 1)) as usize;// Index change if x increased
    let dvy = (max_z - min_z + 1) as usize;// Index change if y increased

    // Get whether each vertex is above or below the triangle and whether it's in the z projection of the triangle
    let mut signs = vec![0; ((max_x - min_x + 1) * (max_y - min_y + 1) * (max_z - min_z + 1)) as usize]; // True means above
    let mut proj_z = vec![false; ((max_x - min_x + 1) * (max_y - min_y + 1) * (max_z - min_z + 1)) as usize]; // True means in the triangle
    {
        let mut index = 0;
        let v0 = (vertices[1].0 - vertices[0].0, vertices[1].1 - vertices[0].1); // Projeced edge of the triangle
        let v1 = (vertices[2].0 - vertices[1].0, vertices[2].1 - vertices[1].1); // Projeced edge of the triangle
        let v2 = (vertices[0].0 - vertices[2].0, vertices[0].1 - vertices[2].1); // Projeced edge of the triangle
        for ix in min_x..=max_x {
            for iy in min_y..=max_y {
                let c0 = (ix - vertices[0].0, iy - vertices[0].1);
                let c1 = (ix - vertices[1].0, iy - vertices[1].1);
                let c2 = (ix - vertices[2].0, iy - vertices[2].1);
                let x0 = v0.0 * c0.1 - v0.1 * c0.0;
                let x1 = v1.0 * c1.1 - v1.1 * c1.0;
                let x2 = v2.0 * c2.1 - v2.1 * c2.0;
                let proj_z_result = (x0 >= 0 && x1 >= 0 && x2 >= 0)||(x0 <= 0 && x1 <= 0 && x2 <= 0);
                for iz in min_z..=max_z {
                    let height = ix as f64 * n.x + iy as f64 * n.y  + iz as f64 * n.z - d;
                    const EPSILON: f64 = 1e-8;
                    signs[index] = if height < -EPSILON {
                        -1 // Vertex was below the triangle
                    } else if height > EPSILON {
                        1 // Vertex was above the triangle
                    } else {
                        0 // Vertex was in the triangle (ignore it)
                    };
                    proj_z[index] = proj_z_result;
                    index += 1;
                }
            }
        }
    }
    // Check x projection
    let mut proj_x = vec![false; ((max_x - min_x + 1) * (max_y - min_y + 1) * (max_z - min_z + 1)) as usize];
    {
        let v0 = (vertices[1].2 - vertices[0].2, vertices[1].1 - vertices[0].1);
        let v1 = (vertices[2].2 - vertices[1].2, vertices[2].1 - vertices[1].1);
        let v2 = (vertices[0].2 - vertices[2].2, vertices[0].1 - vertices[2].1);
        for iz in min_z..=max_z {
            for iy in min_y..=max_y {
                let c0 = (iz - vertices[0].2, iy - vertices[0].1);
                let c1 = (iz - vertices[1].2, iy - vertices[1].1);
                let c2 = (iz - vertices[2].2, iy - vertices[2].1);
                let x0 = v0.0 * c0.1 - v0.1 * c0.0;
                let x1 = v1.0 * c1.1 - v1.1 * c1.0;
                let x2 = v2.0 * c2.1 - v2.1 * c2.0;
                let proj_x_result = (x0 >= 0 && x1 >= 0 && x2 >= 0)||(x0 <= 0 && x1 <= 0 && x2 <= 0);
                for ix in min_x..=max_x {
                    let index = (iz - min_z) as usize + (iy - min_y) as usize * dvy + (ix - min_x) as usize * dvx;
                    proj_x[index] = proj_x_result;
                }
            }
        }
    }
    // Check y projection
    let mut proj_y = vec![false; ((max_x - min_x + 1) * (max_y - min_y + 1) * (max_z - min_z + 1)) as usize];
    {
        let v0 = (vertices[1].2 - vertices[0].2, vertices[1].0 - vertices[0].0); // Projeced edge of the triangle
        let v1 = (vertices[2].2 - vertices[1].2, vertices[2].0 - vertices[1].0); // Projeced edge of the triangle
        let v2 = (vertices[0].2 - vertices[2].2, vertices[0].0 - vertices[2].0); // Projeced edge of the triangle
        for iz in min_z..=max_z {
            for ix in min_x..=max_x {
                let c0 = (iz - vertices[0].2, ix - vertices[0].0);
                let c1 = (iz - vertices[1].2, ix - vertices[1].0);
                let c2 = (iz - vertices[2].2, ix - vertices[2].0);
                let x0 = v0.0 * c0.1 - v0.1 * c0.0;
                let x1 = v1.0 * c1.1 - v1.1 * c1.0;
                let x2 = v2.0 * c2.1 - v2.1 * c2.0;
                let proj_y_result = (x0 >= 0 && x1 >= 0 && x2 >= 0)||(x0 <= 0 && x1 <= 0 && x2 <= 0);
                for iy in min_y..=max_y {
                    let index = (iz - min_z) as usize + (iy - min_y) as usize * dvy + (ix - min_x) as usize * dvx;
                    proj_y[index] = proj_y_result;
                }
            }
        }
    }

    // Iterate through the box to see whether the grid cell intersects the triangle
    let mut index = 0;
    for ix in min_x..max_x {
        for iy in min_y..max_y {
            for iz in min_z..max_z {
                // Check for triangle intersection
                let vertices = [index, index+1, index+dvy, index+dvy+1, index+dvx, index+dvx+1, index+dvx+dvy, index+dvx+dvy+1];
                let mut inside_x = false;
                let mut inside_y = false;
                let mut inside_z = false;
                let mut above = false;
                let mut below = false;
                for vertex in vertices {
                    inside_x |= proj_x[vertex];
                    inside_y |= proj_y[vertex];
                    inside_z |= proj_z[vertex];
                    above |= signs[vertex]==1;
                    below |= signs[vertex]==-1;
                }
                if inside_x && inside_y && inside_z && above && below {
                    // The cell was intersected
                    output.push((ix, iy, iz));
                }
                index += 1;
            }
        }
    }

    output
}