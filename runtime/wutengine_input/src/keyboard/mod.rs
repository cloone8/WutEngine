//! Keyboard interaction and APIs

use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use wutengine_util::IntSet;

mod key;
mod logical_key;

pub use key::*;
pub use logical_key::*;

use super::INPUT_MANAGER;

/// A keyboard input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KeyboardId(winit::event::DeviceId);

impl KeyboardId {
    /// Maps a winit device to a [KeyboardId], if the winit device is valid
    #[inline(always)]
    pub(super) fn from_winit(device: winit::event::DeviceId) -> Option<Self> {
        if device != winit::event::DeviceId::dummy() {
            Some(Self(device))
        } else {
            None
        }
    }
}

/// Data concerning the input of a single keyboard
#[derive(Debug, Clone)]
pub(crate) struct Keyboard {
    /// The held keys in the previous frame
    prev_pressed_keys: IntSet<Key>,

    /// The currently held keys
    pressed_keys: IntSet<Key>,

    /// Logical keyboard inputs, ordered by when they happened. Mostly used by UI
    logical_inputs: Vec<LogicalInput>,
}

/// A logical keyboard input. Includes any OS mappings and composite inputs.
/// Mostly used by UI rendering and text editing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalInput {
    /// A key was pressed
    Pressed(LogicalKey),

    /// Text was entered. This includes control characters and other non-printable strings
    Text(String),

    /// A key was released
    Released(LogicalKey),
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyboard {
    /// New [Keyboard] with no keys pressed
    pub(crate) fn new() -> Self {
        Self {
            prev_pressed_keys: IntSet::default(),
            pressed_keys: IntSet::default(),
            logical_inputs: Vec::new(),
        }
    }

    /// Makes sure that all new input is registered to the next frame
    pub(crate) fn end_frame(&mut self) {
        self.prev_pressed_keys.clone_from(&self.pressed_keys);
        self.logical_inputs.clear();
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

    /// Add a new logical input to the keyboard
    pub(crate) fn add_logical_input(&mut self, logical: LogicalInput) {
        self.logical_inputs.push(logical);
    }
}

#[inline]
fn winit_native_keycode_to_u32(nkc: winit::keyboard::NativeKeyCode) -> Option<u32> {
    match nkc {
        winit::keyboard::NativeKeyCode::Unidentified => None,
        winit::keyboard::NativeKeyCode::Android(scancode) => Some(scancode),
        winit::keyboard::NativeKeyCode::MacOS(scancode) => Some(u32::from(scancode)),
        winit::keyboard::NativeKeyCode::Windows(scancode) => Some(u32::from(scancode)),
        winit::keyboard::NativeKeyCode::Xkb(keycode) => Some(keycode),
    }
}

#[inline]
fn winit_nativekey_to_unknown_logical(kc: winit::keyboard::NativeKey) -> Option<UnknownLogicalKey> {
    match kc {
        winit::keyboard::NativeKey::Unidentified => None,
        winit::keyboard::NativeKey::Android(scancode) => Some(UnknownLogicalKey::Code(scancode)),
        winit::keyboard::NativeKey::MacOS(scancode) => {
            Some(UnknownLogicalKey::Code(u32::from(scancode)))
        }
        winit::keyboard::NativeKey::Windows(scancode) => {
            Some(UnknownLogicalKey::Code(u32::from(scancode)))
        }
        winit::keyboard::NativeKey::Xkb(scancode) => Some(UnknownLogicalKey::Code(scancode)),
        winit::keyboard::NativeKey::Web(s) => Some(UnknownLogicalKey::String(s.to_string())),
    }
}

fn get_keyboard_and<T>(
    to_query: Option<KeyboardId>,
    func: impl FnOnce(Option<&Keyboard>) -> T,
) -> T {
    let keyboards = INPUT_MANAGER.keyboards.read().unwrap();

    let keyboard = match to_query {
        Some(to_query) => {
            let keyboard = keyboards.get_identified_device(&to_query);

            if keyboard.is_none() {
                log::warn!("Keyboard {to_query:?} could not be found, returning default values");
            }

            keyboard
        }
        None => {
            let most_recent_keyboard = *INPUT_MANAGER.most_recent_keyboard.read().unwrap();

            if let Some(latest) = most_recent_keyboard {
                match keyboards.get_identified_device(&latest) {
                    Some(keyboard) => Some(keyboard),
                    None => Some(keyboards.get_any_device()),
                }
            } else {
                Some(keyboards.get_any_device())
            }
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

/// Returns all keys pressed this frame. If the key
/// was not first pressed last frame, this always returns `false`. To get the held keys,
/// even if it they were also held before, see [held_keys]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns an empty set
pub fn pressed_keys(device: Option<KeyboardId>) -> HashSet<Key> {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard
                .pressed_keys
                .difference(&keyboard.prev_pressed_keys)
                .copied()
                .collect()
        } else {
            HashSet::default()
        }
    })
}

/// Returns all keys held this frame. To get the keys that were first pressed
/// this frame, see [pressed_keys]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns an empty set
pub fn held_keys(device: Option<KeyboardId>) -> IntSet<Key> {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard.pressed_keys.clone()
        } else {
            HashSet::default()
        }
    })
}

/// Returns all keys released this frame. If the key
/// was not held down last frame, it will not be included. To get the full
/// set of held keys (which can also be used for checking if a key was not held),
/// see [held_keys]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns an empty set
pub fn released_keys(device: Option<KeyboardId>) -> HashSet<Key> {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard
                .prev_pressed_keys
                .difference(&keyboard.pressed_keys)
                .copied()
                .collect()
        } else {
            HashSet::default()
        }
    })
}

/// Returns all logical inputs that were made this frame, in order. This should
/// mostly be used by UI code and other non-gameplay functionality.
///
/// If `device` is [None], returns the values
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns an empty vector
pub fn logical_inputs(device: Option<KeyboardId>) -> Vec<LogicalInput> {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard.logical_inputs.clone()
        } else {
            Vec::new()
        }
    })
}
