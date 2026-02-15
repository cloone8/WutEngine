//! Runtime and loadtime configuration management

use core::str::FromStr;
use std::path::Path;

use dashmap::DashMap;
use serde::Deserialize;
use smallvec::SmallVec;

use crate::util::InitOnce;

static CONFIG_MANAGER: InitOnce<ConfigManager> = InitOnce::new();

#[derive(Debug)]
struct ConfigManager {
    config: DashMap<String, toml::Value>,
}

pub(crate) fn init_and_load(path: Option<&Path>) {
    let initial_values = match path {
        Some(path) => load_from_file(path),
        None => DashMap::default(),
    };

    InitOnce::init(
        &CONFIG_MANAGER,
        ConfigManager {
            config: initial_values,
        },
    );
}

fn load_from_file(path: &Path) -> DashMap<String, toml::Value> {
    let config_file_content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            log::error!(
                "Could not read config file due to error: {e}. Returning default values for all config requests"
            );
            return DashMap::default();
        }
    };

    let config_toml = match toml::Table::from_str(&config_file_content) {
        Ok(config_toml) => config_toml,
        Err(e) => {
            log::error!(
                "Could not read config file due to error: {e}. Returning default values for all config requests"
            );
            return DashMap::default();
        }
    };

    config_toml.into_iter().collect()
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ConfigKeyErr {
    #[display(
        "Config key needs at least one main category and one subcategory: {}",
        _0
    )]
    NeedsSubcategory(#[error(not(source))] String),
}

fn validate_config_key(key: &str) -> Result<(&str, &str), ConfigKeyErr> {
    let Some((main_category, rest)) = key.split_once('.') else {
        log::warn!("Config key needs at least one main category and one subcategory: {key}");
        return Err(ConfigKeyErr::NeedsSubcategory(key.to_owned()));
    };

    Ok((main_category, rest))
}

#[inline]
pub fn get<'de, T>(key: &str) -> T
where
    T: Deserialize<'de> + Default,
{
    try_get(key).unwrap_or_default()
}

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

fn find_key<'a>(main_cat: &str, key: &str, tab: &'a toml::Table) -> Option<&'a toml::Value> {
    let keys = key.split('.').collect::<SmallVec<[_; 8]>>();
    let categories = &keys[..keys.len() - 1];
    let config_key = *&keys[keys.len() - 1];

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

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum SetConfigErr<T> {
    #[display("Configuration key is invalid: {}", _0)]
    InvalidKey(ConfigKeyErr),

    #[display("Could not convert input into a TOML value: {}", _0)]
    #[from(skip)]
    Convert(T),
}

#[inline]
pub fn set<T: TryInto<toml::Value>>(key: &str, value: T) -> Result<(), SetConfigErr<T::Error>> {
    profiling::function_scope!(key);

    let value = value.try_into().map_err(|e| SetConfigErr::Convert(e))?;

    set_raw(key, value)?;

    Ok(())
}

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

fn set_key(key: &str, category: &mut toml::Value, val: toml::Value) {
    let keys = key.split('.').collect::<Vec<_>>();
    let categories = &keys[..keys.len() - 1];
    let config_key = &keys[keys.len() - 1];

    let mut cur_category = category;

    for &category in categories {
        // Some divergent behaviour depending on the current type of the config key
        if cur_category.is_table() {
            // If the category is already a table, we continue to traverse down the table.

            let as_table = cur_category.as_table_mut().unwrap();

            if as_table.contains_key(category) {
                cur_category = as_table.get_mut(category).unwrap();
            } else {
                // We insert the new subcategory if it does not already exist
                as_table.insert(
                    category.to_owned(),
                    toml::Value::Table(toml::map::Map::default()),
                );

                cur_category = &mut as_table[category];
            }
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

    if cur_category.is_table() {
        let as_table = cur_category.as_table_mut().unwrap();

        if as_table.contains_key(*config_key) {
            as_table[*config_key] = val;
        } else {
            as_table.insert((*config_key).to_owned(), val);
        }
    } else {
        let mut new_table = toml::map::Map::default();
        new_table.insert((*config_key).to_owned(), val);

        *cur_category = toml::Value::Table(new_table);
    }
}
