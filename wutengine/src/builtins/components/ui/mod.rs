//! UI rendering components for WutEngine

use wutengine_core::identifiers::WindowIdentifier;

use crate::component::Component;
use crate::ui::UIPlugin;

#[derive(Debug)]
pub struct ScreenSpaceUICanvas {
    pub window: WindowIdentifier,
}

impl Component for ScreenSpaceUICanvas {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn on_start(&mut self, context: &mut crate::component::Context) {
        if let Some(uiplugin) = context.plugin.get::<UIPlugin>() {
            uiplugin.register_screenspace_canvas(context.gameobject.object.id);
        }
    }

    fn on_destroy(&mut self, context: &mut crate::component::Context) {
        if let Some(uiplugin) = context.plugin.get::<UIPlugin>() {
            uiplugin.deregister_screenspace_canvas(context.gameobject.object.id);
        }
    }
}

impl ScreenSpaceUICanvas {
    pub(crate) fn run_ui(&self, context: &egui::Context) {
        egui::CentralPanel::default().show(context, |ui| {
            context.inspection_ui(ui);
        });
    }
}
