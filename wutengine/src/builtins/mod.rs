use camera::Camera;
use name::Name;
use wutengine_core::component::ComponentTypeId;

use crate::{component::storage::StorageKind, RuntimeInitializer};

pub mod camera;
pub mod name;

pub(crate) const ID_CAMERA: ComponentTypeId = ComponentTypeId::from_int(0);
pub(crate) const ID_NAME: ComponentTypeId = ComponentTypeId::from_int(1);

pub(crate) fn register_builtins(runtime_init: &mut RuntimeInitializer) {
    log::debug!("Registering builtin components");

    runtime_init.add_component_type_with_storage::<Camera>(StorageKind::Array);
    runtime_init.add_component_type_with_storage::<Name>(StorageKind::Array);
}
