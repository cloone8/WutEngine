use wutengine_egui::egui;

use crate::project;

use super::EditorPanel;

/// The panel showing the main level hierarchy tree
#[derive(Debug)]
pub(crate) struct TreePanel;

impl EditorPanel for TreePanel {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Tree"
    }

    fn construct(_id: super::EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized,
    {
        Box::new(Self)
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let open_levels = project::open_levels();

        if open_levels.is_empty() {
            ui.label("No levels loaded. Open a level from the project library panel");
            return;
        }

        for level in open_levels {
            ui.label(format!("🗄️ {}", level));
        }
    }
}
