#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::sync::RwLock;

use wutengine_util::InitOnce;

#[cfg(feature = "phys2d")]
pub mod phys2d;

#[cfg(feature = "phys3d")]
pub mod phys3d;

/// Global physics manager
pub(crate) static PHYSICS_MANAGER: InitOnce<PhysicsManager> = InitOnce::new_checked();

struct PhysicsManager {
    #[cfg(feature = "phys2d")]
    phys2d: RwLock<phys2d::PhysicsManager>,

    #[cfg(feature = "phys3d")]
    phys3d: RwLock<phys3d::PhysicsManager>,
}

impl PhysicsManager {
    fn new() -> Self {
        Self {
            #[cfg(feature = "phys2d")]
            phys2d: RwLock::new(phys2d::PhysicsManager::new()),

            #[cfg(feature = "phys3d")]
            phys3d: RwLock::new(phys3d::PhysicsManager::new()),
        }
    }

    fn step(&self, dt: f32) {
        rayon::join(
            || {
                #[cfg(feature = "phys2d")]
                self.phys2d.write().unwrap().step(dt);
            },
            || {
                #[cfg(feature = "phys3d")]
                self.phys3d.write().unwrap().step(dt);
            },
        );
    }
}

/// Initialize the physics system
#[doc(hidden)]
pub fn init() {
    InitOnce::init(&PHYSICS_MANAGER, PhysicsManager::new());
}

/// Runs the physics simulation for one frame
pub fn step(dt: f32) {
    profiling::function_scope!();

    PHYSICS_MANAGER.step(dt);
}

/// Locks the physics world and calls the given callback, which will receive a handle
/// to an updater struct for each dimension
pub fn update_physics_world(
    #[cfg(feature = "phys2d")] cb_2d: impl FnOnce(&mut phys2d::PhysicsWorldUpdater) + Send,
    #[cfg(feature = "phys3d")] cb_3d: impl FnOnce(&mut phys3d::PhysicsWorldUpdater) + Send,
) {
    profiling::function_scope!();

    rayon::join(
        || {
            #[cfg(feature = "phys2d")]
            {
                profiling::scope!("Update 2D physics world");
                let mut manager_lock = PHYSICS_MANAGER.phys2d.write().unwrap();

                let mut updater = phys2d::PhysicsWorldUpdater {
                    manager: &mut manager_lock,
                };

                cb_2d(&mut updater);
            }
        },
        || {
            #[cfg(feature = "phys3d")]
            {
                profiling::scope!("Update 3D physics world");
                let mut manager_lock = PHYSICS_MANAGER.phys3d.write().unwrap();

                let mut updater = phys3d::PhysicsWorldUpdater {
                    manager: &mut manager_lock,
                };

                cb_3d(&mut updater);
            }
        },
    );
}

/// Easier inline rapier-wutengine type conversion
trait RapierConversion<T> {
    /// Convert to rapier
    fn to_rapier(self) -> T;

    /// Convert from rapier
    #[expect(unused, reason = "Will be used once basic physics have been added")]
    fn from_rapier(val: T) -> Self;
}
