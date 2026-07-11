#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod bind_group;
mod cache;
mod config;
mod init;
pub mod internal_bind_groups;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod queries;
pub mod renderpass;
pub mod sampler;
pub mod shader;
pub mod texture;

pub use bind_group::*;
pub use config::*;
pub use init::{initialize_graphics_context, persist_pipeline_cache};
#[doc(inline)]
pub use wgpu;

use wutengine_util::InitOnce;

/// The global [wgpu::Adapter]
static GFX_ADAPTER: InitOnce<wgpu::Adapter> = InitOnce::new_checked();

/// The global [wgpu::Instance]
static GFX_INSTANCE: InitOnce<wgpu::Instance> = InitOnce::new_checked();

/// The global [wgpu::Device]
static GFX_DEVICE: InitOnce<wgpu::Device> = InitOnce::new_checked();

/// The global [wgpu::Queue]
static GFX_QUEUE: InitOnce<wgpu::Queue> = InitOnce::new_checked();

/// The global active [graphics configuration](GraphicsRuntimeConfig)
static ACTIVE_CONFIG: InitOnce<GraphicsRuntimeConfig> = InitOnce::new_checked();

/// The pipeline cache, if we have one
static PIPELINE_CACHE: InitOnce<Option<wgpu::PipelineCache>> = InitOnce::new_checked();

/// Returns the global graphics adapter
#[inline(always)]
pub fn adapter() -> &'static wgpu::Adapter {
    &GFX_ADAPTER
}

/// Returns the global graphics instance
#[inline(always)]
pub fn instance() -> &'static wgpu::Instance {
    &GFX_INSTANCE
}

/// Returns the global graphics device
#[inline(always)]
pub fn device() -> &'static wgpu::Device {
    &GFX_DEVICE
}

/// Returns the global graphics queue
#[inline(always)]
pub fn queue() -> &'static wgpu::Queue {
    &GFX_QUEUE
}

/// Returns the active [graphics configuration](GraphicsRuntimeConfig)
#[inline(always)]
pub fn active_config() -> &'static GraphicsRuntimeConfig {
    &ACTIVE_CONFIG
}

/// Returns whether all features in `features` are currently supported
#[inline(always)]
pub fn features_supported(features: wgpu::Features) -> bool {
    ACTIVE_CONFIG.features.contains(features)
}

/// Converts a [wutengine color](wutengine_math::Color) to a [wgpu color](wgpu::Color)
pub const fn to_wgpu_color(color: wutengine_math::Color) -> wgpu::Color {
    wgpu::Color {
        r: color.r() as f64,
        g: color.g() as f64,
        b: color.b() as f64,
        a: color.a() as f64,
    }
}

/// All features we could possibly request
pub const fn all_wanted_features(device_type: wgpu::DeviceType) -> wgpu::Features {
    wgpu::Features::all().intersection(unwanted_features_mask(device_type).complement())
}

/// A mask of all unwanted API features. This includes experimental features,
/// and mappable primary buffers on non-shared memory systems
const fn unwanted_features_mask(device_type: wgpu::DeviceType) -> wgpu::Features {
    let mut mask = wgpu::Features::all_experimental_mask();

    if !matches!(device_type, wgpu::DeviceType::IntegratedGpu) {
        mask = mask.union(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
    }

    mask
}

/// Creates a generic label for the [wgpu] API
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
        Some(::alloc::format!($($arg)*).as_str())
    };
}

/// Creates a generic label for the [wgpu] API
#[macro_export]
#[cfg(not(feature = "labels"))]
macro_rules! label {
    ($($arg:tt)+) => {
        None
    };
}
