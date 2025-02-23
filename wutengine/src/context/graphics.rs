use core::marker::PhantomData;
use std::sync::Mutex;

use glam::Mat4;
use wutengine_graphics::renderer::Renderable;

use crate::builtins::assets::{Material, Mesh};

/// The graphics context. Used for interacting with graphics related APIs
#[must_use = "The commands within the context must be consumed"]
#[derive(Debug)]
pub struct GraphicsContext<'a> {
    render_commands: Mutex<Vec<Renderable>>,
    ph: PhantomData<&'a ()>,
}

impl GraphicsContext<'_> {
    /// Creates a new, empty, graphics context
    pub(crate) fn new() -> Self {
        GraphicsContext {
            render_commands: Mutex::new(Vec::new()),
            ph: PhantomData,
        }
    }

    /// Returns the renderable commands contained within the context
    pub(crate) fn consume(self) -> Vec<Renderable> {
        self.render_commands.into_inner().unwrap()
    }

    /// Submits a render command for the given combination of [Mesh] and [Material],
    /// to be rendered using object-to-world [Mat4] `object_to_world`
    pub fn render(&self, mesh: &Mesh, material: &Material, object_to_world: Mat4) {
        let mut locked = self.render_commands.lock().unwrap();

        locked.push(Renderable {
            mesh: mesh.data.clone(),
            material: material.data.clone(),
            object_to_world,
        });
    }
}
