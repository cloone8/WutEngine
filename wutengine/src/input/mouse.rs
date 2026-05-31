//! Mouse interaction and APIs

use std::collections::HashSet;

use glam::Vec2;
use nohash_hasher::IntSet;
use winit::event::ButtonId;

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
