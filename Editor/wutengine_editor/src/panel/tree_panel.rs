use wutengine_egui::egui;

use super::EditorPanel;

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
        ui.label("Level Tree");
    }
}
