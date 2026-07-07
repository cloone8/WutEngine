use std::path::Path;

use wutengine_egui::egui;

use super::EditorPanel;

/// Panel showing the asset library of this project
#[derive(Debug)]
pub(crate) struct LibraryPanel;

impl LibraryPanel {
    fn show_file_tree(&mut self, ui: &mut egui::Ui) {
        let window_response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt("FileTree")
                    .sense(egui::Sense::CLICK),
                |ui| {
                    ui.take_available_space();

                    egui::CollapsingHeader::new("📁 Assets")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label("A");
                            ui.label("B");
                        });
                },
            )
            .response;

        window_response.context_menu(|ui| {
            Self::context_menu("", ui);
        });
    }

    fn show_directory(&mut self, ui: &mut egui::Ui) {
        let window_response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt("ShowDirectory")
                    .sense(egui::Sense::CLICK),
                |ui| {
                    ui.take_available_space();
                },
            )
            .response;

        window_response.context_menu(|ui| {
            Self::context_menu("", ui);
        });
    }

    fn context_menu(root: impl AsRef<Path>, ui: &mut egui::Ui) {
        let root = root.as_ref();

        if ui.button(root.to_string_lossy()).clicked() {
            log::info!("Click");
        };
    }
}

impl EditorPanel for LibraryPanel {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Library"
    }

    fn construct(_id: super::EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized,
    {
        Box::new(Self)
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("LibraryLeft").show(ui, |ui| {
            self.show_file_tree(ui);
        });

        egui::CentralPanel::default_margins().show(ui, |ui| {
            self.show_directory(ui);
        });
    }
}
