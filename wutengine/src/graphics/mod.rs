//! Graphics related functionality

use glam::Mat4;
use internal::RenderCommand;
use internal::objects::OBJECT_QUEUE;
use internal::viewports::VIEWPORT_QUEUE;
use wutengine_graphics::renderer::Viewport;
pub use wutengine_graphics::*;

use crate::builtins::assets::{Material, Mesh};

pub(crate) mod internal;

/// Renders `mesh` with `material`, transforming it with `object_to_world` this frame.
#[profiling::function]
pub fn render_mesh(mesh: &Mesh, material: &Material, object_to_world: Mat4) {
    let mut locked = OBJECT_QUEUE.lock().unwrap();

    locked.push(RenderCommand {
        mesh: mesh.0.clone(),
        material: material.0.clone(),
        object_to_world,
    });
}

/// Queues the rendering of `viewport` this frame
#[profiling::function]
pub fn render_viewport(viewport: Viewport) {
    let mut locked = VIEWPORT_QUEUE.lock().unwrap();
    locked.push(viewport);
}
