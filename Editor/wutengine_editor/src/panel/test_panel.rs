use super::EditorPanel;
use super::EditorPanelId;

#[derive(Debug)]
pub struct TestPanel {
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
        ui.label("Hello from test panel");
    }
}

#[derive(Debug)]
pub struct TestPanelTwo {
    id: EditorPanelId,
}

impl EditorPanel for TestPanelTwo {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Test Panel 2"
    }

    fn construct(id: EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized,
    {
        Box::new(Self { id })
    }

    fn show(&mut self, ui: &mut wutengine_egui::egui::Ui) {
        ui.label("Hello from test panel 2");
    }
}
