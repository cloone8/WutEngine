use core::fmt::Debug;
use std::path::PathBuf;

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
    /// The config file to use. If [None], will not load any dynamic configuration, and will
    /// always return the default values instead
    pub config_file: Option<PathBuf>,

    /// Fixed timestep in nanoseconds
    pub initial_window: Option<InitialWindowConfig>,

    /// The asset loader implementation that this runtime will use
    pub asset_loader: Box<dyn AssetLoader>,

    /// The asset format that the runtime will assume any loaded assets were serialized in
    pub asset_format: AssetSerializationFormat,

    /// The post-initialization callback. Called once the runtime was started and the first native OS event was sent
    pub post_init: Option<Box<dyn FnOnce()>>,
}

impl Debug for StaticRuntimeConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WutEngineConfig")
            .field("config_file", &self.config_file)
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
            config_file: Some(PathBuf::from("wutengine.toml")),
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
