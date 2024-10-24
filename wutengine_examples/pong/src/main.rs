//! Basic Pong example for WutEngine

use std::time::Instant;

use spawn::PongStarterPlugin;
use wutengine::builtins::components::{InputHandler, Transform};
use wutengine::component::{Component, Context};
use wutengine::input::keyboard::{KeyCode, KeyboardInputPlugin};
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::math::{vec3, Vec3};
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;
use wutengine::time::Time;

mod spawn;

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.with_log_config(LogConfig {
        runtime: Some(ComponentLogConfig {
            min_level: log::LevelFilter::Debug,
            output: log::LogOutput::StdOut,
        }),
        ..Default::default()
    });

    runtime.with_plugin(PongStarterPlugin {});
    runtime.with_plugin(KeyboardInputPlugin::new());
    runtime.run::<OpenGLRenderer>();
}

struct BallData {
    start: Instant,
}

impl Component for BallData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update(&mut self, context: &mut Context) {
        let time_since_start = Time::get()
            .frame_start
            .duration_since(self.start)
            .as_secs_f32()
            * 3.0;

        let transform = context.gameobject.get_component_mut::<Transform>().unwrap();

        transform.set_local_pos(vec3(time_since_start.sin() * 0.6, 0.0, 0.0));
    }
}

pub struct PlayerMovement {
    move_speed: f32,
}

impl Component for PlayerMovement {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update(&mut self, context: &mut Context) {
        let move_speed = self.move_speed;
        let movement_this_frame = move_speed * Time::get().delta;

        let input = context.gameobject.get_component::<InputHandler>().unwrap();

        let mut movement_vec = Vec3::ZERO;

        if input.is_pressed(KeyCode::ArrowUp) {
            movement_vec += vec3(0.0, movement_this_frame, 0.0);
        }

        if input.is_pressed(KeyCode::ArrowDown) {
            movement_vec += vec3(0.0, -movement_this_frame, 0.0);
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
