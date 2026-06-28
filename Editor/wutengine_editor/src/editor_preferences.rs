//! Per-user preferences, persistent between different projects

use alloc::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;
use wutengine::profiling;

/// Event fired when an editor preference was changed [set]
pub(crate) struct EditorPrefChanged {
    /// The key that was changed
    pub key: String,

    /// The new value. [None] means the key was deleted
    pub value: Option<serde_json::Value>,
}

/// Sets a global editor preference to the given value
pub(crate) fn set<T: Serialize>(key: &str, value: T) {
    profiling::function_scope!();

    log::debug!("Setting user preference {key}");

    let new_pref_value = match serde_json::to_value(value) {
        Ok(val) => val,
        Err(e) => {
            log::error!(
                "Failed to serialize preference {key} with value of type {} to JSON: {e}",
                core::any::type_name::<T>()
            );
            return;
        }
    };

    let mut cur_prefs = get_stored_preferences();

    cur_prefs.insert(key.to_string(), new_pref_value.clone());

    if let Err(e) = store_preferences(&cur_prefs) {
        log::error!("Failed to save preference {key} to file: {e}");
        return;
    }

    wutengine::event::publish(EditorPrefChanged {
        key: key.to_string(),
        value: Some(new_pref_value),
    });
}

/// Returns the stored setting for the given editor preference, or returns the [Default::default].
/// For a custom default value, see [get_or]
#[inline]
pub(crate) fn get<T: DeserializeOwned + Default>(key: &str) -> T {
    profiling::function_scope!();

    get_or(key, T::default())
}

/// Returns the stored setting for the given editor preference, or returns `default`.
/// To return the [Default::default] instead of a custom one, see [get]
pub(crate) fn get_or<T: DeserializeOwned>(key: &str, default: T) -> T {
    profiling::function_scope!();

    let prefs = get_stored_preferences();

    let Some(stored) = prefs.get(key).cloned() else {
        return default;
    };

    match serde_json::from_value(stored) {
        Ok(val) => val,
        Err(e) => {
            log::error!(
                "Failed to convert stored preference {key} as type {}: {e}",
                core::any::type_name::<T>()
            );
            default
        }
    }
}

/// Deletes a stored editor preference value
pub(crate) fn delete(key: &str) {
    profiling::function_scope!();

    log::info!("Deleting user preference for {key}");

    let mut stored = get_stored_preferences();

    let prev = stored.remove(key);

    if prev.is_some() {
        wutengine::event::publish(EditorPrefChanged {
            key: key.to_string(),
            value: None,
        });
    }
}

/// Returns the path to the preferences file, or an error if the path could not be determined
fn prefs_file_path() -> Result<PathBuf, std::io::Error> {
    dirs::preference_dir()
        .map(|prefs_dir| prefs_dir.join("WutEngine").join("editor_preferences.json"))
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::Unsupported))
}

/// Creates an empty preferences file, if it does not exist
fn create_prefs_file() -> Result<(), std::io::Error> {
    profiling::function_scope!();

    let prefs_file_path = prefs_file_path()?;

    if std::fs::exists(&prefs_file_path)? {
        return Ok(());
    }

    let prefs_file_dir = prefs_file_path
        .parent()
        .expect("Preferences file should be stored in a directory");

    std::fs::create_dir_all(prefs_file_dir)?;

    std::fs::write(prefs_file_path, "{}")
}

fn store_preferences(prefs: &BTreeMap<String, serde_json::Value>) -> Result<(), std::io::Error> {
    profiling::function_scope!();

    create_prefs_file()?;

    let prefs_file_path = prefs_file_path()?;

    let as_string = serde_json::to_string_pretty(prefs).expect("Failed to serialize preferences");

    std::fs::write(prefs_file_path, as_string)
}

fn get_stored_preferences() -> BTreeMap<String, serde_json::Value> {
    profiling::function_scope!();

    if let Err(e) = create_prefs_file() {
        log::error!("Failed to create editor preferences file, defaults will be returned: {e}");
        return BTreeMap::default();
    }

    let prefs_file_path = match prefs_file_path() {
        Ok(pfp) => pfp,
        Err(e) => {
            log::error!(
                "Failed to get editor preferences file path, defaults will be returned: {e}"
            );
            return BTreeMap::default();
        }
    };

    let prefs_string = match std::fs::read_to_string(&prefs_file_path) {
        Ok(prefs) => prefs,
        Err(e) => {
            log::error!("Failed to read editor preferences file, defaults will be returned: {e}");
            return BTreeMap::default();
        }
    };

    let prefs_map: BTreeMap<String, serde_json::Value> = match serde_json::from_str(&prefs_string) {
        Ok(prefs) => prefs,
        Err(e) => {
            log::error!(
                "Failed to deserialize editor preferences file. It might be corrupt. Defaults will be returned: {e}"
            );
            return BTreeMap::default();
        }
    };

    prefs_map
}
