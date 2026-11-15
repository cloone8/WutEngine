//! Running a demo from `/demos`

use core::error::Error;
use std::path::Path;
use std::process::Command;

use clap::Parser;

use crate::errs::ErrContext;
use crate::errs::err;

/// Args for running a demo
#[derive(Debug, Parser)]
pub(crate) struct DemoCommand {
    /// The name of the demo folder in the
    name: String,

    /// Arguments to pass to the demo
    #[arg(last = true)]
    args: Vec<String>,
}

/// Runs the demo given by `cmd`. Returns an error on execution failure or a non-success exit code
pub(crate) fn run_demo(cmd: &DemoCommand) -> Result<(), Box<dyn Error>> {
    let arg_string = cmd.args.join(" ");

    log::info!("Running demo: {} {}", cmd.name, arg_string);

    // Cargo sets `CARGO_MANIFEST_DIR` to this crate's folder at compile time, so this works from any working directory
    let all_demos_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is in a subfolder of the workspace root")
        .join("demos");

    let demo_manifest = all_demos_dir.join(&cmd.name).join("Cargo.toml");

    let mut command = Command::new(env!("CARGO"));

    command
        .arg("run")
        .arg("-m")
        .arg(demo_manifest)
        .arg("--locked")
        .arg("--")
        .args(&cmd.args);

    let mut spawned = command.spawn().context("Failed to spawn demo command")?;

    let exit_status = spawned.wait().context("Executing demo failed")?;

    if !exit_status.success() {
        return match exit_status.code() {
            Some(code) => Err(err(format!("Demo command failed with exit code: {code}"))),
            None => Err(err("Demo command failed without exit code".to_string())),
        };
    }

    Ok(())
}
