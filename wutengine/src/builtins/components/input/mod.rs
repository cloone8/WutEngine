use std::collections::HashMap;

use crate::component::{Component, Context};
use crate::input::gamepad::{Gamepad, GamepadId, GamepadInputPlugin};
use crate::input::keyboard::KeyboardInputPlugin;
use crate::input::keyboard::MAX_KEYCODE;

mod gamepad;
mod keyboard;

pub use gamepad::*;
pub use keyboard::*;

/// The main input handler component. The various input-reading engine plugins will
/// inject their read inputs into each of these components before each Update iteration.
#[derive(Debug)]
pub struct InputHandler {
    prev: InputState,
    cur: InputState,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct InputState {
    /// Map of KeyCode indices to pressed state. To get the index
    /// for a [KeyCode], see [winit_keycode_to_usize]
    keyboard_pressed_keys: [bool; MAX_KEYCODE],

    gamepads: HashMap<GamepadId, Gamepad>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            keyboard_pressed_keys: [false; MAX_KEYCODE],
            gamepads: HashMap::default(),
        }
    }
}

impl InputHandler {
    /// Constructs a new [InputHandler] component
    pub fn new() -> Self {
        Self {
            cur: InputState::default(),
            prev: InputState::default(),
        }
    }

    pub fn keyboard(&self) -> InputHandlerKeyboard {
        InputHandlerKeyboard { handler: self }
    }

    pub fn gamepad(&self) -> InputHandlerGamepad {
        InputHandlerGamepad { handler: self }
    }
}

impl Component for InputHandler {
    fn pre_update(&mut self, context: &mut Context) {
        self.prev = std::mem::take(&mut self.cur);

        if let Some(keyboard_plugin) = context.plugin.get::<KeyboardInputPlugin>() {
            self.cur
                .keyboard_pressed_keys
                .copy_from_slice(&keyboard_plugin.pressed_keys);
        }

        if let Some(gamepad_plugin) = context.plugin.get::<GamepadInputPlugin>() {
            self.cur.gamepads = gamepad_plugin.gamepads.clone();
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
