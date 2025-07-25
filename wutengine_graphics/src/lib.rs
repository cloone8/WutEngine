//! Graphics backend for WutEngine

mod backend;

use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

pub use backend::WutEngineBackend;
use serde::de;
use thiserror::Error;
use wgpu::wgt::DeviceDescriptor;
use wgpu::{
    BackendOptions, CommandBuffer, Features, InstanceDescriptor, InstanceFlags, Limits,
    MemoryBudgetThresholds, MemoryHints, PipelineCacheDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, SurfaceTexture,
};
use wutengine_util::GlobalManager;
use wutengine_windowing::window::WindowIdentifier;

pub mod color;
pub mod format;
pub mod material;
pub mod mesh;
pub(crate) mod passes;
pub mod resource;
pub mod shader;
pub mod texture;

pub(crate) static GRAPHICS_MANAGER: GlobalManager<GraphicsManager> = GlobalManager::new();

#[derive(Debug, Error)]
pub enum InitErr {
    #[error("Could not find compatible adapter: {0}")]
    Adapter(#[from] wgpu::RequestAdapterError),

    #[error("Could not get device from adapter: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
}

fn reinitialize_graphics() {
    resource::increment_device_generation();
    todo!();
}

pub async fn init(backends: WutEngineBackend) -> Result<(), InitErr> {
    log::info!("Initializing WutEngine graphics stack");

    log::debug!("Requested backends: {backends}");
    log::debug!("Compiled backends: {}", WutEngineBackend::IN_BUILD);

    let usable_backends = backends & WutEngineBackend::IN_BUILD;

    log::info!("Using graphics backends: {usable_backends}");

    let instance = wgpu::Instance::new(&InstanceDescriptor {
        backends: usable_backends.into(),
        flags: InstanceFlags::from_build_config(),
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

    device.set_device_lost_callback(|reason, msg| {
        log::error!("Lost device due to {reason:#?}: {msg}");

        reinitialize_graphics();
    });

    device.on_uncaptured_error(Box::new(|e| {
        panic!("AAAA: {e}"); // Todo: replace once we find what triggers this
    }));

    // Set device generation to 1: the main/first
    resource::increment_device_generation();

    let manager = GraphicsManager {
        instance,
        adapter,
        device,
        queue,
        surfaces: RwLock::new(HashMap::new()),
        shader_cache: shader::cache::ShaderCache::new(),
    };

    GlobalManager::init(&GRAPHICS_MANAGER, manager);

    Ok(())
}

#[derive(Debug, Error)]
pub enum InitSurfaceErr {
    #[error("Failed to create a rendering surface for the given window: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),

    #[error("The current adapter does not support rendering to the window")]
    UnsupportedAdapter,
}

pub fn initialize_surface_for_window(
    id: WindowIdentifier,
    inner_size: (u32, u32),
    native: impl Into<wgpu::SurfaceTarget<'static>>,
) -> Result<(), InitSurfaceErr> {
    let surface = GRAPHICS_MANAGER.instance.create_surface(native)?;

    if !GRAPHICS_MANAGER.adapter.is_surface_supported(&surface) {
        return Err(InitSurfaceErr::UnsupportedAdapter);
    }

    let graphics_surface = GraphicsSurface {
        capabilities: surface.get_capabilities(&GRAPHICS_MANAGER.adapter),
        native: surface,
    };

    graphics_surface.reconfigure(&GRAPHICS_MANAGER.device, inner_size);

    let mut locked = GRAPHICS_MANAGER.surfaces.write().unwrap();
    locked.insert(id, graphics_surface);

    Ok(())
}

pub fn resized(id: &WindowIdentifier, inner_size: (u32, u32)) {
    let mut locked = GRAPHICS_MANAGER.surfaces.write().unwrap();

    let surface = if let Some(surface) = locked.get_mut(id) {
        surface
    } else {
        log::warn!("Resizing unknown surface {id}. Ignoring");
        return;
    };

    surface.reconfigure(&GRAPHICS_MANAGER.device, inner_size);
}

pub fn get_window_surface_texture(id: &WindowIdentifier) -> Option<wgpu::SurfaceTexture> {
    let surfaces = GRAPHICS_MANAGER.surfaces.read().unwrap();
    let surface = match surfaces.get(id) {
        Some(surface) => surface,
        None => {
            log::error!("Could not find surface for given window id {id}");
            return None;
        }
    };

    let surface_texture = surface
        .native
        .get_current_texture()
        .expect("failed to acquire next swapchain texture");

    Some(surface_texture)
}

pub fn create_command_encoder(desc: &wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder {
    GRAPHICS_MANAGER.device.create_command_encoder(desc)
}

pub fn submit_command_buffers<I: IntoIterator<Item = CommandBuffer>>(buffers: I) {
    GRAPHICS_MANAGER.queue.submit(buffers);
}

#[derive(Debug)]
struct GraphicsSurface {
    native: wgpu::Surface<'static>,
    capabilities: wgpu::SurfaceCapabilities,
}

impl GraphicsSurface {
    fn reconfigure(&self, device: &wgpu::Device, size: (u32, u32)) {
        dbg!(&self.capabilities.formats);
        let surface_format = self.capabilities.formats[0];

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.0,
            height: size.1,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        self.native.configure(device, &surface_config);
    }
}

#[derive(Debug)]
struct GraphicsManager {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surfaces: RwLock<HashMap<WindowIdentifier, GraphicsSurface>>,
    shader_cache: shader::cache::ShaderCache,
}
