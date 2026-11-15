use core::error::Error;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use tiny_http::Header;
use tiny_http::Method;
use tiny_http::Request;
use tiny_http::Response;
use tiny_http::Server;

use crate::demo::workspace_root;
use crate::errs::ErrContext;
use crate::errs::err;

static WEB_INDEX_TEMPLATE: &str = include_str!("template.html");

/// The port the demo web server listens on. Not 8000, which `python3 -m http.server` uses by default
const WEB_PORT: u16 = 8080;

pub(super) fn build_and_serve(demo_name: &str, manifest: &Path) -> Result<(), Box<dyn Error>> {
    let web_dir = build(demo_name, manifest)?;
    serve(&web_dir)?;

    Ok(())
}

/// Builds the demo for the web and generates its web files. Returns the folder holding them
fn build(demo_name: &str, manifest: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let mut command = Command::new(env!("CARGO"));

    command
        .arg("build")
        .arg("-m")
        .arg(manifest)
        .arg("--locked")
        .arg("--target")
        .arg("wasm32-unknown-unknown");

    let mut spawned = command.spawn().context("Failed to spawn demo command")?;
    let exit_status = spawned.wait().context("Building demo failed")?;

    if !exit_status.success() {
        return match exit_status.code() {
            Some(code) => Err(err(format!("Demo web build failed with exit code: {code}"))),
            None => Err(err("Demo web build failed without exit code".to_string())),
        };
    }

    // Generate the JS glue with the `wasm-bindgen` CLI, installed in the version matching `Cargo.lock`
    let root = workspace_root();
    let target_dir =
        env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);

    let wasm_file = target_dir
        .join("wasm32-unknown-unknown")
        .join("debug")
        .join(format!("{demo_name}.wasm"));

    let version = wasm_bindgen_version(&root.join("Cargo.lock"))?;
    let cli = install_wasm_bindgen_cli(&target_dir, &version)?;

    let out_dir = target_dir.join("web").join(demo_name);

    let status = Command::new(&cli)
        .arg(&wasm_file)
        .arg("--out-dir")
        .arg(&out_dir)
        .args(["--target", "web", "--no-typescript", "--keep-debug"])
        .status()
        .context("Failed to run wasm-bindgen")?;

    if !status.success() {
        return Err(err(format!("wasm-bindgen failed with {status}")));
    }

    // The page that loads the generated glue, which runs the demo's `main`
    let index_html = WEB_INDEX_TEMPLATE.replace("{{DEMO_NAME}}", demo_name);

    fs::write(out_dir.join("index.html"), index_html).context("Failed to write index.html")?;

    log::info!("Generated web files in {}", out_dir.display());

    Ok(out_dir)
}

/// Finds the version of the `wasm-bindgen` crate in the given `Cargo.lock`.
/// The CLI must be exactly this version, or it refuses to process the wasm file
fn wasm_bindgen_version(lock_file: &Path) -> Result<String, Box<dyn Error>> {
    let lock = fs::read_to_string(lock_file).context("Failed to read Cargo.lock")?;
    let mut lines = lock.lines();

    // Each package in the lock file starts with its `name` line, directly followed by its `version` line
    while let Some(line) = lines.next() {
        if line != r#"name = "wasm-bindgen""# {
            continue;
        }

        let version = lines
            .next()
            .and_then(|line| line.strip_prefix(r#"version = ""#))
            .and_then(|version| version.strip_suffix('"'));

        if let Some(version) = version {
            return Ok(version.to_string());
        }
    }

    Err(err(
        "The wasm-bindgen crate is not in Cargo.lock. A web build needs it as a (transitive) dependency of the demo",
    ))
}

/// Installs the `wasm-bindgen` CLI in exactly `version`, unless that version is already installed.
/// Each version gets its own folder in the target directory, so it never clashes with a global install.
/// Returns the path to the executable
fn install_wasm_bindgen_cli(target_dir: &Path, version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let install_root = target_dir
        .join("xtask-tools")
        .join(format!("wasm-bindgen-{version}"));

    let mut executable = install_root.join("bin").join("wasm-bindgen");
    executable.set_extension(env::consts::EXE_EXTENSION);

    if executable.is_file() {
        return Ok(executable);
    }

    log::info!(
        "Installing wasm-bindgen-cli {version} into {}",
        install_root.display()
    );

    // A bare `MAJOR.MINOR.PATCH` version makes `cargo install` install exactly that version
    let status = Command::new(env!("CARGO"))
        .args([
            "install",
            "wasm-bindgen-cli",
            "--locked",
            "--version",
            version,
            "--root",
        ])
        .arg(&install_root)
        .status()
        .context("Failed to run cargo install")?;

    if !status.success() {
        return Err(err(format!(
            "Installing wasm-bindgen-cli {version} failed with {status}"
        )));
    }

    Ok(executable)
}

/// Serves the files in `dir` over HTTP on [`WEB_PORT`], until the process is stopped (Ctrl+C)
fn serve(dir: &Path) -> Result<(), Box<dyn Error>> {
    // Only reachable from this machine, not from the network
    let address = format!("127.0.0.1:{WEB_PORT}");

    let server = Server::http(&address)
        .map_err(|e| err(format!("Failed to start web server on {address}: {e}")))?;

    log::info!(
        "Serving {} at http://{address}/ (Ctrl+C to stop)",
        dir.display()
    );

    for request in server.incoming_requests() {
        handle_request(dir, request);
    }

    Ok(())
}

/// Answers one request with the file it asks for, or an error status
fn handle_request(dir: &Path, request: Request) {
    let response = if !matches!(request.method(), Method::Get | Method::Head) {
        Response::from_string("Method not allowed")
            .with_status_code(405)
            .boxed()
    } else if let Some(file_path) = resolve_path(dir, request.url())
        && let Ok(file) = File::open(&file_path)
    {
        Response::from_file(file)
            .with_header(header("Content-Type", content_type(&file_path)))
            // Always fetch again, so a rebuilt demo is picked up by a plain reload
            .with_header(header("Cache-Control", "no-store"))
            .boxed()
    } else {
        log::warn!("Not found: {}", request.url());
        Response::from_string("Not found")
            .with_status_code(404)
            .boxed()
    };

    log::debug!(
        "{} {} -> {}",
        request.method(),
        request.url(),
        response.status_code().0
    );

    if let Err(e) = request.respond(response) {
        log::warn!("Failed to send response: {e}");
    }
}

/// Maps a request URL to a file in `dir`. `/` means `index.html`. Returns [`None`] for URLs that would leave `dir`
fn resolve_path(dir: &Path, url: &str) -> Option<PathBuf> {
    // Ignore the query string, e.g. `/index.html?some=arg`
    let path = url.split('?').next().unwrap_or_default();

    let mut file_path = dir.to_path_buf();

    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        // Never step outside the served folder
        if segment == ".." || segment.contains(['\\', ':']) {
            return None;
        }

        file_path.push(segment);
    }

    if file_path == dir {
        file_path.push("index.html");
    }

    Some(file_path)
}

/// The `Content-Type` for a served file, by extension. `.wasm` must be `application/wasm`, or browsers can't
/// compile it while it downloads
fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(OsStr::to_str) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

/// A response header from a constant name and value
fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("header names and values are plain ASCII")
}
