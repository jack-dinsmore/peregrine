use cgmath::{Quaternion, Rotation, Vector3};
use part::{Block, PartModel};
use serde::{Deserialize, Serialize};
use tethys::{physics::collisions::{ColliderPackage, GridCollider}, prelude::*};

mod part;
mod panel;
mod grid;
pub mod orientation;

pub use part::{Part, PartLayout,  PartData, PartLoader};
pub use panel::{Panel, PanelModel, PanelLayout};
pub use grid::*;

use crate::util::Save;

/// This is the maximum number of parts, because the panel index will start to take over from here
const PANEL_START_INDEX: usize = 65536;

/// Contains the data of a single ship, including its internal components, its hull model, its 
/// physics data, and its simulated properties
pub struct ShipInterior {
    parts: Vec<Part>,
    part_layouts: Vec<PartLayout>,
    pub panels: Vec<Panel>,
    panel_layouts: Vec<PanelLayout>,

    // Physics
    pub rigid_body: RigidBody,
    collider: Collider,
    placement_objects: Option<Vec<Object>>,

    // Graphics
    panel_objects: Vec<Object>,
    part_objects: Vec<Block>,
}

impl ShipInterior {
    pub fn new(loader: PartLoader, template: SaveShipInterior) -> Self {
        let mut part_objects = Vec::with_capacity(template.parts.len());
        let mut panel_objects = Vec::with_capacity(template.panels.len());
        let mut grid = GridCollider::new();
        for (i, (part, layout)) in template.parts.iter().zip(&template.part_layouts).enumerate() {
            part_objects.append(&mut part.get_objects(loader.clone(), *layout));
            add_part_to_grid(&mut grid, part, *layout, i);
        }
        for (i, (panel, layout)) in template.panels.iter().zip(&template.panel_layouts).enumerate() {
            if let Some(object) = panel.get_object(loader.clone(), *layout) {
                panel_objects.push(object);
            }
            add_panel_to_grid(&mut grid, panel, (PANEL_START_INDEX + i) as isize);
        }
        Self {
            parts: template.parts,
            part_layouts: template.part_layouts,
            collider: Collider::Grid(grid),
            rigid_body: template.rigid_body,
            placement_objects: None,
            panels: template.panels,
            panel_layouts: template.panel_layouts,
            part_objects,
            panel_objects,
        }
    }

    /// Update all the objects within the ship according to the physics component
    pub fn update(&mut self, delta_t: f64) {
        self.rigid_body.update(delta_t);
        self.update_graphics();
    }

    /// Update all the objects within the ship according to the physics component
    pub fn update_graphics(&mut self) {
        // TODO Remove this
        for block in &mut self.part_objects {
            let (position, orientation) = block.layout.as_physical();
            block.object.position = self.rigid_body.pos + self.rigid_body.orientation.rotate_vector(position);
            block.object.orientation = self.rigid_body.orientation * orientation;
        }
        for object in &mut self.panel_objects {
            object.position = self.rigid_body.pos;
            object.orientation = self.rigid_body.orientation;
        }
    }
    
    pub fn objects(&self) -> Vec<ObjectHandle> {
        let mut output = Vec::with_capacity(self.parts.len() + self.panels.len());
        for block in &self.part_objects {
            output.push(ObjectHandle::Ref(&block.object));
        }
        for object in &self.panel_objects {
            output.push(ObjectHandle::Ref(object));
        }
        output
    }
    
    /// Get the list of objects to be painted with the ``placing`` texture
    pub fn get_placement_objects(&self) -> Vec<ObjectHandle> {
        self.placement_objects.as_ref().unwrap().iter().map(|o| ObjectHandle::Ref(&o)).collect::<Vec<_>>()
    }

    /// Get the ship ready for placing parts
    pub fn initialize_placement(&mut self, part_loader: PartLoader) {
        if self.placement_objects.is_none() {
            let mut objects = Vec::new();
            let placement_model = part_loader.load_part(PartModel::Placement);
            let orientation = Quaternion::new(1., 0., 0., 0.,);
            for ((x, y, z), part_number) in self.collider.get_grid_collider().unwrap().indexed_iter() {
                if part_number == -1 {continue;}
                let pos = Vector3::new(x as f64 + 0.5, y as f64 + 0.5,z as f64 + 0.5);
                objects.push(Object::new(part_loader.graphics, placement_model.clone(), pos, orientation));
            }
            self.placement_objects = Some(objects);
        }
    }

    pub(crate) fn collider_package(&self) -> ColliderPackage {
        (&self.collider, &self.rigid_body).into()
    }
    
    pub(crate) fn is_new_part_allowed(&self, part: Part, layout: PartLayout) -> bool {
        for block in part.get_blocks(layout) {
            if self.collider.get_grid_collider().unwrap().get_entry(block.x, block.y, block.z) != -1 {
                return false;
            }
        }
        true
    }
    
    pub(crate) fn is_new_line_allowed(&self, vertices: [(i32, i32, i32); 2]) -> bool {
        let start = Vector3::new(vertices[0].0 as f64, vertices[0].1 as f64, vertices[0].2 as f64);
        let stop = Vector3::new(vertices[1].0 as f64, vertices[1].1 as f64, vertices[1].2 as f64);
        let line = Collider::Line(LineCollider::segment(start, stop-start));
        let collision = Collider::check_intersection(self.collider_package(), (&line).into());
        collision.collision()
    }
    
    pub(crate) fn is_new_panel_allowed(&self, vertices: [(i32, i32, i32); 3]) -> bool {
        let grid = self.collider.get_grid_collider().unwrap();
        for (x, y, z) in get_triangle_intersections(grid, vertices) {
            if grid.get_entry(x, y, z) != -1 {
                return false;
            }
        }
        true
    }

    pub(crate) fn add_part(&mut self, part_loader: PartLoader, part: Part, layout: PartLayout) {
        let part_index = self.parts.len();
        self.parts.push(part);
        self.part_layouts.push(layout);
        let mut objects = part.get_objects(part_loader.clone(), layout);
        self.part_objects.append(&mut objects);
        add_part_to_grid(self.collider.get_grid_collider_mut().unwrap(), &part, layout, part_index);
        
        if let Some(objects) = &mut self.placement_objects {
            let placement_model = part_loader.load_part(PartModel::Placement);
            let orientation = Quaternion::new(1., 0., 0., 0.,);
            for block in part.get_blocks(layout) {
                let pos = Vector3::new(block.x as f64, block.y as f64, block.z as f64);
                objects.push(Object::new(part_loader.graphics, placement_model.clone(), pos, orientation));
            }
        }
    }
    
    pub(crate) fn add_panel(&mut self, loader: PartLoader, panel: Panel, layout: PanelLayout) {
        let panel_index = (self.panels.len() + PANEL_START_INDEX) as isize;
        self.panels.push(panel.clone());
        self.panel_layouts.push(layout);
        if let Some(object) = panel.get_object(loader, layout) {
            self.panel_objects.push(object);
        }
        let grid = self.collider.get_grid_collider_mut().unwrap();
        add_panel_to_grid(grid, &panel, panel_index);
    }
}


#[derive(Serialize, Deserialize)]
pub struct SaveShipInterior {
    pub parts: Vec<Part>,
    pub part_layouts: Vec<PartLayout>,
    pub panels: Vec<Panel>,
    pub panel_layouts: Vec<PanelLayout>,
    pub rigid_body: RigidBody,
}

impl Save<ShipInterior, PartLoader<'_>> for SaveShipInterior {
    fn build(self, loader: PartLoader) -> ShipInterior {
        ShipInterior::new(loader, self)
    }
}