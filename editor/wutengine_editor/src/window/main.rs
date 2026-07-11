use wutengine_egui::egui;

use super::{EditorWindow, panel_container::PanelContainer};
use crate::{
    exit,
    panel::{LibraryPanel, LogPanel, TestPanel, TreePanel},
};

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
        bottom_panels.add::<LibraryPanel>();

        Self {
            left_panels,
            right_panels,
            bottom_panels,
            center_panels: PanelContainer::new(),
        }
    }

    fn show_ui(&mut self, ui: &mut egui::Ui) {
        if exit::exit_requested() {
            let modal = egui::Modal::new(egui::Id::new("Exit Modal")).show(ui.ctx(), |ui| {
                ui.set_width(200.0);
                ui.heading("Are you sure you want to exit?");

                ui.add_space(32.0);

                egui::Sides::new().show(
                    ui,
                    |ui| {
                        if ui.button("Exit").clicked() {
                            exit::allow_exit();
                        }
                    },
                    |ui| {
                        if ui.button("Cancel").clicked() {
                            ui.close();
                        }
                    },
                );
            });

            if modal.should_close() {
                exit::stop_exit();
            }
        }

        egui::Panel::top("Top panel")
            .resizable(false)
            .frame(
                egui::Frame::side_top_panel(ui.style())
                    .inner_margin(egui::Margin::symmetric(8, 1))
                    .fill(we_style::MENU_COLOR),
            )
            .show(ui, |ui| {
                we_menu::show(ui);
            });

        let marginless =
            egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::same(0));

        egui::Panel::left("Left panel")
            .resizable(true)
            .frame(marginless)
            .show(ui, |ui| {
                ui.take_available_space();

                self.left_panels.show(ui);
            });

        egui::Panel::right("Right panel")
            .resizable(true)
            .frame(marginless)
            .show(ui, |ui| {
                ui.take_available_space();

                self.right_panels.show(ui);
            });

        egui::Panel::bottom("Bottom panel")
            .resizable(true)
            .frame(marginless)
            .default_size(250.0)
            .show(ui, |ui| {
                ui.take_available_space();

                self.bottom_panels.show(ui);
            });

        egui::CentralPanel::default()
            .frame(marginless)
            .show(ui, |ui| {
                ui.take_available_space();

                self.center_panels.show(ui);
            });
    }
}
