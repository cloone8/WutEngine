use winit::keyboard::KeyCode;

use crate::component::{Component, Context};
use crate::input::keyboard::MAX_KEYCODE;
use crate::input::keyboard::{winit_keycode_to_usize, KeyboardInputPlugin};

/// The main input handler component. The various input-reading engine plugins will
/// inject their read inputs into each of these components before each Update iteration.
#[derive(Debug)]
pub struct InputHandler {
    /// Same as [Self::keyboard_pressed_keys], but from the previous frame.
    /// Used to check pressed/released instead of just held
    pub(crate) keyboard_pressed_keys_prev: [bool; MAX_KEYCODE],

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
            keyboard_pressed_keys_prev: [false; MAX_KEYCODE],
            keyboard_pressed_keys: [false; MAX_KEYCODE],
        }
    }

    /// Returns whether the key with the given keycode is currently held down
    pub fn is_down(&self, key: KeyCode) -> bool {
        self.keyboard_pressed_keys[winit_keycode_to_usize(key)]
    }

    /// Returns whether the key with the given keycode is currenty not held down
    pub fn is_up(&self, key: KeyCode) -> bool {
        !self.is_down(key)
    }

    /// Returns whether the key with the given keycode was
    /// pressed down this frame.
    pub fn pressed_this_frame(&self, key: KeyCode) -> bool {
        let keycode_idx = winit_keycode_to_usize(key);

        self.keyboard_pressed_keys[keycode_idx] && !self.keyboard_pressed_keys_prev[keycode_idx]
    }

    /// Returns whether the key with the given keycode was
    /// released this frame.
    pub fn released_this_frame(&self, key: KeyCode) -> bool {
        let keycode_idx = winit_keycode_to_usize(key);

        !self.keyboard_pressed_keys[keycode_idx] && self.keyboard_pressed_keys_prev[keycode_idx]
    }
}

impl Component for InputHandler {
    fn pre_update(&mut self, context: &mut Context) {
        if let Some(input_plugin) = context.plugin.get::<KeyboardInputPlugin>() {
            self.keyboard_pressed_keys_prev
                .copy_from_slice(&self.keyboard_pressed_keys);

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
