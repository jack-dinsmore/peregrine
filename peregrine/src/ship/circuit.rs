use cgmath::Vector3;

#[derive(Clone, Copy, PartialEq)]
pub enum Fluid {
    Electricity,
    Hydrogen
}
impl Fluid {
    pub fn get_color(self) -> Vector3<f32> {
        match self {
            Fluid::Electricity => Vector3::new(1., 1., 0.),
            Fluid::Hydrogen => Vector3::new(0.5, 0.5, 1.),
        }
    }
}

pub struct Circuit {
    pub fluid: Fluid,
    parts: Vec<usize>,
    pub connections: Vec<(usize, usize)>,
    quantity: f64,
}