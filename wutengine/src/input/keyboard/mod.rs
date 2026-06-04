//! Keyboard interaction and APIs

use std::collections::HashSet;

mod key;

pub use key::*;

use super::INPUT_MANAGER;
use super::InputManager;
use super::KeyboardId;

/// Data concerning the input of a single keyboard
#[derive(Debug, Clone)]
pub(crate) struct Keyboard {
    /// The held keys in the previous frame
    pub(crate) prev_pressed_keys: HashSet<Key>,

    /// The currently held keys
    pub(crate) pressed_keys: HashSet<Key>,
}

impl Keyboard {
    /// New [Keyboard] with no keys pressed
    pub(crate) fn new() -> Self {
        Self {
            prev_pressed_keys: HashSet::default(),
            pressed_keys: HashSet::default(),
        }
    }

    /// Makes sure that all new input is registered to the next frame
    pub(crate) fn next_frame(&mut self) {
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

fn get_keyboard_and<T>(
    to_query: Option<KeyboardId>,
    func: impl FnOnce(Option<&Keyboard>) -> T,
) -> T {
    let keyboards = INPUT_MANAGER.keyboards.read().unwrap();

    let keyboard = match to_query {
        Some(to_query) => {
            let keyboard = InputManager::get_specific_keyboard(&keyboards, to_query);

            if keyboard.is_none() {
                log::warn!("Keyboard {to_query:?} could not be found, returning default values");
            }

            keyboard
        }
        None => {
            let most_recent_keyboard = *INPUT_MANAGER.most_recent_keyboard.read().unwrap();

            InputManager::get_latest_keyboard(&keyboards, most_recent_keyboard)
        }
    };

    func(keyboard)
}

/// Returns whether the specified keyboard key was pressed this frame. If the key
/// was already pressed last frame, this returns `false`. To check whether the key is held,
/// even if it was already held before, see [key_held]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns `false`
pub fn key_pressed(device: Option<KeyboardId>, key: Key) -> bool {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard.pressed_keys.contains(&key) && !keyboard.prev_pressed_keys.contains(&key)
        } else {
            false
        }
    })
}

/// Returns whether the specified keyboard key was being held this frame. This returns
/// `true` in every frame the key is held. To only get `true` for new presses, see
/// [key_pressed]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns `false`
pub fn key_held(device: Option<KeyboardId>, key: Key) -> bool {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard.pressed_keys.contains(&key)
        } else {
            false
        }
    })
}

/// Returns whether the specified keyboard key was released this frame. If the key
/// was not held down last frame, this always returns `false`. To check whether the key is held,
/// even if it was not held before, see [key_held]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns `false`
pub fn key_released(device: Option<KeyboardId>, key: Key) -> bool {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            !keyboard.pressed_keys.contains(&key) && keyboard.prev_pressed_keys.contains(&key)
        } else {
            false
        }
    })
}
