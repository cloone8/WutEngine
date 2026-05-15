use crate::component::Component;
use crate::graphics::renderpass::RenderPassInfo;

#[derive(Debug, Default)]
pub struct GlobalRenderPass {
    pub pass: Option<RenderPassInfo>,
}

impl Component for GlobalRenderPass {}
