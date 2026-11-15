use core::error::Error;
use std::path::Path;
use std::process::Command;

use crate::errs::ErrContext;
use crate::errs::err;

pub(super) fn run(manifest: &Path, args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new(env!("CARGO"));

    command
        .arg("run")
        .arg("-m")
        .arg(manifest)
        .arg("--locked")
        .arg("--")
        .args(args);

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
