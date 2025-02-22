use wutengine::builtins::components::input::InputHandler;
use wutengine::builtins::components::transform::Transform;
use wutengine::component::{Component, Context};
use wutengine::input::gamepad::{GamepadAxis, GamepadButton};
use wutengine::macros::component_boilerplate;
use wutengine::math::{Vec3, vec3};
use wutengine::time::Time;
use wutengine::winit::keyboard::KeyCode;

use crate::balldata::DoReverseMessage;

#[derive(Debug)]
pub struct PlayerMovement {
    move_speed: f32,
}

impl Component for PlayerMovement {
    component_boilerplate!();

    fn update(&mut self, context: &mut Context) {
        let move_speed = self.move_speed;
        let movement_this_frame = move_speed * Time::get().delta;

        let input = context.gameobject.get_component::<InputHandler>().unwrap();

        let mut movement_vec = Vec3::ZERO;
        let left_stick_val = input.gamepad().axis_value(GamepadAxis::LeftStick).y;

        if input.keyboard().is_down(KeyCode::ArrowUp)
            || input.gamepad().is_down(GamepadButton::DPadUp)
        {
            movement_vec += vec3(0.0, movement_this_frame, 0.0);
        } else if input.keyboard().is_down(KeyCode::ArrowDown)
            || input.gamepad().is_down(GamepadButton::DPadDown)
        {
            movement_vec += vec3(0.0, -movement_this_frame, 0.0);
        } else {
            movement_vec += vec3(0.0, movement_this_frame * left_stick_val, 0.0);
        }

        if input.keyboard().pressed_this_frame(KeyCode::Space)
            || input.gamepad().pressed_this_frame(GamepadButton::South)
        {
            context.message.send_global(DoReverseMessage);
        }

        let transform = context.gameobject.get_component_mut::<Transform>().unwrap();

        transform.set_local_pos(transform.local_pos() + movement_vec);
    }
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerMovement {
    pub fn new() -> Self {
        PlayerMovement { move_speed: 1.0 }
    }
}
