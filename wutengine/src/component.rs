//! WutEngine components and component helpers

use core::any::Any;
use core::any::TypeId;
use std::collections::HashSet;
use std::sync::LazyLock;
use std::sync::RwLock;

static ADDED_DEFAULT_COMPONENT_SYSTEMS: LazyLock<RwLock<HashSet<TypeId>>> =
    LazyLock::new(|| RwLock::new(HashSet::default()));

/// Checks whether the default component systems for `C` should be inserted into the
/// global system manager. If `true`, the caller _should_ call [Component::insert_default_component_systems]
/// for the given component type, because no other call to [should_insert_default_component_systems] will return `true`
/// again
pub(crate) fn should_insert_default_component_systems<T: Component>() -> bool {
    let ty = core::any::TypeId::of::<T>();

    let already_added = {
        ADDED_DEFAULT_COMPONENT_SYSTEMS
            .read()
            .unwrap()
            .contains(&ty)
    };

    if already_added {
        return false;
    }

    // This expression returns `true` if the typeid was succesfully inserted, which could
    // fail if another thread got the read lock for this type before we did
    ADDED_DEFAULT_COMPONENT_SYSTEMS.write().unwrap().insert(ty)
}

/// Trait that should be implemented by types that can be
/// used as components in the WutEngine ECS
pub trait Component: Any + Send + Sync {
    /// The identifier for this component type. Most be globally unique to all
    /// other UUIDs.
    ///
    /// Hint: To obtain one, you can generate a random V4 UUID from many websites,
    /// and then use the [uuid macro](uuid::uuid) to embed it at compile time
    const ID: uuid::Uuid;

    /// Adds the systems that are always used by this component into the given manifest.
    ///
    /// Optional usability helper
    fn insert_default_component_systems(_manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
    }
}
