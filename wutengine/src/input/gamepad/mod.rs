//! Module for the gamepad input handling logic.
//! In order to activate gamepad input, use the [GamepadInputPlugin] engine plugin.
//!
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use gilrs::{Event, Gilrs, GilrsBuilder};
use thiserror::Error;

use crate::plugins::WutEnginePlugin;

mod button;
mod gilrs_mapping;

pub use button::*;

/// The gamepad input reader plugin.
/// Reads the raw gamepad input from the OS gamepad API
/// and delivers it to the InputHandler components in the world.
#[derive(Debug)]
pub struct GamepadInputPlugin {
    gilrs: Mutex<Gilrs>,
    pub gamepads: HashMap<GamepadId, Gamepad>,
}

#[derive(Debug, Clone)]
pub struct Gamepad {
    pub(crate) id: GamepadId,
    pub(crate) connected: bool,
    pub(crate) buttons: [GamepadButtonValue; gamepad_button_count()],
}

impl Gamepad {
    fn new(id: GamepadId, connected: bool) -> Self {
        Self {
            id,
            connected,
            buttons: [GamepadButtonValue::NOT_PRESSED; gamepad_button_count()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GamepadId(gilrs::GamepadId);

fn is_button_event(event: &gilrs::EventType) -> bool {
    matches!(
        event,
        gilrs::EventType::ButtonPressed(_, _)
            | gilrs::EventType::ButtonRepeated(_, _)
            | gilrs::EventType::ButtonReleased(_, _)
            | gilrs::EventType::ButtonChanged(_, _, _)
    )
}

#[derive(Debug, Error)]
enum ButtonMapErr {
    #[error("Unknown button")]
    UnknownButton(#[from] FromGilrsButtonErr),
}

fn get_button_event_button_and_value(
    event: &gilrs::EventType,
) -> Result<(GamepadButton, GamepadButtonValue), ButtonMapErr> {
    debug_assert!(
        is_button_event(event),
        "Non-button event given: {:?}",
        event
    );

    match event {
        gilrs::EventType::ButtonPressed(button, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::PRESSED))
        }
        gilrs::EventType::ButtonRepeated(button, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::PRESSED))
        }
        gilrs::EventType::ButtonReleased(button, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::NOT_PRESSED))
        }
        gilrs::EventType::ButtonChanged(button, val, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::new_continuous(*val)))
        }
        _ => unreachable!(),
    }
}

impl GamepadInputPlugin {
    /// Creates a new default [GamepadInputPlugin]
    pub fn new() -> Self {
        let builder = GilrsBuilder::new();
        Self {
            gilrs: Mutex::new(builder.build().unwrap()),
            gamepads: HashMap::new(),
        }
    }
}

impl Default for GamepadInputPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WutEnginePlugin for GamepadInputPlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn pre_update(&mut self, _context: &mut crate::plugins::Context) {
        let mut gilrs = self.gilrs.lock().unwrap();

        // Examine new events
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            if matches!(event, gilrs::EventType::Dropped) {
                // According to Gilrs we must ignore dropped events
                continue;
            }

            log::trace!("New gamepad event from {}: {:?}", id, event);

            let gamepad = self
                .gamepads
                .entry(id.into())
                .or_insert_with(|| Gamepad::new(id.into(), true));

            if is_button_event(&event) {
                let (button, value) = match get_button_event_button_and_value(&event) {
                    Ok(bv) => bv,
                    Err(e) => {
                        log::debug!("Could not determine gamepad button value: {}", e);
                        continue;
                    }
                };

                gamepad.buttons[button as usize] = value;
            }
        }

        // Increment internal frame counter
        gilrs.inc();
    }
}
