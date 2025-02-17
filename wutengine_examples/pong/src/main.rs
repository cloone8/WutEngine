//! Basic Pong example for WutEngine

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use spawn::PongStarterPlugin;
use wutengine::builtins::components::{input::InputHandler, transform::Transform};
use wutengine::component::{Component, Context};
use wutengine::input::gamepad::GamepadAxis::{self};
use wutengine::input::gamepad::{GamepadButton, GamepadInputPlugin};
use wutengine::input::keyboard::{KeyCode, KeyboardInputPlugin};
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::macros::component_boilerplate;
use wutengine::math::{vec3, Vec2, Vec3, Vec3Swizzles};
use wutengine::physics::{CollisionStart, Physics2DPlugin};
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::messaging::Message;
use wutengine::runtime::RuntimeInitializer;
use wutengine::time::Time;

mod spawn;

fn main() {
    let mut runtime = RuntimeInitializer::new();

    if cfg!(debug_assertions) {
        runtime.with_log_config(LogConfig {
            runtime: Some(ComponentLogConfig {
                min_level: log::LevelFilter::Debug,
                output: log::LogOutput::StdOut,
            }),
            ..Default::default()
        });
    } else {
        runtime.with_log_config(LogConfig {
            runtime: Some(ComponentLogConfig {
                min_level: log::LevelFilter::Info,
                output: log::LogOutput::File(PathBuf::from("./wutengine_runtime.log")),
            }),
            ..Default::default()
        });
    }

    runtime.with_plugin(PongStarterPlugin {});
    runtime.with_plugin(KeyboardInputPlugin::new());
    runtime.with_plugin(GamepadInputPlugin::new());
    runtime.with_plugin(Physics2DPlugin::new());
    runtime.run::<OpenGLRenderer>();
}

#[derive(Debug)]
struct DoReverseMessage;

#[derive(Debug)]
struct BallData {
    speed: f32,
    direction: Vec2,
}

impl BallData {
    fn do_step(&mut self, context: &mut Context) {
        let transform = context.gameobject.get_component_mut::<Transform>().unwrap();
        let cur_pos = transform.local_pos();

        let actual_direction = self.direction.normalize() * self.speed;

        transform.set_local_pos(cur_pos + actual_direction.extend(0.0) * Time::get().fixed_delta);
    }
}

impl Component for BallData {
    component_boilerplate!();

    fn physics_update(&mut self, context: &mut Context) {
        self.do_step(context);

        if context
            .gameobject
            .get_component::<Transform>()
            .unwrap()
            .world_pos()
            .y
            .abs()
            > 1.0
        {
            self.direction.y *= -1.0;
        }
    }

    fn on_message(&mut self, _context: &mut Context, message: &Message) {
        if message.try_cast::<DoReverseMessage>().is_some() {
            self.direction.x *= -1.0;
        }

        if let Some(collision) = message.try_cast::<CollisionStart>() {
            _context.this.destroy();

            let my_pos = _context
                .gameobject
                .get_component::<Transform>()
                .unwrap()
                .world_pos()
                .xy();

            self.direction.x *= -1.0;
            self.direction.y *= -1.0;

            if my_pos.y > collision.other_pos.y {
                self.direction.y += 0.1;
            } else {
                self.direction.y -= 0.1;
            }
        }
    }
}

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
