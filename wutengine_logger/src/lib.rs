#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use std::collections::HashMap;
use std::sync::RwLock;

use wutengine_util::map;

#[derive(derive_more::Debug)]
pub struct LoggerFactory {
    #[debug(skip)]
    pub create_logger:
        Box<dyn Fn(log::LevelFilter, Option<(&str, bool)>) -> Arc<dyn log::Log> + Send + Sync>,
}

pub struct ModuleLogger {
    factory: LoggerFactory,
    per_subsystem: RwLock<HashMap<String, Arc<dyn log::Log>>>,
    default_internal_level: log::LevelFilter,
    default_external_level: log::LevelFilter,
}

impl ModuleLogger {
    pub fn new(
        factory: LoggerFactory,
        default_internal_level: log::LevelFilter,
        default_external_level: log::LevelFilter,
    ) -> Self {
        // Insert the config logger by default because otherwise the initialization
        // of the config logger calls to config, which then logs and tries to configure
        // the config logger, which deadlocks

        let config_log_level = wutengine_config::try_get("wutengine.log.subsystem.config.level")
            .unwrap_or(default_internal_level);

        Self {
            per_subsystem: RwLock::new(map![
                "config" => (factory.create_logger)(config_log_level, Some(("config", true)))
            ]),
            factory,
            default_internal_level,
            default_external_level,
        }
    }

    /// Given a target from a [log::Metadata], returns the subsystem string and a
    /// boolean denoting whether the target is internal or external
    fn subsystem_from_target(target: &str) -> (bool, &str) {
        if let Some(no_wutengine_prefix) = target.strip_prefix("wutengine::") {
            return (true, Self::first_module(no_wutengine_prefix));
        }

        if let Some(no_wutengine_crate_prefix) = target.strip_prefix("wutengine_") {
            return (true, Self::first_module(no_wutengine_crate_prefix));
        }

        let first_module = Self::first_module(target);

        if is_internally_used_external_crate(first_module) {
            return (true, first_module);
        }

        (false, Self::first_module(target))
    }

    /// Returns the first module in the given module string like `mod_a::mod_b::mod_c` or `mod_a`
    fn first_module(full: &str) -> &str {
        if let Some((module, _)) = full.split_once("::") {
            module
        } else {
            full
        }
    }

    /// Retrieves the specific subsystem logger for the given target. If it does not yet exist,
    /// creates it.
    ///
    /// NOTE: [Self::per_subsystem] must be unlocked to avoid a deadlock
    fn with_subsys_logger<T>(&self, target: &str, cb: impl FnOnce(Arc<dyn log::Log>) -> T) -> T {
        let (is_wutengine, subsystem) = Self::subsystem_from_target(target);

        let subsystems = self.per_subsystem.read().unwrap();

        if let Some(logger) = subsystems.get(subsystem).cloned() {
            drop(subsystems);
            return cb(logger);
        };

        drop(subsystems);

        self.insert_new_logger_for_subsystem(subsystem, is_wutengine);

        let subsystems = self.per_subsystem.read().unwrap();

        let logger = subsystems
            .get(subsystem)
            .cloned()
            .expect("Failed to get newly inserted logger");

        drop(subsystems);

        cb(logger)
    }

    /// NOTE: Must be called without the current thread holding a lock on the `per_subsystem` field
    fn insert_new_logger_for_subsystem(&self, subsystem: &str, is_wutengine: bool) {
        profiling::function_scope!();

        // Get the level first to prevent deadlocks
        let level =
            wutengine_config::try_get(&format!("wutengine.log.subsystem.{subsystem}.level"))
                .unwrap_or(if is_wutengine {
                    self.default_internal_level
                } else {
                    self.default_external_level
                });

        let mut subsystems = self.per_subsystem.write().unwrap();

        // Check if someone else inserted the logger in the meantime
        if subsystems.contains_key(subsystem) {
            return;
        }

        // Else, insert a new one
        let logger = (self.factory.create_logger)(level, Some((subsystem, is_wutengine)));

        subsystems.insert(subsystem.to_string(), logger);
    }
}

impl log::Log for ModuleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.with_subsys_logger(metadata.target(), |logger| logger.enabled(metadata))
    }

    fn log(&self, record: &log::Record) {
        self.with_subsys_logger(record.metadata().target(), |logger| logger.log(record))
    }

    fn flush(&self) {
        self.per_subsystem
            .read()
            .unwrap()
            .iter()
            .for_each(|(_, logger)| logger.flush());
    }
}

/// Returns true for crates that are used internally by WutEngine but do not start with `wutengine`
#[allow(clippy::match_like_matches_macro, reason = "Cleaner")]
fn is_internally_used_external_crate(first_module: &str) -> bool {
    match first_module {
        "naga" => true,
        "wgpu_hal" => true,
        "gilrs" => true,
        "wgpu_core" => true,
        _ => false,
    }
}
