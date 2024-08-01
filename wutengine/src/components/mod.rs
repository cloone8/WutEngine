use camera::Camera;
use mesh::Mesh;
use name::Name;
use wutengine_core::ComponentTypeId;

use crate::runtime::RuntimeInitializer;
use crate::storage::StorageKind;

pub mod camera;
pub mod mesh;
pub mod name;

pub(crate) const ID_CAMERA: ComponentTypeId = ComponentTypeId::from_int(0);
pub(crate) const ID_NAME: ComponentTypeId = ComponentTypeId::from_int(1);
pub(crate) const ID_MESH: ComponentTypeId = ComponentTypeId::from_int(2);

pub(crate) fn register_builtins(runtime_init: &mut RuntimeInitializer) {
    log::debug!("Registering builtin components");

    runtime_init.add_builtin::<Camera>(StorageKind::Array);
    runtime_init.add_builtin::<Name>(StorageKind::Array);
    runtime_init.add_builtin::<Mesh>(StorageKind::Array);
}
