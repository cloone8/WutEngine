use wutengine_egui::{egui, egui::Widget};

use super::{EditorPanel, EditorPanelId};
use crate::logger::get_editor_logger;

/// Panel that shows the logs sent through the [log] crate
pub(crate) struct LogPanel;

impl EditorPanel for LogPanel {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Logs"
    }

    fn construct(_id: EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized,
    {
        Box::new(Self)
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let logger = get_editor_logger();

        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Log Level", |ui| {
                if let Some(new_level) =
                    show_log_level_picker(logger.get_external_level(), log::LevelFilter::Trace, ui)
                {
                    logger.set_external_level(new_level);
                    logger.refilter_logs();
                }
            });

            ui.menu_button("WutEngine Log Level", |ui| {
                if let Some(new_level) =
                    show_log_level_picker(logger.get_internal_level(), log::LevelFilter::Info, ui)
                {
                    logger.set_internal_level(new_level);
                    logger.refilter_logs();
                }
            });

            if ui.button("Clear").clicked() {
                logger.logs.lock().unwrap().clear();
            }
        });

        ui.separator();

        let logs = logger.logs.lock().unwrap();
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
}

/// Shows a series of buttons to pick a log level. The current level is highlighted.
/// If a new level was selected, returns [Some] with that level. Otherwise, returns [None]
fn show_log_level_picker(
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
