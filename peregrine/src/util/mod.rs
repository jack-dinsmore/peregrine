use cgmath::Vector3;
use num_traits::AsPrimitive;

pub fn vector_cast<T: AsPrimitive<U>, U: Copy + 'static>(v: Vector3<T>) ->  Vector3<U> {
    Vector3::new(v.x.as_(), v.y.as_(), v.z.as_())
}