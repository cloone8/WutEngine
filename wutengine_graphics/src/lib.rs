//! Graphics backend for WutEngine

mod backend;

use std::collections::HashMap;
use std::sync::Mutex;

pub use backend::WutEngineBackend;
use thiserror::Error;
use wgpu::wgt::DeviceDescriptor;
use wgpu::{
    BackendOptions, Features, InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds,
    MemoryHints, PowerPreference, RequestAdapterOptions,
};
use wutengine_util::GlobalManager;
use wutengine_windowing::window::WindowIdentifier;

pub mod mesh;
pub mod texture;

static GRAPHICS_MANAGER: GlobalManager<GraphicsManager> = GlobalManager::new();

#[derive(Debug, Error)]
pub enum InitErr {
    #[error("Could not find compatible adapter: {0}")]
    Adapter(#[from] wgpu::RequestAdapterError),

    #[error("Could not get device from adapter: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
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

    let manager = GraphicsManager {
        instance,
        adapter,
        device,
        queue,
        surfaces: Mutex::new(HashMap::new()),
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
        inner_size,
        native: surface,
    };

    graphics_surface.reconfigure(&GRAPHICS_MANAGER.device);

    let mut locked = GRAPHICS_MANAGER.surfaces.lock().unwrap();
    locked.insert(id, graphics_surface);

    Ok(())
}

pub fn resized(id: &WindowIdentifier, inner_size: (u32, u32)) {
    let mut locked = GRAPHICS_MANAGER.surfaces.lock().unwrap();

    let surface = if let Some(surface) = locked.get_mut(id) {
        surface
    } else {
        log::warn!("Resizing unknown surface {id}. Ignoring");
        return;
    };

    surface.inner_size = inner_size;
    surface.reconfigure(&GRAPHICS_MANAGER.device);
}

#[derive(Debug)]
struct GraphicsSurface {
    native: wgpu::Surface<'static>,
    capabilities: wgpu::SurfaceCapabilities,
    inner_size: (u32, u32),
}

impl GraphicsSurface {
    fn reconfigure(&self, device: &wgpu::Device) {
        let surface_format = self.capabilities.formats[0];

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.inner_size.0,
            height: self.inner_size.1,
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
    surfaces: Mutex<HashMap<WindowIdentifier, GraphicsSurface>>,
}
