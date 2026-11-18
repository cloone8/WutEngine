use std::env;
use std::fmt;
use std::io;
use std::io::IsTerminal;
use std::io::Stderr;
use std::io::Stdout;
use std::io::Write;

use anstyle::AnsiColor;
use anstyle::Style;
use log::Level;

use crate::LoggerOutput;
use crate::duration_since_logger_start;

/// Simple terminal logger that logs to `stdout`
#[derive(Debug)]
pub struct TerminalLogger {
    output: TerminalOutput,
    color: bool,
}

#[derive(Debug)]
enum TerminalOutput {
    Stdout(Stdout),
    Stderr(Stderr),
}

impl TerminalOutput {
    fn from_mode(mode: OutputMode) -> Self {
        match mode {
            OutputMode::Stdout => Self::Stdout(io::stdout()),
            OutputMode::Stderr => Self::Stderr(io::stderr()),
        }
    }

    #[inline]
    fn with_writer(&self, f: impl FnOnce(&mut dyn Write)) {
        match self {
            Self::Stdout(stdout) => {
                let mut w = stdout;

                f(&mut w);
            }
            Self::Stderr(stderr) => {
                let mut w = stderr;

                f(&mut w);
            }
        }
    }

    fn should_use_color(&self, color_mode: ColorMode) -> bool {
        match self {
            Self::Stdout(stdout) => color_mode.should_use_color(stdout),
            Self::Stderr(stderr) => color_mode.should_use_color(stderr),
        }
    }
}

/// Where a [`TerminalLogger`] writes to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    /// Write to stdout
    #[default]
    Stdout,

    /// Write to stderr
    Stderr,
}

/// Whether [`TerminalLogger`] uses color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    /// Use color for color-supporting terminals, and if the environment does not disable it
    #[default]
    Auto,

    /// Force color output
    On,

    /// Disable color output
    Off,
}

impl ColorMode {
    fn should_use_color(self, target: &impl IsTerminal) -> bool {
        match self {
            Self::On => true,
            Self::Off => false,
            Self::Auto => wants_color(target),
        }
    }
}

impl TerminalLogger {
    /// Create a new [`TerminalLogger`]
    #[must_use]
    pub fn new(output: OutputMode, color: ColorMode) -> Self {
        let writer = TerminalOutput::from_mode(output);
        let use_color = writer.should_use_color(color);

        Self {
            output: writer,
            color: use_color,
        }
    }
}

impl LoggerOutput for TerminalLogger {
    fn write(&self, entry: &crate::LogEntry) {
        let time = duration_since_logger_start(&entry.time).as_secs_f64();

        let level = fmt::from_fn(|fmt| {
            if self.color
                && let Some(level_color) = level_color(entry.level)
            {
                write!(fmt, "{level_color}{}{level_color:#}", entry.level)
            } else {
                write!(fmt, "{}", entry.level)
            }
        });

        self.output.with_writer(|writer| {
            _ = writeln!(
                writer,
                "{time:.3}s [{}] ({}) {}",
                level, entry.category, entry.message
            );
        });
    }

    fn flush(&self) {
        self.output.with_writer(|writer| {
            _ = writer.flush();
        });
    }
}

const fn level_color(level: Level) -> Option<Style> {
    Some(match level {
        Level::Error => AnsiColor::Red.on_default(),
        Level::Warn => AnsiColor::Yellow.on_default(),
        Level::Info => AnsiColor::Blue.on_default(),
        Level::Debug => AnsiColor::Cyan.on_default(),
        Level::Trace => {
            return None;
        }
    })
}

/// Checks common color environment variables for color to be forced on/off, and if none are found,
/// checks whether the target is a terminal that might support color.
fn wants_color(target: &impl IsTerminal) -> bool {
    if env::var_os("CLICOLOR_FORCE").is_some_and(|value| value != "0") {
        return true;
    }

    if let Some(value) = env::var_os("FORCE_COLOR") {
        // This one can force both on and off
        return !(value == "0" || value.eq_ignore_ascii_case("false"));
    }

    if env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()) {
        // NO_COLOR forces off for any non-empty value
        return false;
    }

    if env::var_os("CLICOLOR").is_some_and(|value| value == "0") {
        // Older convention for turning off color
        return false;
    }

    if env::var_os("TERM").is_some_and(|value| value == "dumb") {
        // Yet another convention for simple shells
        return false;
    }

    // No env vars. Color if terminal
    target.is_terminal()
}
