//! Windowing API

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use wutengine_sync::Mutex;
use wutengine_sync::RwLock;

use winit::window::Window as WinitWindow;
use winit::window::WindowAttributes;
use wutengine_macro::unique_id_type32;

use crate::log;

unique_id_type32! {
    /// Handle to a WutEngine window
    pub Window
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

#[derive(Debug)]
enum WindowData {
    Pending {
        requested_attributes: Box<WindowAttributes>,
    },
    Open {
        native: Arc<dyn WinitWindow>,
    },
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

/// Sends a request to create a new window. The handle is returned immediately,
/// and the window itself is created as soon as possible on the main thread
#[must_use]
pub fn create() -> Window {
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
            requested_attributes: Box::new(WindowAttributes::default()),
        },
    );

    handle
}

/// Runs the queued window actions. This actually spawns native windows.
pub(crate) fn initialize_windows(
    mut initializer: impl FnMut(Window, &WindowAttributes) -> Arc<dyn WinitWindow>,
) {
    profiling::function_scope!();

    for (window, data) in window_manager().windows.write().iter_mut() {
        let WindowData::Pending {
            requested_attributes,
        } = data
        else {
            continue;
        };

        profiling::scope!("initialize_window");

        let native_window = initializer(*window, requested_attributes);

        *data = WindowData::Open {
            native: native_window,
        };
    }
}
