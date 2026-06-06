//! Gamepad handling and APIs

use std::collections::HashMap;

use glam::Vec2;

use crate::input::INPUT_MANAGER;

use super::DeviceSet;

/// A gamepad input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From, derive_more::Display)]
#[repr(transparent)]
pub struct GamepadId(gilrs::GamepadId);

/// The data for a gamepad
#[derive(Debug)]
pub(crate) struct Gamepad {
    /// Current button values, between 0.0 and 1.0
    pub(crate) button_values: HashMap<Button, f32>,

    /// Previous button values, between 0.0 and 1.0
    pub(crate) prev_button_values: HashMap<Button, f32>,

    /// Current axis values, between 0.0 and 1.0
    pub(crate) axis_values: HashMap<Axis, Vec2>,

    /// Previous axis values, between 0.0 and 1.0
    pub(crate) prev_axis_values: HashMap<Axis, Vec2>,
}

impl Default for Gamepad {
    fn default() -> Self {
        Self::new()
    }
}

impl Gamepad {
    fn new() -> Self {
        Self {
            button_values: Default::default(),
            prev_button_values: Default::default(),
            axis_values: Default::default(),
            prev_axis_values: Default::default(),
        }
    }

    /// Clears the frame-specific data for this gamepad, ensuring all new
    /// input gets registered to the next frame
    pub(crate) fn next_frame(&mut self) {
        self.prev_button_values.clone_from(&self.button_values);
        self.prev_axis_values.clone_from(&self.axis_values);
    }
}

/// Poll for new gamepad events
pub(crate) fn poll_for_events() {
    profiling::function_scope!();

    let Some(gamepad_manager) = INPUT_MANAGER.gamepad_manager.as_ref() else {
        // We failed to initialize the gamepad manager earlier for some reason,
        // so no gamepad support
        return;
    };

    let mut gamepads = INPUT_MANAGER.gamepads.write().unwrap();
    let mut gamepad_manager = gamepad_manager.lock().unwrap();

    while let Some(event) = gamepad_manager.next_event() {
        if event.is_dropped() {
            continue;
        }

        let gamepad_id = GamepadId(event.id);
        let gilrs_gamepad = gamepad_manager.gamepad(event.id);

        log::trace!("Event for gamepad {}: {:#?}", event.id, event.event);

        match event.event {
            gilrs::EventType::ButtonPressed(button, code) => {
                set_axis_or_button_value(
                    &mut gamepads,
                    &gamepad_id,
                    AxisOrButton::Button(Button::from_gilrs(button, code)),
                    1.0,
                );
            }
            gilrs::EventType::ButtonReleased(button, code) => {
                set_axis_or_button_value(
                    &mut gamepads,
                    &gamepad_id,
                    AxisOrButton::Button(Button::from_gilrs(button, code)),
                    0.0,
                );
            }
            gilrs::EventType::ButtonChanged(button, value, code) => {
                set_axis_or_button_value(
                    &mut gamepads,
                    &gamepad_id,
                    AxisOrButton::Button(Button::from_gilrs(button, code)),
                    value,
                );
            }
            gilrs::EventType::AxisChanged(axis, value, code) => {
                let Some((axis, subaxis)) = Axis::from_gilrs(axis) else {
                    log::warn!(
                        "Unrecognized controller axis: {axis:#?} (native code: {})",
                        code.into_u32()
                    );
                    continue;
                };

                set_axis_or_button_value(
                    &mut gamepads,
                    &gamepad_id,
                    AxisOrButton::Axis(axis, subaxis),
                    value,
                );
            }
            gilrs::EventType::Connected => {
                let name = gilrs_gamepad.name();

                log::info!("Gamepad \"{name}\" with ID {} connected", event.id);

                INPUT_MANAGER.set_most_recent_gamepad(GamepadId(event.id));

                gamepads.update_device(Some(&GamepadId(event.id)), |_| {});
            }
            gilrs::EventType::Disconnected => {
                let name = gilrs_gamepad.name();

                log::info!("Gamepad \"{name}\" with ID {} disconnected", event.id);

                gamepads.remove_device(&GamepadId(event.id));
            }
            _ => {}
        }
    }

    gamepad_manager.inc();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SubAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, derive_more::From)]
enum AxisOrButton {
    Axis(Axis, SubAxis),
    Button(Button),
}

fn set_axis_or_button_value(
    gamepads: &mut DeviceSet<super::GamepadId, Gamepad>,
    gamepad: &super::GamepadId,
    axis_or_button: AxisOrButton,
    value: f32,
) {
    INPUT_MANAGER.set_most_recent_gamepad(*gamepad);

    let Some(gamepad) = gamepads.get_identified_device_mut(gamepad) else {
        log::warn!("Unknown gamepad: {gamepad}");
        return;
    };

    match axis_or_button {
        AxisOrButton::Axis(axis, subaxis) => {
            let cur_value = gamepad.axis_values.entry(axis).or_default();

            match subaxis {
                SubAxis::X => cur_value.x = value,
                SubAxis::Y => cur_value.y = value,
            }
        }
        AxisOrButton::Button(button) => *gamepad.button_values.entry(button).or_default() = value,
    }
}

/// Gamepad buttons.
///
/// Based on [gilrs 0.11.2](https://docs.rs/gilrs/0.11.2/gilrs/)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Button {
    /// Right button pad, south button
    South,

    /// Right button pad, east button
    East,

    /// Right button pad, north button
    North,

    /// Right button pad, west button
    West,

    /// C button. Uncommon
    C,

    /// Z button. Uncommon
    Z,

    /// Left shoulder button, above the trigger
    LeftShoulder,

    /// Left trigger, below the shoulder button
    LeftTrigger,

    /// Right shoulder button, above the trigger
    RightShoulder,

    /// Right trigger button, below the shoulder button
    RightTrigger,

    /// Select button. Usually the left smaller middle button
    Select,

    /// Start button. Usually the right smaller middle button
    Start,

    /// "Mode" button. Usually the main button between start/select. Often
    /// has the shape of the controller vendor
    Mode,

    /// Left thumb stick pressed
    LeftStick,

    /// Right thumb stick pressed
    RightStick,

    /// D-Pad up
    DPadUp,

    /// D-Pad down
    DPadDown,

    /// D-Pad left
    DPadLeft,

    /// D-Pad right
    DPadRight,

    /// Unknown button. Contains the raw OS code
    Raw(u32),
}

impl Button {
    #[inline]
    fn from_gilrs(gilrs_btn: gilrs::Button, gilrs_ev: gilrs::ev::Code) -> Self {
        match gilrs_btn {
            gilrs::Button::South => Self::South,
            gilrs::Button::East => Self::East,
            gilrs::Button::North => Self::North,
            gilrs::Button::West => Self::West,
            gilrs::Button::C => Self::C,
            gilrs::Button::Z => Self::Z,
            gilrs::Button::LeftTrigger => Self::LeftShoulder,
            gilrs::Button::LeftTrigger2 => Self::LeftTrigger,
            gilrs::Button::RightTrigger => Self::RightShoulder,
            gilrs::Button::RightTrigger2 => Self::RightTrigger,
            gilrs::Button::Select => Self::Select,
            gilrs::Button::Start => Self::Start,
            gilrs::Button::Mode => Self::Mode,
            gilrs::Button::LeftThumb => Self::LeftStick,
            gilrs::Button::RightThumb => Self::RightStick,
            gilrs::Button::DPadUp => Self::DPadUp,
            gilrs::Button::DPadDown => Self::DPadDown,
            gilrs::Button::DPadLeft => Self::DPadLeft,
            gilrs::Button::DPadRight => Self::DPadRight,
            gilrs::Button::Unknown => Self::Raw(gilrs_ev.into_u32()),
        }
    }
}

/// Gamepad axes
///
/// Based on [gilrs 0.11.2](https://docs.rs/gilrs/0.11.2/gilrs/)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Axis {
    /// Left stick
    LeftStick,

    /// Right stick
    RightStick,
}

impl Axis {
    #[inline]
    const fn from_gilrs(gilrs_axis: gilrs::Axis) -> Option<(Self, SubAxis)> {
        match gilrs_axis {
            gilrs::Axis::LeftStickX => Some((Self::LeftStick, SubAxis::X)),
            gilrs::Axis::LeftStickY => Some((Self::LeftStick, SubAxis::Y)),
            gilrs::Axis::RightStickX => Some((Self::RightStick, SubAxis::X)),
            gilrs::Axis::RightStickY => Some((Self::RightStick, SubAxis::Y)),
            _ => None,
        }
    }
}

fn get_gamepad_and<T>(
    to_query: Option<super::GamepadId>,
    func: impl FnOnce(Option<&Gamepad>) -> T,
) -> T {
    let gamepads = INPUT_MANAGER.gamepads.read().unwrap();

    let gamepad = match to_query {
        Some(to_query) => {
            let gamepad = gamepads.get_identified_device(&to_query);

            if gamepad.is_none() {
                log::warn!("Gamepad {to_query:?} could not be found, returning default values");
            }

            gamepad
        }
        None => {
            let most_recent_gamepad = *INPUT_MANAGER.most_recent_gamepad.read().unwrap();

            if let Some(latest) = most_recent_gamepad {
                match gamepads.get_identified_device(&latest) {
                    Some(mouse) => Some(mouse),
                    None => Some(gamepads.get_any_device()),
                }
            } else {
                Some(gamepads.get_any_device())
            }
        }
    };

    func(gamepad)
}

/// Returns whether the specified gamepad button was pressed this frame. If the button
/// was already pressed last frame, this returns `false`. To check whether the button is held,
/// even if it was already held before, see [button_held]
///
/// If `device` is [None], returns the value
/// for the latest changed gamepad device.
///
/// If the specified gamepad (or the latest gamepad) could not be found, returns `false`
pub fn button_pressed(device: Option<super::GamepadId>, button: Button) -> bool {
    get_gamepad_and(device, |gamepad| {
        if let Some(gamepad) = gamepad {
            let cur_value = gamepad.button_values.get(&button).copied().unwrap_or(0.0);

            let prev_value = gamepad
                .prev_button_values
                .get(&button)
                .copied()
                .unwrap_or(0.0);

            cur_value > 0.0 && prev_value == 0.0
        } else {
            false
        }
    })
}

/// Returns whether the specified gamepad button was being held this frame. This returns
/// `true` in every frame the button is held. To only get `true` for new presses, see
/// [button_pressed]
///
/// If `device` is [None], returns the value
/// for the latest changed gamepad device.
///
/// If the specified gamepad (or the latest gamepad) could not be found, returns `false`
pub fn button_held(device: Option<super::GamepadId>, button: Button) -> bool {
    get_gamepad_and(device, |gamepad| {
        if let Some(gamepad) = gamepad {
            let cur_value = gamepad.button_values.get(&button).copied().unwrap_or(0.0);

            cur_value > 0.0
        } else {
            false
        }
    })
}

/// Returns whether the specified gamepad button was released this frame. If the button
/// was not held down last frame, this always returns `false`. To check whether the button is held,
/// even if it was not held before, see [button_held]
///
/// If `device` is [None], returns the value
/// for the latest changed gamepad device.
///
/// If the specified gamepad (or the latest gamepad) could not be found, returns `false`
pub fn button_released(device: Option<super::GamepadId>, button: Button) -> bool {
    get_gamepad_and(device, |gamepad| {
        if let Some(gamepad) = gamepad {
            let cur_value = gamepad.button_values.get(&button).copied().unwrap_or(0.0);

            let prev_value = gamepad
                .prev_button_values
                .get(&button)
                .copied()
                .unwrap_or(0.0);

            cur_value == 0.0 && prev_value > 0.0
        } else {
            false
        }
    })
}

/// Returns the raw value for a button on the specified gamepad.
///
/// If `device` is [None], returns the value
/// for the latest changed gamepad device.
///
/// If the specified gamepad (or the latest gamepad) could not be found, returns `false`
pub fn button_value(device: Option<super::GamepadId>, button: Button) -> f32 {
    get_gamepad_and(device, |gamepad| {
        if let Some(gamepad) = gamepad {
            gamepad.button_values.get(&button).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    })
}

/// Returns the raw delta for a button on the specified gamepad.
///
/// If `device` is [None], returns the value
/// for the latest changed gamepad device.
///
/// If the specified gamepad (or the latest gamepad) could not be found, returns `false`
pub fn button_delta(device: Option<super::GamepadId>, button: Button) -> f32 {
    get_gamepad_and(device, |gamepad| {
        if let Some(gamepad) = gamepad {
            let cur_value = gamepad.button_values.get(&button).copied().unwrap_or(0.0);

            let prev_value = gamepad
                .prev_button_values
                .get(&button)
                .copied()
                .unwrap_or(0.0);

            cur_value - prev_value
        } else {
            0.0
        }
    })
}

/// Returns the raw value for an axis on the specified gamepad.
///
/// If `device` is [None], returns the value
/// for the latest changed gamepad device.
///
/// If the specified gamepad (or the latest gamepad) could not be found, returns `false`
pub fn axis_value(device: Option<super::GamepadId>, axis: Axis) -> Vec2 {
    get_gamepad_and(device, |gamepad| {
        if let Some(gamepad) = gamepad {
            gamepad
                .axis_values
                .get(&axis)
                .copied()
                .unwrap_or(Vec2::ZERO)
        } else {
            Vec2::ZERO
        }
    })
}

/// Returns the raw delta for an axis on the specified gamepad.
///
/// If `device` is [None], returns the value
/// for the latest changed gamepad device.
///
/// If the specified gamepad (or the latest gamepad) could not be found, returns `false`
pub fn axis_delta(device: Option<super::GamepadId>, axis: Axis) -> Vec2 {
    get_gamepad_and(device, |gamepad| {
        if let Some(gamepad) = gamepad {
            let cur_value = gamepad
                .axis_values
                .get(&axis)
                .copied()
                .unwrap_or(Vec2::ZERO);

            let prev_value = gamepad
                .prev_axis_values
                .get(&axis)
                .copied()
                .unwrap_or(Vec2::ZERO);

            cur_value - prev_value
        } else {
            Vec2::ZERO
        }
    })
}
