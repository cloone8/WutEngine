use super::EditorPanel;
use super::EditorPanelId;

/// Testing panel
#[derive(Debug)]
pub(crate) struct TestPanel {
    id: EditorPanelId,
}

impl EditorPanel for TestPanel {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Test Panel"
    }

    fn construct(id: EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized,
    {
        Box::new(Self { id })
    }

    fn show(&mut self, ui: &mut wutengine_egui::egui::Ui) {
        ui.label(format!("Hello from test panel {}", self.id));
    }
}
