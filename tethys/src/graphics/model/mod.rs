pub mod container;
mod loading;
mod mesh;
mod material;

use std::sync::Arc;

use crate::graphics::primitives::*;

use super::Graphics;
use container::{Container, Loader, MaybeInstanced};
pub use mesh::Mesh;

pub use loading::{LoadModel, LoadMaterial};
pub use material::{Material, MaterialContainer, MaterialLoader};

pub type Model = MaybeInstanced<(Vec<Mesh>, Vec<Material>)>;
pub type ModelContainer<const CAPACITY: usize> = Container<CAPACITY, (Vec<Mesh>, Vec<Material>)>;
pub type ModelLoader<'a, const CAPACITY: usize> = Loader<'a, CAPACITY, (Vec<Mesh>, Vec<Material>)>;

impl Model {
    pub fn from_obj(graphics: &Graphics, obj: LoadModel) -> Model {
        let mut meshes = Vec::with_capacity(obj.meshes.len());
        for load_mesh in &obj.meshes {
            meshes.push(Mesh::from_obj(graphics, load_mesh));
        }

        let mut materials = Vec::with_capacity(obj.materials.len());
        for load_material in &obj.materials {
            materials.push(Material::new(graphics, load_material));
        }

        Model::Singleton(Arc::new((meshes, materials)))
    }

    /// Generate a model that uses one material
    pub fn from_vertices_and_material<V: Vertex>(graphics: &Graphics, vertices: &[V], indices: &[u16], material: Material) -> Model {
        let meshes = vec![Mesh::from_vertices(graphics, vertices, indices, 0)];
        let materials = vec![material];
        Model::Singleton(Arc::new((meshes, materials)))
    }

    /// Generate a model that uses no materials
    pub fn from_vertices<V: Vertex>(graphics: &Graphics, vertices: &[V], indices: &[u16]) -> Model {
        let meshes = vec![Mesh::from_vertices(graphics, vertices, indices, 0)];
        Model::Singleton(Arc::new((meshes, Vec::new())))
    }

    /// Get an identifier for this model for the sake of sorting the models
    pub(crate) fn identifier(&self) -> usize {
        match self {
            Model::Singleton(_) => 0,
            Model::Instance(instance) => instance.identifier(),
        }
    }
}