use crate::component::Component;
use crate::graphics::renderpass::RenderPass;
use crate::graphics::renderpass::RenderPassInfo;

/// A global render pass. Once inserted into the world, becomes active on all cameras
#[derive(Debug, Default)]
pub struct GlobalRenderPass {
    /// The pass. See [RenderPassInfo] or [Self::new]
    pub pass: Option<RenderPassInfo>,
}

impl GlobalRenderPass {
    /// Construct a new [GlobalRenderPass] component from the given [RenderPass]
    pub fn new<T: RenderPass>() -> Self {
        Self {
            pass: Some(RenderPassInfo::from_pass::<T>()),
        }
    }
}

impl Component for GlobalRenderPass {}
