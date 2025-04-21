//! Graphics related functionality

use glam::Mat4;
use internal::RenderCommand;
use internal::objects::OBJECT_QUEUE;
use internal::viewports::VIEWPORT_QUEUE;
use wutengine_graphics::renderer::Viewport;
pub use wutengine_graphics::*;

use crate::builtins::assets::{Material, Mesh};

pub(crate) mod internal;

#[profiling::function]
pub fn render_mesh(mesh: &Mesh, material: &Material, object_to_world: Mat4) {
    let mut locked = OBJECT_QUEUE.lock().unwrap();

    locked.push(RenderCommand {
        mesh: mesh.0.clone(),
        material: material.0.clone(),
        object_to_world,
    });
}

#[profiling::function]
pub fn render_viewport(viewport: Viewport) {
    let mut locked = VIEWPORT_QUEUE.lock().unwrap();
    locked.push(viewport);
}
