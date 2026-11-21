//! The renderer

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use wgpu::Adapter;
use wgpu::Device;
use wgpu::Instance;
use wgpu::Queue;
use winit::window::Window as WinitWindow;

pub(crate) use init::init_adapter;
pub(crate) use init::init_instance;
pub(crate) use init::init_renderer;

use wgpu::Surface;

use crate::window::Window;
use wutengine_sync::RwLock;

mod init;

/// Creates a generic label for the [`wgpu`] API
#[macro_export]
#[cfg(feature = "labels")]
macro_rules! label {
    () => {
        None
    };

    ($arg:literal) => {
        Some($arg)
    };

    ($arg:expr) => {
        Some($arg)
    };

    ($($arg:tt)+) => {
        Some(format!($($arg)*).as_str())
    };
}

/// Creates a generic label for the [`wgpu`] API
#[macro_export]
#[cfg(not(feature = "labels"))]
macro_rules! label {
    ($($arg:tt)+) => {
        None
    };
}

static INSTANCE: OnceLock<Instance> = OnceLock::new();

/// The global data for the renderer
#[derive(Debug)]
struct Renderer {
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl Renderer {
    fn new(adapter: Adapter, device: Device, queue: Queue) -> Self {
        Self {
            adapter,
            device,
            queue,
        }
    }
}

static RENDERER: OnceLock<Renderer> = OnceLock::new();

fn renderer() -> &'static Renderer {
    RENDERER.get().expect("Renderer not yet initialized")
}

static SURFACES: OnceLock<RwLock<HashMap<Window, Surface<'static>>>> = OnceLock::new();

fn surfaces() -> &'static RwLock<HashMap<Window, Surface<'static>>> {
    SURFACES.get().expect("Surfaces not yet initialized")
}

/// Returns the global [`Instance`]
///
/// Panics if called before [`init::init_renderer`]
#[inline]
pub(crate) fn instance() -> &'static Instance {
    INSTANCE.get().expect("Instance not yet initialized")
}

/// Returns the global [`Adapter`]
///
/// Panics if called before [`init::init_renderer`]
#[inline]
pub(crate) fn adapter() -> &'static Adapter {
    &renderer().adapter
}

/// Returns the global [`Device`]
///
/// Panics if called before [`init::init_renderer`]
#[inline]
pub(crate) fn device() -> &'static Device {
    &renderer().device
}

/// Returns the global [`Queue`]
///
/// Panics if called before [`init::init_renderer`]
#[inline]
pub(crate) fn queue() -> &'static Queue {
    &renderer().queue
}

/// Initializes the rendering surface for the given native window, with the given handle
pub(crate) fn initialize_surface(window: Window, native: Arc<dyn WinitWindow>) {
    profiling::function_scope!();

    let surface = instance()
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
