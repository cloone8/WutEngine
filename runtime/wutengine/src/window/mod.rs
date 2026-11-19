//! Windowing API

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
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
    windows: RwLock<HashMap<Window, WindowData>>,
}

#[derive(Debug)]
enum WindowData {
    Pending {
        requested_attributes: WindowAttributes,
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
            windows: RwLock::new(HashMap::new()),
        })
        .expect("Window manager already initialized");
}

/// Sends a request to create a new window. The handle is returned immediately,
/// and the window itself is created as soon as possible on the main thread
#[must_use]
pub fn create() -> Window {
    let handle = Window::new();

    window_manager().windows.write().insert(
        handle,
        WindowData::Pending {
            requested_attributes: WindowAttributes::default(),
        },
    );

    handle
}

/// Runs the queued window actions. This actually spawns native windows.
/// Return whether queued actions were actually executed
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
