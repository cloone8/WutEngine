use wutengine_math::{Mat4, Quat, Vec3};

use crate::component::Component;

/// A 3D transform component, containing both the local transform and the transform hierarchy
#[derive(Debug)]
pub struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
    local_to_world: Mat4,
}

/// Public API
impl Transform {
    /// Create a new identity transform
    #[inline]
    pub fn new() -> Self {
        Self::new_at_local(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE)
    }

    /// Create a new transform with the given local parameters
    pub fn new_at_local(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let mut new = Self {
            translation: position,
            rotation,
            scale,
            local_to_world: Mat4::NAN,
        };

        new.recalculate_local_to_world();

        new
    }

    /// Returns the current local-to-world matrix
    #[inline(always)]
    pub const fn local_to_world(&self) -> Mat4 {
        self.local_to_world
    }

    /// Returns the current local position
    #[inline(always)]
    pub const fn local_position(&self) -> Vec3 {
        self.translation
    }

    /// Returns the current local rotation
    #[inline(always)]
    pub const fn local_rotation(&self) -> Quat {
        self.rotation
    }

    /// Returns the current local scale
    #[inline(always)]
    pub const fn local_scale(&self) -> Vec3 {
        self.scale
    }

    /// Returns the current world position
    #[inline(always)]
    pub const fn world_position(&self) -> Vec3 {
        self.translation
    }

    /// Returns the current world rotation
    #[inline(always)]
    pub const fn world_rotation(&self) -> Quat {
        self.rotation
    }

    /// Set the local position
    #[inline(always)]
    pub fn set_local_position(&mut self, local_position: Vec3) {
        self.translation = local_position;

        self.recalculate_local_to_world();
    }

    /// Set the local rotation
    #[inline(always)]
    pub fn set_local_rotation(&mut self, local_rotation: Quat) {
        self.rotation = local_rotation;

        self.recalculate_local_to_world();
    }

    /// Set the local scale
    #[inline(always)]
    pub fn set_local_scale(&mut self, local_scale: Vec3) {
        self.scale = local_scale;

        self.recalculate_local_to_world();
    }

    /// Set the world position
    #[inline(always)]
    pub fn set_world_position(&mut self, world_position: Vec3) {
        self.translation = world_position;

        self.recalculate_local_to_world();
    }

    /// Set the world rotation
    #[inline(always)]
    pub fn set_world_rotation(&mut self, world_position: Quat) {
        self.rotation = world_position;

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

impl Component for Transform {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("5462eaa9-fed4-4603-84f4-1edf0bcdaeee")).unwrap();
}
