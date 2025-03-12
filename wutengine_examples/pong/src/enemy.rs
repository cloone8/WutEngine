use wutengine::builtins::components::transform::Transform;
use wutengine::component::{Component, Context};
use wutengine::macros::component_boilerplate;
use wutengine::math::vec3;
use wutengine::time::Time;

use crate::balldata::BallPosition;

#[derive(Debug)]
pub struct Enemy {
    pub max_speed: f32,
    pub acceleration: f32,
    cur_speed: f32,
}

impl Enemy {
    pub fn new(max_speed: f32, acceleration: f32) -> Self {
        Self {
            max_speed,
            acceleration,
            cur_speed: 0.0,
        }
    }
}

impl Component for Enemy {
    component_boilerplate!();

    fn update(&mut self, context: &mut Context) {
        let transform = context.gameobject.get_component_mut::<Transform>().unwrap();

        let ball_global = wutengine::global::find_clone::<BallPosition>().unwrap();

        if ball_global.pos.x < 0.0 {
            self.cur_speed = 0.0;
            return;
        }

        if (ball_global.pos.y - transform.world_pos().y).abs() < 0.02 {
            self.cur_speed = 0.0;
            return;
        }

        if ball_global.pos.y > transform.world_pos().y {
            self.cur_speed = self.cur_speed.max(0.0);

            self.cur_speed += self.acceleration * Time::get().delta;
            self.cur_speed = self.cur_speed.min(self.max_speed);
        } else {
            self.cur_speed = self.cur_speed.min(0.0);

            self.cur_speed -= self.acceleration * Time::get().delta;
            self.cur_speed = self.cur_speed.max(-self.max_speed);
        }

        transform.set_local_pos(
            transform.local_pos() + vec3(0.0, self.cur_speed * Time::get().delta, 0.0),
        );
    }
}
