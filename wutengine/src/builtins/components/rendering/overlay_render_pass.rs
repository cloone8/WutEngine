use wutengine_graphics::wgpu;

use crate::component::Component;
use crate::graphics::RenderPassInfo;
use crate::graphics::renderpass::RenderPass;
use crate::window::Window;

/// A render pass that renders on a raw surface, on top of all cameras
#[derive(Debug, Default)]
pub struct OverlayRenderPass {
    /// The pass. See [RenderPassInfo] or [Self::new]
    pub pass: Option<RenderPassInfo<(Window, wgpu::Texture), ()>>, // TODO: Change [D] parameter to take a hecs world instead, and implement scheduling
}

impl OverlayRenderPass {
    /// Construct a new [OverlayRenderPass] component from the given [RenderPass]
    pub fn new<T: RenderPass<(Window, wgpu::Texture), ()>>() -> Self {
        Self {
            pass: Some(RenderPassInfo::from_pass::<T>()),
        }
    }
}

impl Component for OverlayRenderPass {}
