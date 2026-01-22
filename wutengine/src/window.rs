//! Window creation and management

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[repr(transparent)]
// pub struct WindowId(u32);

use std::path::PathBuf;

use image::GenericImageView;
use nohash_hasher::IntMap;
use winit::window::WindowAttributes;
use wutengine_util_macro::unique_id_type32;

use crate::{assert_main_thread, runtime};

unique_id_type32! {
    /// The unique ID for a single window
    pub WindowId
}

/// Config used to create a new window with [create]
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

/// Creates a new window with the given configuration
pub fn create(mut config: WindowConfig) -> WindowId {
    let id = WindowId::new();

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
pub fn destroy(window: WindowId) {
    log::info!("Closing window with ID {window}");

    runtime::notify_event_loop(runtime::WinitEvent::CloseWindow(window));
}

/// Updates the icon of the given window to the provided one
pub fn set_icon(window: WindowId, icon: Icon) {
    log::trace!("Updating icon for window {window}");

    if let Some(native_icon) = icon.into_native_icon() {
        runtime::notify_event_loop(runtime::WinitEvent::UpdateIcon(window, native_icon));
    }
}

/// The WutEngine window manager. Owns all native windows, and allows interaction with them
#[derive(Debug)]
pub(crate) struct WindowManager {
    /// Mapping from [winit] window IDs to WutEngine window IDs.
    /// use the [Into::into] implementation on [winit::window::WindowId] to convert
    /// it to a [u64] for use in this map
    winit_to_engine: IntMap<u64, crate::window::WindowId>,

    windows: IntMap<crate::window::WindowId, WindowInfo>,
}

#[derive(Debug)]
struct WindowInfo {
    /// The engine-internal ID
    id: WindowId,

    /// The actual native window handle
    native: winit::window::Window,

    /// The physical window size, in pixels `W x H`
    inner_size: (u32, u32),
}

impl WindowInfo {
    /// Creates a new [WindowInfo] struct, containing cached
    /// window information to prevent problems with querying information
    /// on non-main threads
    pub(crate) fn new(id: WindowId, native: winit::window::Window) -> Self {
        let mut new = Self {
            id,
            native,
            inner_size: (0, 0),
        };

        new.refresh();

        new
    }

    /// Refresh all cached window information
    fn refresh(&mut self) {
        assert_main_thread!("WindowInfo::refresh");

        log::trace!("Refreshing cached information for window {}", self.id);

        self.inner_size = self.native.inner_size().into();
    }
}

impl WindowManager {
    /// Creates a new default [WindowManager]
    pub(crate) fn new() -> Self {
        Self {
            winit_to_engine: IntMap::default(),
            windows: IntMap::default(),
        }
    }

    /// Registers a newly created window with the [WindowManager]
    ///
    /// Must be called from the main thread
    pub(crate) fn new_window(
        &mut self,
        id: crate::window::WindowId,
        window: winit::window::Window,
    ) {
        assert_main_thread!("WindowManager::new_window");

        let native_id = window.id();
        let info = WindowInfo::new(id, window);

        self.winit_to_engine.insert(native_id.into(), id);
        self.windows.insert(id, info);
    }

    /// Instructs the [WindowManager] to destroy the given window.
    ///
    /// If the window does not exist, does nothing.
    ///
    /// Must be called from the main thread
    pub(crate) fn close_window(&mut self, id: crate::window::WindowId) {
        assert_main_thread!("WindowManager::close_window");

        let Some(info) = self.windows.remove(&id) else {
            log::warn!("Window {id} was already closed");
            return;
        };

        let removed = self.winit_to_engine.remove(&info.native.id().into());
        assert!(removed.is_some(), "Native to engine ID map was missing");
    }

    /// Updates the icon of the given window to the new one.
    ///
    /// If the window does not exist, does nothing.
    ///
    /// Must be called from the main thread
    pub(crate) fn set_icon(&mut self, id: crate::window::WindowId, icon: winit::window::Icon) {
        let Some(window) = self.windows.get_mut(&id) else {
            log::error!("Could not update icon for window {id} because it could not be found");
            return;
        };

        #[cfg(windows)]
        {
            use winit::platform::windows::WindowExtWindows;

            window.native.set_taskbar_icon(Some(icon.clone()));
        }

        window.native.set_window_icon(Some(icon));
    }

    /// Refreshes the cached info for a given [WindowId]. Should be called by the main
    /// engine runtime when a relevant [winit] event is received, or if some other window
    /// attribute was changed.
    ///
    /// Must be called from the main thread
    pub(crate) fn refresh_cached_info(&mut self, id: &crate::window::WindowId) {
        assert_main_thread!("WindowManager::refresh_cached_info");

        log::trace!("Refreshing cached window info for {id}");

        let Some(window) = self.windows.get_mut(id) else {
            log::error!(
                "Could not refresh cached info for window {id} because it could not be found"
            );
            return;
        };

        window.refresh();
    }

    /// For a given [winit::window::WindowId], returns the WutEngine [WindowId] if one can be found
    pub(crate) fn find_id(&self, native_id: winit::window::WindowId) -> Option<WindowId> {
        self.winit_to_engine.get(&native_id.into()).copied()
    }
}

/// A window icon. The icon dimensions must be equal on both sides, and a power of two
#[derive(Debug, Clone)]
pub enum Icon {
    /// The icon will be set from the given file
    File(PathBuf),

    /// The icon will be set from the given bytes
    /// The bytes will be decoded as if they came straight from an image file
    Bytes(Vec<u8>),
}

impl Icon {
    /// Converts this user-provided icon into a native icon. If this fails, logs
    /// the error and returns [None]
    pub(crate) fn into_native_icon(self) -> Option<winit::window::Icon> {
        let image = match self {
            Self::File(path) => match image::open(path) {
                Ok(img) => img,
                Err(e) => {
                    log::error!("Failed to read icon as an image due to error: {e}");
                    return None;
                }
            },
            Self::Bytes(data) => match image::load_from_memory(&data) {
                Ok(img) => img,
                Err(e) => {
                    log::error!("Failed to read icon bytes as an image due to error: {e}");
                    return None;
                }
            },
        };

        let dims = image.dimensions();

        if dims.0 != dims.1 || !dims.0.is_power_of_two() {
            log::error!(
                "Invalid icon dimensions: ({}, {}). Width/height must be equal and a power of two.",
                dims.0,
                dims.1
            );
            return None;
        }

        let as_rgba = image.into_rgba8().into_raw();

        match winit::window::Icon::from_rgba(as_rgba, dims.0, dims.0) {
            Ok(icon) => Some(icon),
            Err(e) => {
                log::error!("Failed to convert image to a native icon: {e}");
                None
            }
        }
    }
}
