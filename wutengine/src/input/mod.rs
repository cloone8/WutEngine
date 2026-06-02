//! Raw input handling.

use std::collections::HashMap;
use std::sync::RwLock;

use glam::Vec2;
use keyboard::Keyboard;
use mouse::Mouse;
use winit::event::ButtonId;
use winit::event::DeviceId;
use winit::event::ElementState;

use crate::util::InitOnce;

pub mod keyboard;
pub mod mouse;

/// A mouse input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From)]
pub struct MouseDevice(winit::event::DeviceId);

/// A keyboard input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From)]
pub struct KeyboardDevice(winit::event::DeviceId);

/// An input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From)]
pub enum InputDevice {
    /// A [MouseDevice]
    Mouse(MouseDevice),

    /// A [KeyboardDevice]
    Keyboard(KeyboardDevice),
}

/// Raw input manager
#[derive(Debug, Default)]
pub(crate) struct InputManager {
    most_recent_mouse: RwLock<Option<DeviceId>>, //TODO: I think the DeviceId is always 64 bits or less. Unsafe trickery with atomics?
    most_recent_keyboard: RwLock<Option<DeviceId>>,
    mice: RwLock<HashMap<DeviceId, Mouse>>,
    keyboards: RwLock<HashMap<DeviceId, Keyboard>>,
}

/// Private API
impl InputManager {
    fn new() -> Self {
        Self::default()
    }

    fn remove_device(&self, device: DeviceId) {
        {
            let mut mice = self.mice.write().unwrap();
            mice.remove(&device);
        }
        {
            let mut keyboards = self.keyboards.write().unwrap();
            keyboards.remove(&device);
        }
    }

    fn reset_delta(&self) {
        {
            let mut mice = self.mice.write().unwrap();

            for mouse in mice.values_mut() {
                mouse.reset_frame();
            }
        }
        {
            let mut keyboards = self.keyboards.write().unwrap();

            for keyboard in keyboards.values_mut() {
                keyboard.next_frame();
            }
        }
    }

    fn get_specific_mouse(mice: &HashMap<DeviceId, Mouse>, mouse: DeviceId) -> Option<&Mouse> {
        mice.get(&mouse)
    }

    fn get_latest_mouse(
        mice: &HashMap<DeviceId, Mouse>,
        latest: Option<DeviceId>,
    ) -> Option<&Mouse> {
        if let Some(latest) = latest {
            let latest_mouse = mice.get(&latest);

            if latest_mouse.is_some() {
                return latest_mouse;
            }
        }

        // Fallback behaviour for if there is no latest mouse or if it was disconnected
        mice.values().next()
    }

    fn get_specific_keyboard(
        keyboards: &HashMap<DeviceId, Keyboard>,
        keyboard: DeviceId,
    ) -> Option<&Keyboard> {
        keyboards.get(&keyboard)
    }

    fn get_latest_keyboard(
        keyboards: &HashMap<DeviceId, Keyboard>,
        latest: Option<DeviceId>,
    ) -> Option<&Keyboard> {
        if let Some(latest) = latest {
            let latest_keyboard = keyboards.get(&latest);

            if latest_keyboard.is_some() {
                return latest_keyboard;
            }
        }

        // Fallback behaviour for if there is no latest keyboard or if it was disconnected
        keyboards.values().next()
    }

    fn get_mouse_mut(mice: &mut HashMap<DeviceId, Mouse>, mouse: DeviceId) -> &mut Mouse {
        mice.entry(mouse).or_insert_with(Mouse::new)
    }

    fn get_keyboard_mut(
        keyboards: &mut HashMap<DeviceId, Keyboard>,
        keyboard: DeviceId,
    ) -> &mut Keyboard {
        keyboards.entry(keyboard).or_insert_with(Keyboard::new)
    }

    fn set_most_recent_mouse(&self, mouse: DeviceId) {
        let mut most_recent = self.most_recent_mouse.write().unwrap();

        *most_recent = Some(mouse);
    }

    fn set_most_recent_keyboard(&self, keyboard: DeviceId) {
        let mut most_recent = self.most_recent_keyboard.write().unwrap();

        *most_recent = Some(keyboard);
    }

    fn mouse_motion(&self, mouse: DeviceId, delta: Vec2) {
        self.set_most_recent_mouse(mouse);

        let mut mice = self.mice.write().unwrap();
        let mouse = Self::get_mouse_mut(&mut mice, mouse);

        mouse.pos_delta += delta;
    }

    fn mouse_scroll(&self, mouse: DeviceId, delta: Vec2) {
        self.set_most_recent_mouse(mouse);

        let mut mice = self.mice.write().unwrap();
        let mouse = Self::get_mouse_mut(&mut mice, mouse);

        mouse.scroll_delta += delta;
    }

    fn mouse_button(&self, mouse: DeviceId, button: ButtonId, state: ElementState) {
        self.set_most_recent_mouse(mouse);

        let mut mice = self.mice.write().unwrap();
        let mouse = Self::get_mouse_mut(&mut mice, mouse);

        match state {
            ElementState::Pressed => mouse.set_button_pressed(button),
            ElementState::Released => mouse.set_button_released(&button),
        }
    }

    fn keyboard_key(&self, keyboard: DeviceId, key: keyboard::Key, state: ElementState) {
        self.set_most_recent_keyboard(keyboard);

        let mut keyboards = self.keyboards.write().unwrap();
        let keyboard = Self::get_keyboard_mut(&mut keyboards, keyboard);

        match state {
            ElementState::Pressed => keyboard.set_key_pressed(key),
            ElementState::Released => keyboard.set_key_released(&key),
        }
    }
}

/// The global [InputManager]
pub(crate) static INPUT_MANAGER: InitOnce<InputManager> = InitOnce::new();

/// Initializes the global [InputManager]
pub(crate) fn init() {
    InitOnce::init(&INPUT_MANAGER, InputManager::new());
}

/// Inserts a new raw [winit device event](winit::event::DeviceEvent) for the given [device](winit::event::DeviceId)
/// into the input manager for the current frame.
pub fn insert_new_raw_event(device: DeviceId, event: winit::event::DeviceEvent) {
    profiling::function_scope!();

    match event {
        winit::event::DeviceEvent::Added => {
            //Nothing to do
        }
        winit::event::DeviceEvent::Removed => {
            profiling::scope!("Device removed");

            INPUT_MANAGER.remove_device(device);
        }
        winit::event::DeviceEvent::MouseMotion { delta } => {
            profiling::scope!("Mouse motion");

            INPUT_MANAGER.mouse_motion(device, Vec2::new(delta.0 as f32, -delta.1 as f32));
        }
        winit::event::DeviceEvent::MouseWheel { delta } => {
            profiling::scope!("Mouse wheel");

            match delta {
                winit::event::MouseScrollDelta::LineDelta(hor, ver) => {
                    INPUT_MANAGER.mouse_scroll(device, Vec2::new(hor, ver));
                }
                winit::event::MouseScrollDelta::PixelDelta(phys_pos) => {
                    log::error!(
                        "Pixel delta mouse scrolls are not yet supported. Device: {device:#?}. Pixel delta: {phys_pos:#?}"
                    );
                }
            }
        }
        winit::event::DeviceEvent::Motion { .. } => {
            log::trace!("Ignoring unsupported device motion event");
        }
        winit::event::DeviceEvent::Button { button, state } => {
            profiling::scope!("Button");

            INPUT_MANAGER.mouse_button(device, button, state);
        }
        winit::event::DeviceEvent::Key(raw_key_event) => {
            profiling::scope!("Key");

            if let Ok(as_key) = keyboard::Key::try_from(raw_key_event.physical_key) {
                INPUT_MANAGER.keyboard_key(device, as_key, raw_key_event.state);
            }
        }
    }
}

/// Resets all per-frame delta values back to zero.
/// Should be called by the engine runtime at the end of each rendered frame, so
/// all new input events count for the next frame
pub fn reset_delta() {
    profiling::function_scope!();

    INPUT_MANAGER.reset_delta();
}

fn get_mouse_and<T>(to_query: Option<MouseDevice>, func: impl FnOnce(Option<&Mouse>) -> T) -> T {
    let mice = INPUT_MANAGER.mice.read().unwrap();
    let mouse = match to_query {
        Some(to_query) => {
            let mouse = InputManager::get_specific_mouse(&mice, to_query.0);

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

fn get_keyboard_and<T>(
    to_query: Option<KeyboardDevice>,
    func: impl FnOnce(Option<&Keyboard>) -> T,
) -> T {
    let keyboards = INPUT_MANAGER.keyboards.read().unwrap();

    let keyboard = match to_query {
        Some(to_query) => {
            let keyboard = InputManager::get_specific_keyboard(&keyboards, to_query.0);

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

/// Returns the raw mouse position delta.
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns [Vec2::ZERO]
pub fn raw_mouse_pos_delta(device: Option<MouseDevice>) -> Vec2 {
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
pub fn raw_mouse_scroll_delta(device: Option<MouseDevice>) -> Vec2 {
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
/// even if it was already held before, see [raw_mouse_button_held]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns `false`
pub fn raw_mouse_button_pressed(device: Option<MouseDevice>, button: u32) -> bool {
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
/// [raw_mouse_button_pressed]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns `false`
pub fn raw_mouse_button_held(device: Option<MouseDevice>, button: u32) -> bool {
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
/// even if it was not held before, see [raw_mouse_button_held]
///
/// If `device` is [None], returns the value
/// for the latest changed mouse device.
///
/// If the specified mouse (or the latest mouse) could not be found, returns `false`
pub fn raw_mouse_button_released(device: Option<MouseDevice>, button: u32) -> bool {
    get_mouse_and(device, |mouse| {
        if let Some(mouse) = mouse {
            !mouse.pressed_buttons.contains(&button) && mouse.prev_pressed_buttons.contains(&button)
        } else {
            false
        }
    })
}

/// Returns whether the specified keyboard key was pressed this frame. If the key
/// was already pressed last frame, this returns `false`. To check whether the key is held,
/// even if it was already held before, see [raw_keyboard_key_held]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns `false`
pub fn raw_keyboard_key_pressed(device: Option<KeyboardDevice>, key: keyboard::Key) -> bool {
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
/// [raw_keyboard_key_pressed]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns `false`
pub fn raw_keyboard_key_held(device: Option<KeyboardDevice>, key: keyboard::Key) -> bool {
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
/// even if it was not held before, see [raw_keyboard_key_held]
///
/// If `device` is [None], returns the value
/// for the latest changed keyboard device.
///
/// If the specified keyboard (or the latest keyboard) could not be found, returns `false`
pub fn raw_keyboard_key_released(device: Option<KeyboardDevice>, key: keyboard::Key) -> bool {
    get_keyboard_and(device, |keyboard| {
        if let Some(keyboard) = keyboard {
            !keyboard.pressed_keys.contains(&key) && keyboard.prev_pressed_keys.contains(&key)
        } else {
            false
        }
    })
}
