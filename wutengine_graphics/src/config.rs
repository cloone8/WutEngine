//! Graphics config options

use serde::Deserialize;

/// Graphics context initialization config
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct GraphicsConfig {
    /// The backend to use
    pub(crate) backend: GraphicsBackend,

    /// Graphics debug mode
    pub(crate) debug: bool,

    /// Graphics validation mode
    pub(crate) validation: bool,

    /// Graphics GPU based validation mode.
    /// Very slow
    pub(crate) gpu_based_validation: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            backend: Default::default(),
            debug: cfg!(debug_assertions),
            validation: cfg!(debug_assertions),
            gpu_based_validation: false,
        }
    }
}

/// Configurable graphics backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, derive_more::Display)]
#[serde(rename_all = "lowercase")]
pub enum GraphicsBackend {
    /// Vulkan
    Vulkan,

    /// DirectX 12
    #[display("DirectX 12")]
    DX12,

    /// Metal
    Metal,

    /// WebGPU (browser only)
    WebGPU,
}

impl Default for GraphicsBackend {
    fn default() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::WebGPU
        } else if cfg!(windows) {
            Self::DX12
        } else if cfg!(any(target_os = "macos", target_os = "ios")) {
            Self::Metal
        } else if cfg!(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd"
        )) {
            Self::Vulkan
        } else {
            log::warn!(
                "Could not determine appropriate backends for current platform. Using Vulkan"
            );
            Self::Vulkan
        }
    }
}

impl From<GraphicsBackend> for wgpu::Backends {
    fn from(value: GraphicsBackend) -> Self {
        match value {
            GraphicsBackend::Vulkan => Self::VULKAN,
            GraphicsBackend::DX12 => Self::DX12,
            GraphicsBackend::Metal => Self::METAL,
            GraphicsBackend::WebGPU => Self::BROWSER_WEBGPU,
        }
    }
}
