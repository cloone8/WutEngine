#![doc = include_str!("../README.md")]

use wutengine_util::InitOnce;

#[cfg(feature = "phys2d")]
pub mod phys2d;

#[cfg(feature = "phys3d")]
pub mod phys3d;

pub(crate) static PHYSICS_MANAGER: InitOnce<PhysicsManager> = InitOnce::new();

/// Initialize the physics system
pub fn init() {
    InitOnce::init(&PHYSICS_MANAGER, PhysicsManager::new());
}

pub(crate) struct PhysicsManager {
    #[cfg(feature = "phys2d")]
    pub(crate) phys2d: std::sync::RwLock<phys2d::Phys2DManager>,
    #[cfg(feature = "phys3d")]
    pub(crate) phys3d: std::sync::RwLock<phys3d::Phys3DManager>,
}

impl PhysicsManager {
    fn new() -> Self {
        PhysicsManager {
            #[cfg(feature = "phys2d")]
            phys2d: std::sync::RwLock::new(phys2d::Phys2DManager::new()),
            #[cfg(feature = "phys3d")]
            phys3d: std::sync::RwLock::new(phys3d::Phys3DManager::new()),
        }
    }
}
