mod value;
use thiserror::Error;
pub use value::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialGamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
}

impl PartialGamepadAxis {
    pub(super) const fn get_full_axis_and_dir(self) -> (GamepadAxis, PartialGamepadAxisDir) {
        match self {
            PartialGamepadAxis::LeftStickX => (GamepadAxis::LeftStick, PartialGamepadAxisDir::X),
            PartialGamepadAxis::LeftStickY => (GamepadAxis::LeftStick, PartialGamepadAxisDir::Y),
            PartialGamepadAxis::RightStickX => (GamepadAxis::RightStick, PartialGamepadAxisDir::X),
            PartialGamepadAxis::RightStickY => (GamepadAxis::RightStick, PartialGamepadAxisDir::Y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialGamepadAxisDir {
    X,
    Y,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadAxis {
    LeftStick,
    // NOTE: Must be the last variant because it is used to determine the amount of axes
    RightStick,
}

pub(super) const fn gamepad_axis_count() -> usize {
    (GamepadAxis::RightStick as usize) + 1
}

#[derive(Debug, Error)]
pub enum FromGilrsAxisErr {
    #[error("Unknown partial axis: {}", 0)]
    UnknownAxis(gilrs::Axis),
}

impl TryFrom<gilrs::Axis> for PartialGamepadAxis {
    type Error = FromGilrsAxisErr;

    fn try_from(value: gilrs::Axis) -> Result<Self, Self::Error> {
        let mapped = match value {
            gilrs::Axis::LeftStickX => PartialGamepadAxis::LeftStickX,
            gilrs::Axis::LeftStickY => PartialGamepadAxis::LeftStickY,
            gilrs::Axis::RightStickX => PartialGamepadAxis::RightStickX,
            gilrs::Axis::RightStickY => PartialGamepadAxis::RightStickY,
            _ => return Err(FromGilrsAxisErr::UnknownAxis(value)),
        };

        Ok(mapped)
    }
}
