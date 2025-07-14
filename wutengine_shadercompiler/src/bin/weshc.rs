//! WutEngine ShaderCompiler binary. Compiles shaders on the command line for debugging purposes

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::{ExitCode, exit};

use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode};
use wutengine_shadercompiler::{KeywordValue, compile, compile_with_callback};

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
    let outdir = PathBuf::from(std::env::args().nth(2).unwrap());
    let mut file = std::fs::File::open(filepath).unwrap();
    std::fs::create_dir_all(&outdir).unwrap();

    let mut shader = String::new();
    file.read_to_string(&mut shader).unwrap();

    let mut keywords = HashMap::new();

    keywords.insert("HAS_COLOR_MAP", KeywordValue::Range(0..=50));
    keywords.insert("COLOR_MULTIPLIER", KeywordValue::Range(0..=6000));
    keywords.insert("SUPER_TEST", KeywordValue::Range(0..=500));

    compile_with_callback(&shader, &keywords, |result| match result {
        Ok(shader) => {
            let outfile = outdir.join(format!("{}.txt", shader.keyword_hash));
            std::fs::write(outfile, format!("{shader:#?}")).unwrap();
        }
        Err(e) => {
            log::error!("Failed to compile a shader: {e}");
            std::process::exit(1);
        }
    });

    ExitCode::SUCCESS
}
