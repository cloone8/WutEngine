//! Module for the keyboard input handling logic.
//! In order to activate keyboard input, use the [KeyboardInputPlugin] engine plugin.

use winit::event::ElementState;
use winit::keyboard::PhysicalKey;
use wutengine_ecs::world::World;
use wutengine_graphics::windowing::WindowIdentifier;

use crate::builtins::components::InputHandler;
use crate::command::Command;
use crate::windowing::winit::event::{DeviceEvent, DeviceId, WindowEvent};

use crate::plugins::WutEnginePlugin;

pub use crate::windowing::winit::keyboard::KeyCode;

mod winit_mapping;

pub(crate) use winit_mapping::winit_keycode_to_usize;

/// The conservative upper bound for the maximum numerical keycode of a keyboard input.
/// The actual value is likely lower
pub(crate) const MAX_KEYCODE: usize = 2usize.pow((size_of::<KeyCode>() * 8usize) as u32) - 1;

/// The keyboard input reader plugin.
/// Reads the raw keyboard input from the WutEngine window handler
/// and delivers it to the InputHandler components in the world.
#[derive(Debug)]
pub struct KeyboardInputPlugin {
    pressed_keys: [bool; MAX_KEYCODE],
}

impl KeyboardInputPlugin {
    /// Creates a new, default, [KeyboardInputPlugin]
    pub fn new() -> Self {
        Self {
            pressed_keys: [false; MAX_KEYCODE],
        }
    }

    fn handle_physical_key(&mut self, key: PhysicalKey, state: ElementState) {
        if let PhysicalKey::Code(code) = key {
            let code_index = winit_keycode_to_usize(code);

            match state {
                ElementState::Pressed => {
                    self.pressed_keys[code_index] = true;
                }
                ElementState::Released => {
                    self.pressed_keys[code_index] = false;
                }
            }
        }
    }
}

impl Default for KeyboardInputPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WutEnginePlugin for KeyboardInputPlugin {
    fn on_device_event(
        &mut self,
        _world: &mut World,
        _device: DeviceId,
        event: &DeviceEvent,
        _commands: &mut Command,
    ) {
        if let DeviceEvent::Key(raw_key_event) = event {
            self.handle_physical_key(raw_key_event.physical_key, raw_key_event.state);
        }
    }

    fn on_window_event(
        &mut self,
        _world: &mut World,
        _window: &WindowIdentifier,
        event: &WindowEvent,
        _commands: &mut Command,
    ) {
        if let WindowEvent::KeyboardInput {
            device_id: _,
            event,
            is_synthetic: _,
        } = event
        {
            self.handle_physical_key(event.physical_key, event.state);
        }
    }

    fn pre_update(&mut self, world: &mut World, _commands: &mut Command) {
        unsafe {
            world.query(|_id, handler: &mut InputHandler| {
                handler
                    .keyboard_pressed_keys
                    .copy_from_slice(&self.pressed_keys);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use winit::keyboard::KeyCode;

    #[test]
    fn check_winit_keycode_not_too_large() {
        assert!(size_of::<KeyCode>() <= 2);
    }
}
