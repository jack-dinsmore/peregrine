use crate::ship::{Fluid, PartLoader, ShipInterior};
use cgmath::Vector3;
use tethys::prelude::*;

pub struct ConnectionState {
    pub fluid: Fluid,

    // Graphics
    line_model: Model,
    line_objects: Vec<Object>,
    circle_objects: Vec<Object>,
}

impl ConnectionState {
    pub fn new(fluid: Fluid, part_loader: PartLoader, ship: &ShipInterior) -> Self {
        const LINE_VERTICES: [PointVertex; 4] = [
            PointVertex { position: [0., 0., 0.] },
            PointVertex { position: [0., 0., 1.] },
            PointVertex { position: [1., 0., 1.] },
            PointVertex { position: [1., 0., 0.] },
        ];
        const LINE_INDICES: [u16; 6] = [
            0,1,2,0,2,3,
        ];
        const CIRCLE_VERTICES: [PointVertex; 16] = [
            PointVertex { position: [0.0, 0.0, 0.] },
            PointVertex { position: [1.0, 0.0, 0.] },
            PointVertex { position: [0.9135454576426009, 0.40673664307580015, 0.] },
            PointVertex { position: [0.6691306063588582, 0.7431448254773942, 0.] },
            PointVertex { position: [0.30901699437494745, 0.9510565162951535, 0.] },
            PointVertex { position: [-0.10452846326765333, 0.9945218953682734, 0.] },
            PointVertex { position: [-0.49999999999999983, 0.8660254037844387, 0.] },
            PointVertex { position: [-0.8090169943749473, 0.5877852522924732, 0.] },
            PointVertex { position: [-0.9781476007338056, 0.20791169081775973, 0.] },
            PointVertex { position: [-0.9781476007338057, -0.20791169081775907, 0.] },
            PointVertex { position: [-0.8090169943749475, -0.587785252292473, 0.] },
            PointVertex { position: [-0.5000000000000004, -0.8660254037844384, 0.] },
            PointVertex { position: [-0.10452846326765423, -0.9945218953682733, 0.] },
            PointVertex { position: [0.30901699437494723, -0.9510565162951536, 0.] },
            PointVertex { position: [0.6691306063588578, -0.7431448254773946, 0.] },
            PointVertex { position: [0.9135454576426005, -0.406736643075801, 0.] },
        ];
        const CIRCLE_INDICES: [u16; 45] = [
            0,1,2,0,2,3,0,3,4,0,4,5,0,5,6,0,6,7,0,7,8,0,8,9,0,9,10,0,10,11,0,11,12,0,12,13,0,13,14,0,14,15,0,15,1
        ];
        let line_model = Model::from_vertices(part_loader.graphics, &LINE_VERTICES, &LINE_INDICES);
        let circle_model = Model::from_vertices(part_loader.graphics, &CIRCLE_VERTICES, &CIRCLE_INDICES);

        let mut line_objects = Vec::new();
        let mut circle_objects = Vec::new();
        for circuit in &ship.circuits {
            if circuit.fluid != fluid {continue};
            for _connection in &circuit.connections {
                line_objects.push(Object::zeroed::<SolidUniform>(part_loader.graphics, line_model.clone()));
            }
        }
        for part in &ship.parts {
            if part.typical_power_draw().is_some() {
                circle_objects.push(Object::zeroed::<SolidUniform>(part_loader.graphics, circle_model.clone()));
            }
        }

        Self {
            fluid,
            line_model,
            line_objects,
            circle_objects,
        }
    }

    pub fn update(&self, graphics: &Graphics, camera: &Camera, closest_ship: &ShipInterior) {
        self.update_graphics(graphics, camera, closest_ship);
    }

    pub fn update_graphics(&self, graphics: &Graphics, camera: &Camera, ship: &ShipInterior) {
        let mut i = 0;
        for circuit in &ship.circuits {
            if circuit.fluid != self.fluid {continue};
            for connection in &circuit.connections {
                let start = Vector3::new(
                    ship.part_layouts[connection.0].x as f64,
                    ship.part_layouts[connection.0].y as f64,
                    ship.part_layouts[connection.0].z as f64
                );
                let stop = Vector3::new(
                    ship.part_layouts[connection.1].x as f64,
                    ship.part_layouts[connection.1].y as f64,
                    ship.part_layouts[connection.1].z as f64
                );
                let uniform = SolidUniform::line(camera, start, stop, self.fluid.get_color());
                self.line_objects[i].update(graphics, uniform);
                i += 1;
            }
        }
        let mut i = 0;
        for (part, layout) in ship.parts.iter().zip(&ship.part_layouts) {
            if part.typical_power_draw().is_some() {
                let pos = Vector3::new(layout.x as f64 + 0.5, layout.y as f64 + 0.5, layout.z as f64 + 0.5);
                let uniform = SolidUniform::circle(camera, pos, self.fluid.get_color());
                self.circle_objects[i].update(graphics, uniform);
                i += 1;
            }
        }
    }
    
    pub(crate) fn get_connected_objects(&self) -> Vec<ObjectHandle> {
        self.line_objects.iter().map(|o| ObjectHandle::Ref(&o))
            .chain(self.circle_objects.iter().map(|o| ObjectHandle::Ref(&o)))
            .collect::<Vec<_>>()
    }
}
