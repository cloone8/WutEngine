use serde::Deserialize;

mod backend;

pub(crate) use backend::*;

use wgpu::InstanceFlags;

/// Graphics configuration
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub(crate) struct WutEngineGraphicsConfig {
    /// The graphics backends to use. If a backend is enabled here and was compiled
    /// into the runtime, the graphics manager will include graphics adapters exposing
    /// that backend into its search for an appropriate GPU
    pub(crate) backend: GraphicsBackend,

    /// The graphics layer debug validation level
    pub(crate) debug_level: GraphicsValidationLevel,

    /// Whether to ignore graphics layer validation errors and keep running
    pub(crate) ignore_errors: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GraphicsValidationLevel {
    /// No validation. Unsafe in the presence of engine bugs or
    /// manual graphics calls
    Unsafe,

    /// Default validation. Only safety checks
    #[cfg_attr(not(debug_assertions), default)]
    Basic,

    /// Debug validation. Checks for graphics calls for correctness
    #[cfg_attr(debug_assertions, default)]
    Debug,

    /// Advanced and slow debugging
    Advanced,
}

impl From<GraphicsValidationLevel> for wgpu::InstanceFlags {
    fn from(value: GraphicsValidationLevel) -> Self {
        match value {
            GraphicsValidationLevel::Unsafe => InstanceFlags::empty(),
            GraphicsValidationLevel::Basic => InstanceFlags::VALIDATION_INDIRECT_CALL,
            GraphicsValidationLevel::Debug => InstanceFlags::debugging(),
            GraphicsValidationLevel::Advanced => InstanceFlags::advanced_debugging(),
        }
    }
}
