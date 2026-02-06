use std::sync::mpsc::{Receiver, channel};

use crate::util::InitOnce;

use super::DrawCommand;

/// Initializes the global draw command queue, and returns its receiving end.
pub(crate) fn initialize_command_queue() -> Receiver<DrawCommand> {
    let (send, recv) = channel::<DrawCommand>();

    InitOnce::init(&super::DRAW_COMMAND_QUEUE, send);

    recv
}

/// Initializes the global graphics context for WutEngine. Acquires a graphics
/// device and configures it.
/// Returns `true` if graphics initialization was succesful, and `false` otherwise.
pub(crate) fn initialize_graphics_context() -> bool {
    log::debug!("Initializing graphics context");

    log::trace!(
        "Available graphics backends in build: {}",
        backends_to_str(wgpu::Backends::all())
    );

    let wanted_backends = default_backends();

    log::debug!("Using backends: {}", backends_to_str(wanted_backends));

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wanted_backends,
        flags: default_instance_flags(),
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

    InitOnce::init(&super::GFX_ADAPTER, adapter);
    InitOnce::init(&super::GFX_INSTANCE, instance);
    InitOnce::init(&super::GFX_DEVICE, device);
    InitOnce::init(&super::GFX_QUEUE, queue);

    true
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
        },
        noop: wgpu::NoopBackendOptions { enable: false },
    }
}

fn default_instance_flags() -> wgpu::InstanceFlags {
    if cfg!(debug_assertions) {
        wgpu::InstanceFlags::advanced_debugging()
    } else {
        wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL
    }
}

fn default_backends() -> wgpu::Backends {
    if cfg!(target_arch = "wasm32") {
        wgpu::Backends::BROWSER_WEBGPU
    } else if cfg!(windows) {
        wgpu::Backends::VULKAN.union(wgpu::Backends::DX12)
    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
        wgpu::Backends::METAL
    } else if cfg!(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd"
    )) {
        wgpu::Backends::VULKAN
    } else {
        log::warn!(
            "Could not determine appropriate backends for current platform. Using first available"
        );
        wgpu::Backends::all()
    }
}
