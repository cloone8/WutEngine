//! Hello: A tiny demo that starts the logger and writes a log at each level, inside a profiler scope

use std::process::ExitCode;

fn main() -> ExitCode {
    println!("Hello, world!");

    ExitCode::SUCCESS
}
