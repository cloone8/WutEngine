#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use hashbrown::HashMap;
use std::sync::RwLock;

use wutengine_util::map;

/// A clonable [logger](log::Log)
pub type LoggerImplementation = Arc<dyn log::Log>;

/// Function that produces [LoggerImplementation]s
pub type CreateLoggerFunction =
    dyn Fn(log::LevelFilter, &str, bool) -> LoggerImplementation + Send + Sync;

/// Struct that produces per-subsytem loggers, for use in the wutengine [ModuleLogger]
#[derive(derive_more::Debug)]
pub struct LoggerFactory {
    /// Called by the [ModuleLogger] when either a new subsystem is encountered, or the configuration for
    /// a subsystem has changed
    #[debug(skip)]
    pub create_logger: Box<CreateLoggerFunction>,
}

/// [log::Log] implementation that contains multiple loggers, one for each subsystem
#[derive(derive_more::Debug)]
pub struct ModuleLogger {
    /// Factory that produces the actual logger implementations
    factory: LoggerFactory,

    /// Map of loggers per subsystem
    #[debug(skip)]
    per_subsystem: RwLock<HashMap<String, Arc<dyn log::Log>>>,

    /// Default level for internal subsystems
    default_internal_level: log::LevelFilter,

    /// Default level for external subsystems
    default_external_level: log::LevelFilter,
}

impl ModuleLogger {
    /// Create a new [ModuleLogger]
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
                "config" => (factory.create_logger)(config_log_level, "config", true)
            ]),
            factory,
            default_internal_level,
            default_external_level,
        }
    }

    /// Retrieves the specific subsystem logger for the given target. If it does not yet exist,
    /// creates it.
    ///
    /// NOTE: [Self::per_subsystem] must be unlocked to avoid a deadlock
    fn with_subsys_logger<T>(&self, target: &str, cb: impl FnOnce(Arc<dyn log::Log>) -> T) -> T {
        let (is_wutengine, subsystem) = subsystem_from_target(target);

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
        let logger = (self.factory.create_logger)(level, subsystem, is_wutengine);

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

/// Returns the first module in the given module string like `mod_a::mod_b::mod_c` or `mod_a`
fn first_module(full: &str) -> &str {
    if let Some((module, _)) = full.split_once("::") {
        module
    } else {
        full
    }
}

/// Given a target from a [log::Metadata], returns the subsystem string and a
/// boolean denoting whether the target is internal (`true`) or external (`false`)
pub fn subsystem_from_target(target: &str) -> (bool, &str) {
    if let Some(no_wutengine_prefix) = target.strip_prefix("wutengine::") {
        return (true, first_module(no_wutengine_prefix));
    }

    if let Some(no_wutengine_crate_prefix) = target.strip_prefix("wutengine_") {
        return (true, first_module(no_wutengine_crate_prefix));
    }

    let first_module = first_module(target);

    if is_internally_used_external_crate(first_module) {
        return (true, first_module);
    }

    (false, first_module)
}

/// Returns true for crates that are used internally by WutEngine but do not start with `wutengine`
#[allow(clippy::match_like_matches_macro, reason = "Cleaner")]
fn is_internally_used_external_crate(first_module: &str) -> bool {
    match first_module {
        "naga" => true,
        "wgpu_hal" => true,
        "gilrs" => true,
        "wgpu_core" => true,
        "symphonia_core" => true,
        _ => false,
    }
}
