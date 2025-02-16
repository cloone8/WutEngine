use std::collections::HashMap;
use std::sync::Mutex;

use gilrs::{Event, Gilrs, GilrsBuilder};

use crate::input::gamepad::gilrs_mapping::{
    get_axis_event_axis_and_value, get_button_event_button_and_value,
};
use crate::input::gamepad::{is_axis_event, is_button_event};
use crate::plugins::WutEnginePlugin;

use super::{Gamepad, GamepadId};

/// The gamepad input reader plugin.
/// Reads the raw gamepad input from the OS gamepad API
/// and delivers it to the InputHandler components in the world.
#[derive(Debug)]
pub struct GamepadInputPlugin {
    gilrs: Mutex<Gilrs>,

    /// Map of all currently known gamepads
    pub gamepads: HashMap<GamepadId, Gamepad>,
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

            if is_axis_event(&event) {
                let (partial_axis, value) = match get_axis_event_axis_and_value(&event) {
                    Ok(av) => av,
                    Err(e) => {
                        log::debug!("Could not determine gamepad axis value: {}", e);
                        continue;
                    }
                };

                let (axis, dir) = partial_axis.get_full_axis_and_dir();
                gamepad.axes[axis as usize].set_axis(value, dir);
            }
        }

        // Increment internal frame counter
        gilrs.inc();
    }
}
