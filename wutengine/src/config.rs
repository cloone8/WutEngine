use core::fmt::Debug;

use wutengine_asset::{AssetLoader, AssetSerializationFormat, BasicAssetLoader};

use crate::graphics::WutEngineBackend;
use crate::window::WindowIdentifier;

pub struct WutEngineConfig {
    pub fixed_timestep: f32,
    pub backends: WutEngineBackend,
    pub initial_window: Option<InitialWindowConfig>,
    pub asset_loader: Box<dyn AssetLoader>,
    pub asset_format: AssetSerializationFormat,
    pub post_init: Option<Box<dyn FnOnce()>>,
}

impl Debug for WutEngineConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WutEngineConfig")
            .field("fixed_timestep", &self.fixed_timestep)
            .field("backends", &self.backends)
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

impl Default for WutEngineConfig {
    fn default() -> Self {
        Self {
            fixed_timestep: 1.0 / 50.0,
            backends: WutEngineBackend::default(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
