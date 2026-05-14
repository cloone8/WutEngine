use crate::asset::AssetHandle;
use crate::component::Component;
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mesh(&mut self, mesh: AssetHandle<Mesh>) {
        self.mesh = mesh;
    }

    pub fn set_material(&mut self, material: AssetHandle<Material>) {
        self.material = material;
    }
}

impl Component for StaticMeshRenderer {
    fn insert_default_component_systems(manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
        manifest.add_system::<&mut Self>(
            Phase::PreRender,
            Some("StaticMeshRenderer submit draw call"),
            |_, this| {
                this.submit_draw_call();
            },
        );
    }
}

/// System implementations
impl StaticMeshRenderer {
    fn submit_draw_call(&mut self) {
        let (Some(mesh), Some(mat)) = (self.mesh.get(), self.material.get()) else {
            log::trace!(
                "Not rendering static mesh renderer because either the mesh or the material is missing"
            );
            return;
        };

        log::info!("Submitting draw call");
    }
}
