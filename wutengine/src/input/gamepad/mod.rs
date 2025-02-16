//! Module for the gamepad input handling logic.
//! In order to activate gamepad input, use the [GamepadInputPlugin] engine plugin.
//!
use gilrs_mapping::{is_axis_event, is_button_event};

mod axis;
mod button;
mod gilrs_mapping;
mod plugin;

pub use axis::*;
pub use button::*;
pub use plugin::GamepadInputPlugin;

/// All data about a single gamepad
#[derive(Debug, Clone)]
pub struct Gamepad {
    /// The ID of the gamepad
    pub(crate) _id: GamepadId,

    /// Whether the gamepad is currently connected
    pub(crate) connected: bool,

    /// The last known state of the gamepad buttons.
    /// The value for a button `let x: GamepadButton = ...` can be found
    /// with `buttons[x as usize]`
    pub(crate) buttons: [GamepadButtonValue; gamepad_button_count()],

    /// The last known state of the gamepad axes.
    /// The value for an axis `let x: GamepadAxis = ...` can be found
    /// with `axes[x as usize]`
    pub(crate) axes: [GamepadAxisValue; gamepad_axis_count()],
}

impl Gamepad {
    fn new(id: GamepadId, connected: bool) -> Self {
        Self {
            _id: id,
            connected,
            buttons: [GamepadButtonValue::NOT_PRESSED; gamepad_button_count()],
            axes: [GamepadAxisValue::NEUTRAL; gamepad_axis_count()],
        }
    }
}

/// The unique identifier of a gamepad
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GamepadId(gilrs::GamepadId);
