use tethys::graphics::model::LoadModel;

fn main() {
    println!("cargo:rerun-if-changed=assets/parts");

    LoadModel::load_obj("assets/parts/tank-cap.obj").save();
    LoadModel::load_obj("assets/parts/tank-body.obj").save();
    LoadModel::load_obj("assets/parts/placement.obj").save();
    // LoadObj::load_obj("assets/parts/fuel-cell.obj").save();
    // LoadObj::load_obj("assets/parts/box.obj").save();

    LoadModel::load_obj("assets/panels/metal.mtl").save();
}