#![doc = include_str!("../README.md")]

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::time::Duration;
use std::sync::OnceLock;

pub use ::log::Level;
pub use ::log::LevelFilter;
pub use ::log::Metadata;
pub use ::log::Record;
pub use ::log::debug;
pub use ::log::error;
pub use ::log::info;
pub use ::log::log;
pub use ::log::trace;
pub use ::log::warn;
use log::Log;

cfg_select! {
    target_arch = "wasm32" => {
        use web_time::Instant;
    }
    _ => {
        use std::time::Instant;
    }
}

use wutengine_sync::RwLock;

mod outputs;

pub use outputs::*;

/// A single log message
#[derive(Debug, Clone)]
pub struct LogEntry<'a> {
    /// The level
    pub level: Level,

    /// The "target". Either the module path (default) or `target: "my_target"` as given to the log macro's
    pub category: &'a str,

    /// The message
    pub message: String,

    /// The file, if known
    pub file: Option<&'a str>,

    /// The line within [`Self::file`], if known
    pub line: Option<u32>,

    /// The time instant
    pub time: Instant,
}

/// Something that can be used as an output for [the WutEngine logger](crate)
pub trait LoggerOutput: Send + Sync + 'static {
    /// Write a new log entry to this output
    fn write(&self, entry: &LogEntry);

    /// Flush the logger output
    #[inline]
    fn flush(&self) {}

    /// Same as [`flush`](Self::flush), but called only when panicing. Must never block
    #[inline]
    fn flush_on_panic(&self) {
        self.flush();
    }

    /// The most verbose level this output wants
    #[inline]
    fn max_level(&self) -> LevelFilter {
        ::log::STATIC_MAX_LEVEL
    }
}

/// Settings for the [WutEngine logger](crate)
#[derive(Debug, Clone)]
pub struct LogSettings {
    /// The default log level, if a more specific one was not set
    pub default_level: LevelFilter,

    /// Category log levels. Longest matching prefix wins
    pub categories: Vec<(String, LevelFilter)>,
}

impl LogSettings {
    /// The default log settings. `debug` level for builds with `debug_assertions` set, `info` otherwise
    #[must_use]
    pub const fn new() -> Self {
        Self {
            default_level: if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
            categories: Vec::new(),
        }
    }

    /// Returns the highest possible level configured
    #[must_use]
    #[inline]
    pub fn highest_level(&self) -> LevelFilter {
        self.default_level.max(
            self.categories
                .iter()
                .map(|(_, filter)| *filter)
                .max()
                .unwrap_or(LevelFilter::Off),
        )
    }
}

impl Default for LogSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// The global logger manager. Does not log anything by itself, but forwards to
/// a set of arbitrary outputs
#[derive(derive_more::Debug)]
struct WutEngineLogger {
    /// The currently active settings
    settings: RwLock<LogSettings>,

    /// The actual log output sinks
    #[debug(skip)]
    outputs: RwLock<Vec<Box<dyn LoggerOutput>>>,
}

impl WutEngineLogger {
    /// Gets the configured level filter for the category
    fn get_category_level_filter(&self, category: &str) -> LevelFilter {
        let settings = self.settings.read();

        let mut filter = settings.default_level;
        let mut longest_prefix = 0;

        for (cat_prefix, cat_level) in &settings.categories {
            if cat_prefix.len() > longest_prefix && category.starts_with(cat_prefix.as_str()) {
                longest_prefix = cat_prefix.len();
                filter = *cat_level;
            }
        }

        filter
    }
}

impl Log for WutEngineLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= ::log::STATIC_MAX_LEVEL && metadata.level() <= ::log::max_level()
    }

    fn log(&self, record: &Record) {
        let metadata = record.metadata();

        if !self.enabled(metadata) {
            return;
        }

        let category = metadata.target();

        let filter = self.get_category_level_filter(category);

        if record.level() > filter {
            return;
        }

        let entry = LogEntry {
            level: metadata.level(),
            category,
            message: record.args().to_string(), // TODO: Do this without allocating by keeping a per-thread String buffer or something?
            file: record.file(),
            line: record.line(),
            time: Instant::now(),
        };

        for output in &*self.outputs.read() {
            if record.level() <= output.max_level() {
                output.write(&entry);
            }
        }
    }

    #[inline]
    fn flush(&self) {
        flush();
    }
}

/// The global logger
static LOGGER: WutEngineLogger = WutEngineLogger {
    settings: RwLock::new(LogSettings::new()),
    outputs: RwLock::new(Vec::new()),
};

/// The [`Instant`] when [`init`] was called, for resolving [`LogEntry::time`]
static LOG_INIT_TIME: OnceLock<Instant> = OnceLock::new();

/// Installs the logger. Call exactly once, as early as possible.
///
/// Panics when called twice
pub fn init(settings: LogSettings, outputs: Vec<Box<dyn LoggerOutput>>) {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if INITIALIZED.swap(true, Ordering::Relaxed) {
        panic!("Logger was already initialized once!");
    }

    LOG_INIT_TIME
        .set(Instant::now())
        .expect("Init time already set");

    let highest_level = settings.highest_level();

    *LOGGER.settings.write() = settings;
    *LOGGER.outputs.write() = outputs;

    if ::log::set_logger(&LOGGER).is_err() {
        // A logger was already set by someone else. Assume we can log
        log::info!("WutEngine logger not enabled because another logger is already active");
        return;
    }

    ::log::set_max_level(highest_level);
}

/// Adds another output to the logging system
pub fn add_output(output: Box<dyn LoggerOutput>) {
    LOGGER.outputs.write().push(output);
}

/// Replaces the current logging settings
pub fn set_settings(settings: LogSettings) {
    let highest_level = settings.highest_level();

    *LOGGER.settings.write() = settings;

    ::log::set_max_level(highest_level);
}

/// Flush all log outputs
pub fn flush() {
    for output in &*LOGGER.outputs.read() {
        output.flush();
    }
}

/// Flush all log outputs during a panic. Might not be able to flush all outputs,
/// so only use during a panic (usually from the panic hook). For normal flushing, see [`flush`]
pub fn flush_on_panic() {
    let Some(outputs) = LOGGER.outputs.try_read() else {
        return;
    };

    for output in &*outputs {
        output.flush_on_panic();
    }
}

/// Returns the time span between when [`init`] was called and `time`.
/// Panics if called before [`init`] has returned
#[inline]
pub fn duration_since_logger_start(time: &Instant) -> Duration {
    let start_time = LOG_INIT_TIME
        .get()
        .expect("Cannot be called before the logger is initialized");

    time.duration_since(*start_time)
}
