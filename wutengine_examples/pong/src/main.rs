//! Basic Pong example for WutEngine

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use spawn::PongStarterPlugin;
use wutengine::input::gamepad::GamepadInputPlugin;
use wutengine::input::keyboard::KeyboardInputPlugin;
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::physics::Physics2DPlugin;
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;

mod balldata;
mod enemy;
mod player;
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
