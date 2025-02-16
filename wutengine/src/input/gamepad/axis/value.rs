use glam::Vec2;

use super::PartialGamepadAxisDir;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct GamepadAxisValue(Vec2);

impl GamepadAxisValue {
    pub(crate) const NEUTRAL: Self = Self::new(Vec2::ZERO);

    pub(crate) const fn new(value: Vec2) -> Self {
        assert!(value.x >= -1.0 && value.x <= 1.0);
        assert!(value.y >= -1.0 && value.y <= 1.0);

        Self(value)
    }

    pub(crate) const fn set_axis(
        &mut self,
        value: PartialGamepadAxisValue,
        axis: PartialGamepadAxisDir,
    ) {
        match axis {
            PartialGamepadAxisDir::X => self.0.x = value.0,
            PartialGamepadAxisDir::Y => self.0.y = value.0,
        }
    }

    pub(crate) fn is_neutral(self) -> bool {
        self.0 == Vec2::ZERO
    }

    pub(crate) fn value(self) -> Vec2 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PartialGamepadAxisValue(f32);

impl PartialGamepadAxisValue {
    pub(crate) const fn new(value: f32) -> Self {
        assert!(value >= -1.0 && value <= 1.0);

        Self(value)
    }
}
