//! Window creation and management

use core::num::NonZeroU32;

use cursor_icon::CursorIcon;
use manager::display_info::DisplayExclusiveFullscreenMode;
use winit::window::WindowAttributes;
use wutengine_util_macro::{VariantName, unique_id_type32};

use crate::{graphics, runtime};

mod icon;
pub use icon::*;

mod display;
pub use display::*;

pub mod cursor;
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

    /// Fullscreen mode. If [None], is windowed
    pub fullscreen: Option<FullscreenMode>,
}

/// Fullscreen window configuration
#[derive(Debug, Clone)]
pub enum FullscreenMode {
    /// Borderless fullscreen. Preferred fullscreen method on most systems (even for games on all
    /// supported Windows versions), so use this
    /// unless you have a very specific reason not to
    Borderless(BorderlessTarget),

    /// Exclusive fullscreen mode. Not supported everywhere
    Exclusive(DisplayExclusiveFullscreenMode),
}

/// Target display for [FullscreenMode::Borderless]
#[derive(Debug, Clone, Copy)]
pub enum BorderlessTarget {
    /// The display the window is currently (or initially) on
    Current,

    /// The primary display
    Primary,

    /// A specific display
    Specific(Display),
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: None,
            size: (1280, 720),
            min_size: (1, 1),
            max_size: None,
            resizable: true,
            icon: Some(Icon::Bytes(DEFAULT_ICON.to_vec())),
            vsync: None,
            fullscreen: Some(FullscreenMode::Borderless(BorderlessTarget::Primary)),
        }
    }
}

/// An event sent to the main thread that updates a window
#[derive(Debug, VariantName)]
pub(crate) enum WindowUpdateEvent {
    /// Update the icon
    UpdateIcon(winit::window::Icon),

    /// Update the title string
    UpdateTitle(String),

    /// Update the cursor
    UpdateCursor(CursorIcon),

    /// Sets the cursor to visible (true) or invisible (false)
    CursorVisibility(bool),
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

        if let Some(mut fullscreen_mode) = value.fullscreen {
            let (is_set, new_attrs) = try_set_exclusive_fullscreen(attrs, &mut fullscreen_mode);
            attrs = new_attrs;

            if !is_set {
                let (_, new_attrs) = try_set_borderless_fullscreen(attrs, &fullscreen_mode);
                attrs = new_attrs;
            }
        }

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

fn try_set_exclusive_fullscreen(
    attrs: winit::window::WindowAttributes,
    fullscreen_mode: &mut FullscreenMode,
) -> (bool, winit::window::WindowAttributes) {
    let FullscreenMode::Exclusive(mode) = fullscreen_mode else {
        return (false, attrs);
    };

    let backend = graphics::active_config().backend;

    if !backend.supports_exclusive_fullscreen() {
        log::error!(
            "Graphics backend {backend} does not support exclusive fullscreen mode. Falling back to borderless",
        );
        *fullscreen_mode = FullscreenMode::Borderless(BorderlessTarget::Specific(mode.0));
        return (false, attrs);
    }

    // Check is not strictly needed, but not a bad idea anyway
    if crate::window::manager::monitor_handle_from_display(mode.0).is_some() {
        (
            true,
            attrs.with_fullscreen(Some(winit::window::Fullscreen::Exclusive(mode.1.clone()))),
        )
    } else {
        log::error!(
            "Target display {} does not exist anymore. Falling back to borderless on primary",
            mode.0
        );
        *fullscreen_mode = FullscreenMode::Borderless(BorderlessTarget::Primary);
        (false, attrs)
    }
}

fn try_set_borderless_fullscreen(
    attrs: winit::window::WindowAttributes,
    fullscreen_mode: &FullscreenMode,
) -> (bool, winit::window::WindowAttributes) {
    let FullscreenMode::Borderless(borderless_target) = fullscreen_mode else {
        return (false, attrs);
    };

    let display_handle = match borderless_target {
        BorderlessTarget::Current => None,
        BorderlessTarget::Primary => match crate::window::manager::primary_display() {
            Some(disp) => Some(disp),
            None => {
                log::error!(
                    "Failed to determine primary display. Falling back to borderless mode on current display"
                );
                None
            }
        },
        BorderlessTarget::Specific(display) => Some(*display),
    };

    let target_handle = match display_handle {
        Some(disp) => match crate::window::manager::monitor_handle_from_display(disp) {
            Some(handle) => Some(handle),
            None => {
                log::error!(
                    "Target display {disp} does not exist anymore. Falling back to borderless mode on current display"
                );
                None
            }
        },
        None => None,
    };

    #[allow(unused_mut, reason = "Is mutated depending on platform")]
    let mut attrs =
        attrs.with_fullscreen(Some(winit::window::Fullscreen::Borderless(target_handle)));

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        use winit::platform::macos::WindowAttributesExtMacOS;

        attrs = attrs.with_borderless_game(true);
    }

    (true, attrs)
}

impl From<Window> for wutengine_input::WindowIdentifier {
    #[inline]
    fn from(value: Window) -> Self {
        Self::new(u64::from(value.0.get()))
    }
}

impl TryFrom<wutengine_input::WindowIdentifier> for Window {
    type Error = u64;

    #[inline]
    fn try_from(value: wutengine_input::WindowIdentifier) -> Result<Self, Self::Error> {
        match u32::try_from(value.raw()) {
            Ok(as_32) => match NonZeroU32::new(as_32) {
                Some(nonzero) => Ok(Self(nonzero)),
                None => Err(value.raw()),
            },
            Err(_) => Err(value.raw()),
        }
    }
}

/// Proxy APIs for usability purposes
impl Window {
    /// Returns the primary window, if any
    #[inline]
    pub fn primary() -> Option<Self> {
        crate::window::manager::get_windows_and(|windows| {
            windows
                .iter()
                .find(|(_, info)| info.is_primary)
                .map(|(win, _)| *win)
        })
    }

    /// Returns the opened windows, if any
    #[inline]
    pub fn opened() -> Vec<Self> {
        crate::window::manager::get_windows_and(|windows| windows.keys().copied().collect())
    }

    /// Creates a new window with the given configuration
    #[inline]
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

        runtime::send_to_main_thread(runtime::MainThreadEvent::NewWindowRequested(id, config));

        id
    }

    /// Closes the window with the given ID, if it is not already closed
    #[inline]
    pub fn destroy(self) {
        let window = self;
        log::info!("Closing window with ID {window}");

        runtime::send_to_main_thread(runtime::MainThreadEvent::CloseWindow(window));
    }

    /// Updates the icon of the given window to the provided one
    #[inline]
    pub fn set_icon(self, icon: Icon) {
        let window = self;
        log::trace!("Updating icon for window {window}");

        if let Some(native_icon) = icon.into_native_icon() {
            runtime::send_to_main_thread(runtime::MainThreadEvent::UpdateWindow(
                window,
                WindowUpdateEvent::UpdateIcon(native_icon),
            ));
        }
    }

    /// Sets the cursor icon for this window
    #[inline]
    pub fn set_cursor(self, cursor: cursor::CursorIcon) {
        runtime::send_to_main_thread(runtime::MainThreadEvent::UpdateWindow(
            self,
            WindowUpdateEvent::UpdateCursor(cursor),
        ));
    }

    /// Sets the visibility of the cursor
    #[inline]
    pub fn set_cursor_visible(self, visible: bool) {
        runtime::send_to_main_thread(runtime::MainThreadEvent::UpdateWindow(
            self,
            WindowUpdateEvent::CursorVisibility(visible),
        ));
    }

    /// Hides the cursor
    #[inline(always)]
    pub fn hide_cursor(self) {
        self.set_cursor_visible(false);
    }

    /// Shows the cursor
    #[inline(always)]
    pub fn show_cursor(self) {
        self.set_cursor_visible(true);
    }

    /// Returns whether this window is the primary window
    #[inline]
    pub fn is_primary(self) -> bool {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.is_primary).unwrap_or(false)
        })
    }

    /// Appoints this window as the "primary" window
    #[inline]
    pub fn make_primary(self) {
        crate::window::manager::appoint_primary_window(self);
    }

    /// Checks whether this window has been opened by the native windowing system, and
    /// has also not yet been destroyed
    #[inline]
    pub fn is_ready(self) -> bool {
        crate::window::manager::get_window_and(self, |win| win.is_some())
    }

    /// Returns the size of this window in pixels.
    /// If the window is not yet created or is already destroyed, returns (0,0)
    #[inline]
    pub fn get_size(self) -> (u32, u32) {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.inner_size).unwrap_or((0, 0))
        })
    }

    /// Returns the OS scale factor of this window.
    /// If the window is not yet created or is already destroyed, 1.0
    #[inline]
    pub fn get_scale_factor(self) -> f64 {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.os_scale_factor).unwrap_or(1.0)
        })
    }

    /// Returns whether this window is currently focused
    #[inline]
    pub fn is_focused(self) -> bool {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.focused).unwrap_or(true)
        })
    }

    /// Returns whether this window is currently fully occluded.
    /// Not supported by every platform, in which case this will always
    /// return `false`
    #[inline]
    pub fn is_occluded(self) -> bool {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.occluded).unwrap_or(false)
        })
    }

    /// Returns whether this window is currently (known to be) minimized
    #[inline]
    pub fn is_minimized(self) -> bool {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.minimized).unwrap_or(false)
        })
    }

    /// Returns whether this window is currently maximized
    #[inline]
    pub fn is_maximized(self) -> bool {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.maximized).unwrap_or(false)
        })
    }

    /// Returns whether this window is currently in the foreground.
    ///
    /// Shorthand for `!self.is_occluded() && !self.is_minimized() && self.is_focused()`
    #[inline]
    pub fn is_foreground(self) -> bool {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| !win.occluded && win.focused && !win.minimized)
                .unwrap_or(true)
        })
    }

    /// Forces reconfiguration of the window surface. Should usually not be called,
    /// except when vsync or frame settings have recently been manually changed
    #[inline]
    pub fn force_reconfigure(self) {
        log::debug!("Forcing window {self} reconfiguration");

        crate::runtime::send_to_main_thread(runtime::MainThreadEvent::ForceSurfaceReconfigure(
            self,
        ));
    }

    /// Returns the current title of this window
    #[inline]
    pub fn title(self) -> String {
        crate::window::manager::get_window_and(self, |win| {
            win.map(|win| win.title.clone()).unwrap_or(String::new())
        })
    }

    /// Updates the title of the window
    #[inline]
    pub fn set_title(self, title: impl Into<String>) {
        crate::runtime::send_to_main_thread(runtime::MainThreadEvent::UpdateWindow(
            self,
            WindowUpdateEvent::UpdateTitle(title.into()),
        ));
    }
}
