use core::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

use wutengine::builtins::components::Transform;
use wutengine::component::Component;
use wutengine::gameobject::GameObjectId;
use wutengine::math::{vec2, Vec2, Vec3Swizzles};
use wutengine::plugins::{self, WutEnginePlugin};

#[derive(Debug)]
pub struct CollisionPlugin {
    colliders: Mutex<HashMap<u64, BadCollider>>,
    num_colliders: AtomicU64,
}

impl CollisionPlugin {
    pub fn new() -> Self {
        Self {
            colliders: Mutex::new(HashMap::new()),
            num_colliders: AtomicU64::new(0),
        }
    }
    pub fn add_collider(&self, center: Vec2, size: Vec2, gameobject: GameObjectId) -> u64 {
        let new_id = self.num_colliders.fetch_add(1, Ordering::Relaxed);

        self.colliders.lock().unwrap().insert(
            new_id,
            BadCollider {
                gameobject,
                center,
                size,
            },
        );

        new_id
    }

    pub fn update_collider(&self, collider: u64, center: Vec2) {
        let mut locked = self.colliders.lock().unwrap();

        let collider = locked.get_mut(&collider).unwrap();

        collider.center = center;
    }
}

#[derive(Debug, Clone)]
pub struct BadCollider {
    gameobject: GameObjectId,
    center: Vec2,
    size: Vec2,
}

#[derive(Debug)]
pub struct BadColliderComponent {
    collider: Option<u64>,
}

impl BadColliderComponent {
    pub fn new() -> Self {
        BadColliderComponent { collider: None }
    }
}

impl Component for BadColliderComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn pre_update(&mut self, context: &mut wutengine::component::Context) {
        let plugin = context.plugin.get::<CollisionPlugin>();

        if plugin.is_none() {
            return;
        }

        let plugin = plugin.unwrap();
        let transform = context.gameobject.get_component::<Transform>().unwrap();

        if let Some(collider_id) = self.collider {
            plugin.update_collider(collider_id, transform.world_pos().xy());
        } else {
            let collider_id = plugin.add_collider(
                transform.world_pos().xy(),
                transform.lossy_scale().xy(),
                context.gameobject.object.id,
            );

            self.collider = Some(collider_id);
        }
    }
}

#[derive(Debug)]
pub struct CollisionMessage {
    pub other_center: Vec2,
}

impl BadCollider {
    fn corners(&self) -> (Vec2, Vec2) {
        let hsize = self.size.x / 2.0;
        let vsize = self.size.y / 2.0;

        (
            self.center + vec2(-hsize, -vsize),
            self.center + vec2(hsize, vsize),
        )
    }

    fn overlaps_with(&self, other: &Self) -> bool {
        let (my_bottom_left, my_top_right) = self.corners();
        let (other_bottom_left, other_top_right) = other.corners();

        if my_top_right.x < other_bottom_left.x || my_bottom_left.x > other_top_right.x {
            return false;
        }

        if my_top_right.y < other_bottom_left.y || my_bottom_left.y > other_top_right.y {
            return false;
        }

        true
    }
}

impl WutEnginePlugin for CollisionPlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn pre_update(&mut self, context: &mut plugins::Context) {
        let locked = self.colliders.lock().unwrap();

        let colliders: Vec<&BadCollider> = locked.values().collect();

        for i in 0..colliders.len() {
            for j in 0..colliders.len() {
                if j == i {
                    continue;
                }

                let me = colliders[i];
                let other = colliders[j];

                if other.overlaps_with(me) {
                    context.message.send_gameobject(
                        CollisionMessage {
                            other_center: other.center,
                        },
                        me.gameobject,
                    );
                }
            }
        }
    }
}
