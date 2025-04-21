//! Static mesh rendering components

use glam::Mat4;

use crate::builtins::assets::{Material, Mesh};
use crate::builtins::components::transform::Transform;
use crate::component::{Component, Context};
use crate::graphics;

/// A static mesh renderer component. Renders its configured mesh using its configured
/// material
#[derive(Debug)]
pub struct StaticMeshRenderer {
    /// The mesh that is rendered
    pub mesh: Mesh,

    /// The material that is rendered
    pub material: Material,
}

#[profiling::all_functions]
impl Component for StaticMeshRenderer {
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

        graphics::render_mesh(&self.mesh, &self.material, transform);
    }
}
