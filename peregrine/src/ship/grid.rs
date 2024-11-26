use tethys::{physics::collisions::ColliderPackage, prelude::*};
use super::{Panel, PanelLayout, Part, PartLayout};


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

/// Add a part to the grid, where data represents the index of the part
pub(super) fn add_panel_to_grid(collider: &mut GridCollider, panel: &Panel, layout: PanelLayout, data: usize) {
    // The part is guaranteed to lie within the grid, but it may take up many spaces so these should all be replaced with the panel data.
    unimplemented!()
}

/// Get the closest corner to the given ray 
pub(super) fn get_corner(grid: ColliderPackage, line: LineCollider) -> Option<(i32, i32, i32)> {
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