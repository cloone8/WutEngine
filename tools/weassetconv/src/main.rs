//! Asset format conversion utility

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use clap::Parser;

/// Command line arguments
#[derive(Debug, Parser)]
#[command(version, about, author)]
struct CliArgs {
    /// Input sources
    #[command(flatten)]
    input: InputArg,

    /// Output targets
    #[command(flatten)]
    output: OutputArg,

    /// Output format
    #[command(flatten)]
    format: OutputFormat,

    /// The log level used
    #[arg(short, long, default_value_t = if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info })]
    verbosity: log::LevelFilter,
}

/// Input arguments
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct InputArg {
    /// Read input from stdin
    #[arg(long)]
    stdin: bool,

    /// Read input assets from the given paths. Directories are scanned recursively
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    input: Option<PathBuf>,
}

/// Output targets
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct OutputArg {
    /// Write the output to stdout
    #[arg(long)]
    stdout: bool,

    /// Write the output to this directory. Will be created if it does not yet exist
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    output: Option<PathBuf>,
}

/// Output formats
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct OutputFormat {
    /// Write the output as text
    #[arg(long)]
    text: bool,

    /// Write the output as binary
    #[arg(long)]
    binary: bool,
}

fn get_input(input: &InputArg) -> Result<Vec<u8>, Box<dyn core::error::Error>> {
    if input.stdin {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).map_err(Box::new)?;

        return Ok(buf);
    }

    if let Some(path) = &input.input {
        return std::fs::read(path).map_err(|e| Box::new(e) as Box<dyn core::error::Error>);
    }

    unreachable!("One input method must be selected");
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

    log::info!("Hello, world!");

    let input = match get_input(&args.input) {
        Ok(i) => i,
        Err(e) => {
            log::error!("Failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    };

    let asset_types = wutengine_asset_importers::default_asset_types();

    let mut deserialized = None;
    for asset_type in asset_types.values() {
        if let Ok(asset) = asset_type.deserialize_text(&input) {
            log::info!(
                "Deserialized asset as text asset of type {}",
                asset_type.asset_type_name()
            );
            deserialized = Some(asset);
            break;
        }
        if let Ok(asset) = asset_type.deserialize_binary(&input) {
            log::info!(
                "Deserialized asset as binary asset of type {}",
                asset_type.asset_type_name()
            );
            deserialized = Some(asset);
            break;
        }
    }

    if deserialized.is_none() {
        log::error!("Failed to deserialize input as any asset type");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
