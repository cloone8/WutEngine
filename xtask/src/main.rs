#![doc = include_str!("../README.md")]

use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use crate::demo::DemoCommand;
use crate::logger::XTaskLogger;

mod demo;
mod errs;
mod logger;
mod styling;

/// Command-line arguments
#[derive(Debug, Parser)]
#[command(
    version,
    long_version = styling::LONG_VERSION,
    about,
    styles = styling::STYLING
)]
struct Args {
    /// The subcommand to run
    #[command(subcommand)]
    command: Commands,
}

/// The subcommands supported by the `xtask` crate
#[derive(Debug, Subcommand)]
enum Commands {
    /// Run a demo
    Demo(DemoCommand),
}

fn main() -> ExitCode {
    let args = Args::parse();

    XTaskLogger::init();

    match args.command {
        Commands::Demo(cmd) => {
            if let Err(e) = demo::run_demo(&cmd) {
                log::error!("Failed to run demo: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
