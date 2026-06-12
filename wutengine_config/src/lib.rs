//! Runtime and loadtime configuration management

use core::str::FromStr;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

use dashmap::DashMap;
use serde::Deserialize;
use smallvec::SmallVec;

#[doc(inline)]
pub use toml;

use wutengine_util::InitOnce;

/// The global [ConfigManager]
static CONFIG_MANAGER: InitOnce<ConfigManager> = InitOnce::new();

/// A config manager
#[derive(Debug)]
struct ConfigManager {
    /// All config keys
    config: DashMap<String, toml::Value>,
}

/// Initialize the config manager, and loads the configuration from the
/// given path. If no path is provided, no initial configuration is loaded, and
/// all keys will return their default value
#[doc(hidden)]
pub fn init_and_load(path: Option<&Path>) -> Vec<(log::Level, String)> {
    //NOTE: The logger is not yet set up here, so we buffer any messages here and return them
    // to the caller so they can emit them later
    let mut messages = Vec::new();

    let initial_values = match path {
        Some(path) => load_from_file(path, &mut messages),
        None => DashMap::default(),
    };

    InitOnce::init(
        &CONFIG_MANAGER,
        ConfigManager {
            config: initial_values,
        },
    );

    messages
}

/// Loads an initial config map from a path
///
/// NOTE: You cannot call macro's from [log] here because the logger
/// hasn't been set up yet. Instead, place them in `log_messages`
fn load_from_file(
    path: &Path,
    log_messages: &mut Vec<(log::Level, String)>,
) -> DashMap<String, toml::Value> {
    let config_file_content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            log_messages.push((
                log::Level::Info,
                format!(
                    "Config file \"{}\" not found. Assuming default configuration",
                    path.to_string_lossy()
                ),
            ));

            return DashMap::default();
        }
        Err(e) => {
            log_messages.push((
                log::Level::Error,
                format!(
                    "Could not read config file \"{}\" due to I/O error: {e}. Returning default values for all config requests",
                    path.to_string_lossy()
                )
            ));

            return DashMap::default();
        }
    };

    let config_toml = match toml::Table::from_str(&config_file_content) {
        Ok(config_toml) => config_toml,
        Err(e) => {
            log_messages.push((
                log::Level::Error,
                format!(
                    "Could not parse config file due to error: {e}. Returning default values for all config requests"
                )
            ));
            return DashMap::default();
        }
    };

    config_toml.into_iter().collect()
}

/// An invalid config key
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ConfigKeyErr {
    #[display(
        "Config key needs at least one main category and one subcategory: {}",
        _0
    )]
    /// No subcategory was given
    NeedsSubcategory(#[error(not(source))] String),
}

/// Checks that a given config key string is valid, and returns a tuple of the main and subcategories
/// if so
fn validate_config_key(key: &str) -> Result<(&str, &str), ConfigKeyErr> {
    let Some((main_category, rest)) = key.split_once('.') else {
        log::warn!("Config key needs at least one main category and one subcategory: {key}");
        return Err(ConfigKeyErr::NeedsSubcategory(key.to_owned()));
    };

    Ok((main_category, rest))
}

/// Returns the value of the given configuration option, if it exists.
/// If not, returns the default value.
///
/// If the default value is not desired, see [try_get]
#[inline]
pub fn get<'de, T>(key: &str) -> T
where
    T: Deserialize<'de> + Default,
{
    try_get(key).unwrap_or_default()
}

/// Returns the value of the given configuration option, if it exists.
///
/// If options that are not set should return their [Default], see [get]
pub fn try_get<'de, T>(key: &str) -> Option<T>
where
    T: Deserialize<'de>,
{
    profiling::function_scope!(key);

    let raw = get_raw(key)?;

    match raw.try_into::<T>() {
        Ok(deser) => Some(deser),
        Err(e) => {
            log::warn!(
                "Key '{key}' could not be deserialized as a value of type '{}': {e}",
                core::any::type_name::<T>()
            );
            None
        }
    }
}

/// Returns the raw [toml::Value] of a given config key, if it exists.
///
/// For automatic deserialization, see [try_get] or [get]
pub fn get_raw(key: &str) -> Option<toml::Value> {
    profiling::function_scope!(key);

    let (main_category, rest) = match validate_config_key(key) {
        Ok(split) => split,
        Err(e) => {
            log::warn!("Invalid config key: {e}");
            return None;
        }
    };

    let Some(subcategory_value) = CONFIG_MANAGER.config.get(main_category) else {
        log::debug!("Main category not found: {main_category}");
        return None;
    };

    let val = match subcategory_value.as_table() {
        Some(subcategory_table) => find_key(main_category, rest, subcategory_table),
        None => {
            if rest.contains('.') {
                log::warn!(
                    "Key '{key}' specifies a subcategory, but '{main_category}' is a main category with values only"
                );
                return None;
            } else {
                subcategory_value.get(rest)
            }
        }
    };

    val.cloned()
}

/// Finds a config key in the given table
fn find_key<'a>(main_cat: &str, key: &str, tab: &'a toml::Table) -> Option<&'a toml::Value> {
    let keys = key.split('.').collect::<SmallVec<[_; 8]>>();

    // All parts of the key before the last `.`
    let categories = &keys[..keys.len() - 1];

    // The final part of the key after the last `.`
    let config_key = keys[keys.len() - 1];

    let mut containing_category_name = main_cat;
    let mut cur_category = tab;

    for &category in categories {
        let Some(category_val) = cur_category.get(category) else {
            log::debug!("Could not find key {category} in category {containing_category_name}");
            return None;
        };

        let Some(table) = category_val.as_table() else {
            log::warn!(
                "Key {category} in category {containing_category_name} is not a subcategory, but a value of type '{}'",
                category_val.type_str()
            );
            return None;
        };

        cur_category = table;
        containing_category_name = category;
    }

    cur_category.get(config_key)
}

/// An error while trying to override a config option
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum SetConfigErr<T> {
    #[display("Configuration key is invalid: {}", _0)]
    /// Key was invalid
    InvalidKey(ConfigKeyErr),

    #[display("Could not convert input into a TOML value: {}", _0)]
    #[from(skip)]
    /// Serialization into TOML failed
    Convert(T),
}

/// Sets the given config option to the provided value.
#[inline]
pub fn set<T: TryInto<toml::Value>>(key: &str, value: T) -> Result<(), SetConfigErr<T::Error>> {
    profiling::function_scope!(key);

    let value = value.try_into().map_err(SetConfigErr::Convert)?;

    set_raw(key, value)?;

    Ok(())
}

/// Sets the given config option to the provided raw TOML value.
pub fn set_raw(key: &str, value: toml::Value) -> Result<(), ConfigKeyErr> {
    profiling::function_scope!(key);

    let (main_category, rest) = validate_config_key(key)?;

    let mut subcategory_value = CONFIG_MANAGER
        .config
        .entry(main_category.to_owned())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::default()));

    set_key(rest, &mut subcategory_value, value);

    Ok(())
}

/// Sets a config key in the given category table
fn set_key(key: &str, category: &mut toml::Value, val: toml::Value) {
    let keys = key.split('.').collect::<SmallVec<[_; 8]>>();

    // All parts of the key before the final `.`
    let categories = &keys[..keys.len() - 1];

    // The final string after the last `.`
    let config_key = keys[keys.len() - 1];

    let mut cur_category = category;

    for &category in categories {
        // Some divergent behaviour depending on the current type of the config key
        if cur_category.is_table() {
            // If the category is already a table, we continue to traverse down the table.

            let as_table = cur_category.as_table_mut().unwrap();

            if !as_table.contains_key(category) {
                as_table.insert(
                    category.to_owned(),
                    toml::Value::Table(toml::map::Map::default()),
                );
            }

            cur_category = &mut as_table[category];
        } else {
            // If the value is not a table, we make it one and remove the existing values

            let mut new_table = toml::map::Map::default();
            new_table.insert(
                category.to_owned(),
                toml::Value::Table(toml::map::Map::default()),
            );

            *cur_category = toml::Value::Table(new_table);

            cur_category = &mut cur_category[category];
        }
    }

    // We've traversed down the config tree, inserting new tables (or modifying entries into tables) while doing it.
    // Now we insert the actual value at the final part of the key
    if cur_category.is_table() {
        let as_table = cur_category.as_table_mut().unwrap();

        if as_table.contains_key(config_key) {
            as_table[config_key] = val;
        } else {
            as_table.insert(config_key.to_owned(), val);
        }
    } else {
        let mut new_table = toml::map::Map::default();
        new_table.insert(config_key.to_owned(), val);

        *cur_category = toml::Value::Table(new_table);
    }
}

/// Creates a clone of the full config table
pub fn get_all() -> HashMap<String, toml::Value> {
    CONFIG_MANAGER
        .config
        .iter()
        .map(|dm| (dm.key().clone(), dm.value().clone()))
        .collect()
}
