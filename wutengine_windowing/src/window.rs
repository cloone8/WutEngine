use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wutengine_event::WutEngineEvent;
use wutengine_util::GlobalManager;

use crate::WutEngineWinitEvent;
use crate::display::{DISPLAY_MANAGER, DisplayIdentifier, VideoMode};

pub(crate) static WINDOW_MANAGER: GlobalManager<WindowManager> = GlobalManager::new();

#[derive(Debug)]
pub(crate) struct WindowManager {
    event_loop: winit::event_loop::EventLoopProxy<WutEngineWinitEvent>,

    windows: RwLock<WindowMap>,
}

#[derive(Debug, Default)]
struct WindowMap {
    winit_id_map: HashMap<winit::window::WindowId, WindowIdentifier>,
    windows: HashMap<WindowIdentifier, WindowData>,
}

#[derive(Debug)]
struct WindowData {
    native: Arc<winit::window::Window>,
    inner_size: (u32, u32),
}

impl WindowData {
    fn new(native: Arc<winit::window::Window>) -> Self {
        let size = native.inner_size();
        Self {
            native,
            inner_size: size.into(),
        }
    }
}

impl WindowManager {
    pub(crate) fn new(event_loop: winit::event_loop::EventLoopProxy<WutEngineWinitEvent>) -> Self {
        Self {
            event_loop,
            windows: RwLock::new(WindowMap::default()),
        }
    }

    fn register_window(&self, identifier: WindowIdentifier, window: Arc<winit::window::Window>) {
        let mut window_map = self.windows.write().unwrap();

        if window_map.windows.contains_key(&identifier) {
            log::error!("Window with ID {identifier} already exists. Not creating new one");
            return;
        }

        let winit_id = window.id();

        window_map.winit_id_map.insert(winit_id, identifier.clone());
        window_map
            .windows
            .insert(identifier, WindowData::new(window));
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
        .read()
        .unwrap()
        .winit_id_map
        .get(id)
        .cloned()
}

pub fn lock_windows<F>(cb: F)
where
    F: for<'a> FnOnce(Vec<(&'a WindowIdentifier, &'a winit::window::Window)>),
{
    let locked = WINDOW_MANAGER.windows.read().unwrap();

    let as_vec = locked
        .windows
        .iter()
        .map(|(id, index)| (id, index.native.as_ref()))
        .collect();

    cb(as_vec);
}

pub fn window_size(id: &WindowIdentifier) -> Option<(u32, u32)> {
    WINDOW_MANAGER
        .windows
        .read()
        .unwrap()
        .windows
        .get(id)
        .map(|data| data.inner_size)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WindowIdentifier(pub(crate) String);

impl WindowIdentifier {
    pub fn new(s: impl ToString) -> Self {
        Self(s.to_string())
    }
}

impl core::fmt::Display for WindowIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Window(\"{}\")", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum WindowMode {
    Windowed,
    BorderlessFullscreen(Option<DisplayIdentifier>),
    ExclusiveFullscreen(DisplayIdentifier, VideoMode),
}

#[derive(Debug)]
pub struct WindowResizedEvent {
    pub window_id: WindowIdentifier,
    pub new_size: (u32, u32),
}

impl WutEngineEvent for WindowResizedEvent {}
