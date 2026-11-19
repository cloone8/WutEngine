//! The renderer

use std::sync::OnceLock;

use wgpu::Adapter;
use wgpu::Device;
use wgpu::Instance;
use wgpu::Queue;
mod init;

pub(crate) use init::init_instance;
pub(crate) use init::init_renderer;

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

static RENDERER: OnceLock<Renderer> = OnceLock::new();

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
    &RENDERER
        .get()
        .expect("Renderer not yet initialized")
        .adapter
}

/// Returns the global [`Device`]
///
/// Panics if called before [`init::init_renderer`]
#[inline]
pub(crate) fn device() -> &'static Device {
    &RENDERER.get().expect("Renderer not yet initialized").device
}

/// Returns the global [`Queue`]
///
/// Panics if called before [`init::init_renderer`]
#[inline]
pub(crate) fn queue() -> &'static Queue {
    &RENDERER.get().expect("Renderer not yet initialized").queue
}
