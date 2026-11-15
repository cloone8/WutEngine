use crate::LoggerOutput;
use crate::duration_since_logger_start;

/// Simple terminal logger that logs to `stdout`
#[derive(Debug, Clone, Copy)]
pub struct TerminalLogger; // TODO: Add support for color, stderr, and a level above/below which the logs to go stdout/stderr

impl LoggerOutput for TerminalLogger {
    fn write(&self, entry: &crate::LogEntry) {
        let time = duration_since_logger_start(&entry.time).as_secs_f64();

        println!("({time:.3}s) [{}] {}", entry.level, entry.message);
    }
}
