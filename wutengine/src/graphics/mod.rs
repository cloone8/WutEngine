//! WutEngine graphics layer

mod api;
mod bind_group;
mod cache;
mod config;
mod init;
pub(crate) mod internal_bind_groups;
pub mod material;
pub mod mesh;
pub(crate) mod pipeline;
pub mod renderpass;
pub mod sampler;
pub mod shader;
pub mod texture;

pub(crate) use bind_group::*;

pub use api::*;

use std::sync::mpsc::Sender;

use crate::util::InitOnce;

pub(crate) use init::{initialize_command_queue, initialize_graphics_context};

/// The global [wgpu::Adapter]
static GFX_ADAPTER: InitOnce<wgpu::Adapter> = InitOnce::new();

/// The global [wgpu::Instance]
static GFX_INSTANCE: InitOnce<wgpu::Instance> = InitOnce::new();

/// The global [wgpu::Device]
static GFX_DEVICE: InitOnce<wgpu::Device> = InitOnce::new();

/// The global [wgpu::Queue]
static GFX_QUEUE: InitOnce<wgpu::Queue> = InitOnce::new();

/// The global draw command queue
static DRAW_COMMAND_QUEUE: InitOnce<Sender<DrawCommand>> = InitOnce::new();

/// Returns the global graphics adapter
#[inline(always)]
pub(crate) fn adapter() -> &'static wgpu::Adapter {
    &GFX_ADAPTER
}

/// Returns the global graphics instance
#[inline(always)]
pub(crate) fn instance() -> &'static wgpu::Instance {
    &GFX_INSTANCE
}

/// Returns the global graphics device
#[inline(always)]
pub(crate) fn device() -> &'static wgpu::Device {
    &GFX_DEVICE
}

/// Returns the global graphics queue
#[inline(always)]
pub(crate) fn queue() -> &'static wgpu::Queue {
    &GFX_QUEUE
}
