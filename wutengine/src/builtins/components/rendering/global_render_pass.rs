use crate::component::Component;
use crate::graphics::renderpass::RenderPass;
use crate::graphics::renderpass::RenderPassInfo;

#[derive(Debug, Default)]
pub struct GlobalRenderPass {
    pub pass: Option<RenderPassInfo>,
}

impl GlobalRenderPass {
    pub fn new<T: RenderPass>() -> Self {
        Self {
            pass: Some(RenderPassInfo::from_pass::<T>()),
        }
    }
}

impl Component for GlobalRenderPass {}
