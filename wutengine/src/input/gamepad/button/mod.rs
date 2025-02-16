mod value;

use thiserror::Error;
pub use value::GamepadButtonValue;

/// All common gamepad buttons
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    /// The south button. Represents `A` for a common XInput gamepad
    South,
    /// The east button. Represents `B` for a common XInput gamepad
    East,
    /// The north button. Represents `Y` for a common XInput gamepad
    North,
    /// The west button. Represents `X` for a common XInput gamepad
    West,

    /// The left shoulder button (commonly above the left trigger)
    LeftShoulder,

    /// The left trigger button (commonly below the left shoulder button)
    LeftTrigger,

    /// The right shoulder button (commonly above the right trigger)
    RightShoulder,

    /// The right trigger button (commonly below the right shoulder button)
    RightTrigger,

    /// The select button (commonly the small left center button)
    Select,

    /// The start button (commonly the small right center button)
    Start,

    /// The button fired when the left stick is pressed down
    LeftStick,

    /// The button fired when the right stick is pressed down
    RightStick,

    /// D-Pad up
    DPadUp,

    /// D-Pad down
    DPadDown,

    /// D-Pad left
    DPadLeft,

    /// D-Pad right
    DPadRight,

    // NOTE: Defined as the last value so that we can also use it to determine the "max" value of the
    /// The "home" button. Typically the manufacturers logo, or a house logo.
    Home,
}

pub(super) const fn gamepad_button_count() -> usize {
    (GamepadButton::Home as usize) + 1
}

#[doc(hidden)] // Implementation detail
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
