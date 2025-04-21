use winit::keyboard::KeyCode;

use crate::input::keyboard::winit_keycode_to_usize;

use super::InputHandler;

/// The keyboard input state from an [InputHandler]
#[repr(transparent)]
pub struct InputHandlerKeyboard<'a> {
    pub(super) handler: &'a InputHandler,
}

#[profiling::all_functions]
impl InputHandlerKeyboard<'_> {
    /// Returns whether the key with the given keycode is currently held down
    pub fn is_down(&self, key: KeyCode) -> bool {
        self.handler.cur.keyboard_pressed_keys[winit_keycode_to_usize(key)]
    }

    /// Returns whether the key with the given keycode is currenty not held down
    pub fn is_up(&self, key: KeyCode) -> bool {
        !self.is_down(key)
    }

    /// Returns whether the key with the given keycode was
    /// pressed down this frame.
    pub fn pressed_this_frame(&self, key: KeyCode) -> bool {
        let keycode_idx = winit_keycode_to_usize(key);

        self.handler.cur.keyboard_pressed_keys[keycode_idx]
            && !self.handler.prev.keyboard_pressed_keys[keycode_idx]
    }

    /// Returns whether the key with the given keycode was
    /// released this frame.
    pub fn released_this_frame(&self, key: KeyCode) -> bool {
        let keycode_idx = winit_keycode_to_usize(key);

        !self.handler.cur.keyboard_pressed_keys[keycode_idx]
            && self.handler.prev.keyboard_pressed_keys[keycode_idx]
    }
}
