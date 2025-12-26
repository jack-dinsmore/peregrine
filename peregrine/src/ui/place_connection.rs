use crate::{ship::{orientation, Attachment, AttachmentLayout, Fluid, PartLoader, ShipInterior}, ui::place_tools::{grid_shrink, PlacementTools}, util::Save};
use cgmath::{InnerSpace, Rotation, Vector3};
use tethys::prelude::*;

pub struct PlaceConnectionState {
    pub fluid: Fluid,
    tools: PlacementTools,

    /// Index of the existing hub
    existing: Option<usize>,

    /// Index of the attachment it is connected to
    pub selected_index: Option<usize>,
}

impl PlaceConnectionState {
    pub fn new(part_loader: PartLoader, fluid: Fluid, ship: &ShipInterior) -> Self {
        let attachment = match fluid {
            Fluid::Electricity => Attachment::CircuitHub,
            Fluid::Hydrogen => Attachment::HydrogenHub,
        };
        let layout = AttachmentLayout { x: 0., y: 0., z: 0., orientation: 0 };
        let interior = crate::ship::SaveShipInterior {
            attachments: vec![attachment],
            attachment_layouts: vec![layout],
            ..Default::default()
        };
        Self {
            fluid,
            selected_index: None,
            tools: PlacementTools::new(part_loader.clone(), interior.build(part_loader), ship),
            existing: None,
        }
    }

    pub fn get_placed_layout(&self) -> Option<AttachmentLayout> {
        let mut pos = match self.tools.ship_location {
            Some(loc) => loc,
            None => return None,
        };
        let layout = self.tools.interior.attachment_layouts[0];
        pos -= orientation::to_quat(layout.orientation).rotate_vector(Vector3::new(layout.x as f64, layout.y as f64, layout.z as f64));
        Some(AttachmentLayout {
            x: pos.x as f32,
            y: pos.y as f32,
            z: pos.z as f32,
            orientation: orientation::compose(self.tools.roll, layout.orientation),
        })
    }
    
    pub fn rotate(&mut self, axis: Vector3<f64>) {
        self.tools.rotate(axis)
    }

    pub fn update(&mut self, graphics: &Graphics, camera: &Camera, ship: &ShipInterior) {
        //TODO
        let forward = camera.get_forward::<f64>();
        let line = Collider::Line(
            LineCollider::segment(camera.position, forward * super::place_tools::PLACEMENT_REACH)
        );
        let result = Collider::check_intersection(ship.collider_package(), (&line).into());
        let pos_in_grid = if result.collision() {
            let pos = result.positions[0];

            let my_attachment = self.tools.interior.attachments[0];
            
            // Check nearby hubs
            self.existing = None;
            for (index, (attachment, layout)) in ship.attachments.iter().zip(&ship.attachment_layouts).enumerate() {
                if attachment == &my_attachment {
                    let layout_pos = Vector3::new(layout.x, layout.y, layout.z);
                    if (layout_pos - pos.cast().unwrap()).magnitude() < 0.3 {
                        self.existing = Some(index);
                        break;
                    }
                }
            }

            match self.existing {
                Some(_) => {
                    None
                },
                None => {
                    let grid_pos = grid_shrink(pos, forward);
                    let delta = pos - grid_pos;
                    let layout = &mut self.tools.interior.attachment_layouts[0];
                    layout.x = delta.x as f32;
                    layout.y = delta.y as f32;
                    layout.z = delta.z as f32;
                    
                    Some(grid_pos.cast().unwrap())
                }
            }
        } else {
            None
        };
        self.tools.update(graphics, camera, ship, pos_in_grid);
    }

    pub fn place(&mut self, ship: &mut ShipInterior) {
        // Place a hub
        let old_attachment_index = match self.existing {
            Some(i) => i,
            None => match self.get_placed_layout() {
                Some(layout) => {
                    ship.attachments.push(self.tools.interior.attachments[0]);
                    ship.attachment_layouts.push(layout);
                    ship.attachments.len() - 1
                },
                None => return,
            }
        };

        // Now connect the pipe
        match self.selected_index {
            Some(new_attachment_index) => {
                let pair = (new_attachment_index, old_attachment_index);
                match ship.connections.get_mut(&self.fluid) {
                    Some(value) => {value.push(pair);},
                    None => {ship.connections.insert(self.fluid, vec![pair]).unwrap();},
                }
            },
            None => (),
        }
    }

    pub fn get_objects(&self) -> Vec<ObjectHandle<'_>> {
        self.tools.get_placement_objects()
        // TODO draw pipe too
    }
}