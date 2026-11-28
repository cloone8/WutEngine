//! Window management

#![cfg_attr(
    not(feature = "interactive"),
    allow(
        unused,
        reason = "No windows in headless. Blanket allowing unused code for headless builds is cleaner than sprinkling attributes everywhere"
    )
)]

use core::error::Error;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use wutengine_macro::unique_id_type32;
use wutengine_sync::Mutex;
use wutengine_sync::RwLock;
use wutengine_sync::thread_context;

use crate::log;
use crate::runtime;

mod size;

pub use size::*;

unique_id_type32! {
    /// Handle to a WutEngine window
    pub Window
}

/// The current state of a [`Window`]
#[derive(Debug, Clone)]
pub enum WindowState {
    /// Window is queued for creation, but not actually created yet
    Pending,

    /// Window is open. Normal state
    Open,

    /// Window has been requested to close
    Closing,

    /// Window is closed
    Closed,

    /// Window creation failed. See contained error for reason
    Error(Arc<dyn Error + Send + Sync>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display, derive_more::Error)]
#[display("Headless runs do not support window creation")]
struct HeadlessErr;

impl Window {
    /// Sends a request to create a new window. The handle is returned immediately,
    /// and the window itself is created as soon as possible on the main thread
    #[must_use]
    pub fn create(settings: WindowSettings) -> Self {
        let handle = Window::new();

        log::info!("Creating new window with handle {handle}");

        let mut window_map = window_manager().windows.write();

        if runtime::is_headless() {
            window_map.insert(
                handle,
                WindowData::Failed {
                    snapshot: Box::new(WindowSnapshot::guess_from_settings(&settings)),
                    err: Arc::new(HeadlessErr),
                },
            );
        } else {
            window_map.insert(
                handle,
                WindowData::Pending {
                    settings: Box::new(settings),
                    close_requested: AtomicBool::new(false),
                },
            );
        }

        handle
    }

    /// Gets the current state of the window
    #[must_use]
    pub fn state(self) -> WindowState {
        self.with_data(|data| match data {
            WindowData::Pending {
                close_requested, ..
            } => {
                if close_requested.load(Ordering::Relaxed) {
                    WindowState::Closing
                } else {
                    WindowState::Pending
                }
            }
            WindowData::Open {
                close_requested, ..
            } => {
                if close_requested.load(Ordering::Relaxed) {
                    WindowState::Closing
                } else {
                    WindowState::Open
                }
            }
            WindowData::Closed(_) => WindowState::Closed,
            WindowData::Failed { err, .. } => WindowState::Error(Arc::clone(err)),
        })
    }

    /// Sends a request to close this window as soon as possible.
    /// If the window is the main window, also closes the game.
    pub fn close(self) {
        if self.is_main() {
            log::info!("Closing main window {self}");
            runtime::quit();
            return;
        }

        log::info!("Closing window {self}");

        let window_map = window_manager().windows.read();

        let window = window_map.get(&self).expect("Missing window");

        match window {
            WindowData::Pending {
                close_requested, ..
            } => close_requested.store(true, Ordering::Relaxed),
            WindowData::Open {
                close_requested, ..
            } => close_requested.store(true, Ordering::Relaxed),
            WindowData::Closed(_) | WindowData::Failed { .. } => {
                // Nothing to do
            }
        }
    }

    /// Returns a handle to the main window
    #[must_use]
    #[inline]
    pub fn main() -> Option<Self> {
        *window_manager().main_window.lock()
    }

    /// Returns `true` if this window is the main window
    #[must_use]
    #[inline]
    pub fn is_main(self) -> bool {
        window_manager()
            .main_window
            .lock()
            .is_some_and(|main_window| main_window == self)
    }

    /// Marks this window as the "main" window.
    /// This is the window used for frame pacing, and also the window
    /// that quits the application if closed.
    pub(crate) fn mark_main(self) {
        self.with_data(|data| {
            if !matches!(data, WindowData::Pending { .. } | WindowData::Open { .. }) {
                log::error!("Cannot mark non pending/open window {self} as main window");
                return;
            }

            *window_manager().main_window.lock() = Some(self);
            log::debug!("Marked {self} as main window");
        });
    }

    /// Returns the logical size of the window.
    /// To get the size in pixels, see [`Self::pixels`]
    #[must_use]
    #[inline]
    pub fn size(self) -> LogicalSize {
        self.with_data(|data| match data {
            WindowData::Pending { settings, .. } => settings.size,
            WindowData::Open { state, .. } => state.size,
            WindowData::Closed(window_snapshot) => window_snapshot.size,
            WindowData::Failed { snapshot, .. } => snapshot.size,
        })
    }

    /// Returns the physical size (in pixels) of the window. If the window has not yet been created
    /// by the OS, the scale factor is not yet known, so the size in pixels might be off.
    #[must_use]
    #[inline]
    pub fn pixels(self) -> PixelSize {
        self.with_data(|data| match data {
            WindowData::Pending { settings, .. } => PixelSize::from_logical(settings.size, 1.0),
            WindowData::Open { state, .. } => state.pixels,
            WindowData::Closed(window_snapshot) => window_snapshot.pixels,
            WindowData::Failed { snapshot, .. } => snapshot.pixels,
        })
    }

    /// Returns the OS scale factor for this window
    #[must_use]
    #[inline]
    pub fn scale_factor(self) -> f64 {
        self.with_data(|data| match data {
            WindowData::Pending { .. } => 1.0, // Can only guess
            WindowData::Open { state, .. } => state.scale_factor,
            WindowData::Closed(window_snapshot) => window_snapshot.scale_factor,
            WindowData::Failed { snapshot, .. } => snapshot.scale_factor,
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

#[derive(Debug)]
enum WindowData {
    Pending {
        settings: Box<WindowSettings>,
        close_requested: AtomicBool,
    },
    Open {
        // /// The frame of the last outstanding redraw request, if any
        state: Box<WindowSnapshot>,
        close_requested: AtomicBool,
    },
    Closed(Box<WindowSnapshot>),
    Failed {
        snapshot: Box<WindowSnapshot>,
        err: Arc<dyn Error + Send + Sync>,
    },
}

/// A snapshot of the state of a native window,
/// filled by the driver
#[derive(Debug, Clone, Copy)]
pub(crate) struct WindowSnapshot {
    /// The size of the (inner) window in pixels
    pub(crate) pixels: PixelSize,

    /// The size of the (inner) window in points
    pub(crate) size: LogicalSize,

    /// The scale factor of the native window
    pub(crate) scale_factor: f64,
}

impl WindowSnapshot {
    fn guess_from_settings(settings: &WindowSettings) -> Self {
        Self {
            pixels: PixelSize::from_logical(settings.size, 1.0),
            size: settings.size,
            scale_factor: 1.0,
        }
    }
}

static WINDOW_MANAGER: OnceLock<WindowManager> = OnceLock::new();

/// Returns the [`WindowManager`]
///
/// Panics if not yet initialized
#[inline]
fn window_manager() -> &'static WindowManager {
    WINDOW_MANAGER
        .get()
        .expect("Window manager not yet initialized")
}

/// Initializes the windowing subsystem
pub(crate) fn init() {
    profiling::function_scope!();
    thread_context::assert_main_context();

    log::debug!("Initializing window manager");

    WINDOW_MANAGER
        .set(WindowManager {
            main_window: Mutex::new(None),
            windows: RwLock::new(HashMap::new()),
        })
        .expect("Window manager already initialized");
}

/// Runs the queued window actions. This actually spawns native windows.
///
/// NOTE: The [`Window`] API cannot be used in the `initializer` callback as it will
/// deadlock, with the exception of [`Window::main`] and [`Window::is_main`].
pub(crate) fn initialize_windows(
    mut initializer: impl FnMut(
        Window,
        &WindowSettings,
    ) -> Result<WindowSnapshot, Arc<dyn Error + Send + Sync>>,
) {
    profiling::function_scope!();
    thread_context::assert_main_context();

    for (window, data) in window_manager().windows.write().iter_mut() {
        let WindowData::Pending {
            settings,
            close_requested: wants_to_close,
        } = data
        else {
            continue;
        };

        profiling::scope!("initialize_window");

        if wants_to_close.load(Ordering::Relaxed) {
            // If the window is pending but already wants to close, we won't
            // open it
            continue;
        }

        let window_snapshot = match initializer(*window, settings.as_ref()) {
            Ok(snapshot) => snapshot,
            Err(e) => {
                // Driver failed to open the window

                *data = WindowData::Failed {
                    snapshot: Box::new(WindowSnapshot::guess_from_settings(settings)),
                    err: e,
                };

                continue;
            }
        };

        *data = WindowData::Open {
            state: Box::new(window_snapshot),
            close_requested: AtomicBool::new(false),
        };
    }
}

/// Closes any open windows that have been requested to close. Returns the closed windows
/// with their IDs and native windows
pub(crate) fn close_windows() -> Vec<Window> {
    profiling::function_scope!();
    thread_context::assert_main_context();

    let mut closed: Vec<Window> = Vec::new();

    for (&window, data) in window_manager().windows.write().iter_mut() {
        match data {
            WindowData::Pending {
                settings,
                close_requested,
            } => {
                if close_requested.load(Ordering::Relaxed) {
                    // Change state to closed, but don't return to the runtime. It was never open
                    *data =
                        WindowData::Closed(Box::new(WindowSnapshot::guess_from_settings(settings)));
                }
            }
            WindowData::Open {
                state,
                close_requested,
            } => {
                if close_requested.load(Ordering::Relaxed) {
                    // Change state to closed and return to the runtime
                    closed.push(window);
                    *data = WindowData::Closed(state.clone());
                }
            }
            WindowData::Closed(_) => {
                // Already closed
            }
            WindowData::Failed { .. } => {
                // Nothing to handle here
            }
        }
    }

    closed
}

/// Refreshes the cached window data for this window to the new snapshot.
/// The window must be open
pub(crate) fn update_window_snapshot(window: Window, snapshot: WindowSnapshot) {
    profiling::function_scope!();
    thread_context::assert_main_context();

    let mut windows = window_manager().windows.write();
    let Some(window_data) = windows.get_mut(&window) else {
        log::error!("Cannot refresh data for unknown window {window}");
        return;
    };

    let WindowData::Open { state, .. } = window_data else {
        log::error!("Cannot refresh data for non-open window {window}");
        return;
    };

    *state.as_mut() = snapshot;

    log::trace!("Window snapshot for window {window} updated");
}
