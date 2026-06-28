use wutengine::runtime;
use wutengine_egui::egui;

use crate::panel::EditorPanel;
use crate::panel::EditorPanelId;
use crate::panel::LogPanel;
use crate::panel::TestPanel;
use crate::panel::TestPanelTwo;

use super::EditorWindow;
use super::panel_container::PanelContainer;

#[derive(derive_more::Debug)]
pub(crate) struct MainEditorWindow {
    left_panels: PanelContainer,
    right_panels: PanelContainer,
    bottom_panels: PanelContainer,
    center_panels: PanelContainer,
}

impl EditorWindow for MainEditorWindow {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.show_ui(ui);
    }
}

impl MainEditorWindow {
    pub(crate) fn new() -> Self {
        let mut left_panels = PanelContainer::new();

        left_panels.add::<TestPanel>().add::<TestPanelTwo>();

        let mut bottom_panels = PanelContainer::new();
        bottom_panels.add::<LogPanel>();

        Self {
            left_panels,
            right_panels: PanelContainer::new(),
            bottom_panels,
            center_panels: PanelContainer::new(),
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

                self.left_panels.show(ui);
            });

        egui::Panel::right("Right panel")
            .resizable(true)
            .show(ui, |ui| {
                ui.take_available_space();

                self.right_panels.show(ui);
            });

        egui::Panel::bottom("Bottom panel")
            .resizable(true)
            .default_size(250.0)
            .show(ui, |ui| {
                ui.take_available_space();

                self.bottom_panels.show(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.take_available_space();

            self.center_panels.show(ui);
        });
    }
}
