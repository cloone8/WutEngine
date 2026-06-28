use wutengine::runtime;
use wutengine_egui::egui;

use crate::panel::EditorPanel;
use crate::panel::EditorPanelId;
use crate::panel::LogPanel;

use super::EditorWindow;

#[derive(derive_more::Debug)]
pub(crate) struct MainEditorWindow {
    #[debug(skip)]
    log_panel: Box<dyn EditorPanel>,
}

impl EditorWindow for MainEditorWindow {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.show_ui(ui);
    }
}

impl MainEditorWindow {
    pub(crate) fn new() -> Self {
        Self {
            log_panel: LogPanel::construct(EditorPanelId::new()),
        }
    }

    fn show_ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("Top panel")
            .resizable(false)
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New Project").clicked() {
                            log::info!("New project");
                        }

                        ui.separator();

                        if ui.button("Exit").clicked() {
                            runtime::exit();
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo").clicked() {
                            log::info!("Undo");
                        }

                        if ui.button("Redo").clicked() {
                            log::info!("Redo");
                        }
                    });
                });
            });

        egui::Panel::left("Left panel")
            .resizable(true)
            .show(ui, |ui| {
                ui.take_available_space();
                ui.label("Hello from WutEngine Editor Left");
            });

        egui::Panel::right("Right panel")
            .resizable(true)
            .show(ui, |ui| {
                ui.take_available_space();
                ui.label("Hello from WutEngine Editor Right");
            });

        egui::Panel::bottom("Bottom panel")
            .resizable(true)
            .default_size(250.0)
            .show(ui, |ui| {
                ui.take_available_space();

                self.log_panel.show(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("Hello from WutEngine Editor");
        });
    }
}
