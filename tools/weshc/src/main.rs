//! Freestanding shader compiler for WutEngine

use core::error::Error;
use core::num::NonZero;
use core::range::Range;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use clap::Parser;
use wutengine_assets::assets::shader::PrecompiledShader;
use wutengine_cli_tools::clap::OutputFormat;
use wutengine_cli_tools::clap::OutputFormatArg;

/// Freestanding WutEngine shader compiler for compiling raw WutEngine WGSL shaders into pre-compiled shader assets
#[derive(Debug, Parser)]
#[command(version, about, author, styles = wutengine_cli_tools::clap::STYLING)]
struct CliArgs {
    /// The input source.
    #[command(flatten)]
    input: InputArg,

    /// The output source
    #[command(flatten)]
    output: OutputArg,

    /// If `true`, the shader is formatted as text instead of binary
    #[command(flatten)]
    format: OutputFormatArg,

    /// How many shaders will be compiled at once until the compiler throttles itself
    #[arg(long, default_value_t = NonZero::new(32usize).unwrap())]
    buffer_size: NonZero<usize>,

    /// Keywords. If an explicit value is not given, `1` is used
    #[arg(short, long, value_name = "KEY{=VALUE} or KEY{=START..END}", value_parser = parse_keyword)]
    keyword: Vec<(String, Range<u64>)>,

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

/// Output arguments
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct OutputArg {
    /// Write the compiled shaders to stdout.
    /// Each compiled shader is seperated by an empty line.
    /// If stdout is selected, the output format is forced to text.
    #[arg(long)]
    stdout: bool,

    /// Write the compiled shaders to a directory as assets
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    output: Option<PathBuf>,
}

/// Parse a single key-value pair.
fn parse_keyword(s: &str) -> Result<(String, Range<u64>), Box<dyn Error + Send + Sync + 'static>> {
    let Some(pos) = s.find('=') else {
        return Ok((s.to_string(), Range { start: 1, end: 2 }));
    };

    let name = s[..pos].to_string();

    let value = &s[pos + 1..];

    if let Some((start, end)) = value.split_once("..") {
        let start: u64 = start.parse()?;
        let end: u64 = end.parse()?;

        Ok((name, Range { start, end }))
    } else {
        let single_value: u64 = value.parse()?;

        Ok((
            name,
            Range {
                start: single_value,
                end: single_value + 1,
            },
        ))
    }
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

    let output_formatter = if args.output.stdout {
        OutputFormatter::Stdout
    } else {
        let Some(out_dir) = &args.output.output else {
            unreachable!()
        };

        if let Err(e) = std::fs::create_dir_all(out_dir) {
            log::error!("Failed to create output directory: {e}");
            return ExitCode::FAILURE;
        }

        OutputFormatter::Dir(
            args.format
                .determine_format()
                .unwrap_or(OutputFormat::Binary),
            out_dir.clone(),
        )
    };

    let output_channel = wutengine_shadercompiler2::compile_multiple(
        &input,
        wutengine_shadercompiler2::MultiConfig {
            buf_size: args.buffer_size,
            keywords: args.keyword.into_iter().collect(),
            shader_resolver: None,
        },
    );

    for result in output_channel {
        match result {
            Ok(compiled) => {
                if let Err(e) = output_formatter.write(&compiled) {
                    log::error!("Failed to write shader to output: {e}");
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                log::error!("Error while compiling one shader: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

/// Simple abstraction for formatting a shader into the correct format, and writing it to the requested sink
enum OutputFormatter {
    /// Write as text to stdout
    Stdout,

    /// Write according to the given format and into the given directory
    Dir(OutputFormat, PathBuf),
}

impl OutputFormatter {
    /// Write a shader to this formatter
    fn write(&self, shader: &PrecompiledShader) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Stdout => {
                let serialized = serde_json::to_string_pretty(&shader).map_err(Box::new)?;

                println!("{serialized}\n");

                Ok(())
            }
            Self::Dir(output_format, output_dir) => {
                let serialized = match output_format {
                    OutputFormat::Binary => postcard::to_allocvec(&shader).map_err(Box::new)?,
                    OutputFormat::Text => serde_json::to_string_pretty(&shader)
                        .map_err(Box::new)?
                        .into_bytes(),
                };

                let name = shader.hash;
                let extension = match output_format {
                    OutputFormat::Binary => ".we-binasset",
                    OutputFormat::Text => ".we-txtasset",
                };

                let file_name = format!("{name}{extension}");

                let path = output_dir.join(file_name);

                std::fs::write(path, serialized).map_err(Box::new)?;

                Ok(())
            }
        }
    }
}
