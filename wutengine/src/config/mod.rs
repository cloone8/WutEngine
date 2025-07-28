use core::fmt::Debug;

use serde::{Deserialize, Serialize};
use wutengine_asset::{AssetLoader, AssetSerializationFormat, BasicAssetLoader};

use crate::window::WindowIdentifier;

pub use wutengine_config::*;

/// Baked-in runtime configuration
/// This struct contains the WutEngine configuration options
/// that cannot be loaded dynamically from a config file, because
/// they either contain implementations of engine systems or other callback-type
/// types
pub struct StaticRuntimeConfig {
    /// Fixed timestep in nanoseconds
    pub initial_window: Option<InitialWindowConfig>,
    pub asset_loader: Box<dyn AssetLoader>,
    pub asset_format: AssetSerializationFormat,
    pub post_init: Option<Box<dyn FnOnce()>>,
}

impl Debug for StaticRuntimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WutEngineConfig")
            .field("initial_window", &self.initial_window)
            .field("asset_loader", &self.asset_loader)
            .field("asset_format", &self.asset_format)
            .field(
                "post_init",
                if self.post_init.is_some() {
                    &"<has callback>"
                } else {
                    &"<no callback>"
                },
            )
            .finish()
    }
}

impl Default for StaticRuntimeConfig {
    fn default() -> Self {
        Self {
            initial_window: Some(InitialWindowConfig::default()),
            asset_loader: Box::new(BasicAssetLoader::default()),
            asset_format: AssetSerializationFormat::Text,
            post_init: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitialWindowConfig {
    pub id: WindowIdentifier,
    pub title: String,
    pub mode: InitialWindowMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitialWindowMode {
    Windowed,
    BorderlessFullscreen,
    ExclusiveFullscreen,
}

impl Default for InitialWindowConfig {
    fn default() -> Self {
        Self {
            id: WindowIdentifier::new("main".to_string()),
            title: "WutEngine".to_string(),
            mode: InitialWindowMode::BorderlessFullscreen,
        }
    }
}
