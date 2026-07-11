//! CLI asset importer for WutEngine

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::any::Any;
use core::error::Error;
use core::num::NonZero;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Condvar;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;

use clap::Args;
use clap::Parser;
use wutengine_asset_importers::AssetImporter;
use wutengine_asset_importers::ImportedAsset;
use wutengine_assets::SerializedAsset;
use wutengine_assets::assets::texture::SerializedTexture;

static IMPORTERS: LazyLock<HashMap<&'static str, Vec<Arc<Importer>>>> =
    LazyLock::new(get_importers);

static ASSET_TYPES: LazyLock<HashMap<uuid::NonNilUuid, SerializedAssetType>> =
    LazyLock::new(get_asset_types);

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

type ImportFn = Box<
    dyn Fn(&str, &[u8], Option<&Path>) -> Result<Vec<ImportedAsset>, Box<dyn Error>> + Send + Sync,
>;

struct Importer {
    name: &'static str,
    file_types: Vec<&'static str>,
    import_from_bytes: ImportFn,
}

impl Importer {
    fn from_asset_importer<T: AssetImporter>() -> Self {
        Self {
            name: core::any::type_name::<T>(),
            file_types: T::supported_file_types(),
            import_from_bytes: Box::new(|ftype, bytes, orig_path| {
                T::from_bytes(bytes, ftype, orig_path)
            }),
        }
    }
}

fn get_importers() -> HashMap<&'static str, Vec<Arc<Importer>>> {
    let known_importers = [Importer::from_asset_importer::<
        wutengine_asset_importers::ImageAssetImporter,
    >()];

    let mut importer_map: HashMap<&str, Vec<Arc<Importer>>> = HashMap::new();

    for importer in known_importers {
        let importer = Arc::new(importer);

        for &supported_file_type in &importer.file_types {
            importer_map
                .entry(supported_file_type)
                .or_default()
                .push(importer.clone());
        }
    }

    importer_map
}

struct SerializedAssetType {
    id: uuid::NonNilUuid,
    name: String,
    prefer_binary: bool,
    serialize_binary: Box<
        dyn Fn(Box<dyn Any + Send + Sync + 'static>) -> Result<Vec<u8>, Box<dyn Error>>
            + Send
            + Sync,
    >,
    serialize_text: Box<
        dyn Fn(Box<dyn Any + Send + Sync + 'static>) -> Result<Vec<u8>, Box<dyn Error>>
            + Send
            + Sync,
    >,
}

impl SerializedAssetType {
    fn new_from_asset<T: SerializedAsset>() -> Self {
        Self {
            id: T::ID,
            name: core::any::type_name::<T>()
                .split("::")
                .last()
                .unwrap()
                .to_lowercase()
                .to_string(),
            prefer_binary: T::PREFER_BINARY_SERIALIZATION,
            serialize_binary: Box::new(|asset| {
                let as_typed: Box<T> = asset.downcast::<T>().expect("Invalid downcast");

                Ok(postcard::to_allocvec(as_typed.as_ref()).map_err(Box::new)?)
            }),
            serialize_text: Box::new(|asset| {
                let as_typed: Box<T> = asset.downcast::<T>().expect("Invalid downcast");

                Ok(serde_json::to_vec_pretty(as_typed.as_ref()).map_err(Box::new)?)
            }),
        }
    }
}

fn get_asset_types() -> HashMap<uuid::NonNilUuid, SerializedAssetType> {
    let known_asset_types = [SerializedAssetType::new_from_asset::<SerializedTexture>()];

    let mut asset_type_map = HashMap::new();

    for known_asset_type in known_asset_types {
        asset_type_map.insert(known_asset_type.id, known_asset_type);
    }

    asset_type_map
}

#[derive(Debug)]
enum InputIterator {
    Empty,
    Stdin(String),
    Paths(VecDeque<PathBuf>),
}

impl InputIterator {
    fn from_stdin(file_type: String) -> Self {
        Self::Stdin(file_type)
    }

    fn from_input_paths(input_paths: &[impl AsRef<Path>]) -> Self {
        let mut paths = VecDeque::new();

        for input_path in input_paths {
            Self::add_path(input_path.as_ref(), &mut paths);
        }

        Self::Paths(paths)
    }

    fn add_dir_recursive(dir: &Path, out: &mut VecDeque<PathBuf>) {
        let dir_iter = match std::fs::read_dir(dir) {
            Ok(di) => di,
            Err(e) => {
                log::error!(
                    "Failed to list contents of directory {}, skipping: {e}",
                    dir.to_string_lossy()
                );
                return;
            }
        };

        for dir_entry in dir_iter {
            let dir_entry = match dir_entry {
                Ok(de) => de,
                Err(e) => {
                    log::error!(
                        "Failed to read an entry in directory {}, skipping entry: {e}",
                        dir.to_string_lossy()
                    );
                    continue;
                }
            };

            Self::add_path(&dir_entry.path(), out);
        }
    }

    fn add_path(path: &Path, out: &mut VecDeque<PathBuf>) {
        let meta = match std::fs::metadata(path) {
            Ok(meta) => meta,
            Err(e) => {
                log::error!(
                    "Failed to read metadata for path \"{}\", skipping: {e}",
                    path.to_string_lossy()
                );
                return;
            }
        };

        if meta.is_file() {
            if get_file_type_from_path(path).is_some() {
                out.push_back(path.to_path_buf());
            } else {
                log::error!(
                    "Skipping file \"{}\" because the file has no extension and thus no conclusive file type",
                    path.to_string_lossy()
                );
            }
        } else if meta.is_dir() {
            Self::add_dir_recursive(path, out);
        } else if meta.is_symlink() {
            log::warn!("Symlinks are not yet supported");
        } else {
            log::error!(
                "Unknown file type for path \"{}\", not a file, directory, or symlink according to OS",
                path.to_string_lossy()
            );
        }
    }
}

fn get_file_type_from_path(path: &Path) -> Option<&str> {
    let ext = path.extension()?;

    ext.to_str()
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
enum InputIteratorError {
    #[display("Failed to read from stdin due to error: {}", _0)]
    Stdin(std::io::Error),

    #[display("Failed to read file at path {} due to error: {}", path.to_string_lossy(), err)]
    File { path: PathBuf, err: std::io::Error },
}

impl Iterator for InputIterator {
    type Item = Result<(Option<PathBuf>, String, Vec<u8>), InputIteratorError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Empty => None,
            Self::Stdin(file_type) => {
                let file_type = core::mem::take(file_type);
                *self = Self::Empty;

                let mut bytes = Vec::new();

                if let Err(e) = std::io::stdin().read_to_end(&mut bytes) {
                    return Some(Err(InputIteratorError::Stdin(e)));
                }

                Some(Ok((None, file_type, bytes)))
            }
            Self::Paths(path_bufs) => {
                let next = path_bufs.pop_front()?;

                let file_type = get_file_type_from_path(&next)
                    .expect("Invalid paths should have been filtered")
                    .to_string();

                let bytes = match std::fs::read(&next) {
                    Ok(b) => b,
                    Err(e) => {
                        return Some(Err(InputIteratorError::File { path: next, err: e }));
                    }
                };

                Some(Ok((Some(next), file_type, bytes)))
            }
        }
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
enum ImportAssetErr {
    #[display("No importer could be found for asset of type \"{}\"", _0)]
    NoImporter(#[error(not(source))] String),

    #[display("Asset importer {} failed: {}", importer_name, err)]
    ImporterFailed {
        importer_name: &'static str,
        err: Box<dyn Error>,
    },

    UnknownAssetType(#[error(not(source))] uuid::NonNilUuid),

    #[display("Failed to serialize imported asset: {}", _0)]
    SerializationFailed(Box<dyn Error>),
}

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

    let Some(importer) = IMPORTERS
        .get(file_type.as_str())
        .map(|importers| importers.first().expect("Empty importer array").clone())
    else {
        return Err(ImportAssetErr::NoImporter(file_type));
    };

    let imported_assets = match (importer.import_from_bytes)(&file_type, &bytes, file_path) {
        Ok(imported) => imported,
        Err(e) => {
            return Err(ImportAssetErr::ImporterFailed {
                importer_name: importer.name,
                err: e,
            });
        }
    };

    let mut imported = Vec::with_capacity(imported_assets.len());

    for imported_asset in imported_assets {
        let asset_unique_idx = ASSET_IDX.fetch_add(1, Ordering::AcqRel);

        let target_type = ASSET_TYPES
            .get(&imported_asset.id)
            .ok_or_else(|| ImportAssetErr::UnknownAssetType(imported_asset.id))?;

        let serialized = if target_type.prefer_binary {
            log::debug!("Serializing as binary");
            (target_type.serialize_binary)(imported_asset.asset)
        } else {
            log::debug!("Serializing as text");
            (target_type.serialize_text)(imported_asset.asset)
        };

        match serialized {
            Ok(ser) => {
                let name = imported_asset
                    .name
                    .unwrap_or_else(|| format!("{}_{}", target_type.name, asset_unique_idx));
                let extension = if target_type.prefer_binary {
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

fn write_asset_to_disk_thread(
    output_root: Option<PathBuf>,
    output_recv: Receiver<(Arc<JobToken>, String, Vec<u8>)>,
) {
    let mut num_imported = 0;

    for (job_token, asset_file_name, content) in output_recv.iter() {
        //TODO: Write to disk

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

        drop(job_token);
    }

    log::info!("Imported {num_imported} assets");
}

#[derive(Debug, Clone)]
#[repr(transparent)]
struct JobQueue {
    job_budget: Arc<(Mutex<usize>, Condvar)>,
}

impl JobQueue {
    fn new(budget: NonZero<usize>) -> Self {
        let job_budget_mtx = Mutex::new(budget.get());
        let job_budget_condvar = Condvar::new();

        let job_budget = Arc::new((job_budget_mtx, job_budget_condvar));

        Self { job_budget }
    }

    fn issue_job(&self) -> JobToken {
        // Wait for a slot to open up

        let job_budget_lock = self.job_budget.0.lock().unwrap();
        let mut job_budget_lock = self
            .job_budget
            .1
            .wait_while(job_budget_lock, |budget| *budget == 0)
            .unwrap();

        *job_budget_lock -= 1;
        drop(job_budget_lock);

        JobToken {
            job_budget: self.job_budget.clone(),
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct JobToken {
    job_budget: Arc<(Mutex<usize>, Condvar)>,
}

impl Drop for JobToken {
    fn drop(&mut self) {
        let mut job_budget_lock = self.job_budget.0.lock().unwrap();

        *job_budget_lock += 1;

        drop(job_budget_lock);
        self.job_budget.1.notify_all();
    }
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

    const DEFAULT_QUEUE_SIZE: usize = 8;

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
