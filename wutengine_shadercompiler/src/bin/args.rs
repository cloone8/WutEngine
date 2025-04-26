//! Command line arguments

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// The input folder containing the individual stages.
    #[arg(short, long, required = true)]
    pub input: PathBuf,

    /// The output folder
    #[arg(short, long)]
    pub output: PathBuf,
}
