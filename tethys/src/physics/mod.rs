use cgmath::{InnerSpace, Quaternion, Rotation, Vector3, Zero};

pub mod collisions;


#[derive(Debug)]
pub struct RigidBody {
    pub pos: Vector3<f64>,
    pub orientation: Quaternion<f64>,
    pub vel: Vector3<f64>,
    pub angvel: Quaternion<f64>,
    pub force: Vector3<f64>,
    pub torque: Quaternion<f64>,
    pub mass: f64,
    pub moi: (f64, f64, f64),
}

impl RigidBody {
    pub fn new(pos: Vector3<f64>, vel: Vector3<f64>, orientation: Quaternion<f64>, angvel: Vector3<f64>, mass: f64, moi: (f64, f64, f64)) -> Self {
        Self {
            pos,
            orientation,
            vel,
            angvel: Quaternion::from_sv(0., angvel),
            mass,
            moi,
            force: Vector3::new(0., 0., 0.),
            torque: Quaternion::new(0., 0., 0., 0.),
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        self.vel += self.force * delta_t / self.mass;
        self.angvel += Quaternion::new(
            0.,
            self.torque.v.x * delta_t / self.moi.0,
            self.torque.v.y * delta_t / self.moi.1,
            self.torque.v.z * delta_t / self.moi.2
        );
        self.pos += self.vel * delta_t;
        self.orientation += self.angvel * self.orientation * 0.5 * delta_t;
        self.orientation = self.orientation.normalize();

        self.force = Vector3::new(0., 0., 0.);
        self.torque = Quaternion::new(0., 0., 0., 0.);
    }
    
    pub fn to_local(&self, v: Vector3<f64>) -> Vector3<f64> {
        self.orientation.invert().rotate_vector(v - self.pos)
    }
    
    pub fn to_global(&self, v: Vector3<f64>) -> Vector3<f64> {
        self.orientation.rotate_vector(v) + self.pos
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new(
            Vector3::zero(),
            Vector3::zero(),
            Quaternion::new(1., 0., 0., 0.),
            Vector3::zero(),
            1.,
            (1., 1., 1.)
        )
    }
}