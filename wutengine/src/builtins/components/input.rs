use winit::keyboard::KeyCode;

use crate::component::{Component, Context};
use crate::input::keyboard::MAX_KEYCODE;
use crate::input::keyboard::{winit_keycode_to_usize, KeyboardInputPlugin};

/// The main input handler component. The various input-reading engine plugins will
/// inject their read inputs into each of these components before each Update iteration.
pub struct InputHandler {
    /// Map of KeyCode indices to pressed state. To get the index
    /// for a [KeyCode], see [winit_keycode_to_usize]
    pub(crate) keyboard_pressed_keys: [bool; MAX_KEYCODE],
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    /// Constructs a new [InputHandler] component
    pub fn new() -> Self {
        Self {
            keyboard_pressed_keys: [false; MAX_KEYCODE],
        }
    }

    /// Returns whether the key with the given keycode is currently pressed
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.keyboard_pressed_keys[winit_keycode_to_usize(key)]
    }
}

impl Component for InputHandler {
    fn pre_update(&mut self, context: &mut Context) {
        if let Some(input_plugin) = context.plugin.get::<KeyboardInputPlugin>() {
            self.keyboard_pressed_keys
                .copy_from_slice(&input_plugin.pressed_keys);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
