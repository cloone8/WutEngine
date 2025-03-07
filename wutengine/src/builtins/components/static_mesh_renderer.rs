//! Static mesh rendering components

use std::any::Any;

use glam::Mat4;

use crate::builtins::assets::{Material, Mesh};
use crate::builtins::components::transform::Transform;
use crate::component::{Component, Context};

/// A static mesh renderer component. Renders its configured mesh using its configured
/// material
#[derive(Debug)]
pub struct StaticMeshRenderer {
    /// The mesh that is rendered
    pub mesh: Mesh,

    /// The material that is rendered
    pub material: Material,
}

impl Component for StaticMeshRenderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn pre_render(&mut self, context: &mut Context) {
        let transform = if let Some(transform) = context.gameobject.get_component::<Transform>() {
            transform.local_to_world()
        } else {
            log::trace!("Transformless renderable entity found, rendering at origin");
            Mat4::IDENTITY
        };

        log::trace!(
            "Pushing renderable mesh {:#?} with material {:#?} and transform {}",
            &self.mesh,
            &self.material,
            transform
        );

        context
            .graphics
            .render(&self.mesh, &self.material, transform);
    }
}
