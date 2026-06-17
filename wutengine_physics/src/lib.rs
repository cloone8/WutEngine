#![cfg_attr(phys2d, doc = include_str!("../../wutengine_physics2d/README.md"))]
#![cfg_attr(phys3d, doc = include_str!("../../wutengine_physics3d/README.md"))]

use std::sync::RwLock;
use std::sync::mpsc::Receiver;

use collider::ColliderId;
use nohash_hasher::IntMap;
use wutengine_util::InitOnce;

#[cfg(all(phys2d, phys3d))]
compile_error!("Cannot enable both 2D and 3D physics");

pub mod collider;
pub mod rigidbody;

#[cfg(phys2d)]
pub use rapier2d as rapier;

#[cfg(phys3d)]
use rapier3d as rapier;

use rapier::prelude::*;

#[cfg(phys2d)]
mod types {
    #![allow(clippy::missing_docs_in_private_items, reason = "Type aliases only")]
    //! Dynamic type aliases. Used to abstract over 2D/3D physics

    pub(crate) type VecX = wutengine_math::Vec2;

    pub(crate) type ColliderPose = (wutengine_math::Vec2, f32);
}

#[cfg(phys3d)]
mod types {
    #![allow(clippy::missing_docs_in_private_items, reason = "Type aliases only")]
    //! Dynamic type aliases. Used to abstract over 2D/3D physics

    pub(crate) type VecX = wutengine_math::Vec3;

    pub(crate) type ColliderPose = (wutengine_math::Vec3, wutengine_math::Quat);
}

use types::*;

/// Global physics manager
pub(crate) static PHYSICS_MANAGER: InitOnce<RwLock<PhysicsManager>> = InitOnce::new();

/// Initialize the physics system
#[doc(hidden)]
pub fn init() {
    InitOnce::init(&PHYSICS_MANAGER, RwLock::new(PhysicsManager::new()));
}

/// API entrypoint in order to update the physics world synchronously
#[derive(derive_more::Debug)]
pub struct PhysicsWorldUpdater<'a> {
    /// A reference to the manager
    #[debug(skip)]
    manager: &'a mut PhysicsManager,
}

impl<'a> PhysicsWorldUpdater<'a> {
    /// Adds a new collider to the world, returning a handle to it
    pub fn add_collider(&mut self, mut builder: ColliderBuilder) -> crate::collider::Collider {
        let id = ColliderId::new();
        builder = builder.active_events(ActiveEvents::all());
        builder = builder.active_collision_types(ActiveCollisionTypes::all());

        let collider = builder.build();

        log::info!(
            "Adding new collider {id} of type {:?}",
            collider.shape().shape_type()
        );

        let handle = self.manager.collider_set.insert(collider);

        self.manager.collider_map.insert(id, handle);

        crate::collider::Collider(id)
    }

    /// Deletes an existing collider from the physics world
    pub fn delete_collider(&mut self, collider: crate::collider::Collider) {
        let Some(handle) = self.manager.collider_map.remove(&collider.0) else {
            log::error!("Tried to delete unknown collider: {}", collider.0);
            return;
        };

        log::info!("Deleting collider {}", collider.0);

        let old = self.manager.collider_set.remove(
            handle,
            &mut self.manager.island_manager,
            &mut self.manager.rigidbody_set,
            true,
        );

        assert!(old.is_some(), "Removed collider unknown in rapier");
    }

    /// Moves an existing collider to a new position in world space
    pub fn move_collider(&mut self, collider: &crate::collider::Collider, pose: ColliderPose) {
        log::debug!("Moving collider {} to {} {}", collider.0, pose.0, pose.1);

        let handle = self.manager.collider_map.get(&collider.0).unwrap();
        let collider = self.manager.collider_set.get_mut(*handle).unwrap();

        if collider.parent().is_some() {
            collider.set_position_wrt_parent(collider::make_pose(pose));
        } else {
            collider.set_position(collider::make_pose(pose));
        }
    }
}

/// Locks the physics world and calls the given callback, which will receive a handle
/// to a [PhysicsWorldUpdater].
pub fn update_physics_world(cb: impl FnOnce(&mut PhysicsWorldUpdater)) {
    let mut manager_lock = PHYSICS_MANAGER.write().unwrap();

    let mut updater = PhysicsWorldUpdater {
        manager: &mut manager_lock,
    };

    cb(&mut updater);
}

/// Runs the physics simulation for one frame
pub fn step(dt: f32) {
    profiling::function_scope!();

    PHYSICS_MANAGER.write().unwrap().step(dt);
}

/// Physics manager
pub(crate) struct PhysicsManager {
    /// Gravity vector
    gravity: VecX,

    /// All rigidbodies
    rigidbody_set: RigidBodySet,

    /// Map from public collider IDs to rapier IDs
    collider_map: IntMap<ColliderId, ColliderHandle>,

    /// All colliders
    collider_set: ColliderSet,

    /// Integration parameters
    integration_parameters: IntegrationParameters,

    /// The physics pipeline
    physics_pipeline: PhysicsPipeline,

    /// The rapier islang manager
    island_manager: IslandManager,

    /// Broad phase
    broad_phase: DefaultBroadPhase,

    /// Narrow phase
    narrow_phase: NarrowPhase,

    /// All impulse joints
    impulse_joint_set: ImpulseJointSet,

    /// All multibody joints
    multibody_joint_set: MultibodyJointSet,

    /// CCD solver
    ccd_solver: CCDSolver,
}

impl PhysicsManager {
    /// Create a new, empty, physics manager
    fn new() -> Self {
        PhysicsManager {
            gravity: VecX::ZERO.with_y(-9.81),
            rigidbody_set: RigidBodySet::new(),
            collider_map: IntMap::default(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    /// Step the physics simulation by a given delta-time
    fn step(&mut self, dt: f32) {
        profiling::function_scope!();

        log::trace!("Stepping simulation with dt: {dt}");

        self.integration_parameters.dt = dt;

        let (collision_send, collision_recv) = std::sync::mpsc::channel();
        let (contact_force_send, contact_force_recv) = std::sync::mpsc::channel();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        self.physics_pipeline.step(
            self.gravity.to_rapier(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbody_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &event_handler,
        );

        let result_handler = PhysicsResultHandler {
            collisions: collision_recv,
            contact_force: contact_force_recv,
        };

        result_handler.handle();
    }
}

/// Handles the results of a physics step
#[derive(Debug)]
struct PhysicsResultHandler {
    /// Collision event receiver
    collisions: Receiver<rapier::geometry::CollisionEvent>,

    /// Contact force event receiver
    contact_force: Receiver<rapier::geometry::ContactForceEvent>,
}

impl PhysicsResultHandler {
    /// Handles all pending events
    fn handle(&self) {
        profiling::function_scope!();

        {
            profiling::scope!("Collisions");

            for collision in self.collisions.try_iter() {
                Self::handle_collision_event(collision);
            }
        }

        {
            profiling::scope!("Contact Forces");

            for contact_force in self.contact_force.try_iter() {
                Self::handle_contact_force_event(contact_force);
            }
        }
    }

    /// Handles a single collision event
    fn handle_collision_event(collision: rapier::geometry::CollisionEvent) {
        profiling::function_scope!();

        log::trace!("Handling collision event {:?}", collision);
    }

    /// Handles a single contact force event
    fn handle_contact_force_event(contact_force: rapier::geometry::ContactForceEvent) {
        profiling::function_scope!();

        log::trace!("Handling contact force event {:#?}", contact_force);
    }
}

/// Easier inline rapier-wutengine type conversion
trait RapierConversion<T> {
    /// Convert to rapier
    fn to_rapier(self) -> T;

    /// Convert from rapier
    #[expect(unused, reason = "Will be used once basic physics have been added")]
    fn from_rapier(val: T) -> Self;
}

impl RapierConversion<rapier::math::Vector> for VecX {
    #[inline(always)]
    fn to_rapier(self) -> rapier::math::Vector {
        rapier::math::Vector::from_array(self.to_array())
    }

    #[inline(always)]
    fn from_rapier(val: rapier::math::Vector) -> Self {
        Self::from_array(val.to_array())
    }
}
