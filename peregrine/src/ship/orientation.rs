
    // OPTIMIZE
    use core::f64;

    use cgmath::{Deg, InnerSpace, Quaternion, Rotation, Rotation3, Vector3};
    const RZ0: Quaternion<f64> = Quaternion::new(1., 0., 0., 0.);
    const RZ1: Quaternion<f64> = Quaternion::new(0., 0., 0., 1.);
    const RZ2: Quaternion<f64> = Quaternion::new(-1., 0., 0., 0.);
    const RZ3: Quaternion<f64> = Quaternion::new(0., 0., 0., -1.);

    pub fn from_quat(q: Quaternion<f64>) -> u8 {
        let possible_quats = [
            RZ0,
            RZ1,
            RZ2,
            RZ3,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ0,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ1,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ2,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ3,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ0,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ1,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ2,
            Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ3,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ0,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ1,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ2,
            Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ3,
        ];
        let mut best_index = 0;
        let mut best_mag2 = f64::INFINITY;
        for (i, r) in possible_quats.iter().enumerate() {
            let mag2 = (q - r).magnitude2();
            if mag2 < best_mag2 {
                best_index = i as u8;
                best_mag2 = mag2;
            }
        }
        best_index
    }

    pub fn to_quat(orientation: u8) -> Quaternion<f64> {
        match orientation {
            0 => RZ0,
            1 => RZ1,
            2 => RZ2,
            3 => RZ3,

            4 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ0,
            5 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ1,
            6 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ2,
            7 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(90.)) * RZ3,

            8 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ0,
            9 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ1,
            10 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ2,
            11 => Quaternion::from_axis_angle(Vector3::new(0., 1., 0.), Deg(90.)) * RZ3,

            12 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ0,
            13 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ1,
            14 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ2,
            15 => Quaternion::from_axis_angle(Vector3::new(1., 0., 0.), Deg(180.)) * RZ3,
            _ => panic!("Orientation not supported"),
        }
    }

    pub fn compose(a: u8, b: u8) -> u8 {
        from_quat(to_quat(a) * to_quat(b))
    }

    pub fn rotate_by_quat(a: u8, q: Quaternion<f64>) -> u8 {
        from_quat(to_quat(a) * q)
    }
    
    /// Rotates a set of integers around the origin
    pub(crate) fn rotate_integer(orientation: u8, x: i32, y: i32, z: i32) -> (i32, i32, i32) {
        let quat = to_quat(orientation);
        let vec = Vector3::new(x as f64, y as f64, z as f64);
        let point = quat.rotate_vector(vec);
        (
            point.x.round() as i32,
            point.y.round() as i32,
            point.z.round() as i32,
        )
    }