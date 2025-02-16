use glam::Vec2;

use super::PartialGamepadAxisDir;

/// The value of a gamepad axis. Each axis is a value
/// between `-1.0` and `1.0` (both inclusive)
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct GamepadAxisValue(Vec2);

impl GamepadAxisValue {
    /// The neutral state
    pub(crate) const NEUTRAL: Self = Self::new(Vec2::ZERO);

    /// Creates a new axis value from a raw value between `-1.0` and `1.0`
    pub(crate) const fn new(value: Vec2) -> Self {
        assert!(value.x >= -1.0 && value.x <= 1.0);
        assert!(value.y >= -1.0 && value.y <= 1.0);

        Self(value)
    }

    /// Changes only a single axis to a new value
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

    /// Whether this axis is in the neutral state
    pub(crate) fn is_neutral(self) -> bool {
        self.0 == Vec2::ZERO
    }

    /// The raw axis value
    pub(crate) fn value(self) -> Vec2 {
        self.0
    }
}

/// The value of a partial gamepad axis
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PartialGamepadAxisValue(f32);

impl PartialGamepadAxisValue {
    /// Creates a new partial gamepad axis with the given raw value between `-1.0` and `1.0`
    pub(crate) const fn new(value: f32) -> Self {
        assert!(value >= -1.0 && value <= 1.0);

        Self(value)
    }
}
