//! Keyboard interaction and APIs

use std::collections::HashSet;

mod key;

pub use key::*;

#[derive(Debug, Clone)]
pub(crate) struct Keyboard {
    /// The held keys in the previous frame
    pub(crate) prev_pressed_keys: HashSet<Key>,

    /// The currently held keys
    pub(crate) pressed_keys: HashSet<Key>,
}

impl Keyboard {
    pub(crate) fn new() -> Self {
        Self {
            prev_pressed_keys: HashSet::default(),
            pressed_keys: HashSet::default(),
        }
    }
    pub(crate) fn reset_frame(&mut self) {
        self.prev_pressed_keys.clone_from(&self.pressed_keys);
    }

    /// Registers the given key as pressed
    pub(crate) fn set_key_pressed(&mut self, key: Key) {
        let was_released = self.pressed_keys.insert(key);

        if !was_released {
            log::trace!("Pressed key {key:?}, which was already pressed");
        }
    }

    /// Registers the given key as released
    pub(crate) fn set_key_released(&mut self, key: &Key) {
        let was_held = self.pressed_keys.remove(key);

        if !was_held {
            log::trace!("Released key {key:?}, which was not pressed");
        }
    }
}
