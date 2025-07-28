//! WutEngine runtime configuration and config management

use core::str::FromStr;
use std::collections::HashMap;
use std::path::Path;

use wutengine_util::GlobalManager;

/// The config manager singleton
static CONFIG_MANAGER: GlobalManager<ConfigManager> = GlobalManager::new();

/// The main config manager. Basically just stores a hashmap of config values
struct ConfigManager {
    /// The top-level config
    config: HashMap<String, toml::Value>,
}

impl ConfigManager {
    /// Reads the config file from the given path,
    fn read_config(&mut self, path: &Path) {
        let config_file_content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                log::error!(
                    "Could not read config file due to error: {e}. Returning default values for all config requests"
                );
                return;
            }
        };

        let config_toml = match toml::Table::from_str(&config_file_content) {
            Ok(config_toml) => config_toml,
            Err(e) => {
                log::error!(
                    "Could not read config file due to error: {e}. Returning default values for all config requests"
                );
                return;
            }
        };

        self.config = config_toml.into_iter().collect();
    }
}

/// Initializes the configuration manager. Should not be called manually, and is automatically called
/// by the WutEngine runtime upon startup
#[doc(hidden)]
pub fn init(config_path: Option<&Path>) {
    let mut config_manager = ConfigManager {
        config: HashMap::default(),
    };

    match config_path {
        Some(path) => {
            config_manager.read_config(path);
        }
        None => {
            log::info!("No config file given. Returning default values for all config requests");
        }
    }

    GlobalManager::init(&CONFIG_MANAGER, config_manager);
}

/// Returns a wutengine-internal configuration value. Mainly used by the engine runtime.
/// For reading custom per-application config values instead, see [get] and [try_get]
#[doc(hidden)]
pub fn get_wutengine<'de, T>(value: &str) -> T
where
    T: serde::Deserialize<'de>,
    T: Default,
{
    get("wutengine", value)
}

/// Returns a config value from the given category and key. If it does not exist
/// or is otherwise corrupt, returns [Default::default] for `T` instead
pub fn get<'de, T>(category: &str, key: &str) -> T
where
    T: serde::Deserialize<'de>,
    T: Default,
{
    try_get(category, key).unwrap_or_default()
}

/// Returns a config value from the given category and key. If it does not exist
/// or is otherwise corrupt, returns [None] instead
pub fn try_get<'de, T>(category: &str, key: &str) -> Option<T>
where
    T: serde::Deserialize<'de>,
{
    CONFIG_MANAGER
        .config
        .get(category)
        .and_then(|cat| cat.get(key))
        .and_then(|val| match val.clone().try_into::<T>() {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                log::error!(
                    "Could not parse config value {}.{} as type {}: {}",
                    category,
                    key,
                    core::any::type_name::<T>(),
                    e
                );
                None
            }
        })
}
