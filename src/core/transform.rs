pub struct Transform {
    pub position: glam::Vec3A,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3A,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: glam::Vec3A::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3A::ONE,
        }
    }
}
