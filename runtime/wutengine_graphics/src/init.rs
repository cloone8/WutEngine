//! Graphics initialization

use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use wutengine_util::InitOnce;

use crate::{
    ACTIVE_CONFIG, GFX_DEVICE, GraphicsRuntimeConfig, PIPELINE_CACHE, config::GraphicsConfig,
    features_supported, label,
};

/// Initializes the global graphics context for WutEngine. Acquires a graphics
/// device and configures it.
/// Returns `true` if graphics initialization was succesful, and `false` otherwise.
pub fn initialize_graphics_context() -> bool {
    log::debug!("Initializing graphics context");

    log::trace!(
        "Available graphics backends in build: {}",
        backends_to_str(wgpu::Backends::all())
    );

    let config = wutengine_config::get::<GraphicsConfig>("wutengine.graphics");

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
        apply_limit_buckets: false,
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

    // We request all features supported by the device, excluding "unwanted" ones.
    let adapter_requested_features = adapter
        .features()
        .intersection(super::all_wanted_features(adapter_info.device_type));

    if log::log_enabled!(log::Level::Debug) {
        let mut features_string = String::new();

        let mut features = Vec::new();

        for (feature, _) in adapter_requested_features.iter_names() {
            features.push(feature);
        }

        features.sort();

        for feature in features {
            features_string = format!("{features_string}\n\t{feature}");
        }

        log::debug!("Supported features:{features_string}");
    }

    let (device, queue) =
        match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: label!("WutEngine Main GPU"),
            required_features: adapter_requested_features,
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
    InitOnce::init(
        &super::ACTIVE_CONFIG,
        GraphicsRuntimeConfig {
            backend: config.backend,
            adapter: adapter_info,
            features: super::GFX_DEVICE.features(),
            limits: super::GFX_DEVICE.limits(),
        },
    );

    if features_supported(wgpu::Features::PIPELINE_CACHE) {
        log::debug!("Pipeline cache supported, trying to obtain one");
        InitOnce::init(&super::PIPELINE_CACHE, get_pipeline_cache());
    } else {
        log::debug!("Pipeline cache not supported");
        InitOnce::init(&super::PIPELINE_CACHE, None);
    }

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

fn gather_instance_flags(config: &GraphicsConfig) -> wgpu::InstanceFlags {
    let mut flags = wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL;

    flags.set(wgpu::InstanceFlags::DEBUG, config.debug);
    flags.set(wgpu::InstanceFlags::VALIDATION, config.validation);
    flags.set(
        wgpu::InstanceFlags::GPU_BASED_VALIDATION,
        config.gpu_based_validation,
    );
    flags.set(
        wgpu::InstanceFlags::DISCARD_HAL_LABELS,
        cfg!(not(feature = "labels")),
    );

    if flags.contains(wgpu::InstanceFlags::GPU_BASED_VALIDATION) {
        log::warn!(
            "Using GPU based graphics validation. This can have a very significant performance impact"
        );
    }

    log::trace!("Flags: {:?}", flags);

    flags
}

fn get_pipeline_cache() -> Option<wgpu::PipelineCache> {
    profiling::function_scope!();

    match get_existing_pipeline_cache() {
        Ok(cache) => Some(cache),
        Err(e) => {
            if matches!(e, PipelineCacheErr::NotExists) {
                log::debug!("No pipeline cache exists on disk, creating new one: {e}");
            } else {
                log::error!(
                    "Existing pipeline on disk is corrupt or otherwise impossible to obtain, creating new one: {e}"
                );
            }

            // SAFETY: This should be safe because we're creating a fresh pipeline, not an existing
            // one
            let new_cache = match with_checked_errs(|| unsafe {
                super::GFX_DEVICE.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    label: label!("WutEngine Pipeline Cache"),
                    data: None,
                    fallback: false,
                })
            }) {
                Ok(cache) => cache,
                Err(e) => {
                    log::warn!("Failed to create new pipeline cache due to error: {e}");
                    return None;
                }
            };

            Some(new_cache)
        }
    }
}

#[derive(Debug, derive_more::Display, derive_more::From, derive_more::Error)]
enum PipelineCacheErr {
    NotExists,
    UnknownAdapterKey,
    UnknownExecutableName,
    NoCacheDir,
    Invalid,
    IO(std::io::Error),
}

fn get_pipeline_cache_path() -> Result<PathBuf, PipelineCacheErr> {
    let adapter_cache_key = wgpu::util::pipeline_cache_key(&ACTIVE_CONFIG.adapter)
        .ok_or(PipelineCacheErr::UnknownAdapterKey)?;

    let mut cache_filename = OsString::new();

    cache_filename.push(
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.file_stem().map(OsStr::to_os_string))
            .ok_or(PipelineCacheErr::UnknownExecutableName)?,
    );
    cache_filename.push("_");
    cache_filename.push(adapter_cache_key);
    cache_filename.push(".we-pipelinecache");

    let sys_cache_dir = dirs::cache_dir().ok_or(PipelineCacheErr::NoCacheDir)?;

    Ok(sys_cache_dir
        .join("WutEngine")
        .join("pipeline_cache")
        .join(cache_filename))
}

fn get_existing_pipeline_cache() -> Result<wgpu::PipelineCache, PipelineCacheErr> {
    profiling::function_scope!();

    let cache_path = get_pipeline_cache_path()?;

    log::debug!(
        "Trying to find existing pipeline cache at path {}",
        cache_path.to_string_lossy()
    );

    if !(std::fs::exists(&cache_path)?) || !(std::fs::metadata(&cache_path)?.is_file()) {
        return Err(PipelineCacheErr::NotExists);
    }

    let cache_bytes = std::fs::read(&cache_path)?;

    // SAFETY: This _should_ come from a pipeline cache we saved to disk, due to the naming. This is almost
    // impossible to verify though, due to the filesystem not being under our control when we're not running
    let cache = match with_checked_errs(move || unsafe {
        GFX_DEVICE.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
            label: label!("WutEngine Pipeline Cache"),
            data: Some(&cache_bytes),
            fallback: false,
        })
    }) {
        Ok(cache) => cache,
        Err(e) => {
            log::warn!("Could not use existing pipeline cache due to error: {e}");
            return Err(PipelineCacheErr::Invalid);
        }
    };

    Ok(cache)
}

/// Persists the currently active pipeline cache to disk
pub fn persist_pipeline_cache() {
    let Some(cache_bytes) = PIPELINE_CACHE.as_ref().and_then(|pcache| pcache.get_data()) else {
        // Nothing to cache
        return;
    };

    let cache_path = match get_pipeline_cache_path() {
        Ok(cpath) => cpath,
        Err(e) => {
            log::error!(
                "Failed to determine pipeline cache path, so can't persist pipeline cache: {e}"
            );
            return;
        }
    };

    let Some(cache_parent_dir) = cache_path.parent() else {
        log::error!("Could not determine pipeline cache parent directory");
        return;
    };

    if let Err(e) = std::fs::create_dir_all(cache_parent_dir) {
        log::error!("Could not create pipeline cache directory: {e}");
        return;
    }

    if let Err(e) = std::fs::write(cache_path, cache_bytes) {
        log::error!("Could not write pipeline cache bytes to disk: {e}");
        return;
    }

    log::debug!("Succesfully persisted pipeline cache to disk");
}

/// Run the given function with checked errors. Does not check for OOM errors.
/// Slow. Use during initialization only.
fn with_checked_errs<T>(f: impl FnOnce() -> T) -> Result<T, wgpu::Error> {
    let es_val = GFX_DEVICE.push_error_scope(wgpu::ErrorFilter::Validation);
    let es_int = GFX_DEVICE.push_error_scope(wgpu::ErrorFilter::Internal);

    let val = f();

    if let Some(e) = pollster::block_on(es_int.pop()) {
        return Err(e);
    }

    if let Some(e) = pollster::block_on(es_val.pop()) {
        return Err(e);
    }

    Ok(val)
}
