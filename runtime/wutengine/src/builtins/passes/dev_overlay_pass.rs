use wutengine_graphics::{renderpass::RenderPass, wgpu};
use wutengine_input::WindowIdentifier;

use crate::window::Window;

/// The main pass for color rendering
#[derive(Debug)]
pub struct DevOverlayPass;

impl DevOverlayPass {
    /// Development overlay is always last
    pub const ORDER: u64 = u64::MAX;
}

impl RenderPass<(Window, wgpu::Texture), hecs::World> for DevOverlayPass {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Development Overlay"
    }

    fn order() -> u64 {
        Self::ORDER
    }

    fn construct() -> Box<dyn RenderPass<(Window, wgpu::Texture), hecs::World>>
    where
        Self: Sized,
    {
        Box::new(DevOverlayPass)
    }

    fn execute(
        &mut self,
        cmd: &mut wgpu::CommandEncoder,
        target: &(Window, wgpu::Texture),
        _drawable: &hecs::World,
    ) {
        profiling::function_scope!();

        log::trace!("Running development overlay pass");

        crate::development_overlay::render_overlay_if_window_eq(
            &WindowIdentifier::from(target.0),
            &target.1,
            cmd,
        );
    }
}
