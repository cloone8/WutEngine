#![doc = include_str!("../README.md")]

use std::sync::OnceLock;

pub use init::init_adapter;
pub use init::init_instance;
pub use init::init_renderer;
use wgpu::Adapter;
use wgpu::Device;
use wgpu::Instance;
use wgpu::Queue;
use wutengine_log as log;

mod init;
pub mod render;
pub mod surfaces;

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

/// The global [`wgpu`] instance
static INSTANCE: OnceLock<Instance> = OnceLock::new();

/// The global data for the renderer
#[derive(Debug)]
struct Renderer {
    /// The [`Adapter`]
    adapter: Adapter,

    /// The [`Device`]
    device: Device,

    /// The [`Queue`]
    queue: Queue,
}

impl Renderer {
    /// Create a new [`Renderer`] containing the given values
    fn new(adapter: Adapter, device: Device, queue: Queue) -> Self {
        Self {
            adapter,
            device,
            queue,
        }
    }
}

/// The global renderer instance
static RENDERER: OnceLock<Renderer> = OnceLock::new();

/// Returns the global renderer.
///
/// If the renderer is not yet initialized, panics.
fn renderer() -> &'static Renderer {
    RENDERER.get().expect("Renderer not yet initialized")
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

/// Converts an array of floats to a [wgpu Color](wgpu::Color). Performs
/// no conversions, so those have to be done beforehand manually.
#[inline]
const fn array_to_wgpu_color(color: [f32; 4]) -> wgpu::Color {
    wgpu::Color {
        r: color[0] as f64,
        g: color[1] as f64,
        b: color[2] as f64,
        a: color[3] as f64,
    }
}
