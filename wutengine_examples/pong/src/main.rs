use core::str::FromStr;
use std::{fs::File, io::BufReader, path::PathBuf};

use ball::Ball;
use enemy::Enemy;
use player::Player;
use wutengine::{
    loading::script::ScriptLoaders,
    math::{Quat, Vec3},
    renderer::HeadlessRenderer,
    script::Script,
    serialization::{
        format::{json::Json, SerializationFormat},
        object::SerializedObject,
        scene::SerializedScene,
        script::SerializedScript,
        transform::SerializedTransform,
    },
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

    let engine = WutEngine::<HeadlessRenderer, Json>::new(
        1,
        script_loaders,
        PathBuf::from_str("wutengine_examples/pong/assets/main_scene.json")
            .unwrap()
            .as_path(),
    );

    // engine.run().unwrap();
}
