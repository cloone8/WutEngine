//! Window creation and management

use winit::window::WindowAttributes;
use wutengine_util_macro::unique_id_type32;

use crate::runtime;

mod icon;
pub use icon::*;

pub(crate) mod manager;
pub(crate) mod pacer;

unique_id_type32! {
    /// The handle to a WutEngine window
    pub Window
}

/// Config used to create a new window with [Window::create]
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// The title of the window.
    /// If set to [None], will use a generic title instead
    pub title: Option<String>,

    /// The inner size of the window. Defaults to 720p
    pub size: (u32, u32),

    /// The minimum inner size of the window. Must be at least 1x1
    pub min_size: (u32, u32),

    /// The maximum inner size of the window. Defaults to [None]. If given, must be larger or equal to
    /// [Self::min_size]
    pub max_size: Option<(u32, u32)>,

    /// Whether the window is resizable. Defaults to true
    pub resizable: bool,

    /// The icon. Defaults to [None]
    pub icon: Option<Icon>,

    /// Whether vsync is enabled. Defaults to [None], which picks whatever is configured through the config manager
    pub vsync: Option<bool>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: None,
            size: (1280, 720),
            min_size: (1, 1),
            max_size: None,
            resizable: true,
            icon: None,
            vsync: None,
        }
    }
}

fn clamp_min_size(min_size: (u32, u32)) -> (u32, u32) {
    let clamped = (min_size.0.max(1), min_size.1.max(1));

    if clamped != min_size {
        log::error!(
            "Invalid minimum window size given: ({}, {}). Both axes must be at least 1 pixel",
            min_size.0,
            min_size.1
        );
    }

    clamped
}

fn clamp_max_size(min_size: (u32, u32), max_size: (u32, u32)) -> (u32, u32) {
    let clamped = (max_size.0.max(min_size.0), max_size.1.max(min_size.1));

    if clamped != max_size {
        log::error!(
            "Invalid maximum window size given: ({}, {}). Both axes must be larger or equal to the minimum size: ({}, {})",
            max_size.0,
            max_size.1,
            min_size.0,
            min_size.1
        );
    }

    clamped
}

impl From<WindowConfig> for winit::window::WindowAttributes {
    fn from(value: WindowConfig) -> Self {
        let mut attrs = WindowAttributes::default();

        if let Some(title) = value.title {
            attrs = attrs.with_title(title);
        };

        let mut inner_size = value.size;

        let min_size_clamped = clamp_min_size(value.min_size);

        inner_size = (
            inner_size.0.max(min_size_clamped.0),
            inner_size.1.max(min_size_clamped.1),
        );

        attrs = attrs.with_min_inner_size(winit::dpi::PhysicalSize::new(
            min_size_clamped.0,
            min_size_clamped.1,
        ));

        if let Some(max_size) = value.max_size {
            let max_size_clamped = clamp_max_size(min_size_clamped, max_size);

            attrs = attrs.with_max_inner_size(winit::dpi::PhysicalSize::new(
                max_size_clamped.0,
                max_size_clamped.1,
            ));

            inner_size = (
                inner_size.0.min(max_size_clamped.0),
                inner_size.1.min(max_size_clamped.1),
            );
        };

        attrs = attrs.with_resizable(value.resizable);
        attrs = attrs.with_inner_size(winit::dpi::PhysicalSize::new(inner_size.0, inner_size.1));
        attrs = attrs.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

        if let Some(icon) = value.icon
            && let Some(native_icon) = icon.into_native_icon()
        {
            #[cfg(windows)]
            {
                use winit::platform::windows::WindowAttributesExtWindows;

                attrs = attrs.with_taskbar_icon(Some(native_icon.clone()));
            }

            attrs = attrs.with_window_icon(Some(native_icon));
        }

        attrs
    }
}

/// Proxy APIs for usability purposes
impl Window {
    /// Creates a new window with the given configuration
    pub fn create(config: WindowConfig) -> Self {
        let mut config = config;
        let id = Window::new();

        if config.title.is_none() {
            config.title = Some(format!("Window {}", id.0));
        }

        log::info!(
            "Creating new window with ID {id} and title: {}",
            config.title.as_ref().unwrap()
        );

        runtime::notify_event_loop(runtime::WinitEvent::NewWindowRequested(id, config));

        id
    }

    /// Closes the window with the given ID, if it is not already closed
    pub fn destroy(self) {
        let window = self;
        log::info!("Closing window with ID {window}");

        runtime::notify_event_loop(runtime::WinitEvent::CloseWindow(window));
    }

    /// Updates the icon of the given window to the provided one
    pub fn set_icon(self, icon: Icon) {
        let window = self;
        log::trace!("Updating icon for window {window}");

        if let Some(native_icon) = icon.into_native_icon() {
            runtime::notify_event_loop(runtime::WinitEvent::UpdateIcon(window, native_icon));
        }
    }

    /// Returns the size of this window in pixels.
    /// If the window is not yet created or is already destroyed, returns (0,0)
    #[inline]
    pub fn get_size(self) -> (u32, u32) {
        crate::window::manager::get_size(self).unwrap_or((0, 0))
    }
}
