use core::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use glam::{vec2, Vec2};
use rapier2d::prelude::*;

use crate::plugins::{Context, WutEnginePlugin};
use crate::time::Time;

/// A 2D physics pipeline
#[derive(Debug)]
pub struct Physics2DPlugin {
    rapier: Mutex<RapierStructs2D>,
}

struct RapierStructs2D {
    rigids: RigidBodySet,
    colliders: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    parameters: IntegrationParameters,
    gravity: Vec2,
    island_manager: IslandManager,
    broad: DefaultBroadPhase,
    narrow: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl Debug for RapierStructs2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RapierStructs2D")
            .field("rigids", &self.rigids)
            .field("colliders", &self.colliders)
            .field("physics_pipeline", &"<no debug>")
            .field("parameters", &self.parameters)
            .field("gravity", &self.gravity)
            .field("island_manager", &"<no debug>")
            .field("broad", &"<no debug>")
            .field("narrow", &"<no debug>")
            .field("impulse_joints", &self.impulse_joints)
            .field("multibody_joints", &self.multibody_joints)
            .field("ccd_solver", &"<no debug>")
            .field("query_pipeline", &"<no debug>")
            .finish()
    }
}

impl Default for Physics2DPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Physics2DPlugin {
    /// Creates and initializes a new 2D physics pipeline
    pub fn new() -> Self {
        Self {
            rapier: Mutex::new(RapierStructs2D {
                rigids: RigidBodySet::new(),
                colliders: ColliderSet::new(),
                physics_pipeline: PhysicsPipeline::new(),
                parameters: IntegrationParameters::default(),
                gravity: vec2(0.0, -9.81),
                island_manager: IslandManager::new(),
                broad: DefaultBroadPhase::new(),
                narrow: NarrowPhase::new(),
                impulse_joints: ImpulseJointSet::new(),
                multibody_joints: MultibodyJointSet::new(),
                ccd_solver: CCDSolver::new(),
                query_pipeline: QueryPipeline::new(),
            }),
        }
    }

    /// Adds a raw collider to the solver
    pub(crate) fn add_collider(&self, collider: Collider) -> ColliderHandle {
        self.rapier.lock().unwrap().colliders.insert(collider)
    }

    /// Updates a collider to a new translation
    pub(crate) fn update_collider(&self, collider: ColliderHandle, translation: Vec2) {
        self.rapier
            .lock()
            .unwrap()
            .colliders
            .get_mut(collider)
            .unwrap()
            .set_translation(translation.into());
    }

    /// Steps the physics library
    pub(crate) fn step(&mut self, dt: f32, context: &mut Context) {
        log::trace!("Stepping 2D physics solver");

        let rapier = self.rapier.get_mut().unwrap();

        rapier.parameters.dt = dt;

        let (event_handler, collision_recv, contact_force_recv) =
            SimpleChannelEventCollector::new();

        rapier.physics_pipeline.step(
            &rapier.gravity.into(),
            &rapier.parameters,
            &mut rapier.island_manager,
            &mut rapier.broad,
            &mut rapier.narrow,
            &mut rapier.rigids,
            &mut rapier.colliders,
            &mut rapier.impulse_joints,
            &mut rapier.multibody_joints,
            &mut rapier.ccd_solver,
            Some(&mut rapier.query_pipeline),
            &(),
            &event_handler,
        );

        while let Ok(collision_event) = collision_recv.try_recv() {
            log::info!("COLLISION EVENT: {:?}", collision_event);
            if !collision_event.started() {
                continue;
            }

            context.message.send_global(Collision2D {
                handle1: collision_event.collider1(),
                handle2: collision_event.collider2(),
            });
        }

        while let Ok(contact_force_event) = contact_force_recv.try_recv() {
            log::info!("CONTACT FORCE EVENT: {:?}", contact_force_event);
        }
    }
}

impl WutEnginePlugin for Physics2DPlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn physics_solver_update(&mut self, context: &mut Context) {
        self.step(Time::get().fixed_delta, context);
    }
}

#[derive(Debug)]
struct SimpleChannelEventCollector {
    collision_sender: Sender<CollisionEvent>,
    contact_force_sender: Sender<ContactForceEvent>,
}

impl SimpleChannelEventCollector {
    fn new() -> (Self, Receiver<CollisionEvent>, Receiver<ContactForceEvent>) {
        let (coll_send, coll_recv) = channel();
        let (contact_send, contact_recv) = channel();

        let collector = Self {
            collision_sender: coll_send,
            contact_force_sender: contact_send,
        };

        (collector, coll_recv, contact_recv)
    }
}

impl EventHandler for SimpleChannelEventCollector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        self.collision_sender.send(event).unwrap();
    }

    fn handle_contact_force_event(
        &self,
        dt: f32,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: f32,
    ) {
        let result = ContactForceEvent::from_contact_pair(dt, contact_pair, total_force_magnitude);
        self.contact_force_sender.send(result).unwrap();
    }
}

#[derive(Debug)]
pub struct Collision2D {
    pub handle1: ColliderHandle,
    pub handle2: ColliderHandle,
}
