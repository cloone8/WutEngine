//! Initialization routine

use std::sync::Arc;

use log::Level;
use wgpu::Adapter;
use wgpu::Backends;
use wgpu::DeviceType;
use wgpu::DownlevelFlags;
use wgpu::Features;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::InstanceFlags;
use wgpu::MemoryBudgetThresholds;
use wgpu::PowerPreference;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderModel;
use wgpu::Surface;
use wgpu::wgt::DeviceDescriptor;

use crate::INSTANCE;
use crate::RENDERER;
use crate::Renderer;
use crate::instance;
use crate::label;
use crate::log;
use crate::surfaces::init_surface_map;

macro_rules! dump_bitflag_string {
    ($flags:expr) => {{
        let mut flag_names = Vec::new();

        for (flag_name, _) in $flags.iter_names() {
            flag_names.push(flag_name);
        }

        flag_names.sort_unstable();

        let flag_string: String = format!("\t{}", flag_names.join("\n\t"));

        flag_string
    }};
}

/// Initializes the [`Instance`] based on the config of the renderer
pub fn init_instance() {
    log::debug!("Initializing WGPU instance");

    log::trace!(
        "Available graphics backends in build: {}",
        backends_to_str(wgpu::Backends::all())
    );

    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        display: None,
        flags: InstanceFlags::default(),
        memory_budget_thresholds: MemoryBudgetThresholds::default(),
        backend_options: default_backend_options(),
    });

    INSTANCE
        .set(instance)
        .expect("Rendering instance already initialized");

    init_surface_map();
}

/// Requests a new [`Adapter`] compatible with the provided [`Surface`] and returns the future that provides it
pub fn init_adapter(
    surface: &Surface<'_>,
) -> impl Future<Output = Option<(Adapter, Features)>> + use<> {
    log::debug!("Initializing adapter");

    let instance = instance();
    let req_adapter_fut = instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(surface),
        apply_limit_buckets: false,
    });

    async {
        let adapter = match req_adapter_fut.await {
            Ok(a) => a,
            Err(e) => {
                log::error!("Failed to find a graphics adapter: {e}");
                return None;
            }
        };

        let adapter_info = adapter.get_info();

        log::info!(
            "Found graphics device '{}' with backend '{}' and driver '{} {}'",
            adapter_info.name,
            adapter_info.backend,
            adapter_info.driver,
            adapter_info.driver_info
        );

        if !adapter_supported(&adapter) {
            return None;
        }

        // We request all features supported by the device, excluding "unwanted" ones.
        let adapter_requested_features = adapter
            .features()
            .intersection(all_wanted_features(adapter_info.device_type));

        if log::log_enabled!(Level::Debug) {
            let features_string = dump_bitflag_string!(adapter_requested_features);
            log::debug!("Supported features:\n{features_string}");
        }

        Some((adapter, adapter_requested_features))
    }
}

fn adapter_supported(adapter: &Adapter) -> bool {
    // The base compliant flags, except for `SURFACE_VIEW_FORMATS`, which is
    // unsupported on Vulkan on Android. Requiring it would lock us out of Android
    // support
    const REQUIRED_FLAGS: DownlevelFlags =
        DownlevelFlags::compliant().difference(DownlevelFlags::SURFACE_VIEW_FORMATS);

    let downlevel_capabilities = adapter.get_downlevel_capabilities();

    if downlevel_capabilities.shader_model < ShaderModel::Sm5 {
        log::error!(
            "Adapter does not support at least shader model 5: {:?}",
            downlevel_capabilities.shader_model
        );
        return false;
    }

    if !downlevel_capabilities.flags.contains(REQUIRED_FLAGS) {
        let missing = REQUIRED_FLAGS.difference(downlevel_capabilities.flags);

        let missing_string = dump_bitflag_string!(missing);

        log::error!(
            "Adapter does not support the required minimum compliant feature set. Missing features:\n{missing_string}"
        );
        return false;
    }

    true
}

/// Initializes the renderer, making sure it supports at least the given surface
pub async fn init_renderer(adapter: Adapter, adapter_features: Features) -> bool {
    log::debug!("Initializing renderer");

    let (device, queue) = match adapter
        .request_device(&DeviceDescriptor {
            label: label!("WutEngine Main GPU"),
            required_features: adapter_features,
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
        .set(Renderer::new(adapter, device, queue))
        .expect("Renderer already initialized");

    true
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
