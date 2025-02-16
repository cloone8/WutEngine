mod value;
use thiserror::Error;
pub use value::*;

/// A partial [GamepadAxis]. Represents either the X or Y axis of a full gamepad axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialGamepadAxis {
    /// A partial left-stick axis
    LeftStick(PartialGamepadAxisDir),

    /// A partial right-stick axis
    RightStick(PartialGamepadAxisDir),
}

impl PartialGamepadAxis {
    pub(super) const fn get_full_axis_and_dir(self) -> (GamepadAxis, PartialGamepadAxisDir) {
        match self {
            PartialGamepadAxis::LeftStick(PartialGamepadAxisDir::X) => {
                (GamepadAxis::LeftStick, PartialGamepadAxisDir::X)
            }
            PartialGamepadAxis::LeftStick(PartialGamepadAxisDir::Y) => {
                (GamepadAxis::LeftStick, PartialGamepadAxisDir::Y)
            }
            PartialGamepadAxis::RightStick(PartialGamepadAxisDir::X) => {
                (GamepadAxis::RightStick, PartialGamepadAxisDir::X)
            }
            PartialGamepadAxis::RightStick(PartialGamepadAxisDir::Y) => {
                (GamepadAxis::RightStick, PartialGamepadAxisDir::Y)
            }
        }
    }
}

/// The direction of a partial gamepad axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialGamepadAxisDir {
    /// left/right
    X,

    /// up/down
    Y,
}

/// A continuous gamepad axis
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadAxis {
    /// The left stick
    LeftStick,

    // NOTE: Must be the last variant because it is used to determine the amount of axes
    /// The right stick
    RightStick,
}

pub(super) const fn gamepad_axis_count() -> usize {
    (GamepadAxis::RightStick as usize) + 1
}

#[doc(hidden)] // Implementation detail
#[derive(Debug, Error)]
pub enum FromGilrsAxisErr {
    #[error("Unknown partial axis: {}", 0)]
    UnknownAxis(gilrs::Axis),
}

impl TryFrom<gilrs::Axis> for PartialGamepadAxis {
    type Error = FromGilrsAxisErr;

    fn try_from(value: gilrs::Axis) -> Result<Self, Self::Error> {
        let mapped = match value {
            gilrs::Axis::LeftStickX => PartialGamepadAxis::LeftStick(PartialGamepadAxisDir::X),
            gilrs::Axis::LeftStickY => PartialGamepadAxis::LeftStick(PartialGamepadAxisDir::Y),
            gilrs::Axis::RightStickX => PartialGamepadAxis::RightStick(PartialGamepadAxisDir::X),
            gilrs::Axis::RightStickY => PartialGamepadAxis::RightStick(PartialGamepadAxisDir::Y),
            _ => return Err(FromGilrsAxisErr::UnknownAxis(value)),
        };

        Ok(mapped)
    }
}
