//! Build tool for WutEngine

use std::path::PathBuf;
use std::process::ExitCode;

use cargo_metadata::Metadata;
use cargo_metadata::TargetKind;

/// Default output directory for build artifacts
const OUTPUT_DIR: &str = "dist";

fn main() -> ExitCode {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    log::info!("Starting WutEngine build tool");

    let metadata = match cargo_metadata::MetadataCommand::new().exec() {
        Ok(meta) => meta,
        Err(e) => {
            log::error!("Failed to get workspace metadata: {e}");
            return ExitCode::FAILURE;
        }
    };

    if !build_package(&metadata, "weassetconv") {
        return ExitCode::FAILURE;
    }

    if !build_package(&metadata, "weimport") {
        return ExitCode::FAILURE;
    }

    if !build_package(&metadata, "weshc") {
        return ExitCode::FAILURE;
    }

    if !build_package(&metadata, "wutengine_editor") {
        return ExitCode::FAILURE;
    }

    log::info!("All packages built");

    ExitCode::SUCCESS
}

/// Builds a single package
fn build_package(metadata: &Metadata, package_name: &str) -> bool {
    log::info!("Building package `{package_name}`");

    let packages = &metadata.packages;

    let Some(package_info) = packages
        .iter()
        .find(|package_info| package_info.name == package_name)
    else {
        log::error!("Failed to find package info for package: `{package_name}`");
        return false;
    };

    let mut artifacts = Vec::new();

    for target in &package_info.targets {
        let target_name = target.name.clone();

        for target_kind in &target.kind {
            match target_kind {
                TargetKind::Bin => {
                    if cfg!(windows) {
                        artifacts.push(format!("{target_name}.exe"));
                    } else {
                        artifacts.push(target_name.clone());
                    }
                }
                TargetKind::CDyLib | TargetKind::DyLib => {
                    if cfg!(windows) {
                        artifacts.push(format!("{target_name}.dll"));
                    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
                        artifacts.push(format!("{target_name}.dylib"));
                    } else {
                        artifacts.push(format!("{target_name}.so"));
                    }
                }
                _ => {}
            }
        }
    }

    let dist_dir = PathBuf::from(&metadata.workspace_root).join(OUTPUT_DIR);
    let target_dir = PathBuf::from(&metadata.target_directory).join("release");

    if let Err(e) = std::fs::create_dir_all(&dist_dir) {
        log::error!("Failed to create dist directory for build artifacts: {e}");
        return false;
    }

    let cargo = std::env::var("CARGO").unwrap_or("cargo".to_string());

    let mut spawned = match std::process::Command::new(cargo)
        .arg("build")
        .arg("--release")
        .arg("--locked")
        .arg("--package")
        .arg(package_name)
        .spawn()
    {
        Ok(spawned) => spawned,
        Err(e) => {
            log::error!("Failed to spawn build process for package `{package_name}`: {e}");
            return false;
        }
    };

    let build_result = match spawned.wait() {
        Ok(res) => res,
        Err(e) => {
            log::error!("Failed while waiting for build process for `{package_name}`: {e}");
            return false;
        }
    };

    if !build_result.success() {
        if let Some(code) = build_result.code() {
            log::error!("Build for package `{package_name}` failed with exit code {code}");
        } else {
            log::error!("Build for package `{package_name}` failed with no exit code");
        }

        return false;
    }

    let artifacts_str = artifacts
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");

    log::info!("Finished building package `{package_name}`. Copying artifacts: {artifacts_str}");

    for artifact in artifacts {
        let src = target_dir.join(&artifact);
        let dst = dist_dir.join(artifact);

        if let Err(e) = std::fs::copy(&src, &dst) {
            log::error!(
                "Failed to copy {} to {}: {e}",
                src.to_string_lossy(),
                dst.to_string_lossy()
            );
            return false;
        }
    }

    true
}
