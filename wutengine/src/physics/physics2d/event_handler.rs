use std::sync::mpsc::{channel, Receiver, Sender};

use rapier2d::prelude::*;

#[derive(Debug)]
pub(super) struct SimpleChannelEventCollector {
    collision_sender: Sender<CollisionEvent>,
    contact_force_sender: Sender<ContactForceEvent>,
}

impl SimpleChannelEventCollector {
    pub(super) fn new() -> (Self, Receiver<CollisionEvent>, Receiver<ContactForceEvent>) {
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
