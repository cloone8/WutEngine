//! Editor logger

use alloc::sync::Arc;
use std::sync::Mutex;

use wutengine_egui::egui;

#[derive(Debug, Clone)]
pub(crate) struct EditorLogger {
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl EditorLogger {
    pub(crate) fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        let logs = self.logs.lock().unwrap();

        let text_style_height = ui.text_style_height(&egui::TextStyle::Body);
        egui::ScrollArea::vertical()
            .max_height(text_style_height * 20.0)
            .show_rows(ui, text_style_height, logs.len(), |ui, range| {
                for log in logs.iter().skip(range.start).take(range.end - range.start) {
                    log.show(ui);
                }
            });
    }
}

#[derive(Debug)]
struct LogEntry {
    message: String,
}

impl LogEntry {
    fn show(&self, ui: &mut egui::Ui) {
        ui.label(format!("Message: {}", self.message));
    }
}

impl log::Log for EditorLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        todo!()
    }

    fn log(&self, record: &log::Record) {
        todo!()
    }

    fn flush(&self) {
        //no-op
    }
}
