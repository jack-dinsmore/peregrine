use cgmath::{InnerSpace, Quaternion, Rotation, Vector3};
use tethys::prelude::*;

use crate::{ship::{orientation::{self, from_quat}, Part, PartLayout, PartLoader, SaveShipInterior, ShipInterior}, util::Save};

const MAX_DISTANCE: f64 = 5.;

pub struct PlacePartState {
    // Part information
    interior: ShipInterior,
    part: Part,
    display: bool,
    place_coord: Vector3<f64>,// The coordinate on interior that should go where the mouse is
    part_orientation: u8, // Orientation the part would have
    layout: Option<PartLayout>,

    // Placement information
    placement_model: Model,
    placement_objects: Vec<Object>,
}

impl PlacePartState {
    pub fn new(part_loader: PartLoader, part: Part, ship: &ShipInterior) -> Self {
        // Initialize the new part
        let rigid_body = RigidBody::default();
        let layout = PartLayout { x: 0, y: 0, z: 0, orientation: 0 };
        let save = SaveShipInterior {
            parts: vec![part.clone()],
            part_layouts: vec![layout.clone()],
            panels: Vec::new(),
            panel_layouts: Vec::new(),
            rigid_body,
        };

        // Initialize the placement blocks
        const PLACEMENT_VERTICES: [PointVertex ; 8] = [
            PointVertex { position: [0.5, 0.5, 0.5] },
            PointVertex { position: [-0.5, 0.5, 0.5] },
            PointVertex { position: [0.5, -0.5, 0.5] },
            PointVertex { position: [-0.5, -0.5, 0.5] },
            PointVertex { position: [0.5, 0.5, -0.5] },
            PointVertex { position: [-0.5, 0.5, -0.5] },
            PointVertex { position: [0.5, -0.5, -0.5] },
            PointVertex { position: [-0.5, -0.5, -0.5] },
        ];
        const PLACEMENT_INDICES: [u16; 24] = [
            0, 1, 1, 3, 3, 2, 2, 0,
            4, 5, 5, 7, 7, 6, 6, 4,
            0, 4, 1, 5, 2, 6, 3, 7,
        ];
        let placement_model = Model::from_vertices(&part_loader.graphics, &PLACEMENT_VERTICES, &PLACEMENT_INDICES);
        let mut placement_objects = Vec::new();
        for (_, part_number) in ship.collider.get_grid_collider().unwrap().indexed_iter() {
            if part_number == -1 {continue;}
            placement_objects.push(Object::zeroed::<ObjectUniform>(part_loader.graphics, placement_model.clone()));
        }

        Self {
            part_orientation: from_quat(save.rigid_body.orientation),
            interior:  save.build(part_loader),
            display: false,
            place_coord: Vector3::new(0., 0., 0.),
            part,
            layout: None,
            placement_objects,
            placement_model,
        }
    }

    pub fn rotate(&mut self, axis: Vector3<f32>) {
        let reorient = if axis.x.abs() > axis.y.abs() && axis.x.abs() > axis.z.abs() {
            if axis.x > 0. {
                // Rotate around +x
                Quaternion::new(0., 1., 0., 0.)
            } else {
                // Rotate around -x
                Quaternion::new(0., 1., 0., 0.)
            }
        } else if axis.y.abs() > axis.x.abs() && axis.y.abs() > axis.z.abs() {
            if axis.y > 0. {
                // Rotate around +y
                Quaternion::new(0., 0., 1., 0.)
            } else {
                // Rotate around -y
                Quaternion::new(0., 0., -1., 0.)
            }
        } else {
            if axis.z > 0. {
                // Rotate around +z
                Quaternion::new(0., 0., 0., 1.)
            } else {
                // Rotate around -z
                Quaternion::new(0., 0., 0., -1.)
            }
        };
        self.place_coord = reorient.rotate_vector(self.place_coord);
        self.part_orientation = orientation::rotate_by_quat(self.part_orientation, reorient);
    }

    pub fn update(&mut self, graphics: &Graphics, camera: &Camera, closest_ship: &ShipInterior) {
        self.display = false;
        
        // Get the intersection of the mouse pointer with the body
        let forward = camera.get_forward::<f64>().normalize();
        let line = Collider::Line(
            LineCollider::segment(camera.position, forward * MAX_DISTANCE)
        );
        let result = Collider::check_intersection(closest_ship.collider_package(), (&line).into());
        if !result.collision() { return; }

        // Check to see if the part can be placed
        let mut pos_in_grid = closest_ship.rigid_body.to_local(result.positions[0] - forward * 0.001) - self.place_coord;
        if forward.x > 0. {
            pos_in_grid.x = pos_in_grid.x.floor();
        } else {
            pos_in_grid.x = pos_in_grid.x.ceil();
        }
        if forward.y > 0. {
            pos_in_grid.y = pos_in_grid.y.floor();
        } else {
            pos_in_grid.y = pos_in_grid.y.ceil();
        }
        if forward.z > 0. {
            pos_in_grid.z = pos_in_grid.z.floor();
        } else {
            pos_in_grid.z = pos_in_grid.z.ceil();
        }
        let layout = PartLayout {
            x: pos_in_grid.x as i32,
            y: pos_in_grid.y as i32,
            z: pos_in_grid.z as i32,
            orientation: self.part_orientation,
        };
        if !closest_ship.is_new_part_allowed(self.part, layout) { return; }

        // Show the part
        self.layout = Some(layout);
        self.interior.rigid_body.orientation = closest_ship.rigid_body.orientation * orientation::to_quat(self.part_orientation);
        self.interior.rigid_body.pos = closest_ship.rigid_body.to_global(pos_in_grid);
        self.display = true;
        self.update_graphics(graphics, camera, closest_ship);
    }

    pub fn update_graphics(&self, graphics: &Graphics, camera: &Camera, ship: &ShipInterior) {
        self.interior.update_graphics(graphics, camera);

        let mut i = 0;
        let orientation = Quaternion::new(1., 0., 0., 0.,);
        for ((x, y, z), part_number) in ship.collider.get_grid_collider().unwrap().indexed_iter() {
            if part_number == -1 {continue;}
            let pos = Vector3::new(x as f64 + 0.5, y as f64 + 0.5,z as f64 + 0.5);
            self.placement_objects[i].update(graphics, ObjectUniform::new(camera, pos, orientation));
            i += 1;
        }
    }

    pub fn get_placement_objects(&self) -> Vec<ObjectHandle> {
        self.placement_objects.iter().map(|o| ObjectHandle::Ref(&o)).collect::<Vec<_>>()
    }

    pub fn place(&mut self, part_loader: PartLoader, closest_ship: &mut ShipInterior) {
        if let Some(layout) = self.layout {
            closest_ship.add_part(part_loader.clone(), self.part, layout);
            for _block in self.part.get_blocks(layout) {
                self.placement_objects.push(Object::zeroed::<ObjectUniform>(part_loader.graphics, self.placement_model.clone()));
            }
        }
    }
    
    pub fn object(&self) -> Vec<ObjectHandle> {
        if self.display {
            self.interior.objects()
        }
        else {
            Vec::new()
        }
    }
}