//! Basic Pong example for WutEngine

use std::time::Instant;

use spawn::PongStarterPlugin;
use wutengine::builtins::components::util::framerate_counter_system;
use wutengine::builtins::components::Transform;
use wutengine::command::Command;
use wutengine::core::{EntityId, SystemPhase};
use wutengine::input::keyboard::KeyboardInputPlugin;
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::macros::{system, Component};
use wutengine::math::vec3;
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
