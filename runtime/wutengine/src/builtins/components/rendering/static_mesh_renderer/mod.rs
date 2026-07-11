use wutengine_asset_server::AutoLoad;
use wutengine_math::Mat4;

use crate::{
    builtins::components::Transform,
    component::Component,
    graphics,
    graphics::{material::Material, mesh::Mesh},
    system::Phase,
};

/// A static mesh renderer
#[derive(Debug, Default)]
pub struct StaticMeshRenderer {
    mesh: AutoLoad<Mesh>,
    material: AutoLoad<Material>,
}

/// Public API
impl StaticMeshRenderer {
    /// Returns a new unconfigured [StaticMeshRenderer]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the mesh to render to the provided mesh
    pub fn set_mesh(&mut self, mesh: AutoLoad<Mesh>) {
        self.mesh = mesh;
    }

    /// Sets the material this renderer uses to the provided material
    pub fn set_material(&mut self, material: AutoLoad<Material>) {
        self.material = material;
    }
}

impl Component for StaticMeshRenderer {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("cb4802f7-5810-4354-be68-d51a2c44f1f9")).unwrap();

    fn insert_default_component_systems(manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
        manifest.add_system::<(&Self, Option<&Transform>)>(
            Phase::PreRender,
            "StaticMeshRenderer submit draw call",
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
        let (Some(mesh), Some(mat)) = (self.mesh.try_get(), self.material.try_get()) else {
            log::trace!(
                "Not rendering static mesh renderer because either the mesh or the material is missing"
            );
            return;
        };

        log::trace!("Submitting draw call for static mesh renderer");

        graphics::render_mesh(mesh, mat, transform);
    }
}
