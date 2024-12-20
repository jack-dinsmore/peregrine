use tethys::graphics::model::{LoadModel, LoadMaterial};

fn main() {
    println!("cargo:rerun-if-changed=assets/parts");
    println!("cargo:rerun-if-changed=assets/panels");

    LoadModel::load_obj("assets/parts/tank-cap.obj").save();
    LoadModel::load_obj("assets/parts/tank-body.obj").save();
    LoadModel::load_obj("assets/parts/scaffold.obj").save();
    LoadModel::load_obj("assets/parts/thruster.obj").save();
    LoadModel::load_obj("assets/parts/battery.obj").save();

    LoadMaterial::load_mtl("assets/panels/metal.mtl").save();
}