use core::str::FromStr;
use std::path::PathBuf;

use ball::Ball;
use enemy::Enemy;
use player::Player;
use wutengine_opengl::OpenGLRenderer;
use wutengine_runtime::{
    loading::script::ScriptLoaders, serialization::format::json::Json, settings::Settings,
    WutEngine,
};

mod ball;
mod enemy;
mod player;

fn main() {
    let logconfig = simplelog::ConfigBuilder::new()
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set logger time offset to local")
        .build();

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Debug,
        logconfig,
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .expect("Could not initialize logger");

    let ball = Ball {};

    println!("{}", serde_json::to_string_pretty(&ball).unwrap());

    let mut script_loaders = ScriptLoaders::new();

    script_loaders.register_script::<Ball>();
    script_loaders.register_script::<Enemy>();
    script_loaders.register_script::<Player>();

    let engine = WutEngine::<OpenGLRenderer, Json>::new(
        Settings::default(),
        script_loaders,
        PathBuf::from_str("wutengine_examples/pong/assets/main_scene.json")
            .unwrap()
            .as_path(),
    );

    engine.run().unwrap();
}
