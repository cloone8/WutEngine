//! WutEngine graphics layer

extern crate alloc;

mod bind_group;
mod cache;
mod config;
mod init;
pub mod internal_bind_groups;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod renderpass;
pub mod sampler;
pub mod shader;
pub mod texture;

#[doc(inline)]
pub use wgpu;

pub use bind_group::*;

pub use config::*;

pub use init::initialize_graphics_context;

use wutengine_util::InitOnce;

/// The global [wgpu::Adapter]
static GFX_ADAPTER: InitOnce<wgpu::Adapter> = InitOnce::new();

/// The global [wgpu::Instance]
static GFX_INSTANCE: InitOnce<wgpu::Instance> = InitOnce::new();

/// The global [wgpu::Device]
static GFX_DEVICE: InitOnce<wgpu::Device> = InitOnce::new();

/// The global [wgpu::Queue]
static GFX_QUEUE: InitOnce<wgpu::Queue> = InitOnce::new();

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

pub const fn to_wgpu_color(color: wutengine_math::Color) -> wgpu::Color {
    wgpu::Color {
        r: color.r() as f64,
        g: color.g() as f64,
        b: color.b() as f64,
        a: color.a() as f64,
    }
}
