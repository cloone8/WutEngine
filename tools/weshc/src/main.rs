//! Freestanding shader compiler for WutEngine

use core::error::Error;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::io::stdout;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use clap::Parser;
use wutengine_cli_tools::clap::OutputFormat;
use wutengine_cli_tools::clap::OutputFormatArg;

/// Freestanding WutEngine shader compiler for compiling raw WutEngine WGSL shaders into pre-compiled shader assets
#[derive(Debug, Parser)]
#[command(version, about, author, styles = wutengine_cli_tools::clap::STYLING)]
struct CliArgs {
    /// The input source.
    #[command(flatten)]
    input: InputArg,

    /// If `true`, the shader is formatted as text instead of binary
    #[command(flatten)]
    format: OutputFormatArg,

    /// Keywords. If an explicit value is not given, `1` is used
    #[arg(short, long, value_name = "KEY{=VALUE}", value_parser = parse_keyword)]
    keyword: Vec<(String, u64)>,

    /// The log level used
    #[arg(short, long, default_value_t = if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info })]
    verbosity: log::LevelFilter,
}

/// Input arguments
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct InputArg {
    /// Read the input shader from stdin
    #[arg(long)]
    stdin: bool,

    /// Read the input shader from the given file
    #[arg(value_hint = clap::ValueHint::FilePath)]
    file: Option<PathBuf>,
}

/// Parse a single key-value pair.
fn parse_keyword(s: &str) -> Result<(String, u64), Box<dyn Error + Send + Sync + 'static>> {
    let Some(pos) = s.find('=') else {
        return Ok((s.to_string(), 1));
    };

    Ok((s[..pos].to_string(), s[pos + 1..].parse()?))
}

/// An error while reading shader input
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
enum ReadInputErr {
    /// I/O error
    #[display("I/O Error: {_0}")]
    IO(std::io::Error),
}

/// Returns the program shader input according to the given source in `source`
fn read_input(source: InputArg) -> Result<String, ReadInputErr> {
    let mut input_reader: Box<dyn std::io::BufRead> = if source.stdin {
        log::info!("Reading shader from stdin");

        debug_assert!(source.file.is_none(), "Input file should be none");

        Box::new(BufReader::new(std::io::stdin()))
    } else {
        let file_path = source.file.expect("Input file should have been given");

        log::info!("Reading shader from file: {}", file_path.to_string_lossy());

        Box::new(BufReader::new(std::fs::File::open(file_path)?))
    };

    let mut input_buf = String::new();

    input_reader.read_to_string(&mut input_buf)?;

    Ok(input_buf)
}

fn main() -> ExitCode {
    let args = CliArgs::parse();

    simplelog::TermLogger::init(
        args.verbosity,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    let input = match read_input(args.input) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    };

    log::debug!("Input shader:\n{input}");

    let output = match wutengine_shadercompiler2::compile(
        &input,
        &wutengine_shadercompiler2::Config {
            keywords: args.keyword.into_iter().collect(),
            shader_resolver: None,
        },
    ) {
        Ok(o) => o,
        Err(e) => {
            log::error!("Failed to compile shader: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut out_stream = stdout();

    let format = args
        .format
        .determine_format(Some(&out_stream))
        .unwrap_or(OutputFormat::Binary);

    let serialize_result = match format {
        OutputFormat::Binary => {
            postcard::to_allocvec(&output).map_err(|e| Box::new(e) as Box<dyn Error>)
        }
        OutputFormat::Text => serde_json::to_string_pretty(&output)
            .map(String::into_bytes)
            .map_err(|e| Box::new(e) as Box<dyn Error>),
    };

    let serialized = match serialize_result {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Failed to serialize compiled shader: {e}");
            return ExitCode::FAILURE;
        }
    };

    out_stream
        .write_all(&serialized)
        .expect("Failed to write to output stream");

    ExitCode::SUCCESS
}
