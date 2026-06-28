//! Editor panels

use wutengine_egui::egui;
use wutengine_util_macro::unique_id_type32;

mod log_panel;
pub(crate) use log_panel::*;

mod test_panel;
pub(crate) use test_panel::*;

mod tree_panel;
pub(crate) use tree_panel::*;

unique_id_type32! {
    /// Unique identifier for an active [EditorPanel]
    pub(crate) EditorPanelId
}

/// An editor panel. Can be freely rearranged throughout the editor windows
pub(crate) trait EditorPanel: Send + Sync {
    /// The name of the panel
    fn name() -> &'static str
    where
        Self: Sized;

    /// Constructs a new instance of this panel
    fn construct(id: EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized;

    /// Shows this panel within the given ui
    fn show(&mut self, ui: &mut egui::Ui);
}
