use wutengine_egui::egui;

use crate::panel::LogPanel;
use crate::panel::TestPanel;
use crate::panel::TreePanel;

use super::EditorWindow;
use super::panel_container::PanelContainer;

/// The main editor window
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
    /// Creates a new main editor window with the default layout
    pub(crate) fn new() -> Self {
        let mut left_panels = PanelContainer::new();

        left_panels.add::<TreePanel>();

        let mut right_panels = PanelContainer::new();

        right_panels.add::<TestPanel>();
        right_panels.add::<TestPanel>();
        right_panels.add::<TestPanel>();

        let mut bottom_panels = PanelContainer::new();
        bottom_panels.add::<LogPanel>();

        Self {
            left_panels,
            right_panels,
            bottom_panels,
            center_panels: PanelContainer::new(),
        }
    }

    fn show_ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("Top panel")
            .resizable(false)
            .show(ui, |ui| {
                we_menu::show(ui);
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
