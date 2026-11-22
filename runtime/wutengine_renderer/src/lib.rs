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

/// Converts a [WutEngine color](wutengine_math::Color) to a [WGPU color](wgpu::Color)
#[inline]
#[must_use]
pub const fn color_to_wgpu(color: wutengine_math::Color) -> wgpu::Color {
    let color_array = color.as_vec4().to_array();
    wgpu::Color {
        r: color_array[0] as f64,
        g: color_array[1] as f64,
        b: color_array[2] as f64,
        a: color_array[3] as f64,
    }
}

/// Converts a [WGPU color](wgpu::Color) to a [WutEngine color](wutengine_math::Color)
#[inline]
#[must_use]
pub const fn color_from_wgpu(color: wgpu::Color) -> wutengine_math::Color {
    wutengine_math::Color::new(
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    )
}
