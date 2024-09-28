//! Basic Pong example for WutEngine

use std::time::Instant;

use spawn::PongStarterPlugin;
use wutengine::builtins::components::util::framerate_counter_system;
use wutengine::builtins::components::{InputHandler, Transform};
use wutengine::command::Command;
use wutengine::core::{EntityId, SystemPhase};
use wutengine::input::keyboard::{KeyCode, KeyboardInputPlugin};
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::macros::{system, Component};
use wutengine::math::{vec3, Vec3};
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;

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
    runtime.with_system::<ball_mover>(SystemPhase::Update);
    runtime.with_system::<player_movement>(SystemPhase::Update);
    runtime.with_system::<framerate_counter_system>(SystemPhase::Update);
    runtime.run::<OpenGLRenderer>();
}

#[derive(Component)]
struct BallData {
    start: Instant,
}

#[system]
fn ball_mover(
    _commands: &mut Command,
    _entity: EntityId,
    ball: &BallData,
    transform: &mut Transform,
) {
    let time = Instant::now().duration_since(ball.start).as_secs_f32() * 3.0;
    transform.set_local_pos(vec3(time.sin() * 0.6, 0.0, 0.0));
}

#[derive(Component)]
pub struct PlayerMovement {
    prev_time: Instant,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerMovement {
    pub fn new() -> Self {
        PlayerMovement {
            prev_time: Instant::now(),
        }
    }
}

#[system]
fn player_movement(
    _commands: &mut Command,
    _entity: EntityId,
    input: &InputHandler,
    player: &mut PlayerMovement,
    transform: &mut Transform,
) {
    let cur_time = Instant::now();
    let delta = cur_time.duration_since(player.prev_time).as_secs_f64();
    player.prev_time = cur_time;

    let move_speed = 1.0;
    let movement_this_frame = move_speed * delta;

    let mut movement_vec = Vec3::ZERO;

    if input.is_pressed(KeyCode::ArrowUp) {
        movement_vec += vec3(0.0, movement_this_frame as f32, 0.0);
    }

    if input.is_pressed(KeyCode::ArrowDown) {
        movement_vec += vec3(0.0, -movement_this_frame as f32, 0.0);
    }

    transform.set_local_pos(transform.local_pos() + movement_vec);
}
