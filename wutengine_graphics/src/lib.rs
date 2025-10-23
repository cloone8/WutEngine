//! Graphics backend for WutEngine

use std::collections::HashMap;
use std::sync::RwLock;

use thiserror::Error;
use wgpu::wgt::DeviceDescriptor;
use wgpu::{
    BackendOptions, CommandBuffer, Features, InstanceDescriptor, Limits, MemoryBudgetThresholds,
    MemoryHints, PowerPreference, RequestAdapterOptions,
};
use wutengine_util::GlobalManager;
use wutengine_windowing::window::{WindowIdentifier, WindowResizedEvent};

use crate::config::{GraphicsBackend, WutEngineGraphicsConfig};
use crate::surface::{Surface, on_resized_event};

pub mod buffer;
pub mod color;
mod config;
pub(crate) mod debug;
pub mod format;
pub mod material;
pub mod mesh;
pub mod passes;
pub mod pipeline;
pub mod shader;
pub mod surface;
pub mod texture;
pub mod viewport;

pub use wgpu;

pub(crate) static GRAPHICS_MANAGER: GlobalManager<GraphicsManager> = GlobalManager::new();

#[derive(Debug, Error)]
pub enum InitErr {
    #[error("Could not find compatible adapter: {0}")]
    Adapter(#[from] wgpu::RequestAdapterError),

    #[error("Could not get device from adapter: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
}

/// Called by WGPU when we lose the graphics device. Pretty much fatal
fn device_lost_callback(reason: wgpu::DeviceLostReason, message: String) {
    log::error!("Lost device due to {reason:#?}: {message}");
    panic!("Lost graphics device: {reason:#?}: {message}");
}

pub async fn init() -> Result<(), InitErr> {
    log::info!("Initializing WutEngine graphics stack");
    let config = wutengine_config::get_wutengine::<WutEngineGraphicsConfig>("graphics");

    log::debug!("Requested backends: {}", config.backend);
    log::debug!("Compiled backends: {}", GraphicsBackend::IN_BUILD);

    let usable_backends = config.backend & GraphicsBackend::IN_BUILD;

    log::info!("Using graphics backends: {usable_backends}");

    let instance = wgpu::Instance::new(&InstanceDescriptor {
        backends: usable_backends.into(),
        flags: config.debug_level.into(),
        memory_budget_thresholds: MemoryBudgetThresholds::default(),
        backend_options: BackendOptions::default(),
    });

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await?;

    log::info!("Using adapter: {:#?}", adapter.get_info());

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: Some("Main Device"),
            required_features: Features::default(),
            required_limits: Limits::defaults(),
            memory_hints: MemoryHints::default(),
            trace: wgpu::Trace::Off,
        })
        .await?;

    device.set_device_lost_callback(device_lost_callback);

    device.on_uncaptured_error(Box::new(move |e| {
        log::error!("Encountered graphics device error: {e}");

        if !config.ignore_errors {
            panic!("Graphics validation error");
        }
    }));

    let manager = GraphicsManager {
        instance,
        adapter,
        device,
        queue,
        shader_cache: shader::cache::ShaderCache::new(),
        pipeline_cache: pipeline::cache::PipelineCache::new(),
        buffer_cache: buffer::cache::BufferCache::new(),
    };

    GlobalManager::init(&GRAPHICS_MANAGER, manager);

    // Initialize the surface manager
    surface::init();

    wutengine_event::subscribe::<wutengine_windowing::window::WindowResizedEvent>(on_resized_event);

    Ok(())
}

pub fn raw_device() -> &'static wgpu::Device {
    &GRAPHICS_MANAGER.device
}

pub fn create_command_encoder(desc: &wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder {
    GRAPHICS_MANAGER.device.create_command_encoder(desc)
}

pub fn submit_command_buffers<I: IntoIterator<Item = CommandBuffer>>(buffers: I) {
    GRAPHICS_MANAGER.queue.submit(buffers);
}

pub fn get_limits() -> wgpu::Limits {
    GRAPHICS_MANAGER.device.limits()
}

#[derive(Debug)]
struct GraphicsManager {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader_cache: shader::cache::ShaderCache,
    pipeline_cache: pipeline::cache::PipelineCache,
    buffer_cache: buffer::cache::BufferCache,
}
