//! Input related components

use std::collections::{HashMap, HashSet};

use crate::component::{Component, Context};
use crate::input::gamepad::{Gamepad, GamepadId, GamepadInputPlugin};
use crate::input::keyboard::KeyboardInputPlugin;
use crate::input::keyboard::MAX_KEYCODE;
use crate::input::mouse::MouseInputPlugin;

mod gamepad;
mod keyboard;
mod mouse;

pub use gamepad::InputHandlerGamepad;
pub use keyboard::InputHandlerKeyboard;
pub use mouse::InputHandlerMouse;

use glam::Vec2;
use winit::event::ButtonId;

/// The main input handler component. The various input-reading engine plugins will
/// inject their read inputs into each of these components before each Update iteration.
#[derive(Debug)]
pub struct InputHandler {
    prev: InputState,
    cur: InputState,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct InputState {
    /// Map of KeyCode indices to pressed state. To get the index
    /// for a [KeyCode], see [winit_keycode_to_usize]
    keyboard_pressed_keys: [bool; MAX_KEYCODE],

    mouse_pressed_buttons: HashSet<ButtonId>,
    mouse_delta: Vec2,
    mouse_scroll_delta: Vec2,

    gamepads: HashMap<GamepadId, Gamepad>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            keyboard_pressed_keys: [false; MAX_KEYCODE],
            mouse_pressed_buttons: HashSet::default(),
            mouse_delta: Vec2::ZERO,
            mouse_scroll_delta: Vec2::ZERO,
            gamepads: HashMap::default(),
        }
    }
}

impl InputHandler {
    /// Constructs a new [InputHandler] component
    pub fn new() -> Self {
        Self {
            cur: InputState::default(),
            prev: InputState::default(),
        }
    }

    /// The keyboard input state
    pub fn keyboard(&self) -> InputHandlerKeyboard {
        InputHandlerKeyboard { handler: self }
    }

    /// The gamepad input state
    pub fn gamepad(&self) -> InputHandlerGamepad {
        InputHandlerGamepad { handler: self }
    }

    /// The mouse input state
    pub fn mouse(&self) -> InputHandlerMouse {
        InputHandlerMouse { handler: self }
    }
}

impl Component for InputHandler {
    fn pre_update(&mut self, context: &mut Context) {
        self.prev = std::mem::take(&mut self.cur);

        if let Some(keyboard_plugin) = context.plugin.get::<KeyboardInputPlugin>() {
            self.cur
                .keyboard_pressed_keys
                .copy_from_slice(&keyboard_plugin.pressed_keys);
        }

        if let Some(mouse_plugin) = context.plugin.get::<MouseInputPlugin>() {
            self.cur.mouse_pressed_buttons = mouse_plugin.buttons.clone();
            self.cur.mouse_delta = mouse_plugin.mouse_delta;
            self.cur.mouse_scroll_delta = mouse_plugin.scroll_delta;
        }

        if let Some(gamepad_plugin) = context.plugin.get::<GamepadInputPlugin>() {
            self.cur.gamepads = gamepad_plugin.gamepads.clone();
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
