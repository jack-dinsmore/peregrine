use cgmath::Vector3;
use tethys::prelude::*;
use crate::ship::{get_corner, SaveShipInterior};
use crate::ship::{Panel, PanelLayout, PartLoader, ShipInterior};
use crate::ui::place_tools::{PlacementTools, PLACEMENT_REACH};
use crate::util::Save;

pub struct PlacePanelState {
    // Panel information
    panel_layout: PanelLayout,
    num_vertices_placed: usize,

    tools: PlacementTools,
}
impl PlacePanelState {
    pub fn new(loader: PartLoader, panel: Panel, ship: &ShipInterior) -> Self {
        let layout = PanelLayout {
            vertices: [(0, 0, 0); 3],
        };
        let save = SaveShipInterior {
            panels: vec![panel.clone()],
            panel_layouts: vec![layout.clone()],
            ..Default::default()
        };

        Self {
            panel_layout: layout,
            num_vertices_placed: 0,
            tools: PlacementTools::new(loader.clone(), save.build(loader), ship),
        }
    }

    pub(crate) fn update(&mut self, graphics: &Graphics, camera: &Camera, ship: &ShipInterior) {
        // Get the intersection of the mouse pointer with the body
        let line = LineCollider::segment(camera.position, camera.get_forward::<f64>() * PLACEMENT_REACH);
        let corner = get_corner(ship.collider_package(), line);
        let layout = &mut self.tools.interior.panel_layouts[0];
        let pos_in_grid = match corner {
            Some(corner) => match self.num_vertices_placed {
                0 => {
                    layout.vertices[0] = corner;
                    Some(Vector3::new(0., 0., 0.))
                },
                1 => {
                    if !ship.is_new_line_allowed([layout.vertices[0], corner]) {
                        None
                    } else {
                        layout.vertices[1] = corner;
                        Some(Vector3::new(0., 0., 0.))
                    }
                },
                2.. => {
                    if ship.is_new_panel_allowed([layout.vertices[0], layout.vertices[1], corner]) {
                        layout.vertices[2] = corner;
                        Some(Vector3::new(0., 0., 0.))
                    } else {
                        None
                    }
                },
            },
            None => None
        };

        if let Some(_) = &pos_in_grid {
            self.num_vertices_placed += 1;
        }
        self.tools.update(graphics, camera, ship, pos_in_grid);
    }
    
    /// Add the panel vertex. If fewer than three vertices have been selected so far, this will not place the panel.
    pub(crate) fn place(&mut self, loader: PartLoader, ship: &mut ShipInterior) {
        let panel = &mut self.tools.interior.panels[0];
        if self.num_vertices_placed == 2 {
            ship.add_panel(loader, panel.clone(), self.panel_layout);
        }
    }

    pub fn get_objects(&self) -> Vec<ObjectHandle<'_>> {
        self.tools.get_placement_objects()
    }
}