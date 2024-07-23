extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn load_obj(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let path = input.value();
    let base_path = std::path::Path::new(&path).parent().unwrap();

    let obj_buf = fs::read_to_string(&path).expect(&format!("Failed to read OBJ file {:?}", path));
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

    let mesh_code = models.iter().map(|model| {
        let mesh = &model.mesh;
        let positions = mesh.positions.clone();
        let normals = mesh.normals.clone();
        let texcoords = mesh.texcoords.clone();
        let indices = mesh.indices.clone();
        let material_id = mesh.material_id.unwrap_or(0);
        
        quote! {
            LoadMesh {
                positions: vec![#(#positions),*],
                normals: vec![#(#normals),*],
                texcoords: vec![#(#texcoords),*],
                indices: vec![#(#indices),*],
                material_id: #material_id,
            }
        }
    });

    let material_code = materials.unwrap_or_default().into_iter().map(|material| {
        let name = &material.name;
        let diffuse = &material.diffuse.unwrap_or([0., 0., 0.]);
        let specular = &material.specular.unwrap_or([0., 0., 0.]);
        let shininess = material.shininess.unwrap_or(0.);
        let normal_texture = match material.normal_texture {
            Some(path) => {
                let path = base_path.join(path);
                fs::read(&path).expect(&format!("Could not open texture {:?}", path))
            },
            None => Vec::new()
        };
        let diffuse_texture = match material.diffuse_texture {
            Some(path) => {
                let path = base_path.join(path);
                fs::read(&path).expect(&format!("Could not open texture {:?}", path))
            },
            None => Vec::new()
        };

        quote! {
            LoadMaterial {
                name: #name.to_string(),
                diffuse: [#(#diffuse),*],
                specular: [#(#specular),*],
                shininess: #shininess,
                normal_texture: vec![#(#normal_texture),*],
                diffuse_texture: vec![#(#diffuse_texture),*],
            }
        }
    });

    let expanded = quote! {
        LoadedObj {
            meshes: vec![
                #(#mesh_code),*
            ],
            materials: vec![
                #(#material_code),*
            ]
        }
    };

    expanded.into()
}
