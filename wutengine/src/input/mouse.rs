//! Mouse interaction and APIs

use std::collections::HashSet;

use glam::Vec2;
use nohash_hasher::IntSet;
use winit::event::ButtonId;

use super::INPUT_MANAGER;
use super::InputManager;
use super::MouseId;

/// Left mouse button
pub const LEFT: u32 = 0;

/// Right mouse button
pub const RIGHT: u32 = 1;

/// Middle mouse button
pub const MIDDLE: u32 = 2;

/// The data belonging to a single mouse
#[derive(Debug, Clone)]
pub(crate) struct Mouse {
    /// The scroll delta in the current frame in "lines".
    /// Positive X means towards the right, positive Y means up
    pub(crate) scroll_delta: Vec2,

    /// The position delta in the current frame, in opaque units. The scale
    /// depends on the DPI scale of the mouse.
    pub(crate) pos_delta: Vec2,

    /// The held buttons in the previous frame
    pub(crate) prev_pressed_buttons: IntSet<ButtonId>,

    /// The currently held buttons
    pub(crate) pressed_buttons: IntSet<ButtonId>,
}

impl Mouse {
    /// Create a new, empty, [Mouse]
    pub(crate) fn new() -> Self {
        Self {
            scroll_delta: Vec2::ZERO,
            pos_delta: Vec2::ZERO,
            prev_pressed_buttons: HashSet::default(),
            pressed_buttons: HashSet::default(),
        }
    }

    /// Clears the frame-specific data for this mouse, ensuring all new
    /// input gets registered to the next frame
    pub(crate) fn reset_frame(&mut self) {
        self.scroll_delta = Vec2::ZERO;
        self.pos_delta = Vec2::ZERO;

        self.prev_pressed_buttons.clone_from(&self.pressed_buttons);
    }

    /// Registers the given button as pressed
    pub(crate) fn set_button_pressed(&mut self, button: ButtonId) {
        let was_released = self.pressed_buttons.insert(button);

        if !was_released {
            log::trace!("Pressed button {button}, which was already pressed");
        }
    }

    /// Registers the given button as released
    pub(crate) fn set_button_released(&mut self, button: &ButtonId) {
        let was_held = self.pressed_buttons.remove(button);

        if !was_held {
            log::trace!("Released button {button}, which was not pressed");
        }
    }
}

fn get_mouse_and<T>(to_query: Option<MouseId>, func: impl FnOnce(Option<&Mouse>) -> T) -> T {
    let mice = INPUT_MANAGER.mice.read().unwrap();
    let mouse = match to_query {
        Some(to_query) => {
            let mouse = InputManager::get_specific_mouse(&mice, to_query);

            if mouse.is_none() {
                log::warn!("Mouse {to_query:?} could not be found, returning default values");
            }

            mouse
        }
        None => {
            let most_recent_mouse = *INPUT_MANAGER.most_recent_mouse.read().unwrap();

            InputManager::get_latest_mouse(&mice, most_recent_mouse)
        }
    };

    func(mouse)
}

/// Returns the raw mouse position delta.
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns [Vec2::ZERO]
pub fn pos_delta(device: Option<MouseId>) -> Vec2 {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            mouse.pos_delta
        } else {
            Vec2::ZERO
        }
    })
}

/// Returns the raw mouse scroll delta.
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns [Vec2::ZERO]
pub fn scroll_delta(device: Option<MouseId>) -> Vec2 {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            mouse.scroll_delta
        } else {
            Vec2::ZERO
        }
    })
}

/// Returns whether the specified mouse button was pressed this frame. If the button
/// was already pressed last frame, this returns `false`. To check whether the button is held,
/// even if it was already held before, see [button_held]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns `false`
pub fn button_pressed(device: Option<MouseId>, button: u32) -> bool {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            mouse.pressed_buttons.contains(&button) && !mouse.prev_pressed_buttons.contains(&button)
        } else {
            false
        }
    })
}

/// Returns whether the specified mouse button was being held this frame. This returns
/// `true` in every frame the button is held. To only get `true` for new presses, see
/// [button_pressed]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns `false`
pub fn button_held(device: Option<MouseId>, button: u32) -> bool {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            mouse.pressed_buttons.contains(&button)
        } else {
            false
        }
    })
}

/// Returns whether the specified mouse button was released this frame. If the button
/// was not held down last frame, this always returns `false`. To check whether the button is held,
/// even if it was not held before, see [button_held]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns `false`
pub fn button_released(device: Option<MouseId>, button: u32) -> bool {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            !mouse.pressed_buttons.contains(&button) && mouse.prev_pressed_buttons.contains(&button)
        } else {
            false
        }
    })
}
