mod value;

use thiserror::Error;
pub use value::GamepadButtonValue;
pub(super) use value::*;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    // Triggers
    LeftShoulder,
    LeftTrigger,
    RightShoulder,
    RightTrigger,
    // Menu Pad
    Select,
    Start,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,

    // NOTE: Defined as the last value so that we can also use it to determine the "max" value of the
    Home,
}

pub(super) const fn gamepad_button_count() -> usize {
    (GamepadButton::Home as usize) + 1
}

#[derive(Debug, Error)]
pub enum FromGilrsButtonErr {
    #[error("Unknown button: {}", 0)]
    UnknownButton(gilrs::Button),
}

impl TryFrom<gilrs::Button> for GamepadButton {
    type Error = FromGilrsButtonErr;

    fn try_from(value: gilrs::Button) -> Result<Self, Self::Error> {
        let mapped = match value {
            gilrs::Button::South => Self::South,
            gilrs::Button::East => Self::East,
            gilrs::Button::North => Self::North,
            gilrs::Button::West => Self::West,
            gilrs::Button::LeftTrigger => Self::LeftShoulder,
            gilrs::Button::LeftTrigger2 => Self::LeftTrigger,
            gilrs::Button::RightTrigger => Self::RightShoulder,
            gilrs::Button::RightTrigger2 => Self::RightTrigger,
            gilrs::Button::Select => Self::Select,
            gilrs::Button::Start => Self::Start,
            gilrs::Button::Mode => Self::Home,
            gilrs::Button::LeftThumb => Self::LeftStick,
            gilrs::Button::RightThumb => Self::RightStick,
            gilrs::Button::DPadUp => Self::DPadUp,
            gilrs::Button::DPadDown => Self::DPadDown,
            gilrs::Button::DPadLeft => Self::DPadLeft,
            gilrs::Button::DPadRight => Self::DPadRight,
            _ => return Err(FromGilrsButtonErr::UnknownButton(value)),
        };

        Ok(mapped)
    }
}
