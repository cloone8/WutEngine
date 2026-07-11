//! A filter string

use alloc::string::String;

/// A filter string used to filter scopes
#[derive(Clone, Debug, Default)]
pub(crate) struct Filter {
    filter: String,
}

impl Filter {
    /// Show this filter
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Scope filter"));
            self.filter = self.filter.to_lowercase();

            if ui.button("ｘ").clicked() {
                self.filter.clear();
            }
        });
    }

    /// if true, show everything
    pub(crate) fn is_empty(&self) -> bool {
        self.filter.is_empty()
    }

    /// Whether to include the given ID
    pub(crate) fn include(&self, id: &str) -> bool {
        if self.filter.is_empty() {
            true
        } else {
            id.to_lowercase().contains(&self.filter)
        }
    }

    /// Update this filter
    pub(crate) fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }
}
