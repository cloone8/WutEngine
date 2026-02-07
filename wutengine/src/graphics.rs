//! WutEngine graphics layer

pub(crate) mod cache;
mod init;
pub mod material;
pub mod shader;
pub mod texture;

use std::sync::mpsc::Sender;

use crate::builtins::components::CameraId;
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

/// Submits a raw draw command to the command queue
#[inline(always)]
pub fn submit_raw_draw_command(command: DrawCommand) {
    DRAW_COMMAND_QUEUE.send(command).expect("Runtime stopped")
}

/// A single draw command submitted to the WutEngine graphics backend.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// The camera this draw call applies to. If [None], renders on all cameras
    pub camera: Option<CameraId>,
}
