//! 2D Physics logic

use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::Mutex;

use collider_meta::ColliderMeta;
use event_handler::SimpleChannelEventCollector;
use glam::{Vec2, vec2};
use rapier::RapierStructs2D;
use rapier2d::prelude::*;

use crate::gameobject::GameObjectId;
use crate::physics::{CollisionEnd, CollisionStart};
use crate::plugins::{Context, WutEnginePlugin};
use crate::time::Time;

mod collider_meta;
mod event_handler;
mod id;
mod rapier;

pub use id::*;

/// A 2D physics pipeline
#[derive(Debug)]
pub struct Physics2DPlugin {
    collider_meta: Mutex<HashMap<Collider2DID, ColliderMeta>>,
    rapier: Mutex<RapierStructs2D>,
}

impl Default for Physics2DPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[profiling::all_functions]
impl Physics2DPlugin {
    /// Creates and initializes a new 2D physics pipeline
    pub fn new() -> Self {
        Self {
            collider_meta: Mutex::new(HashMap::default()),
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
    pub(crate) fn add_collider(
        &self,
        collider: Collider,
        gameobject: GameObjectId,
    ) -> Collider2DID {
        let id = Collider2DID::new(self.rapier.lock().unwrap().colliders.insert(collider));

        let mut locked = self.collider_meta.lock().unwrap();
        locked.insert(id, ColliderMeta { gameobject });

        id
    }

    /// Updates a collider to a new translation
    pub(crate) fn update_collider(&self, collider: Collider2DID, translation: Vec2) {
        self.rapier
            .lock()
            .unwrap()
            .colliders
            .get_mut(collider.raw)
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
            let id1 = Collider2DID::new(collision_event.collider1());
            let id2 = Collider2DID::new(collision_event.collider2());
            let collider1_pos = Vec2::from(
                rapier
                    .colliders
                    .get(collision_event.collider1())
                    .unwrap()
                    .position()
                    .translation,
            );

            let collider2_pos = Vec2::from(
                rapier
                    .colliders
                    .get(collision_event.collider2())
                    .unwrap()
                    .position()
                    .translation,
            );

            let collider_meta_map = self.collider_meta.get_mut().unwrap();

            let meta1 = collider_meta_map.get(&id1).expect("Missing collider meta!");
            let meta2 = collider_meta_map.get(&id2).expect("Missing collider meta!");

            if collision_event.started() {
                context.message.send_gameobject(
                    CollisionStart {
                        other: meta2.gameobject,
                        other_pos: collider2_pos,
                    },
                    meta1.gameobject,
                );

                context.message.send_gameobject(
                    CollisionStart {
                        other: meta1.gameobject,
                        other_pos: collider1_pos,
                    },
                    meta2.gameobject,
                );
            } else {
                context.message.send_gameobject(
                    CollisionEnd {
                        other: meta2.gameobject,
                    },
                    meta1.gameobject,
                );

                context.message.send_gameobject(
                    CollisionEnd {
                        other: meta1.gameobject,
                    },
                    meta2.gameobject,
                );
            }
        }

        while let Ok(contact_force_event) = contact_force_recv.try_recv() {
            log::info!("CONTACT FORCE EVENT: {:?}", contact_force_event);
        }
    }
}

#[profiling::all_functions]
impl WutEnginePlugin for Physics2DPlugin {
    fn physics_solver_update(&mut self, context: &mut Context) {
        self.step(Time::get().fixed_delta, context);
    }
}
