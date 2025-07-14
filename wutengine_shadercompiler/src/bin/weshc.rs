//! WutEngine ShaderCompiler binary. Compiles shaders on the command line for debugging purposes

use std::collections::HashMap;
use std::io::Read;
use std::process::{ExitCode, exit};

use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode};
use wutengine_shadercompiler::{KeywordValue, compile};

fn main() -> ExitCode {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    simplelog::TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    let filepath = std::env::args().nth(1).unwrap();
    let mut file = std::fs::File::open(filepath).unwrap();

    let mut shader = String::new();
    file.read_to_string(&mut shader).unwrap();

    let mut keywords = HashMap::new();

    keywords.insert("HAS_COLOR_MAP", KeywordValue::Range(0..=1));
    keywords.insert("COLOR_MULTIPLIER", KeywordValue::Range(0..=5));

    let compiled = match compile(&shader, &keywords) {
        Ok(shaders) => shaders,
        Err(e) => {
            log::error!("Failed to compile a shader: {e}");
            return ExitCode::FAILURE;
        }
    };

    dbg!(compiled);

    ExitCode::SUCCESS
}
