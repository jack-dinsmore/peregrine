use tethys::graphics::model::LoadedObj;

fn main() {
    println!("cargo:rerun-if-changed=assets/parts");

    LoadedObj::load_obj("assets/parts/tank-cap.obj").save();
    LoadedObj::load_obj("assets/parts/tank-body.obj").save();
    LoadedObj::load_obj("assets/parts/placement.obj").save();
    // LoadedObj::load_obj("assets/parts/fuel-cell.obj").save();
    // LoadedObj::load_obj("assets/parts/box.obj").save();
}