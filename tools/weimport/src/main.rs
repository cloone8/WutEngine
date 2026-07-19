//! CLI asset importer for WutEngine

extern crate alloc;

use alloc::sync::Arc;
use core::error::Error;
use core::num::NonZero;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;

use clap::Args;
use clap::Parser;

use crate::input_iterator::InputIterator;
use crate::job_queue::JobQueue;
use crate::job_queue::JobToken;

mod input_iterator;
mod job_queue;

/// Default I/O queue size
const DEFAULT_QUEUE_SIZE: usize = 8;

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

    /// How many files we buffer before we throttle reading them from disk. If set to `0`, uses the default
    #[arg(long, default_value_t = 0)]
    io_queue_size: usize,

    /// The log level used
    #[arg(short, long, default_value_t = if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info })]
    verbosity: log::LevelFilter,
}

/// Input arguments
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct InputArg {
    /// Read input from stdin, and interpret it as the given file type
    #[arg(long)]
    stdin: Option<String>,

    /// Read input assets from the given paths. Directories are scanned recursively
    #[arg(short, long, value_hint = clap::ValueHint::AnyPath)]
    input: Vec<PathBuf>,
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

/// An asset could not be imported
#[derive(Debug, derive_more::Display, derive_more::Error)]
enum ImportAssetErr {
    /// There is no importer for the type
    #[display("No importer could be found for asset of type \"{}\"", _0)]
    NoImporter(#[error(not(source))] String),

    /// An asset importer returned an error
    #[display("Asset importer {} failed: {}", importer_name, err)]
    ImporterFailed {
        /// The name of the failed importer
        importer_name: &'static str,

        /// The returned error
        err: Box<dyn Error>,
    },

    /// An asset type returned by an importer failed
    UnknownAssetType(#[error(not(source))] uuid::NonNilUuid),

    /// Asset serialization failed
    #[display("Failed to serialize imported asset: {}", _0)]
    SerializationFailed(Box<dyn Error>),
}

/// Import an asset, and return an array of asset names and serialized contents
fn import_asset(
    file_path: Option<&Path>,
    file_type: String,
    bytes: Vec<u8>,
) -> Result<Vec<(String, Vec<u8>)>, ImportAssetErr> {
    // Just used to ensure a unique name for all unnamed assets
    static ASSET_IDX: AtomicUsize = AtomicUsize::new(0);

    log::info!(
        "Importing asset of type \"{file_type}\" from path {}",
        match file_path {
            Some(fp) => {
                format!("\"{}\"", fp.to_string_lossy())
            }
            None => "<STDIN>".to_string(),
        }
    );

    let Some(importer) = wutengine_asset_importers::default_importers()
        .get(file_type.as_str())
        .map(|importers| importers.first().expect("Empty importer array").clone())
    else {
        return Err(ImportAssetErr::NoImporter(file_type));
    };

    let imported_assets = match importer.import_from_bytes(&file_type, &bytes, file_path) {
        Ok(imported) => imported,
        Err(e) => {
            return Err(ImportAssetErr::ImporterFailed {
                importer_name: importer.name(),
                err: e,
            });
        }
    };

    let mut imported = Vec::with_capacity(imported_assets.len());

    for imported_asset in imported_assets {
        let asset_unique_idx = ASSET_IDX.fetch_add(1, Ordering::AcqRel);

        let target_type = wutengine_asset_importers::default_asset_types()
            .get(&imported_asset.asset_type_id)
            .ok_or_else(|| ImportAssetErr::UnknownAssetType(imported_asset.asset_type_id))?;

        let serialized = if target_type.prefers_binary() {
            log::debug!("Serializing as binary");
            target_type.serialize_binary(imported_asset.asset.as_ref())
        } else {
            log::debug!("Serializing as text");
            target_type.serialize_text(imported_asset.asset.as_ref())
        };

        match serialized {
            Ok(ser) => {
                let name = imported_asset.name.unwrap_or_else(|| {
                    format!("{}_{}", target_type.asset_type_name(), asset_unique_idx)
                });

                let extension = if target_type.prefers_binary() {
                    ".we-binasset"
                } else {
                    ".we-txtasset"
                };

                let asset_file_name = format!("{name}{extension}");

                imported.push((asset_file_name, ser));
            }
            Err(e) => {
                return Err(ImportAssetErr::SerializationFailed(e));
            }
        }
    }

    Ok(imported)
}

/// Thread main function for the output I/O thread
fn write_asset_to_disk_thread(
    output_root: Option<PathBuf>,
    output_recv: Receiver<(Arc<JobToken>, String, Vec<u8>)>,
) {
    let mut num_imported = 0;

    for (job_token, asset_file_name, content) in output_recv.iter() {
        match &output_root {
            Some(output_root) => {
                let complete_path = output_root.join(asset_file_name);

                log::debug!(
                    "Writing asset of {} bytes to \"{}\"",
                    content.len(),
                    complete_path.to_string_lossy()
                );

                if let Err(e) = std::fs::write(&complete_path, content) {
                    log::error!(
                        "Failed to write asset to path \"{}\": {e}",
                        complete_path.to_string_lossy()
                    );
                } else {
                    num_imported += 1;
                }
            }
            None => {
                if let Err(e) = std::io::stdout().write_all(&content) {
                    log::error!("Failed to write an asset to stdout: {e}");
                } else {
                    num_imported += 1;
                }
            }
        }

        // This frees up a slot in the job queue
        drop(job_token);
    }

    log::info!("Imported {num_imported} assets");
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

    if let Some(output_dir) = &args.output.output
        && let Err(e) = std::fs::create_dir_all(output_dir)
    {
        log::error!("Failed to create output directory due to I/O error: {e}");
        return ExitCode::FAILURE;
    }

    let input_iter = if let Some(stdin_file_type) = args.input.stdin {
        InputIterator::from_stdin(stdin_file_type)
    } else {
        InputIterator::from_input_paths(&args.input.input)
    };

    let queue_size = if args.io_queue_size == 0 {
        DEFAULT_QUEUE_SIZE
    } else {
        args.io_queue_size
    };

    let job_queue = JobQueue::new(NonZero::new(queue_size).unwrap());

    let (send, recv) = channel::<(Arc<JobToken>, String, Vec<u8>)>();

    let recv_thread_handle = {
        let output_root = if let Some(output_dir) = args.output.output {
            match output_dir.canonicalize() {
                Ok(od) => Some(od),
                Err(e) => {
                    log::error!("Failed to canonicalize output directory: {e}");
                    return ExitCode::FAILURE;
                }
            }
        } else {
            None
        };

        std::thread::spawn(move || write_asset_to_disk_thread(output_root, recv))
    };

    for input in input_iter {
        if let Err(e) = input {
            log::error!("{e}");
            continue;
        }

        let job_token = job_queue.issue_job();

        let (file_path, file_type, bytes) = input.unwrap();

        let send = send.clone();

        rayon::spawn(
            move || match import_asset(file_path.as_deref(), file_type, bytes) {
                Ok(imported_assets) => {
                    let job_token_arc = Arc::new(job_token);

                    for (output_path, content) in imported_assets {
                        send.send((job_token_arc.clone(), output_path, content))
                            .expect("Failed to send import result");
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to import asset at path \"{}\": {e}",
                        file_path
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "<STDIN>".to_string())
                    );
                    drop(job_token);
                }
            },
        );
    }

    // Drop our sender, so the only senders are in the individual import functions
    drop(send);
    recv_thread_handle.join().unwrap();

    ExitCode::SUCCESS
}
