use glam::Mat4;

use crate::asset::AssetHandle;
use crate::builtins::components::Transform;
use crate::component::Component;
use crate::graphics;
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::system::Phase;

/// A static mesh renderer
#[derive(Debug, Default)]
pub struct StaticMeshRenderer {
    mesh: AssetHandle<Mesh>,
    material: AssetHandle<Material>,
}

/// Public API
impl StaticMeshRenderer {
    /// Returns a new unconfigured [StaticMeshRenderer]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the mesh to render to the provided mesh
    pub fn set_mesh(&mut self, mesh: AssetHandle<Mesh>) {
        self.mesh = mesh;
    }

    /// Sets the material this renderer uses to the provided material
    pub fn set_material(&mut self, material: AssetHandle<Material>) {
        self.material = material;
    }
}

impl Component for StaticMeshRenderer {
    fn insert_default_component_systems(manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
        manifest.add_system::<(&Self, Option<&Transform>)>(
            Phase::PreRender,
            Some("StaticMeshRenderer submit draw call"),
            |_, (this, transform)| {
                this.submit_draw_call(
                    transform
                        .map(|xform| xform.local_to_world())
                        .unwrap_or(Mat4::IDENTITY),
                );
            },
        );
    }
}

/// System implementations
impl StaticMeshRenderer {
    fn submit_draw_call(&self, transform: Mat4) {
        let (Some(mesh), Some(mat)) = (self.mesh.get_arc(), self.material.get_arc()) else {
            log::trace!(
                "Not rendering static mesh renderer because either the mesh or the material is missing"
            );
            return;
        };

        log::trace!("Submitting draw call for static mesh renderer");

        graphics::render_mesh(mesh, mat, transform);
    }
}
