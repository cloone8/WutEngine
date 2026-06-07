//! Low-level physics systems

use std::sync::RwLock;

use phys2d::Phys2DManager;
use phys3d::Phys3DManager;

use wutengine_util::InitOnce;

pub mod phys2d;
pub mod phys3d;

pub(crate) static PHYSICS_MANAGER: InitOnce<PhysicsManager> = InitOnce::new();

/// Initialize the physics system
pub(crate) fn init() {
    InitOnce::init(&PHYSICS_MANAGER, PhysicsManager::new());
}

pub(crate) struct PhysicsManager {
    pub(crate) phys2d: RwLock<Phys2DManager>,
    pub(crate) phys3d: RwLock<Phys3DManager>,
}

impl PhysicsManager {
    fn new() -> Self {
        PhysicsManager {
            phys2d: RwLock::new(Phys2DManager::new()),
            phys3d: RwLock::new(Phys3DManager::new()),
        }
    }
}
