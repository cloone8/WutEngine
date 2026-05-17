use glam::Mat4;
use glam::Quat;
use glam::Vec3;

use crate::component::Component;

#[derive(Debug)]
pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    local_to_world: Mat4,
}

/// Public API
impl Transform {
    #[inline]
    pub fn new() -> Self {
        Self::new_at(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE)
    }

    pub fn new_at(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let mut new = Self {
            position,
            rotation,
            scale,
            local_to_world: Mat4::NAN,
        };

        new.recalculate_local_to_world();

        new
    }

    #[inline(always)]
    pub const fn local_to_world(&self) -> Mat4 {
        self.local_to_world
    }
}

/// Private API
impl Transform {
    fn recalculate_local_to_world(&mut self) {
        self.local_to_world =
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Transform {}
