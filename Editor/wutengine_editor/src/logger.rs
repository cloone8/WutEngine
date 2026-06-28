//! Editor logger

use alloc::collections::VecDeque;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
use std::sync::Mutex;
use wutengine_egui::egui::Widget;
use wutengine_util::InitOnce;

use wutengine_egui::egui;

use crate::editor_preferences;

static EDITOR_LOGGER: InitOnce<EditorLogger, false> = InitOnce::new();

/// Initializes and sets the editor logger
pub(crate) fn init() {
    InitOnce::init(&EDITOR_LOGGER, EditorLogger::new());

    let logger_ref: &EditorLogger = &EDITOR_LOGGER;
    let dyn_logger: &dyn log::Log = logger_ref;

    log::set_logger(dyn_logger).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

/// Returns the editor logger
#[inline(always)]
pub(crate) fn get_editor_logger() -> &'static EditorLogger {
    &EDITOR_LOGGER
}

/// Logger that gathers log messages and displays them in an editor window
#[derive(Debug)]
pub(crate) struct EditorLogger {
    internal_level: AtomicU8,
    external_level: AtomicU8,
    max_logs: usize,
    logs: Mutex<VecDeque<LogEntry>>,
}

impl EditorLogger {
    const INTERNAL_LOG_LEVEL_PREF: &str = "editor.internal_log_level";
    const EXTERNAL_LOG_LEVEL_PREF: &str = "editor.external_log_level";

    fn new() -> Self {
        let stored_internal_level =
            editor_preferences::get_or(Self::INTERNAL_LOG_LEVEL_PREF, log::LevelFilter::Warn);
        let stored_external_level =
            editor_preferences::get_or(Self::EXTERNAL_LOG_LEVEL_PREF, log::LevelFilter::Debug);

        // Clamp to `info` if an invalid level was stored
        let stored_internal_level = stored_internal_level.min(log::LevelFilter::Info);

        Self {
            internal_level: AtomicU8::new(Self::levelfilter_to_int(stored_internal_level)),
            external_level: AtomicU8::new(Self::levelfilter_to_int(stored_external_level)),
            max_logs: 1_000,
            logs: Mutex::new(VecDeque::new()),
        }
    }

    /// Shows the log UI
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        let logs = self.logs.lock().unwrap();
        let text_style_height = ui.text_style_height(&egui::TextStyle::Body);

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .stick_to_bottom(true)
            .show_rows(ui, text_style_height, logs.len(), |ui, range| {
                ui.take_available_space();

                for log in logs.iter().skip(range.start).take(range.end - range.start) {
                    log.show(ui);
                }
            });
    }

    /// Returns the current level filter for internal logs
    pub(crate) fn get_internal_level(&self) -> log::LevelFilter {
        Self::int_to_levelfilter(self.internal_level.load(Ordering::Acquire))
            .expect("Stored invalid levelfilter")
    }

    /// Sets the level filter for internal logs
    pub(crate) fn set_internal_level(&self, level_filter: log::LevelFilter) {
        self.internal_level
            .store(Self::levelfilter_to_int(level_filter), Ordering::Release);

        editor_preferences::set(Self::INTERNAL_LOG_LEVEL_PREF, level_filter);
    }

    /// Returns the current level filter for external logs
    pub(crate) fn get_external_level(&self) -> log::LevelFilter {
        Self::int_to_levelfilter(self.external_level.load(Ordering::Acquire))
            .expect("Stored invalid levelfilter")
    }

    /// Sets the level filter for external logs
    pub(crate) fn set_external_level(&self, level_filter: log::LevelFilter) {
        self.external_level
            .store(Self::levelfilter_to_int(level_filter), Ordering::Release);

        editor_preferences::set(Self::EXTERNAL_LOG_LEVEL_PREF, level_filter);
    }

    /// Filters the currently stored logs according to the configured filters
    pub(crate) fn refilter_logs(&self) {
        let internal_level = self.get_internal_level();
        let external_level = self.get_external_level();

        let mut logs = self.logs.lock().unwrap();

        logs.retain(|log| {
            let filter = if log.is_internal() {
                internal_level
            } else {
                external_level
            };

            log.level() <= filter
        });
    }

    const fn levelfilter_to_int(filter: log::LevelFilter) -> u8 {
        match filter {
            log::LevelFilter::Off => 0,
            log::LevelFilter::Error => 1,
            log::LevelFilter::Warn => 2,
            log::LevelFilter::Info => 3,
            log::LevelFilter::Debug => 4,
            log::LevelFilter::Trace => 5,
        }
    }

    const fn int_to_levelfilter(int: u8) -> Option<log::LevelFilter> {
        match int {
            0 => Some(log::LevelFilter::Off),
            1 => Some(log::LevelFilter::Error),
            2 => Some(log::LevelFilter::Warn),
            3 => Some(log::LevelFilter::Info),
            4 => Some(log::LevelFilter::Debug),
            5 => Some(log::LevelFilter::Trace),
            _ => None,
        }
    }
}

#[derive(Debug, derive_more::IsVariant)]
enum LogEntry {
    Internal {
        level: log::Level,
        message: String,
        subsys: String,
    },

    External {
        level: log::Level,
        message: String,
        file: Option<String>,
        line: Option<u32>,
    },
}

impl LogEntry {
    fn new(record: &log::Record) -> Self {
        let (is_internal, subsys) = wutengine::log::subsystem_from_target(record.target());

        match is_internal {
            true => Self::Internal {
                level: record.level(),
                message: format!("{}", record.args()),
                subsys: subsys.to_string(),
            },
            false => Self::External {
                level: record.level(),
                message: format!("{}", record.args()),
                file: record.file().map(ToString::to_string),
                line: record.line(),
            },
        }
    }

    const fn level(&self) -> log::Level {
        match self {
            Self::Internal { level, .. } => *level,
            Self::External { level, .. } => *level,
        }
    }

    const fn message(&self) -> &str {
        match self {
            Self::Internal { message, .. } => message.as_str(),
            Self::External { message, .. } => message.as_str(),
        }
    }

    fn level_to_color(level: log::Level) -> egui::Color32 {
        match level {
            log::Level::Error => egui::Color32::RED,
            log::Level::Warn => egui::Color32::YELLOW,
            log::Level::Info => egui::Color32::BLUE,
            log::Level::Debug => egui::Color32::LIGHT_BLUE,
            log::Level::Trace => egui::Color32::LIGHT_GRAY,
        }
    }

    fn show(&self, ui: &mut egui::Ui) {
        let on_hover = |ui: &mut egui::Ui| match self {
            Self::Internal { subsys, .. } => {
                ui.label(format!("{} (internal)", subsys.as_str()));
            }
            Self::External { file, line, .. } => {
                if let Some(file) = file.as_deref() {
                    ui.label(file);
                }

                if let Some(line) = line {
                    ui.label(format!("line: {line}"));
                }
            }
        };

        egui::Frame::NONE
            .inner_margin(egui::Margin {
                left: 10,
                right: 10,
                top: 5,
                bottom: 5,
            })
            .stroke(egui::Stroke::new(0.5, egui::Color32::LIGHT_GRAY))
            .corner_radius(3.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.take_available_width();
                    ui.colored_label(Self::level_to_color(self.level()), self.level().to_string());
                    ui.label(self.message());
                });
            })
            .response
            .on_hover_ui_at_pointer(on_hover);
    }
}

impl log::Log for EditorLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let (is_internal, _subsys) = wutengine::log::subsystem_from_target(metadata.target());

        let filter = if is_internal {
            self.get_internal_level()
        } else {
            self.get_external_level()
        };

        metadata.level() <= filter
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut logs = self.logs.lock().unwrap();

        if logs.len() >= self.max_logs {
            logs.drain(..self.max_logs);
        }

        if self.max_logs == 0 {
            return;
        }

        logs.push_back(LogEntry::new(record));
    }

    fn flush(&self) {
        //no-op
    }
}

/// Shows a series of buttons to pick a log level. The current level is highlighted.
/// If a new level was selected, returns [Some] with that level. Otherwise, returns [None]
pub(crate) fn show_log_level_picker(
    cur_level: log::LevelFilter,
    min_level: log::LevelFilter,
    ui: &mut egui::Ui,
) -> Option<log::LevelFilter> {
    macro_rules! show_button {
        ($level:expr, $name:literal) => {
            if $level <= min_level {
                if egui::Button::new($name)
                    .selected(cur_level == $level)
                    .ui(ui)
                    .clicked()
                {
                    return Some($level);
                }
            }
        };
    }

    show_button!(log::LevelFilter::Trace, "Trace");
    show_button!(log::LevelFilter::Debug, "Debug");
    show_button!(log::LevelFilter::Info, "Info");
    show_button!(log::LevelFilter::Warn, "Warning");
    show_button!(log::LevelFilter::Error, "Error");

    None
}
