//! Command line arguments

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// The input file or files. Must be at least one
    #[arg(short, long, required = true)]
    pub input: Vec<PathBuf>,

    /// The output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}
