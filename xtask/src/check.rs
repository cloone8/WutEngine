//! Formatting and lint checks for the whole workspace

use std::process::Command;

use crate::demo::workspace_root;

/// The target for the web checks
const WASM_TARGET: &str = "wasm32-unknown-unknown";

/// Runs every check, all of them even when an earlier one fails, so one run shows every problem.
/// Returns whether all checks passed
pub(crate) fn run_checks() -> bool {
    let checks = [
        ("fmt", fmt_command()),
        ("clippy (native)", clippy_command(None)),
        ("clippy (wasm)", clippy_command(Some(WASM_TARGET))),
        ("clippy (headless)", clippy_headless_command()),
    ];

    let root = workspace_root();
    let mut failed = Vec::new();

    for (name, mut command) in checks {
        log::info!("Running {name}");

        match command.current_dir(&root).status() {
            Ok(status) if status.success() => log::info!("{name}: passed"),
            Ok(status) => {
                log::error!("{name}: failed ({status})");
                failed.push(name);
            }
            Err(e) => {
                log::error!("{name}: could not be run: {e}");
                failed.push(name);
            }
        }
    }

    if failed.is_empty() {
        log::info!("All checks passed");
        true
    } else {
        log::error!("Failed checks: {}", failed.join(", "));
        false
    }
}

/// `cargo fmt --check` on the nightly toolchain, because `rustfmt.toml` uses unstable options.
/// Goes through `rustup run`, since the toolchain's own `cargo` (as in `CARGO`) doesn't accept `+nightly`
fn fmt_command() -> Command {
    let mut command = Command::new("rustup");
    command.args(["run", "nightly", "cargo", "fmt", "--all", "--check"]);
    command
}

/// `cargo clippy` over the whole workspace with all features, failing on any warning.
/// With a `target`, clippy checks for that target instead of this machine, without `xtask`, which only runs here
fn clippy_command(target: Option<&str>) -> Command {
    let mut command = Command::new(env!("CARGO"));
    command.args([
        "clippy",
        "--workspace",
        "--all-features",
        "--all-targets",
        "--locked",
    ]);

    if let Some(target) = target {
        command.args(["--target", target, "--exclude", "xtask"]);
    }

    command.args(["--", "-D", "warnings"]);
    command
}

/// `cargo clippy` over the headless build: the engine without default features, and the headless demo, failing on
/// any warning. Native only, since there is no headless build on the web.
///
/// Selects the packages instead of the whole workspace, because Cargo unifies features over every selected package:
/// the other demos use the engine's default features and would turn the interactive build back on
fn clippy_headless_command() -> Command {
    let mut command = Command::new(env!("CARGO"));
    command.args([
        "clippy",
        "--package",
        "wutengine",
        "--package",
        "headless",
        "--no-default-features",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ]);
    command
}
