//! Module for the gamepad input handling logic.
//! In order to activate gamepad input, use the [GamepadInputPlugin] engine plugin.
//!
use std::sync::Mutex;

use gilrs::{Event, Gilrs, GilrsBuilder};

use crate::plugins::WutEnginePlugin;

/// The gamepad input reader plugin.
/// Reads the raw gamepad input from the OS gamepad API
/// and delivers it to the InputHandler components in the world.
#[derive(Debug)]
pub struct GamepadInputPlugin {
    gilrs: Mutex<Gilrs>,
}

impl GamepadInputPlugin {
    /// Creates a new default [GamepadInputPlugin]
    pub fn new() -> Self {
        let builder = GilrsBuilder::new();
        Self {
            gilrs: Mutex::new(builder.build().unwrap()),
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
            log::info!("New gamepad event from {}: {:?}", id, event);
        }
    }
}
