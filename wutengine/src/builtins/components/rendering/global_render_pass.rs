use crate::component::Component;
use crate::graphics::DrawCommand;
use crate::graphics::RenderPassInfo;
use crate::graphics::renderpass::RenderPass;

use super::Camera;

/// A global render pass. Once inserted into the world, becomes active on all cameras
#[derive(Debug, Default)]
pub struct GlobalRenderPass {
    /// The pass. See [RenderPassInfo] or [Self::new]
    pub pass: Option<RenderPassInfo<Camera, DrawCommand>>,
}

impl GlobalRenderPass {
    /// Construct a new [GlobalRenderPass] component from the given [RenderPass]
    pub fn new<T: RenderPass<Camera, DrawCommand>>() -> Self {
        Self {
            pass: Some(RenderPassInfo::from_pass::<T>()),
        }
    }
}

impl Component for GlobalRenderPass {}
