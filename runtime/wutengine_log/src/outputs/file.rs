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

impl FileLogger {
    /// Creates a new [`FileLogger`] that logs to the given file, in the organization and app directory.
    /// Creates the directories recursively, if it does not already exist.
    ///
    /// If `{dir}/{file_name}.log` already exists, it is first moved to `{dir}/{file_name}_prev.log`. Any existing `_prev` log is deleted.
    pub fn new(
        org_name: &str,
        app_name: &str,
        file_name: impl AsRef<Path>,
    ) -> Result<Self, io::Error> {
        let org_name = org_name.trim();
        let app_name = app_name.trim();
        assert!(!org_name.is_empty(), "Org name must not be empty");
        assert!(!app_name.is_empty(), "App name must not be empty");

        //TODO: Handle properly
        let base_log_dir = dirs::data_local_dir().expect("No data_local_dir");
        let dir = base_log_dir.join(org_name).join(app_name).join("logs");

        fs::create_dir_all(&dir)?;

        let file_name = file_name.as_ref();

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
    //TODO: Test log file name formatting and resolution
}
