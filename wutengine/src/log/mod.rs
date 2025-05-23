//! Logging and logging configuration

use core::str::FromStr;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

pub use log::*;
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, SharedLogger, TermLogger, TerminalMode,
    ThreadLogMode, WriteLogger,
};

/// Configuration for WutEngine logging.
#[derive(Debug)]
pub struct LogConfig {
    /// The log configuration for the core runtime.
    pub runtime: Option<ComponentLogConfig>,

    /// The log configuration for the rendering backend.
    pub renderer: Option<ComponentLogConfig>,

    /// The log configuration for anything else.
    /// This includes user-created code.
    pub other: Option<ComponentLogConfig>,
}

/// A log configuration for a specific WutEngine component.
#[derive(Debug, Clone)]
pub struct ComponentLogConfig {
    /// The minimum level a log needs
    /// to have before it is actually logged.
    pub min_level: LevelFilter,

    /// Where the log will end up
    pub output: LogOutput,
}

/// A log output target
#[derive(Debug, Clone)]
pub enum LogOutput {
    /// Logged to stdout
    StdOut,

    /// Logged to stderr
    StdErr,

    /// Logged to a file at a given path
    File(PathBuf),
}

impl Default for LogConfig {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self {
                runtime: Some(ComponentLogConfig {
                    min_level: LevelFilter::Info,
                    output: LogOutput::StdOut,
                }),
                renderer: Some(ComponentLogConfig {
                    min_level: LevelFilter::Warn,
                    output: LogOutput::StdOut,
                }),
                other: Some(ComponentLogConfig {
                    min_level: LevelFilter::Debug,
                    output: LogOutput::StdOut,
                }),
            }
        } else {
            Self {
                runtime: Some(ComponentLogConfig {
                    min_level: LevelFilter::Info,
                    output: LogOutput::File(PathBuf::from_str("./wutengine_runtime.log").unwrap()),
                }),
                renderer: Some(ComponentLogConfig {
                    min_level: LevelFilter::Warn,
                    output: LogOutput::File(PathBuf::from_str("./wutengine_renderer.log").unwrap()),
                }),
                other: Some(ComponentLogConfig {
                    min_level: LevelFilter::Info,
                    output: LogOutput::File(PathBuf::from_str("./game.log").unwrap()),
                }),
            }
        }
    }
}

impl ComponentLogConfig {
    fn make_simplelog_logger(&self, cfg: Config) -> Result<Box<dyn SharedLogger>, Box<dyn Error>> {
        let logger: Box<dyn SharedLogger> = match &self.output {
            LogOutput::StdOut => {
                TermLogger::new(self.min_level, cfg, TerminalMode::Stdout, ColorChoice::Auto)
            }
            LogOutput::StdErr => {
                TermLogger::new(self.min_level, cfg, TerminalMode::Stderr, ColorChoice::Auto)
            }
            LogOutput::File(path) => {
                let file = File::options().create(true).append(true).open(path)?;
                let writer = BufWriter::new(file);

                WriteLogger::new(self.min_level, cfg, writer)
            }
        };

        Ok(logger)
    }
}

impl LogConfig {
    #[allow(clippy::type_complexity)]
    fn make_simplelog_loggers(
        &self,
        template_cfg: &ConfigBuilder,
    ) -> (Vec<Box<dyn SharedLogger>>, Vec<Box<dyn Error>>) {
        let mut output = Vec::new();
        let mut errs = Vec::new();

        if let Some(runtime) = &self.runtime {
            let cfg = template_cfg
                .clone()
                .add_filter_allow_str("wutengine::")
                .build();

            match runtime.make_simplelog_logger(cfg) {
                Ok(logger) => output.push(logger),
                Err(err) => errs.push(err),
            }
        }

        if let Some(renderer) = &self.renderer {
            let cfg = template_cfg
                .clone()
                .add_filter_allow_str("wutengine_graphics")
                .add_filter_allow_str("wutengine_opengl")
                .add_filter_allow_str("wutengine_shadercompiler")
                .build();

            match renderer.make_simplelog_logger(cfg) {
                Ok(logger) => output.push(logger),
                Err(err) => errs.push(err),
            }
        }

        if let Some(other) = &self.other {
            // Filter all other components that we specified previously
            let cfg = template_cfg
                .clone()
                .add_filter_ignore_str("wutengine")
                .add_filter_ignore_str("naga")
                .build();

            match other.make_simplelog_logger(cfg) {
                Ok(logger) => output.push(logger),
                Err(err) => errs.push(err),
            }
        }

        (output, errs)
    }
}

/// Initializes the global [log] crate according to the
/// provided config.
pub(crate) fn initialize_loggers(config: &LogConfig) {
    let mut log_init_errs: Vec<String> = Vec::new();

    let mut builder = ConfigBuilder::new();
    builder
        .set_thread_mode(ThreadLogMode::Both)
        .set_location_level(LevelFilter::Trace)
        .set_target_level(LevelFilter::Error)
        .set_time_format_rfc3339();

    match builder.set_time_offset_to_local() {
        Ok(_) => (),
        Err(_) => {
            log_init_errs.push(
                "Could not set log time offset to local, using non-offset times. Error: {}"
                    .to_owned(),
            );
        }
    };

    let (loggers, logger_create_errs) = config.make_simplelog_loggers(&builder);

    if let Err(e) = CombinedLogger::init(loggers) {
        eprintln!(
            "Could not configure logger. Logging output will not be available: {}",
            e
        );
    }

    for err in logger_create_errs {
        log::error!(
            "Error configuring one of the loggers, logging output for that component will not be available: {}",
            err
        );
    }
}
