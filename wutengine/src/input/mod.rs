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
                keyboard.reset_frame();
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

fn get_mouse_and<T>(func: impl FnOnce(Option<&Mouse>) -> T) -> T {
    let most_recent_mouse = *INPUT_MANAGER.most_recent_mouse.read().unwrap();
    let mice = INPUT_MANAGER.mice.read().unwrap();

    let mouse = InputManager::get_latest_mouse(&mice, most_recent_mouse);

    func(mouse)
}

fn get_keyboard_and<T>(func: impl FnOnce(Option<&Keyboard>) -> T) -> T {
    let most_recent_keyboard = *INPUT_MANAGER.most_recent_keyboard.read().unwrap();
    let keyboards = INPUT_MANAGER.keyboards.read().unwrap();

    let keyboard = InputManager::get_latest_keyboard(&keyboards, most_recent_keyboard);

    func(keyboard)
}

///TODO: Add argument for querying specific mouse
pub fn raw_mouse_pos_delta() -> Vec2 {
    get_mouse_and(|mouse| {
        if let Some(mouse) = mouse {
            mouse.pos_delta
        } else {
            Vec2::ZERO
        }
    })
}

///TODO: Add argument for querying specific mouse
pub fn raw_mouse_scroll_delta() -> Vec2 {
    get_mouse_and(|mouse| {
        if let Some(mouse) = mouse {
            mouse.scroll_delta
        } else {
            Vec2::ZERO
        }
    })
}

///TODO: Add argument for querying specific mouse
pub fn raw_mouse_button_pressed(button: u32) -> bool {
    get_mouse_and(|mouse| {
        if let Some(mouse) = mouse {
            mouse.pressed_buttons.contains(&button) && !mouse.prev_pressed_buttons.contains(&button)
        } else {
            false
        }
    })
}

///TODO: Add argument for querying specific mouse
pub fn raw_mouse_button_held(button: u32) -> bool {
    get_mouse_and(|mouse| {
        if let Some(mouse) = mouse {
            mouse.pressed_buttons.contains(&button)
        } else {
            false
        }
    })
}

///TODO: Add argument for querying specific mouse
pub fn raw_mouse_button_released(button: u32) -> bool {
    get_mouse_and(|mouse| {
        if let Some(mouse) = mouse {
            !mouse.pressed_buttons.contains(&button) && mouse.prev_pressed_buttons.contains(&button)
        } else {
            false
        }
    })
}

///TODO: Add argument for querying specific keyboard
pub fn raw_keyboard_button_pressed(key: keyboard::Key) -> bool {
    get_keyboard_and(|keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard.pressed_keys.contains(&key) && !keyboard.prev_pressed_keys.contains(&key)
        } else {
            false
        }
    })
}

///TODO: Add argument for querying specific keyboard
pub fn raw_keyboard_button_held(key: keyboard::Key) -> bool {
    get_keyboard_and(|keyboard| {
        if let Some(keyboard) = keyboard {
            keyboard.pressed_keys.contains(&key)
        } else {
            false
        }
    })
}

///TODO: Add argument for querying specific keyboard
pub fn raw_keyboard_button_released(key: keyboard::Key) -> bool {
    get_keyboard_and(|keyboard| {
        if let Some(keyboard) = keyboard {
            !keyboard.pressed_keys.contains(&key) && keyboard.prev_pressed_keys.contains(&key)
        } else {
            false
        }
    })
}
