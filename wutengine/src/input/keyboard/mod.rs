//! Module for the keyboard input handling logic.
//! In order to activate keyboard input, use the [KeyboardInputPlugin] engine plugin.

use winit::event::ElementState;
use winit::keyboard::PhysicalKey;
use wutengine_core::identifiers::WindowIdentifier;

use crate::winit::event::{DeviceEvent, DeviceId, WindowEvent};

use crate::plugins::{Context, WutEnginePlugin};

#[doc(inline)]
pub use crate::winit::keyboard::KeyCode;

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
    /// The most up-to-date list of currently pressed keys
    pub(crate) pressed_keys: [bool; MAX_KEYCODE],
}

#[profiling::all_functions]
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

#[profiling::all_functions]
impl WutEnginePlugin for KeyboardInputPlugin {
    fn on_device_event(&mut self, _device: DeviceId, event: &DeviceEvent, _context: &mut Context) {
        if let DeviceEvent::Key(raw_key_event) = event {
            self.handle_physical_key(raw_key_event.physical_key, raw_key_event.state);
        }
    }

    fn on_window_event(
        &mut self,
        _window: &WindowIdentifier,
        event: &WindowEvent,
        _context: &mut Context,
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
}

#[cfg(test)]
mod tests {
    use winit::keyboard::KeyCode;

    #[test]
    fn check_winit_keycode_not_too_large() {
        assert!(size_of::<KeyCode>() <= 2);
    }
}
