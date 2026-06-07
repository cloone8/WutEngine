//! Graphics initialization

use alloc::sync::Arc;

use crate::config::GraphicsConfig;
use wutengine_util::InitOnce;

/// Initializes the global graphics context for WutEngine. Acquires a graphics
/// device and configures it.
/// Returns `true` if graphics initialization was succesful, and `false` otherwise.
pub fn initialize_graphics_context(config: GraphicsConfig) -> bool {
    log::debug!("Initializing graphics context");

    log::trace!(
        "Available graphics backends in build: {}",
        backends_to_str(wgpu::Backends::all())
    );

    log::debug!("Using backend: {}", config.backend);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: config.backend.into(),
        display: None,
        flags: gather_instance_flags(&config),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: default_backend_options(),
    });

    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    })) {
        Ok(a) => a,
        Err(e) => {
            log::error!("Failed to find a graphics adapter: {e}");
            return false;
        }
    };

    let adapter_info = adapter.get_info();

    log::info!(
        "Using graphics device '{}' with backend '{}' and driver '{} {}'",
        adapter_info.name,
        adapter_info.backend,
        adapter_info.driver,
        adapter_info.driver_info
    );

    let (device, queue) =
        match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("WutEngine Main GPU"),
            ..Default::default()
        })) {
            Ok(dq) => dq,
            Err(e) => {
                log::error!("Failed to get a device from the graphics adapter: {e}");
                return false;
            }
        };

    device.set_device_lost_callback(on_device_lost);
    device.on_uncaptured_error(Arc::new(on_uncaptured_error));

    InitOnce::init(&super::GFX_ADAPTER, adapter);
    InitOnce::init(&super::GFX_INSTANCE, instance);
    InitOnce::init(&super::GFX_DEVICE, device);
    InitOnce::init(&super::GFX_QUEUE, queue);

    true
}

fn on_device_lost(reason: wgpu::DeviceLostReason, message: String) {
    let reason_str = match reason {
        wgpu::DeviceLostReason::Unknown => "<unknown>",
        wgpu::DeviceLostReason::Destroyed => "<device destroyed>",
    };

    panic!("Fatal graphics error: Graphics device lost due to {reason_str}: {message}");
}

fn on_uncaptured_error(error: wgpu::Error) {
    log::error!("Graphics error: {error}");

    if wutengine_config::try_get::<bool>("wutengine.graphics.exit_on_graphics_error")
        .unwrap_or(cfg!(debug_assertions))
    {
        panic!("Fatal graphics error: {}", error);
    }
}

fn backends_to_str(backends: wgpu::Backends) -> String {
    backends
        .iter_names()
        .map(|be| be.0)
        .collect::<Vec<_>>()
        .join(", ")
}

fn default_backend_options() -> wgpu::BackendOptions {
    wgpu::BackendOptions {
        gl: wgpu::GlBackendOptions::default(),
        dx12: wgpu::Dx12BackendOptions {
            shader_compiler: wgpu::Dx12Compiler::Fxc,
            presentation_system: wgpu::Dx12SwapchainKind::DxgiFromHwnd,
            latency_waitable_object: wgpu::Dx12UseFrameLatencyWaitableObject::Wait,
            force_shader_model: wgpu::ForceShaderModelToken::default(),
            agility_sdk: None,
        },
        noop: wgpu::NoopBackendOptions { enable: false },
    }
}

fn gather_instance_flags(config: &GraphicsConfig) -> wgpu::InstanceFlags {
    let mut flags = wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL;

    if config.debug {
        flags |= wgpu::InstanceFlags::DEBUG;
    }

    if config.validation {
        flags |= wgpu::InstanceFlags::VALIDATION;
    }

    if config.gpu_based_validation {
        flags |= wgpu::InstanceFlags::GPU_BASED_VALIDATION;
        log::warn!(
            "Using GPU based graphics validation. This can have a very significant performance impact"
        );
    }

    log::trace!("Flags: {:?}", flags);

    flags
}
