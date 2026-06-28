use crate::component::Component;
use crate::graphics::DrawCommand;
use crate::graphics::RenderPassInfo;
use crate::graphics::renderpass::RenderPass;

use super::Camera;

/// A render pass that executes draw commands for a camera.
#[derive(Debug, Default)]
pub struct CameraRenderPass {
    /// The pass. See [RenderPassInfo] or [Self::new]
    pub pass: Option<RenderPassInfo<Camera, [DrawCommand]>>,
}

impl CameraRenderPass {
    /// Construct a new [GlobalRenderPass] component from the given [RenderPass]
    pub fn new<T: RenderPass<Camera, [DrawCommand]>>() -> Self {
        Self {
            pass: Some(RenderPassInfo::from_pass::<T>()),
        }
    }
}

impl Component for CameraRenderPass {
    const ID: uuid::Uuid = uuid::uuid!("62e189f7-ee97-42bf-aab3-2e8c9842343d");
}
