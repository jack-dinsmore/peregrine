#[derive(Clone, Copy)]
pub enum Fluid {
    Electricity,
    Hydrogen
}
impl Fluid {
    fn get_color(self) -> Vector3<f64> {

    }
}

pub struct Circuit {
    fluid: Fluid,
    parts: Vec<usize>,
    connections: Vec<(usize, usize)>,
    quantity: f64,
}