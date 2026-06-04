//! Raw input handling.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::RwLock;

use gamepad::Gamepad;
use gilrs::Gilrs;
use glam::Vec2;
use keyboard::Keyboard;
use mouse::Mouse;
use winit::event::ButtonId;
use winit::event::DeviceId;
use winit::event::ElementState;

use crate::util::InitOnce;

pub mod gamepad;
pub mod keyboard;
pub mod mouse;

/// A mouse input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From)]
#[repr(transparent)]
pub struct MouseId(winit::event::DeviceId);

/// A keyboard input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From)]
#[repr(transparent)]
pub struct KeyboardId(winit::event::DeviceId);

/// A gamepad input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From, derive_more::Display)]
#[repr(transparent)]
pub struct GamepadId(gilrs::GamepadId);

/// An input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub enum InputDeviceId {
    /// A mouse
    Mouse(MouseId),

    /// A keyboard
    Keyboard(KeyboardId),

    /// A gamepad
    Gamepad(GamepadId),
}

/// Raw input manager
#[derive(Debug)]
pub(crate) struct InputManager {
    gamepad_manager: Option<Mutex<Gilrs>>,
    most_recent_mouse: RwLock<Option<MouseId>>, //TODO: I think the DeviceId is always 64 bits or less. Unsafe trickery with atomics?
    most_recent_keyboard: RwLock<Option<KeyboardId>>,
    most_recent_gamepad: RwLock<Option<GamepadId>>,
    mice: RwLock<HashMap<MouseId, Mouse>>,
    keyboards: RwLock<HashMap<KeyboardId, Keyboard>>,
    gamepads: RwLock<HashMap<GamepadId, Gamepad>>,
}

impl Default for InputManager {
    fn default() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(grs) => Some(Mutex::new(grs)),
            Err(e) => {
                log::error!("Failed to initialize Gilrs for reading gamepad input: {e}");
                None
            }
        };

        Self {
            gamepad_manager: gilrs,
            most_recent_mouse: Default::default(),
            most_recent_keyboard: Default::default(),
            most_recent_gamepad: Default::default(),
            mice: Default::default(),
            keyboards: Default::default(),
            gamepads: Default::default(),
        }
    }
}

/// Private API
impl InputManager {
    fn new() -> Self {
        Self::default()
    }

    fn remove_device(&self, device: DeviceId) {
        {
            let mut mice = self.mice.write().unwrap();
            mice.remove(&MouseId(device));
        }
        {
            let mut keyboards = self.keyboards.write().unwrap();
            keyboards.remove(&KeyboardId(device));
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
        {
            let mut gamepads = self.gamepads.write().unwrap();

            for gamepad in gamepads.values_mut() {
                gamepad.next_frame();
            }
        }
    }

    fn get_specific_mouse(mice: &HashMap<MouseId, Mouse>, mouse: MouseId) -> Option<&Mouse> {
        mice.get(&mouse)
    }

    fn get_latest_mouse(mice: &HashMap<MouseId, Mouse>, latest: Option<MouseId>) -> Option<&Mouse> {
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
        keyboards: &HashMap<KeyboardId, Keyboard>,
        keyboard: KeyboardId,
    ) -> Option<&Keyboard> {
        keyboards.get(&keyboard)
    }

    fn get_latest_keyboard(
        keyboards: &HashMap<KeyboardId, Keyboard>,
        latest: Option<KeyboardId>,
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

    fn get_specific_gamepad(
        gamepads: &HashMap<GamepadId, Gamepad>,
        gamepad: GamepadId,
    ) -> Option<&Gamepad> {
        gamepads.get(&gamepad)
    }

    fn get_latest_gamepad(
        gamepads: &HashMap<GamepadId, Gamepad>,
        latest: Option<GamepadId>,
    ) -> Option<&Gamepad> {
        if let Some(latest) = latest {
            let latest_gamepad = gamepads.get(&latest);

            if latest_gamepad.is_some() {
                return latest_gamepad;
            }
        }

        // Fallback behaviour for if there is no latest gamepad or if it was disconnected
        gamepads.values().next()
    }

    fn get_mouse_mut(mice: &mut HashMap<MouseId, Mouse>, mouse: MouseId) -> &mut Mouse {
        mice.entry(mouse).or_insert_with(Mouse::new)
    }

    fn get_keyboard_mut(
        keyboards: &mut HashMap<KeyboardId, Keyboard>,
        keyboard: KeyboardId,
    ) -> &mut Keyboard {
        keyboards.entry(keyboard).or_insert_with(Keyboard::new)
    }

    fn set_most_recent_mouse(&self, mouse: MouseId) {
        let mut most_recent = self.most_recent_mouse.write().unwrap();

        *most_recent = Some(mouse);
    }

    fn set_most_recent_keyboard(&self, keyboard: KeyboardId) {
        let mut most_recent = self.most_recent_keyboard.write().unwrap();

        *most_recent = Some(keyboard);
    }

    fn mouse_motion(&self, mouse: MouseId, delta: Vec2) {
        self.set_most_recent_mouse(mouse);

        let mut mice = self.mice.write().unwrap();
        let mouse = Self::get_mouse_mut(&mut mice, mouse);

        mouse.pos_delta += delta;
    }

    fn mouse_scroll(&self, mouse: MouseId, delta: Vec2) {
        self.set_most_recent_mouse(mouse);

        let mut mice = self.mice.write().unwrap();
        let mouse = Self::get_mouse_mut(&mut mice, mouse);

        mouse.scroll_delta += delta;
    }

    fn mouse_button(&self, mouse: MouseId, button: ButtonId, state: ElementState) {
        self.set_most_recent_mouse(mouse);

        let mut mice = self.mice.write().unwrap();
        let mouse = Self::get_mouse_mut(&mut mice, mouse);

        match state {
            ElementState::Pressed => mouse.set_button_pressed(button),
            ElementState::Released => mouse.set_button_released(&button),
        }
    }

    fn keyboard_key(&self, keyboard: KeyboardId, key: keyboard::Key, state: ElementState) {
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

            INPUT_MANAGER.mouse_motion(MouseId(device), Vec2::new(delta.0 as f32, -delta.1 as f32));
        }
        winit::event::DeviceEvent::MouseWheel { delta } => {
            profiling::scope!("Mouse wheel");

            match delta {
                winit::event::MouseScrollDelta::LineDelta(hor, ver) => {
                    INPUT_MANAGER.mouse_scroll(MouseId(device), Vec2::new(hor, ver));
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

            INPUT_MANAGER.mouse_button(MouseId(device), button, state);
        }
        winit::event::DeviceEvent::Key(raw_key_event) => {
            profiling::scope!("Key");

            if let Ok(as_key) = keyboard::Key::try_from(raw_key_event.physical_key) {
                INPUT_MANAGER.keyboard_key(KeyboardId(device), as_key, raw_key_event.state);
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
