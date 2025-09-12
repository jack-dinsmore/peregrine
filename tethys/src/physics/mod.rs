use cgmath::{InnerSpace, Matrix4, Quaternion, Rotation, Vector3, Zero};
use serde::{Deserialize, Serialize};

pub mod collisions;


#[derive(Debug, Serialize, Deserialize)]
pub struct RigidBody {
    #[serde(with = "serde_vector3")]
    pub pos: Vector3<f64>,
    #[serde(with = "serde_quaternion")]
    pub orientation: Quaternion<f64>,
    #[serde(with = "serde_vector3")]
    pub vel: Vector3<f64>,
    #[serde(with = "serde_quaternion")]
    pub angvel: Quaternion<f64>,
    #[serde(with = "serde_vector3")]
    pub force: Vector3<f64>,
    #[serde(with = "serde_quaternion")]
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
    
    pub fn get_mvp(&self, camera: &crate::prelude::Camera) -> Matrix4<f32> {
        let rot = Matrix4::from(Quaternion::new(
            self.orientation.s as f32,
            self.orientation.v.x as f32,
            self.orientation.v.y as f32,
            self.orientation.v.z as f32,
        ));
        let world = Matrix4::from_translation((self.pos - camera.position).cast::<f32>().unwrap()) * rot;
        return world * camera.get_view() * camera.get_proj();
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

mod serde_vector3 {
    use cgmath::Vector3;
    use serde::{Serialize, Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(v: &Vector3<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [v.x, v.y, v.z].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vector3<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let arr = <[f64; 3]>::deserialize(deserializer)?;
        Ok(Vector3::new(arr[0], arr[1], arr[2]))
    }
}

mod serde_quaternion {
    use cgmath::Quaternion;
    use serde::{Serialize, Deserialize, Serializer, Deserializer};
    pub fn serialize<S>(q: &Quaternion<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [q.s, q.v.x, q.v.y, q.v.z].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Quaternion<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let arr = <[f64; 4]>::deserialize(deserializer)?;
        Ok(Quaternion::new(arr[0], arr[1], arr[2], arr[3]))
    }
}