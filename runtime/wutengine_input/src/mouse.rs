//! Mouse interaction and APIs

use std::collections::HashSet;

use nohash_hasher::IntSet;
use winit::event::ButtonId;
use wutengine_math::Vec2;

use super::INPUT_MANAGER;
use crate::WindowIdentifier;

/// Left mouse button
pub const BUTTON_LEFT: u32 = 0;

/// Right mouse button
pub const BUTTON_RIGHT: u32 = 1;

/// Middle mouse button
pub const BUTTON_MIDDLE: u32 = 2;

/// "Back" mouse button
pub const BUTTON_BACK: u32 = 3;

/// "Forward" mouse button
pub const BUTTON_FORWARD: u32 = 4;

/// A mouse input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MouseId(winit::event::DeviceId);

impl MouseId {
    /// Maps a winit device to a [MouseId], if the winit device is valid
    #[inline(always)]
    pub(super) fn from_winit(device: winit::event::DeviceId) -> Option<Self> {
        if device != winit::event::DeviceId::dummy() {
            Some(Self(device))
        } else {
            None
        }
    }
}

/// The data belonging to a single mouse
#[derive(Debug, Clone)]
pub(crate) struct Mouse {
    /// The scroll delta in the current frame in "lines".
    /// Positive X means towards the right, positive Y means up
    scroll_delta: Vec2,

    /// The position delta in the current frame, in opaque units. The scale
    /// depends on the DPI scale of the mouse.
    pos_delta: Vec2,

    /// The held buttons in the previous frame
    prev_pressed_buttons: IntSet<ButtonId>,

    /// The currently held buttons
    pressed_buttons: IntSet<ButtonId>,

    /// The position of the mouse cursor relative to a window.
    /// If [None], the cursor is not currently on any WutEngine window
    window_position: Option<(WindowIdentifier, Vec2)>,
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}

impl Mouse {
    /// Create a new, empty, [Mouse]
    pub(crate) fn new() -> Self {
        Self {
            scroll_delta: Vec2::ZERO,
            pos_delta: Vec2::ZERO,
            prev_pressed_buttons: HashSet::default(),
            pressed_buttons: HashSet::default(),
            window_position: None,
        }
    }

    /// Clears the frame-specific data for this mouse, ensuring all new
    /// input gets registered to the next frame
    pub(crate) fn end_frame(&mut self) {
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

    /// Adds raw mouse cursor movement.
    pub(crate) fn add_raw_position_delta(&mut self, delta: Vec2) {
        self.pos_delta += delta;
    }

    /// Adds raw mouse scroll wheel movement.
    pub(crate) fn add_raw_scroll_delta(&mut self, delta: Vec2) {
        self.scroll_delta += delta;
    }

    /// Sets the position of this mouse relative to a window. If [None],
    /// the mouse is not currently on any window.
    pub(crate) fn set_window_position(&mut self, position: Option<(WindowIdentifier, Vec2)>) {
        self.window_position = position;
    }
}

fn get_mouse_and<T>(to_query: Option<MouseId>, func: impl FnOnce(Option<&Mouse>) -> T) -> T {
    let mice = INPUT_MANAGER.mice.read().unwrap();

    let mouse = match to_query {
        Some(to_query) => {
            let mouse = mice.get_identified_device(&to_query);

            if mouse.is_none() {
                log::warn!("Mouse {to_query:?} could not be found, returning default values");
            }

            mouse
        }
        None => {
            let most_recent_mouse = *INPUT_MANAGER.most_recent_mouse.read().unwrap();

            if let Some(latest) = most_recent_mouse {
                match mice.get_identified_device(&latest) {
                    Some(mouse) => Some(mouse),
                    None => Some(mice.get_any_device()),
                }
            } else {
                Some(mice.get_any_device())
            }
        }
    };

    func(mouse)
}

/// If the mouse is currently on any window managed by WutEngine, returns the ID of that
/// window and the pixel position of the cursor.
///
/// If you want to know the position on a specific window, see [window_pos]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns [None]
pub fn screen_pos(device: Option<MouseId>) -> Option<(WindowIdentifier, Vec2)> {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            mouse.window_position
        } else {
            None
        }
    })
}

/// If the mouse is currently on the given window, returns
/// the pixel position of the cursor.
///
/// If you want to know the position on any window, see [screen_pos]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns [None]
pub fn window_pos(device: Option<MouseId>, window: WindowIdentifier) -> Option<Vec2> {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            mouse
                .window_position
                .and_then(|(win, pos)| if win == window { Some(pos) } else { None })
        } else {
            None
        }
    })
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
