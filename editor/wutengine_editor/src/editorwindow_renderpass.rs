//! Rendering editor windows

use wutengine::graphics::renderpass::RenderPass;
use wutengine::graphics::wgpu;
use wutengine::hecs;
use wutengine::time;
use wutengine::window::Window;
use wutengine_egui::egui;

use crate::EGUI_RESOURCES;
use crate::EguiWindowContainer;

/// Overlay pass that renders all editor windows
#[derive(Debug)]
pub(crate) struct EditorWindowRenderPass {
    last_free: usize,
    to_free: Vec<egui::TextureId>,
}

impl EditorWindowRenderPass {
    const ORDER: u64 = u64::MAX / 2;
}

impl RenderPass<(Window, wgpu::Texture), hecs::World> for EditorWindowRenderPass {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Editor Window Renderpass"
    }

    fn order() -> u64
    where
        Self: Sized,
    {
        Self::ORDER
    }

    fn construct() -> Box<dyn RenderPass<(Window, wgpu::Texture), hecs::World>>
    where
        Self: Sized,
    {
        Box::new(Self {
            last_free: 0,
            to_free: Vec::new(),
        })
    }

    fn execute(
        &mut self,
        cmd: &mut wgpu::CommandEncoder,
        target: &(Window, wgpu::Texture),
        drawable: &hecs::World,
    ) {
        if self.last_free < time::frame_num() {
            self.last_free = time::frame_num();
            EGUI_RESOURCES.free_removed(self.to_free.drain(..));
        }

        let mut target_window: Option<&wutengine_egui::EguiWindow> = None;
        let mut query = drawable.query::<&EguiWindowContainer>();

        for window_container in query.iter() {
            if let Some(window_handle) = window_container.window_handle()
                && window_handle == target.0
            {
                target_window = Some(window_container.egui_window());
                break;
            }
        }

        let Some(target_window) = target_window else {
            return;
        };

        target_window.render_window(&target.1, &EGUI_RESOURCES, cmd, &mut self.to_free);
    }
}
