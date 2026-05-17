use glam::Mat4;
use glam::Quat;
use glam::Vec3;

use crate::component::Component;

#[derive(Debug)]
pub struct Transform {
    translation: Vec3,
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
            translation: position,
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

    #[inline(always)]
    pub const fn local_position(&self) -> Vec3 {
        self.translation
    }

    #[inline(always)]
    pub const fn local_rotation(&self) -> Quat {
        self.rotation
    }

    #[inline(always)]
    pub const fn local_scale(&self) -> Vec3 {
        self.scale
    }

    #[inline(always)]
    pub fn set_local_position(&mut self, local_position: Vec3) {
        self.translation = local_position;

        self.recalculate_local_to_world();
    }

    #[inline(always)]
    pub fn set_local_rotation(&mut self, local_rotation: Quat) {
        self.rotation = local_rotation;

        self.recalculate_local_to_world();
    }

    #[inline(always)]
    pub fn set_local_scale(&mut self, local_scale: Vec3) {
        self.scale = local_scale;

        self.recalculate_local_to_world();
    }
}

/// Private API
impl Transform {
    fn recalculate_local_to_world(&mut self) {
        self.local_to_world =
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Transform {}
