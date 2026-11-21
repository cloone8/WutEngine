//! Window management

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use winit::dpi::Size;
use wutengine_sync::Mutex;
use wutengine_sync::RwLock;

use winit::window::Window as WinitWindow;
use winit::window::WindowAttributes;
use wutengine_macro::unique_id_type32;

use crate::log;

mod size;

pub use size::*;

unique_id_type32! {
    /// Handle to a WutEngine window
    pub Window
}

impl Window {
    /// Sends a request to create a new window. The handle is returned immediately,
    /// and the window itself is created as soon as possible on the main thread
    #[must_use]
    pub fn create(settings: WindowSettings) -> Self {
        let handle = Window::new();

        log::info!("Creating new window with handle {handle}");

        let mut window_map = window_manager().windows.write();

        if window_map.is_empty() {
            log::debug!("Setting main window to {handle}");
            // Set the main window if this is the first window to be created
            *window_manager().main_window.lock() = Some(handle);
        }

        window_map.insert(
            handle,
            WindowData::Pending {
                settings: Box::new(settings),
            },
        );

        handle
    }

    /// Returns `true` if this window is the main window
    #[must_use]
    pub fn is_main(self) -> bool {
        window_manager()
            .main_window
            .lock()
            .is_some_and(|main_window| main_window == self)
    }

    /// Returns the logical size of the window.
    /// To get the size in pixels, see [`Self::pixels`]
    #[must_use]
    pub fn size(self) -> LogicalSize {
        self.with_data(|data| match data {
            WindowData::Pending { settings } => settings.size,
            WindowData::Open { state, .. } => state.size,
            WindowData::Closed(window_snapshot) => window_snapshot.size,
        })
    }

    /// Returns the physical size (in pixels) of the window. If the window has not yet been created
    /// by the OS, the scale factor is not yet known, so the size in pixels might be off.
    #[must_use]
    pub fn pixels(self) -> PixelSize {
        self.with_data(|data| match data {
            WindowData::Pending { settings } => {
                PixelSize::from((settings.size.width as u32, settings.size.height as u32))
            }
            WindowData::Open { state, .. } => state.pixels,
            WindowData::Closed(window_snapshot) => window_snapshot.pixels,
        })
    }

    /// Returns the OS scale factor for this window
    #[must_use]
    pub fn scale_factor(self) -> f64 {
        self.with_data(|data| match data {
            WindowData::Pending { .. } => 1.0, // Can only guess
            WindowData::Open { state, .. } => state.scale_factor,
            WindowData::Closed(window_snapshot) => window_snapshot.scale_factor,
        })
    }
}

impl Window {
    #[inline]
    fn with_data<T>(self, cb: impl FnOnce(&WindowData) -> T) -> T {
        let windows = window_manager().windows.read();

        let data = windows
            .get(&self)
            .expect("Window should have been registered");

        cb(data)
    }
}

#[derive(Debug)]
struct WindowManager {
    /// Which window is the main window. Should only be [`None`] if the [`Self::windows`] map is
    /// empty.
    ///
    /// TODO: Change to an atomic type later.
    main_window: Mutex<Option<Window>>,

    /// All currently active windows
    windows: RwLock<HashMap<Window, WindowData>>,
}

/// Used when opening a window with [`create`]
#[derive(Debug, Clone)]
pub struct WindowSettings {
    /// The window title
    pub title: String,

    /// The requested window size, in logical points. The
    /// actual resolution might be different if the OS scale factor is not 1.0
    pub size: LogicalSize,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            title: "WutEngine".to_string(),
            size: (1280.0, 720.0).into(),
        }
    }
}

impl From<&WindowSettings> for WindowAttributes {
    fn from(value: &WindowSettings) -> Self {
        let mut attrs = WindowAttributes::default();

        attrs.surface_size = Some(Size::Logical(value.size.into()));
        attrs.title.clone_from(&value.title);

        attrs
    }
}

#[derive(Debug)]
enum WindowData {
    Pending {
        settings: Box<WindowSettings>,
    },
    Open {
        native: Arc<dyn WinitWindow>,
        state: Box<WindowSnapshot>,
    },
    Closed(Box<WindowSnapshot>),
}

#[derive(Debug, Clone, Copy)]
struct WindowSnapshot {
    pixels: PixelSize,
    size: LogicalSize,
    scale_factor: f64,
}

impl WindowSnapshot {
    fn create(native: &dyn WinitWindow) -> Self {
        Self {
            pixels: native.surface_size().into(),
            scale_factor: native.scale_factor(),
            size: native
                .surface_size()
                .to_logical(native.scale_factor())
                .into(),
        }
    }
}

static WINDOW_MANAGER: OnceLock<WindowManager> = OnceLock::new();

/// Returns the [`WindowManager`]
///
/// Panics if not yet initialized
fn window_manager() -> &'static WindowManager {
    WINDOW_MANAGER
        .get()
        .expect("Window manager not yet initialized")
}

/// Initializes the windowing subsystem
pub(crate) fn init() {
    profiling::function_scope!();

    log::debug!("Initializing window manager");

    WINDOW_MANAGER
        .set(WindowManager {
            main_window: Mutex::new(None),
            windows: RwLock::new(HashMap::new()),
        })
        .expect("Window manager already initialized");
}

/// Runs the queued window actions. This actually spawns native windows.
pub(crate) fn initialize_windows(
    mut initializer: impl FnMut(Window, &WindowSettings) -> Arc<dyn WinitWindow>,
) {
    profiling::function_scope!();

    for (window, data) in window_manager().windows.write().iter_mut() {
        let WindowData::Pending { settings } = data else {
            continue;
        };

        profiling::scope!("initialize_window");

        let native_window = initializer(*window, settings.as_ref());

        *data = WindowData::Open {
            state: Box::new(WindowSnapshot::create(native_window.as_ref())),
            native: native_window,
        };
    }
}

pub(crate) fn refresh_window_data(window: Window) {
    profiling::function_scope!();
    log::trace!("Refreshing window data for window {window}");

    let mut windows = window_manager().windows.write();
    let Some(window_data) = windows.get_mut(&window) else {
        log::error!("Cannot refresh data for unknown window {window}");
        return;
    };

    let WindowData::Open { native, state } = window_data else {
        log::error!("Cannot refresh data for non-open window {window}");
        return;
    };

    *state.as_mut() = WindowSnapshot::create(native.as_ref());
}
