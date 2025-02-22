use wutengine::component::{Component, Context};
use wutengine::macros::component_boilerplate;

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

    fn update(&mut self, context: &mut Context) {}
}
