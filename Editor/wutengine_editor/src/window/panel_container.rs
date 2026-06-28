//! Container for editor panels

use wutengine_egui::egui;
use wutengine_egui::egui::Widget;

use crate::panel::EditorPanel;
use crate::panel::EditorPanelId;

#[derive(Debug)]
pub(super) struct PanelContainer {
    selected: usize,
    panels: Vec<PanelMeta>,
}

impl PanelContainer {
    pub(super) fn new() -> Self {
        Self {
            selected: 0,
            panels: Vec::new(),
        }
    }

    pub(super) fn add<P: EditorPanel>(&mut self) -> &mut Self {
        self.panels.push(PanelMeta::new::<P>());
        self
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui) {
        self.clamp_selected();

        egui::MenuBar::new().ui(ui, |ui| {
            for (i, panel) in self.panels.iter_mut().enumerate() {
                let select_panel = egui::Button::new(panel.name)
                    .selected(self.selected == i)
                    .ui(ui)
                    .clicked();

                if select_panel {
                    self.selected = i;
                }
            }
        });

        ui.separator();

        if self.panels.is_empty() {
            return;
        }

        let panel = self
            .panels
            .get_mut(self.selected)
            .expect("Panel index out of range. Internal engine error");

        panel.panel.show(ui);
    }

    fn clamp_selected(&mut self) {
        self.selected = self.selected.min(self.panels.len().saturating_sub(1));
    }
}

#[derive(derive_more::Debug)]
struct PanelMeta {
    name: &'static str,

    #[debug(skip)]
    panel: Box<dyn EditorPanel>,
}

impl PanelMeta {
    fn new<P: EditorPanel>() -> Self {
        let new_id = EditorPanelId::new();

        Self {
            name: P::name(),
            panel: P::construct(new_id),
        }
    }
}
