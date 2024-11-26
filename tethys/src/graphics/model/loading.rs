use std::{fs::{self, File}, io::Write};

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct LoadModel {
    pub file_path: String,
    pub meshes: Vec<LoadMesh>,
    pub materials: Vec<LoadMaterial>,
}

#[derive(Serialize, Deserialize)]
pub struct LoadMesh {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub indices: Vec<u32>,
    pub material_id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct LoadMaterial {
    pub name: String,
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub normal_texture: Vec<u8>,
    pub diffuse_texture: Vec<u8>,
}

impl LoadMaterial {
    pub fn load_mtl(file_path: &str) -> Self {
        let base_path = std::path::Path::new(&file_path).parent().unwrap();
        let mat_buf = fs::read_to_string(&file_path).expect(&format!("Failed to read MTL file {:?}", file_path));
        let materials = tobj::load_mtl_buf(&mut mat_buf.as_bytes()).unwrap_or_default().0;
        if materials.len() == 0 {
            panic!("The material file was empty");
        }
        Self::load_mtl_type(&materials[0], base_path)
    }

    fn load_mtl_type(material: &tobj::Material, base_path: &std::path::Path) -> Self {
        let name = &material.name;
        let diffuse = material.diffuse.unwrap_or([0., 0., 0.]);
        let specular = material.specular.unwrap_or([0., 0., 0.]);
        let shininess = material.shininess.unwrap_or(0.);
        let normal_texture: Vec<u8> = match &material.normal_texture {
            Some(path) => {
                let path = base_path.join(path);
                fs::read(&path).expect(&format!("Could not open texture {:?}", path))
            },
            None => Vec::new()
        };
        let diffuse_texture = match &material.diffuse_texture {
            Some(path) => {
                let path = base_path.join(path);
                fs::read(&path).expect(&format!("Could not open texture {:?}", path))
            },
            None => Vec::new()
        };

        LoadMaterial {
            name: name.to_string(),
            diffuse,
            specular,
            shininess, 
            normal_texture,
            diffuse_texture,
        }
    }

    pub fn save(&self) {
        let serialized = bincode::serialize(&self).unwrap();

        let mut file = File::create(format!("build/{}.bin", self.name)).unwrap();
        file.write_all(&serialized).unwrap();
    }
}

impl LoadModel {
    pub fn load_obj(file_path: &str) -> Self {
        let base_path = std::path::Path::new(&file_path).parent().unwrap();
    
        let obj_buf = fs::read_to_string(&file_path).expect(&format!("Failed to read OBJ file {:?}", file_path));
        let (models, materials) = tobj::load_obj_buf(
            &mut obj_buf.as_bytes(),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| {
                let mtl_path = base_path.join(p);
                let mat_buf = fs::read_to_string(&mtl_path).expect(&format!("Failed to read MTL file {:?}", p));
                tobj::load_mtl_buf(&mut mat_buf.as_bytes())
            },
        ).expect("Failed to load OBJ data");
    
        let meshes = models.iter().map(|model| {
            let mesh = &model.mesh;
            let positions = mesh.positions.clone();
            let normals = mesh.normals.clone();
            let texcoords = mesh.texcoords.clone();
            let indices = mesh.indices.clone();
            let material_id = mesh.material_id.unwrap_or(0);
            LoadMesh {
                positions,
                normals,
                texcoords,
                indices,
                material_id,
            }
        }).collect::<Vec<_>>();

        let materials = materials.unwrap_or_default().iter().map(|material| {
            LoadMaterial::load_mtl_type(material, base_path)
        }).collect::<Vec<_>>();
    
        Self {
            file_path: file_path.to_owned(),
            meshes,
            materials
        }
    }

    pub fn save(&self) {
        let end = std::path::Path::new(&self.file_path).file_stem().unwrap().to_str().unwrap();
        let serialized = bincode::serialize(&self).unwrap();

        let mut file = File::create(format!("build/{}.bin", end)).unwrap();
        file.write_all(&serialized).unwrap();
    }
}

#[macro_export]
macro_rules! include_model {
    ($file:expr) => {{
        let bytes: &[u8] = include_bytes!(concat!("../../build/", $file, ".bin"));
        bincode::deserialize::<LoadModel>(bytes).expect("Failed to deserialize the file")
    }};
}

#[macro_export]
macro_rules! include_material {
    ($file:expr) => {{
        let bytes: &[u8] = include_bytes!(concat!("../../build/", $file, ".bin"));
        bincode::deserialize::<LoadMaterial>(bytes).expect("Failed to deserialize the file")
    }};
}