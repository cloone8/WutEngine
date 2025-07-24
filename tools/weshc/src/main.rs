//! WutEngine ShaderCompiler binary. Compiles shaders on the command line for debugging purposes

use core::error::Error;
use core::str::FromStr;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc::{Sender, channel};

use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Color, Effects, Style};
use clap::{Parser, ValueEnum};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode};
use wutengine_shadercompiler::{CompileStage, KeywordValue, ShaderOutput, compile_shader};

// Styling for clap CLI

/// Header styling
const HEADER: Style = AnsiColor::Green
    .on_default()
    .effects(Effects::UNDERLINE.insert(Effects::BOLD));

/// Usage styling
const USAGE: Style = AnsiColor::BrightYellow.on_default().effects(Effects::BOLD);

/// Literal styling
const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);

/// Placeholder styling
const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();

/// Error styling
const ERROR: Style = AnsiColor::BrightRed.on_default().effects(Effects::BOLD);

/// Valid styling
const VALID: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);

/// Invalid styling
const INVALID: Style = AnsiColor::BrightMagenta.on_default().effects(Effects::BOLD);

/// Context styling
const CONTEXT: Style = AnsiColor::BrightCyan.on_default();

/// Context valeu styling
const CONTEXT_VALUE: Style = LITERAL;

/// Clap CLI styling
const CLAP_STYLING: Styles = Styles::styled()
    .header(HEADER)
    .usage(USAGE)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(ERROR)
    .context(CONTEXT)
    .context_value(CONTEXT_VALUE)
    .valid(VALID)
    .invalid(INVALID);

/// Command line arguments
#[derive(Debug, Parser)]
#[command(version, about, author, styles = CLAP_STYLING)]
struct Args {
    /// Input shader file
    #[arg(value_hint = clap::ValueHint::FilePath)]
    input: PathBuf,

    /// Output shader directory. All compiled variants will be placed in this directory
    #[arg(value_hint = clap::ValueHint::DirPath)]
    output: PathBuf,

    /// Keywords to set. Can be used multiple times. Single values are set with `KEYWORD=value`, ranges
    /// are set with `KEYWORD=start..end`
    #[arg(short, long, value_parser = parse_key_val::<String, DefineValue>)]
    define: Vec<(String, DefineValue)>,

    /// Log level to run with
    #[arg(short, long, value_enum, default_value_t)]
    verbosity: LogLevel,

    /// What stage of compilation to process the shaders to
    #[arg(long, value_enum, default_value_t)]
    stage: CliCompileStage,
}

/// CLI-compatible log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
enum LogLevel {
    /// No logging at all
    Off,

    /// Errors only
    Error,

    /// Warnings or higher
    #[cfg_attr(not(debug_assertions), default)]
    Warn,

    /// Info or higher
    #[cfg_attr(debug_assertions, default)]
    Info,

    /// Debug or higher
    Debug,

    /// All logs
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// Compilation stages
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum CliCompileStage {
    /// Preprocess only
    Preprocess,

    /// Fully compile
    #[default]
    Full,
}

impl From<CliCompileStage> for CompileStage {
    fn from(value: CliCompileStage) -> Self {
        match value {
            CliCompileStage::Preprocess => CompileStage::Preprocess,
            CliCompileStage::Full => CompileStage::Full,
        }
    }
}

/// A value for a CLI keyword definition
#[derive(Debug, Clone, Copy)]
enum DefineValue {
    /// Set the keyword to a single value
    Single(i64),

    /// Set the keyword to a range of values
    Range(i64, i64),
}

impl FromStr for DefineValue {
    type Err = <i64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.split_once("..") {
            Some((left, right)) => DefineValue::Range(left.parse()?, right.parse()?),
            None => DefineValue::Single(s.parse()?),
        })
    }
}

impl From<DefineValue> for KeywordValue {
    fn from(value: DefineValue) -> Self {
        match value {
            DefineValue::Single(x) => KeywordValue::Single(x),
            DefineValue::Range(x, y) => KeywordValue::Range(x..=y),
        }
    }
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    const DELIM: char = '=';

    let pos = s
        .find(DELIM)
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;

    Ok((s[..pos].parse()?, s[pos + DELIM.len_utf8()..].parse()?))
}

/// An output file for the I/O thread
struct OutputFile {
    /// The file path
    path: PathBuf,

    /// The file content
    content: Vec<u8>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let level = args.verbosity;

    simplelog::TermLogger::init(
        level.into(),
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    let mut file = std::fs::File::open(&args.input).unwrap();
    std::fs::create_dir_all(&args.output).unwrap();

    let mut shader = String::new();
    file.read_to_string(&mut shader).unwrap();

    let keywords = HashMap::from_iter(
        args.define
            .iter()
            .map(|def| (def.0.as_str(), KeywordValue::from(def.1))),
    );

    log::debug!("Using keywords: {keywords:#?}");

    let stage = CompileStage::from(args.stage);

    let (send, recv) = channel::<OutputFile>();

    let io_thread = std::thread::spawn(|| io_thread(recv));

    compile_shader(&shader, &keywords, stage, |result| {
        shader_compiled_callback(&send, &args.output, result)
    });

    drop(send);

    log::info!("Waiting for I/O thread to finish");
    io_thread.join().unwrap();

    ExitCode::SUCCESS
}

/// Callback called for each compiled shader.
/// Formats the shader into a byte-buffer ready for writing directly as the content of a file,
/// and sends the formatted shader to the I/O thread
fn shader_compiled_callback(
    io_sender: &Sender<OutputFile>,
    root_output_dir: impl AsRef<Path>,
    result: Result<ShaderOutput<'_>, wutengine_shadercompiler::Error>,
) {
    let outdir = root_output_dir.as_ref();
    match result {
        Ok(shader) => {
            let outfile = match shader {
                ShaderOutput::Preprocessed {
                    source,
                    keyword_hash,
                    keywords: _,
                } => {
                    let outpath = outdir.join(format!("{keyword_hash:032x}.wgsl"));
                    OutputFile {
                        path: outpath,
                        content: source.into_bytes(),
                    }
                }
                ShaderOutput::Compiled {
                    source,
                    keyword_hash,
                    keywords: _,
                } => {
                    let outpath = outdir.join(format!("{keyword_hash:032x}.we_shader"));
                    OutputFile {
                        path: outpath,
                        content: postcard::to_stdvec(&source).unwrap(),
                    }
                }
            };

            io_sender.send(outfile).unwrap();
        }
        Err(e) => {
            log::error!("Failed to compile a shader: {e}");
            std::process::exit(1);
        }
    }
}

/// I/O thread function. Listens to the I/O receiver channel and writes all files to disk
fn io_thread(recv: std::sync::mpsc::Receiver<OutputFile>) {
    while let Ok(msg) = recv.recv() {
        std::fs::write(msg.path, msg.content).unwrap();
    }
}
