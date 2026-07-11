#![doc = include_str!("../README.md")]

use core::fmt::Display;
use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
};

use gamepad::{Gamepad, GamepadId};
use gilrs::Gilrs;
use keyboard::{Keyboard, KeyboardId};
use mouse::{Mouse, MouseId};
use winit::event::{ButtonId, DeviceId, ElementState};
use wutengine_math::Vec2;
use wutengine_util::InitOnce;

pub mod gamepad;
pub mod keyboard;
pub mod mouse;

/// Opaque identifier for a window that can receive input.
/// Users of the library should convert to-and-from this type
/// from their own actual window handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct WindowIdentifier(u64);

impl WindowIdentifier {
    /// Creates a new WindowIdentifier from a raw int
    #[inline(always)]
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    /// Returns the raw integer for this identifier
    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl From<u64> for WindowIdentifier {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl Display for WindowIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

//TODO: Make input device trait?

/// A set of input devices, either uniquely identifier ([Self::Identified]) or not ([Self::Unidentified])
#[derive(Debug)]
enum DeviceSet<K, V> {
    /// Only unidentified devices.
    Unidentified(V),

    /// At least one device has been identified.
    Identified(HashMap<K, V>),
}

impl<K: Eq + core::hash::Hash + Clone, V: Default> DeviceSet<K, V> {
    /// Removes an identified device from the set
    fn remove_device(&mut self, device: &K) {
        if let Self::Identified(devices) = self {
            devices.remove(device);

            if devices.is_empty() {
                *self = Self::default();
            }
        }
    }

    /// Executes a function for each device
    fn for_each(&mut self, mut func: impl FnMut(&mut V)) {
        match self {
            Self::Unidentified(device) => func(device),
            Self::Identified(devices) => devices.values_mut().for_each(func),
        }
    }

    /// Updates a device (identified or not) in the set.
    /// If an identified device is updated but does not yet exist, the device
    /// is automatically added to the set.
    fn update_device(&mut self, device: Option<&K>, mut func: impl FnMut(&mut V)) {
        if let Self::Unidentified(unidentified_device) = self {
            match device {
                Some(device) => {
                    let mut new_devices_map = HashMap::default();

                    new_devices_map.insert(device.clone(), core::mem::take(unidentified_device));

                    *self = Self::Identified(new_devices_map);
                }
                None => {
                    func(unidentified_device);
                    return;
                }
            }
        }

        if let Self::Identified(devices) = self {
            assert!(!devices.is_empty(), "Empty identified device list");

            match device {
                Some(device) => {
                    func(devices.entry(device.clone()).or_default());
                }
                None => {
                    devices.values_mut().for_each(func);
                }
            }
        }
    }

    /// Returns an identified device, if it exists
    fn get_identified_device(&self, device: &K) -> Option<&V> {
        if let Self::Identified(devices) = self {
            devices.get(device)
        } else {
            None
        }
    }

    /// Returns an identified device mutably, if it exists
    fn get_identified_device_mut(&mut self, device: &K) -> Option<&mut V> {
        if let Self::Identified(devices) = self {
            devices.get_mut(device)
        } else {
            None
        }
    }

    /// Returns any device in the set
    fn get_any_device(&self) -> &V {
        match self {
            Self::Unidentified(unidentified_device) => unidentified_device,
            Self::Identified(devices) => devices
                .values()
                .next()
                .expect("Empty identified devices map"),
        }
    }
}

impl<K, V: Default> Default for DeviceSet<K, V> {
    fn default() -> Self {
        Self::Unidentified(V::default())
    }
}

/// Raw input manager
#[derive(Debug)]
pub(crate) struct InputManager {
    /// Gamepad manager
    gamepad_manager: Option<Mutex<Gilrs>>,

    /// The most recently used mouse
    most_recent_mouse: RwLock<Option<MouseId>>,

    /// The most recently used keyboard
    most_recent_keyboard: RwLock<Option<KeyboardId>>,

    /// The most recently used gamepad
    most_recent_gamepad: RwLock<Option<GamepadId>>,

    /// All known mice
    mice: RwLock<DeviceSet<MouseId, Mouse>>,

    /// All known keyboards
    keyboards: RwLock<DeviceSet<KeyboardId, Keyboard>>,

    /// All known gamepads
    gamepads: RwLock<DeviceSet<GamepadId, Gamepad>>,
}

impl Default for InputManager {
    fn default() -> Self {
        let mut gamepads = DeviceSet::default();
        let gilrs = match gilrs::GilrsBuilder::new()
            .add_included_mappings(true)
            .add_env_mappings(false)
            .build()
        {
            Ok(grs) => {
                // Poll for the initial set of gamepads
                for (id, gamepad) in grs.gamepads() {
                    log::info!(
                        "Found already connected gamepad \"{}\" with ID {}",
                        gamepad.name(),
                        gamepad.id()
                    );

                    gamepads.update_device(Some(&GamepadId(id)), |_| {});
                }

                Some(Mutex::new(grs))
            }
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
            gamepads: RwLock::new(gamepads),
        }
    }
}

/// Private API
impl InputManager {
    /// Returns a new default [InputManager]
    fn new() -> Self {
        Self::default()
    }

    /// Removes a winit device from the manager
    fn remove_winit_device(&self, device: DeviceId) {
        {
            if let Some(mouse_id) = MouseId::from_winit(device) {
                self.mice.write().unwrap().remove_device(&mouse_id);
            }
        }
        {
            if let Some(keyboard_id) = KeyboardId::from_winit(device) {
                self.keyboards.write().unwrap().remove_device(&keyboard_id);
            }
        }
    }

    /// Advances the input manager and all devices to the next frame
    fn end_frame(&self) {
        self.mice.write().unwrap().for_each(Mouse::end_frame);
        self.keyboards
            .write()
            .unwrap()
            .for_each(Keyboard::end_frame);
        self.gamepads.write().unwrap().for_each(Gamepad::end_frame);
    }

    /// Sets the most recent mouse
    fn set_most_recent_mouse(&self, mouse: MouseId) {
        log::trace!("Setting most recent mouse to {mouse:?}");

        let mut most_recent = self.most_recent_mouse.write().unwrap();

        *most_recent = Some(mouse);
    }

    /// Sets the most recent keyboard
    fn set_most_recent_keyboard(&self, keyboard: KeyboardId) {
        log::trace!("Setting most recent keyboard to {keyboard:?}");

        let mut most_recent = self.most_recent_keyboard.write().unwrap();

        *most_recent = Some(keyboard);
    }

    /// Sets the most recent gamepad
    fn set_most_recent_gamepad(&self, gamepad: GamepadId) {
        log::trace!("Setting most recent gamepad to {gamepad:?}");

        let mut most_recent = self.most_recent_gamepad.write().unwrap();

        *most_recent = Some(gamepad);
    }

    /// Adds physical non-mapped mouse motion to the given mouse
    fn mouse_motion(&self, mouse: Option<MouseId>, delta: Vec2) {
        if let Some(identified_mouse) = mouse {
            self.set_most_recent_mouse(identified_mouse);
        }

        let mut mice = self.mice.write().unwrap();

        mice.update_device(mouse.as_ref(), |mouse| {
            mouse.add_raw_position_delta(delta);
        });
    }

    /// Sets the mouse scroll value for the given mouse
    fn mouse_scroll(&self, mouse: Option<MouseId>, delta: Vec2) {
        if let Some(identified_mouse) = mouse {
            self.set_most_recent_mouse(identified_mouse);
        }

        let mut mice = self.mice.write().unwrap();

        mice.update_device(mouse.as_ref(), |mouse| {
            mouse.add_raw_scroll_delta(delta);
        });
    }

    /// Sets the physical mouse button state for a given mouse
    fn mouse_button(&self, mouse: Option<MouseId>, button: ButtonId, state: ElementState) {
        if let Some(identified_mouse) = mouse {
            self.set_most_recent_mouse(identified_mouse);
        }

        let mut mice = self.mice.write().unwrap();

        mice.update_device(mouse.as_ref(), |mouse| match state {
            ElementState::Pressed => mouse.set_button_pressed(button),
            ElementState::Released => mouse.set_button_released(&button),
        });
    }

    /// Sets the mouse position for a mouse
    fn mouse_window_position(
        &self,
        mouse: Option<MouseId>,
        position: Option<(WindowIdentifier, Vec2)>,
    ) {
        if let Some(identified_mouse) = mouse {
            self.set_most_recent_mouse(identified_mouse);
        }

        let mut mice = self.mice.write().unwrap();

        mice.update_device(mouse.as_ref(), |mouse| {
            mouse.set_window_position(position);
        });
    }

    /// Adds a physical key state to the given keyboard
    fn keyboard_key(&self, keyboard: Option<KeyboardId>, key: keyboard::Key, state: ElementState) {
        if let Some(identified_keyboard) = keyboard {
            self.set_most_recent_keyboard(identified_keyboard);
        }

        let mut keyboards = self.keyboards.write().unwrap();

        keyboards.update_device(keyboard.as_ref(), |kbd| match state {
            ElementState::Pressed => kbd.set_key_pressed(key),
            ElementState::Released => kbd.set_key_released(&key),
        });
    }

    /// Adds a logical key input to the given keyboard
    fn keyboard_logical_key(
        &self,
        keyboard: Option<KeyboardId>,
        logical_key: keyboard::LogicalKey,
        state: ElementState,
    ) {
        if let Some(identified_keyboard) = keyboard {
            self.set_most_recent_keyboard(identified_keyboard);
        }

        let mut keyboards = self.keyboards.write().unwrap();

        keyboards.update_device(keyboard.as_ref(), |kbd| match state {
            ElementState::Pressed => {
                kbd.add_logical_input(keyboard::LogicalInput::Pressed(logical_key.clone()))
            }
            ElementState::Released => {
                kbd.add_logical_input(keyboard::LogicalInput::Released(logical_key.clone()))
            }
        });
    }

    /// Adds text input to the given keyboard
    fn keyboard_text(&self, keyboard: Option<KeyboardId>, text: &str) {
        assert!(!text.is_empty(), "Cannot send empty text to keyboard");

        if let Some(identified_keyboard) = keyboard {
            self.set_most_recent_keyboard(identified_keyboard);
        }

        let mut keyboards = self.keyboards.write().unwrap();

        keyboards.update_device(keyboard.as_ref(), |kbd| {
            kbd.add_logical_input(keyboard::LogicalInput::Text(text.to_string()));
        });
    }
}

/// The global [InputManager]
pub(crate) static INPUT_MANAGER: InitOnce<InputManager, false> = InitOnce::new_checked();

/// Initializes the global [InputManager]
#[doc(hidden)]
pub fn init() {
    InitOnce::init(&INPUT_MANAGER, InputManager::new());
}

/// Inserts a new raw [winit device event](winit::event::DeviceEvent) for the given [device](winit::event::DeviceId)
/// into the input manager for the current frame.
pub fn insert_raw_device_event(device: DeviceId, event: winit::event::DeviceEvent) {
    profiling::function_scope!();

    log::trace!(
        "Device event for device {device:?} on frame {}: {event:#?}",
        wutengine_time::frame_num()
    );

    match event {
        winit::event::DeviceEvent::Added => {
            profiling::scope!("Device added");

            log::debug!("Raw input device added: {device:?}");
        }
        winit::event::DeviceEvent::Removed => {
            profiling::scope!("Device removed");

            log::info!("Raw input device added: {device:?}");

            INPUT_MANAGER.remove_winit_device(device);
        }
        winit::event::DeviceEvent::MouseMotion { delta } => {
            profiling::scope!("Mouse motion");

            INPUT_MANAGER.mouse_motion(
                MouseId::from_winit(device),
                Vec2::new(delta.0 as f32, -delta.1 as f32),
            );
        }
        winit::event::DeviceEvent::MouseWheel { delta } => {
            profiling::scope!("Mouse wheel");

            match delta {
                winit::event::MouseScrollDelta::LineDelta(hor, ver) => {
                    INPUT_MANAGER.mouse_scroll(MouseId::from_winit(device), Vec2::new(hor, ver));
                }
                winit::event::MouseScrollDelta::PixelDelta(phys_pos) => {
                    const PIXELS_PER_LINE: f32 = 50.0;
                    const LINES_PER_PIXEL: f32 = 1.0 / PIXELS_PER_LINE;

                    let hor = (phys_pos.x as f32) * LINES_PER_PIXEL;
                    let ver = (phys_pos.y as f32) * LINES_PER_PIXEL;

                    INPUT_MANAGER.mouse_scroll(MouseId::from_winit(device), Vec2::new(hor, ver));
                }
            }
        }
        winit::event::DeviceEvent::Motion { .. } => {
            log::trace!("Ignoring unsupported device motion event");
        }
        winit::event::DeviceEvent::Button { button, state } => {
            profiling::scope!("Button");

            INPUT_MANAGER.mouse_button(MouseId::from_winit(device), button, state);
        }
        winit::event::DeviceEvent::Key(raw_key_event) => {
            profiling::scope!("Key");

            if let Ok(as_key) = keyboard::Key::try_from(raw_key_event.physical_key) {
                INPUT_MANAGER.keyboard_key(
                    KeyboardId::from_winit(device),
                    as_key,
                    raw_key_event.state,
                );
            }
        }
    }
}

/// Inserts a new raw [winit window event](winit::event::WindowEvent) for the given [Window]
/// into the input manager for the current frame. Returns whether the window event was an input-related event,
/// and has thus been handled
pub fn insert_raw_window_event(
    window: WindowIdentifier,
    event: &winit::event::WindowEvent,
) -> bool {
    profiling::function_scope!();

    match event {
        winit::event::WindowEvent::KeyboardInput {
            device_id,
            event,
            is_synthetic,
        } => {
            profiling::scope!("Keyboard input");

            if *is_synthetic {
                // Generated by winit. Report as handled, because it's still an input,
                // but don't actually do anything with it
                return true;
            }

            let keyboard_id = KeyboardId::from_winit(*device_id);

            if let Ok(as_key) = keyboard::Key::try_from(event.physical_key) {
                INPUT_MANAGER.keyboard_key(keyboard_id, as_key, event.state);
            }

            if let Some(logical_key) = keyboard::LogicalKey::try_from_winit(&event.logical_key) {
                INPUT_MANAGER.keyboard_logical_key(keyboard_id, logical_key, event.state);
            }

            if event.state.is_pressed()
                && let Some(text) = event
                    .text
                    .as_deref()
                    .or_else(|| event.logical_key.to_text())
                && !text.is_empty()
            {
                INPUT_MANAGER.keyboard_text(keyboard_id, text);
            }
        }
        winit::event::WindowEvent::ModifiersChanged(_) => {}
        winit::event::WindowEvent::Ime(_) => {}
        winit::event::WindowEvent::CursorMoved {
            device_id,
            position,
        } => {
            profiling::scope!("Cursor moved");

            INPUT_MANAGER.mouse_window_position(
                MouseId::from_winit(*device_id),
                Some((window, Vec2::new(position.x as f32, position.y as f32))),
            );
        }
        winit::event::WindowEvent::CursorEntered { .. } => {
            // Winit also sends a cursor-moved event, so we don't have to explicitely handle this
        }
        winit::event::WindowEvent::CursorLeft { device_id } => {
            profiling::scope!("Cursor left");

            INPUT_MANAGER.mouse_window_position(MouseId::from_winit(*device_id), None);
        }
        winit::event::WindowEvent::MouseWheel {
            device_id, delta, ..
        } => {
            profiling::scope!("MouseWheel");

            match *delta {
                winit::event::MouseScrollDelta::LineDelta(hor, ver) => {
                    INPUT_MANAGER
                        .mouse_scroll(MouseId::from_winit(*device_id), Vec2::new(hor, ver));
                }
                winit::event::MouseScrollDelta::PixelDelta(phys_pos) => {
                    const PIXELS_PER_LINE: f32 = 50.0;
                    const LINES_PER_PIXEL: f32 = 1.0 / PIXELS_PER_LINE;

                    let hor = (phys_pos.x as f32) * LINES_PER_PIXEL;
                    let ver = (phys_pos.y as f32) * LINES_PER_PIXEL;

                    INPUT_MANAGER
                        .mouse_scroll(MouseId::from_winit(*device_id), Vec2::new(hor, ver));
                }
            }
        }
        winit::event::WindowEvent::MouseInput {
            device_id,
            state,
            button,
        } => {
            profiling::scope!("MouseInput");

            INPUT_MANAGER.mouse_button(
                MouseId::from_winit(*device_id),
                logical_mouse_to_button_id(button),
                *state,
            );
        }
        winit::event::WindowEvent::PinchGesture { .. } => {}
        winit::event::WindowEvent::PanGesture { .. } => {}
        winit::event::WindowEvent::DoubleTapGesture { .. } => {}
        winit::event::WindowEvent::RotationGesture { .. } => {}
        winit::event::WindowEvent::TouchpadPressure { .. } => {}
        winit::event::WindowEvent::AxisMotion { .. } => {}
        winit::event::WindowEvent::Touch(_) => {}
        _ => return false,
    }

    log::trace!(
        "Handled window input event in frame {} on window {window}: {event:#?}",
        wutengine_time::frame_num()
    );

    true
}

/// Maps a logical winit mouse button to a raw id
const fn logical_mouse_to_button_id(logical: &winit::event::MouseButton) -> winit::event::ButtonId {
    match logical {
        winit::event::MouseButton::Left => mouse::BUTTON_LEFT,
        winit::event::MouseButton::Right => mouse::BUTTON_RIGHT,
        winit::event::MouseButton::Middle => mouse::BUTTON_MIDDLE,
        winit::event::MouseButton::Back => mouse::BUTTON_BACK,
        winit::event::MouseButton::Forward => mouse::BUTTON_FORWARD,
        winit::event::MouseButton::Other(other) => *other as winit::event::ButtonId,
    }
}

/// Resets all per-frame delta values back to zero.
/// Should be called by the engine runtime at the end of each rendered frame, so
/// all new input events count for the next frame
pub fn end_frame() {
    profiling::function_scope!();

    INPUT_MANAGER.end_frame();
}
