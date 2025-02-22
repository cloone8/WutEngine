use wutengine::builtins::components::transform::Transform;
use wutengine::component::{Component, Context};
use wutengine::log;
use wutengine::macros::component_boilerplate;
use wutengine::math::{Vec2, Vec3Swizzles};
use wutengine::physics::CollisionStart;
use wutengine::runtime::messaging::Message;
use wutengine::time::Time;

#[derive(Debug)]
pub struct DoReverseMessage;

#[derive(Debug)]
pub struct BallData {
    pub speed: f32,
    pub direction: Vec2,
}

impl BallData {
    fn do_step(&mut self, context: &mut Context) {
        let transform = context.gameobject.get_component_mut::<Transform>().unwrap();
        let cur_pos = transform.local_pos();

        let actual_direction = self.direction.normalize() * self.speed;

        transform.set_local_pos(cur_pos + actual_direction.extend(0.0) * Time::get().fixed_delta);

        wutengine::global::replace(BallPosition {
            pos: transform.world_pos().xy(),
        })
        .unwrap();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BallPosition {
    pub pos: Vec2,
}

impl Component for BallData {
    component_boilerplate!();

    fn on_start(&mut self, context: &mut Context) {
        let transform = context.gameobject.get_component::<Transform>().unwrap();

        wutengine::global::create(BallPosition {
            pos: transform.world_pos().xy(),
        })
        .unwrap();
    }

    fn physics_update(&mut self, context: &mut Context) {
        self.do_step(context);

        let transform = context.gameobject.get_component::<Transform>().unwrap();
        let cur_pos = transform.world_pos();

        if cur_pos.x.abs() > 1.2 {
            let player_won = cur_pos.x.is_sign_positive();

            if player_won {
                log::info!("PLAYER WON");
            } else {
                log::info!("ENEMY WON");
            }

            wutengine::runtime::exit();
        }

        if cur_pos.y.abs() > (1.0 - (transform.lossy_scale().max_element() / 2.0)) {
            self.direction.y *= -1.0;
        }
    }

    fn on_message(&mut self, _context: &mut Context, message: &Message) {
        if message.try_cast::<DoReverseMessage>().is_some() {
            self.direction.x *= -1.0;
        }

        if let Some(collision) = message.try_cast::<CollisionStart>() {
            let my_pos = _context
                .gameobject
                .get_component::<Transform>()
                .unwrap()
                .world_pos()
                .xy();

            self.direction.x *= -1.0;
            self.direction.x += 0.3 * self.direction.x.signum();

            let new_y_speed = self.direction.y.abs() + 0.3;

            if my_pos.y > collision.other_pos.y {
                self.direction.y = new_y_speed;
            } else {
                self.direction.y = -new_y_speed;
            }
        }
    }
}
