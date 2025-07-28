use serde::Deserialize;

mod backend;

pub(crate) use backend::*;

use wgpu::InstanceFlags;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub(crate) struct WutEngineGraphicsConfig {
    pub(crate) backend: GraphicsBackend,
    pub(crate) debug_level: GraphicsValidationLevel,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GraphicsValidationLevel {
    Unsafe,

    #[cfg_attr(not(debug_assertions), default)]
    Basic,

    #[cfg_attr(debug_assertions, default)]
    Debug,

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
