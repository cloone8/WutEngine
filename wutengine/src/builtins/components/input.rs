use winit::keyboard::KeyCode;
use wutengine_core::Component;

use crate::input::keyboard::winit_keycode_to_usize;
use crate::input::keyboard::MAX_KEYCODE;

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

impl Component for InputHandler {}
