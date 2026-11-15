//! Running a demo from `/demos`

use core::error::Error;
use std::env;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

mod native;
mod web;

/// Args for running a demo
#[derive(Debug, Parser)]
pub(crate) struct DemoCommand {
    /// The name of the demo folder in the
    name: String,

    /// Serve the demo for the web
    #[arg(long)]
    web: bool,

    /// Arguments to pass to the demo
    #[arg(last = true)]
    args: Vec<String>,
}

/// Runs the demo given by `cmd`. Returns an error on execution failure or a non-success exit code
pub(crate) fn run_demo(cmd: &DemoCommand) -> Result<(), Box<dyn Error>> {
    let arg_string = cmd.args.join(" ");

    log::info!("Running demo: {} {}", cmd.name, arg_string);

    let all_demos_dir = workspace_root().join("demos");

    let demo_manifest = all_demos_dir.join(&cmd.name).join("Cargo.toml");

    if cmd.web {
        web::build_and_serve(&cmd.name, &demo_manifest)
    } else {
        native::run(&demo_manifest, &cmd.args)
    }
}

/// The workspace root folder: the parent of this crate's folder
pub(crate) fn workspace_root() -> PathBuf {
    // Cargo sets `CARGO_MANIFEST_DIR` to this crate's folder at compile time, so this works from any working directory
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is in a subfolder of the workspace root")
        .to_path_buf()
}
