use std::sync::Arc;

use wgpu::Backends;
use wgpu::DeviceType;
use wgpu::Features;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::InstanceFlags;
use wgpu::MemoryBudgetThresholds;
use wgpu::PowerPreference;
use wgpu::RequestAdapterOptions;
use wgpu::Surface;
use wgpu::wgt::DeviceDescriptor;

use crate::label;
use crate::log;
use crate::renderer::RENDERER;
use crate::renderer::Renderer;
use log::Level;

/// Initializes the [`Instance`] based on the config of the renderer
pub(crate) fn init_instance() -> Instance {
    log::debug!("Initializing WGPU instance");

    log::trace!(
        "Available graphics backends in build: {}",
        backends_to_str(wgpu::Backends::all())
    );

    Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        display: None,
        flags: InstanceFlags::default(),
        memory_budget_thresholds: MemoryBudgetThresholds::default(),
        backend_options: default_backend_options(),
    })
}

/// Initializes the renderer, making sure it supports at least the given surface
pub(crate) async fn init_renderer(instance: Instance, surface: &Surface<'_>) -> bool {
    log::debug!("Initializing renderer");

    let adapter = match instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
            apply_limit_buckets: false,
        })
        .await
    {
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

    // We request all features supported by the device, excluding "unwanted" ones.
    let adapter_requested_features = adapter
        .features()
        .intersection(all_wanted_features(adapter_info.device_type));

    if log::log_enabled!(Level::Debug) {
        log_features(adapter_requested_features);
    }

    let (device, queue) = match adapter
        .request_device(&DeviceDescriptor {
            label: label!("WutEngine Main GPU"),
            required_features: adapter_requested_features,
            ..Default::default()
        })
        .await
    {
        Ok(dq) => dq,
        Err(e) => {
            log::error!("Failed to get a device from the graphics adapter: {e}");
            return false;
        }
    };

    device.set_device_lost_callback(on_device_lost);
    device.on_uncaptured_error(Arc::new(on_uncaptured_error));

    RENDERER
        .set(Renderer {
            instance,
            adapter,
            device,
            queue,
        })
        .expect("Renderer already initialized");

    true
}

fn log_features(features: Features) {
    let mut features_string = String::new();

    let mut feature_names = Vec::new();

    for (feature, _) in features.iter_names() {
        feature_names.push(feature);
    }

    feature_names.sort_unstable();

    for feature in feature_names {
        features_string = format!("{features_string}\n\t{feature}");
    }

    log::debug!("Supported features:{features_string}");
}

#[expect(clippy::needless_pass_by_value, reason = "WGPU API")]
fn on_device_lost(reason: wgpu::DeviceLostReason, message: String) {
    let reason_str = match reason {
        wgpu::DeviceLostReason::Unknown => "<unknown>",
        wgpu::DeviceLostReason::Destroyed => "<device destroyed>",
    };

    panic!("Fatal graphics error: Graphics device lost due to {reason_str}: {message}");
}

#[expect(clippy::needless_pass_by_value, reason = "WGPU API")]
fn on_uncaptured_error(error: wgpu::Error) {
    //TODO: Not always panic
    panic!("Fatal graphics error: {error}");
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
        noop: wgpu::NoopBackendOptions {
            enable: false,
            limits: None,
            features: None,
            device_type: None,
            subgroup_min_size: None,
            subgroup_max_size: None,
        },
    }
}

/// All features we could possibly request
const fn all_wanted_features(device_type: wgpu::DeviceType) -> wgpu::Features {
    wgpu::Features::all().intersection(unwanted_features_mask(device_type).complement())
}

/// A mask of all unwanted API features. This includes experimental features,
/// and mappable primary buffers on non-shared memory systems
const fn unwanted_features_mask(device_type: wgpu::DeviceType) -> wgpu::Features {
    let mut mask = wgpu::Features::all_experimental_mask();

    if !matches!(device_type, DeviceType::IntegratedGpu | DeviceType::Cpu) {
        mask = mask.union(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
    }

    mask
}
