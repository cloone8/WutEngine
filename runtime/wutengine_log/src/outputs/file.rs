use std::fs;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use wutengine_sync::Mutex;

use crate::LoggerOutput;
use crate::duration_since_logger_start;

/// A simple file logger that writes all logs to the given file
#[derive(Debug)]
pub struct FileLogger {
    file: Mutex<BufWriter<File>>,
}

/// An error in [`FileLogger::new`]
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum CreateFileLoggerErr {
    /// The file name contains a parent directory
    #[display("File name contains a path seperator")]
    PathSep,

    /// No `data_local_dir` is known for this platform
    #[display("No data_local_dir is known for this platform")]
    UnknownDir,

    /// I/O error
    #[display("I/O error: {_0}")]
    Io(io::Error),
}

impl FileLogger {
    /// Creates a new [`FileLogger`] that logs to the given file, in the organization and app directory.
    /// Creates the directories recursively, if it does not already exist.
    ///
    /// If `{dir}/{file_name}.log` already exists, it is first moved to `{dir}/{file_name}_prev.log`. Any existing `_prev` log is deleted.
    pub fn new(
        org_name: &str,
        app_name: &str,
        file_name: impl AsRef<Path>,
    ) -> Result<Self, CreateFileLoggerErr> {
        let org_name = org_name.trim();
        let app_name = app_name.trim();

        assert!(!org_name.is_empty(), "Org name must not be empty");
        assert!(!app_name.is_empty(), "App name must not be empty");

        let file_name = file_name.as_ref();

        if file_name.parent().is_some_and(|parent| !parent.is_empty()) {
            return Err(CreateFileLoggerErr::PathSep);
        }

        let Some(base_log_dir) = dirs::data_local_dir() else {
            return Err(CreateFileLoggerErr::UnknownDir);
        };

        let dir = base_log_dir.join(org_name).join(app_name).join("logs");

        fs::create_dir_all(&dir)?;

        let (file_name, file_name_prev) = get_log_file_names(file_name);

        let log_file_path = dir.join(file_name);
        let prev_log_file_path = dir.join(file_name_prev);

        // Move <file> to <file>_prev
        if fs::exists(&log_file_path)? {
            if fs::exists(&prev_log_file_path)? {
                fs::remove_file(&prev_log_file_path)?;
            }

            fs::rename(&log_file_path, prev_log_file_path)?;
        }

        // Should not fail because we just checked (and moved) the file in the block above
        let opened_log_file = File::options()
            .create_new(true)
            .write(true)
            .open(log_file_path)?;

        let writer = BufWriter::new(opened_log_file);

        Ok(Self {
            file: Mutex::new(writer),
        })
    }
}

impl LoggerOutput for FileLogger {
    fn write(&self, entry: &crate::LogEntry) {
        let time = duration_since_logger_start(&entry.time).as_secs_f64();

        let mut writer = self.file.lock();

        _ = writeln!(writer, "({time:.3}s) [{}] {}", entry.level, entry.message);
    }

    fn flush(&self) {
        // Nothing to do here. It's not like we can log it
        _ = self.file.lock().flush();
    }

    fn flush_on_panic(&self) {
        let Some(mut locked) = self.file.try_lock() else {
            // Cannot block here
            return;
        };

        _ = locked.flush();
    }
}

fn get_log_file_names(file_name: &Path) -> (PathBuf, PathBuf) {
    let mut file_name = file_name.to_path_buf();

    let mut file_name_prev = file_name.clone();
    file_name_prev.as_mut_os_string().push("_prev");

    file_name.add_extension("log");
    file_name_prev.add_extension("log");

    (file_name, file_name_prev)
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::get_log_file_names;

    /// A plain name gets `.log`, and the previous file gets `_prev` before the extension
    #[test]
    fn plain_name() {
        let (current, prev) = get_log_file_names(Path::new("engine"));

        assert_eq!(current, Path::new("engine.log"), "current log file name");
        assert_eq!(prev, Path::new("engine_prev.log"), "previous log file name");
    }

    /// A name that already has an extension keeps it, and `.log` is added after it
    #[test]
    fn name_with_extension() {
        let (current, prev) = get_log_file_names(Path::new("engine.render"));

        assert_eq!(
            current,
            Path::new("engine.render.log"),
            "current log file name"
        );
        assert_eq!(
            prev,
            Path::new("engine.render_prev.log"),
            "previous log file name"
        );
    }
}
