pub mod container;
mod loading;
mod mesh;
mod material;

use super::Graphics;
use container::{Container, Loader, MaybeInstanced};
pub use mesh::Mesh;

pub use loading::{LoadModel, LoadMaterial};
pub use material::{Material, MaterialContainer, MaterialLoader};

pub type Model = MaybeInstanced<(Vec<Mesh>, Vec<Material>)>;
pub type ModelContainer<const CAPACITY: usize> = Container<CAPACITY, (Vec<Mesh>, Vec<Material>)>;
pub type ModelLoader<'a, const CAPACITY: usize> = Loader<'a, CAPACITY, (Vec<Mesh>, Vec<Material>)>;

impl Model {
    pub fn new(graphics: &Graphics, obj: LoadModel) -> Model {
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

    /// Get an identifier for this model for the sake of sorting the models
    pub(crate) fn identifier(&self) -> usize {
        match self {
            Model::Singleton(_) => 0,
            Model::Instance(instance) => instance.identifier(),
        }
    }
}