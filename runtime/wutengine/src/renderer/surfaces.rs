//! Rendering surface management

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use wgpu::Surface;
use wutengine_sync::RwLock;

use crate::renderer;
use crate::window::Window;

use winit::window::Window as WinitWindow;

static SURFACES: OnceLock<RwLock<HashMap<Window, Surface<'static>>>> = OnceLock::new();

fn surfaces() -> &'static RwLock<HashMap<Window, Surface<'static>>> {
    SURFACES.get().expect("Surfaces not yet initialized")
}

pub(super) fn init_surface_map() {
    SURFACES
        .set(RwLock::new(HashMap::new()))
        .expect("Surface map already initialized");
}

/// Initializes the rendering surface for the given native window, with the given handle
pub(crate) fn initialize_surface(window: Window, native: Arc<dyn WinitWindow>) {
    profiling::function_scope!();

    let surface = renderer::instance()
        .create_surface(native)
        .expect("Failed to initialize surface for window"); //TODO: Handle properly

    surfaces().write().insert(window, surface);
}

/// Runs the callback with the [`Surface`] for the given window
pub(crate) fn get_surface<T>(window: Window, cb: impl FnOnce(&Surface<'static>) -> T) -> T {
    profiling::function_scope!();

    let surfaces = surfaces().read();

    let surface = surfaces.get(&window).expect("Missing surface for window");

    cb(surface)
}
