mod container;
mod loading;
mod mesh;
mod material;
mod texture;

use super::Graphics;
use container::ModelInstance;
use mesh::Mesh;
use material::Material;

pub use container::{ModelContainer, ModelLoader};
pub use loading::LoadedObj;
pub use texture::Texture;

pub(crate) type ModelInner = (Vec<Mesh>, Vec<Material>);

#[allow(private_interfaces)]
pub enum Model {
    Singleton (ModelInner),
    Instance (ModelInstance),
}
impl Model {
    pub fn new(graphics: &Graphics, obj: LoadedObj) -> Model {
        let mut meshes = Vec::with_capacity(obj.meshes.len());
        for load_mesh in &obj.meshes {
            meshes.push(Mesh::new(graphics, load_mesh));
        }

        let mut materials = Vec::with_capacity(obj.materials.len());
        for load_material in &obj.materials {
            materials.push(Material::new(graphics, load_material));
        }

        Model::Singleton((meshes, materials))
    }

    pub(crate) fn inner<'a>(&'a self) -> &'a ModelInner {
        match self {
            Self::Singleton (inner) => inner,
            Self::Instance (instance) => instance.as_ref()
        }
    }

    /// Get an identifier for this model for the sake of sorting the models
    pub(crate) fn identifier(&self) -> usize {
        match self {
            Model::Singleton(_) => 0,
            Model::Instance(instance) => instance.identifier(),
        }
    }
}

impl Clone for Model {
    fn clone(&self) -> Self {
        match self {
            Self::Singleton(_) => panic!("You must not clone a singleton model"),
            Self::Instance(instance) => Self::Instance(instance.clone()),
        }
    }
}