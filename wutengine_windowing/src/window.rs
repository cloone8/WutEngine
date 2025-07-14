use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use wutengine_util::GlobalManager;

use crate::WutEngineWinitEvent;
use crate::display::{DISPLAY_MANAGER, DisplayIdentifier, VideoMode};

pub(crate) static WINDOW_MANAGER: GlobalManager<WindowManager> = GlobalManager::new();

#[derive(Debug)]
pub(crate) struct WindowManager {
    event_loop: winit::event_loop::EventLoopProxy<WutEngineWinitEvent>,

    windows: Mutex<WindowMap>,
}

#[derive(Debug, Default)]
struct WindowMap {
    winit_id_map: HashMap<winit::window::WindowId, WindowIdentifier>,
    wutengine_id_map: HashMap<WindowIdentifier, usize>,
    windows: Vec<Arc<winit::window::Window>>,
}

impl WindowManager {
    pub(crate) fn new(event_loop: winit::event_loop::EventLoopProxy<WutEngineWinitEvent>) -> Self {
        Self {
            event_loop,
            windows: Mutex::new(WindowMap::default()),
        }
    }

    fn register_window(&self, identifier: WindowIdentifier, window: Arc<winit::window::Window>) {
        let mut locked = self.windows.lock().unwrap();

        if locked.wutengine_id_map.contains_key(&identifier) {
            log::error!("Window with ID {identifier} already exists. Not creating new one");
            return;
        }

        let index = locked.windows.len();
        let winit_id = window.id();

        locked.windows.push(window);
        locked.winit_id_map.insert(winit_id, identifier.clone());
        locked.wutengine_id_map.insert(identifier, index);
    }
}

pub fn register_window(identifier: WindowIdentifier, window: Arc<winit::window::Window>) {
    WINDOW_MANAGER.register_window(identifier, window);
}

#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub title: String,
    pub mode: WindowMode,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "WutEngine".to_string(),
            mode: WindowMode::Windowed,
        }
    }
}

#[derive(Debug, Error)]
pub enum CreateWindowError {
    #[error("Display {0} does not support video mode {1}")]
    UnsupportedVideoMode(DisplayIdentifier, VideoMode),
}

impl TryFrom<WindowOptions> for winit::window::WindowAttributes {
    type Error = Box<CreateWindowError>;
    fn try_from(value: WindowOptions) -> Result<Self, Self::Error> {
        use winit::window::Fullscreen;

        let fullscreen_mode = match value.mode {
            WindowMode::Windowed => None,
            WindowMode::BorderlessFullscreen(display_identifier) => {
                Some(Fullscreen::Borderless(display_identifier.map(|di| di.0)))
            }
            WindowMode::ExclusiveFullscreen(display_identifier, video_mode) => {
                let display = DISPLAY_MANAGER
                    .get_display(&display_identifier)
                    .expect("Unknown display");
                let mode_handle = display.get_mode_handle(video_mode).ok_or(
                    CreateWindowError::UnsupportedVideoMode(display_identifier, video_mode),
                )?;

                Some(Fullscreen::Exclusive(mode_handle))
            }
        };

        Ok(Self::default()
            .with_fullscreen(fullscreen_mode)
            .with_title(value.title)
            .with_min_inner_size(winit::dpi::PhysicalSize::new(1, 1)))
    }
}

pub fn create(id: WindowIdentifier, options: WindowOptions) {
    log::debug!("Requesting creation of new window with ID {id} and options {options:#?}");

    let attributes = match winit::window::WindowAttributes::try_from(options) {
        Ok(attrs) => attrs,
        Err(e) => {
            log::error!(
                "Could not open window with identifier {id} because the given options were invalid: {e}"
            );
            return;
        }
    };

    WINDOW_MANAGER
        .event_loop
        .send_event(WutEngineWinitEvent::CreateWindowRequested(id, attributes))
        .unwrap();
}

pub fn identifier_for_native_id(id: &winit::window::WindowId) -> Option<WindowIdentifier> {
    WINDOW_MANAGER
        .windows
        .lock()
        .unwrap()
        .winit_id_map
        .get(id)
        .cloned()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowIdentifier(pub(crate) String);

impl WindowIdentifier {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl core::fmt::Display for WindowIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum WindowMode {
    Windowed,
    BorderlessFullscreen(Option<DisplayIdentifier>),
    ExclusiveFullscreen(DisplayIdentifier, VideoMode),
}
