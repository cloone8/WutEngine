use std::sync::Mutex;

use glam::Mat4;

use crate::builtins::assets::{Material, Mesh};
use crate::renderer::queue::RenderCommand;

/// The graphics context. Used for interacting with graphics related APIs
#[must_use = "The commands within the context must be consumed"]
#[derive(Debug)]
pub struct GraphicsContext {
    render_commands: Mutex<Vec<RenderCommand>>,
}

impl GraphicsContext {
    /// Creates a new, empty, graphics context
    pub(crate) fn new() -> Self {
        GraphicsContext {
            render_commands: Mutex::new(Vec::new()),
        }
    }

    /// Returns the renderable commands contained within the context
    pub(crate) fn consume(self) -> Vec<RenderCommand> {
        self.render_commands.into_inner().unwrap()
    }

    /// Submits a render command for the given combination of [Mesh] and [Material],
    /// to be rendered using object-to-world [Mat4] `object_to_world`
    pub fn render(&self, mesh: &Mesh, material: &Material, object_to_world: Mat4) {
        let mut locked = self.render_commands.lock().unwrap();

        locked.push(RenderCommand {
            mesh: mesh.0.clone(),
            material: material.0.clone(),
            object_to_world,
        });
    }
}
