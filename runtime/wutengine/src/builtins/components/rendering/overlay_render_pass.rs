use wutengine_graphics::wgpu;

use crate::{
    component::Component,
    graphics::{RenderPassInfo, renderpass::RenderPass},
    window::Window,
};

/// A render pass that renders on a raw surface, on top of all cameras
#[derive(Debug, Default)]
pub struct OverlayRenderPass {
    /// The pass. See [RenderPassInfo] or [Self::new]
    pub pass: Option<RenderPassInfo<(Window, wgpu::Texture), hecs::World>>, // TODO: Change [D] parameter to take a hecs world instead, and implement scheduling
}

impl OverlayRenderPass {
    /// Construct a new [OverlayRenderPass] component from the given [RenderPass]
    pub fn new<T: RenderPass<(Window, wgpu::Texture), hecs::World>>() -> Self {
        Self {
            pass: Some(RenderPassInfo::from_pass::<T>()),
        }
    }
}

impl Component for OverlayRenderPass {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("993f53f6-ea80-4b48-9194-508a9d32f7a0")).unwrap();
}
