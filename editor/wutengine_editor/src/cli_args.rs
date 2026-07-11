//! Command-line arguments and types

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use wutengine::graphics;

/// Command line arguments for the WutEngine Editor
#[derive(Debug, Parser)]
#[command(version, about, author)]
pub(crate) struct CliArgs {
    /// The project to open. If not given, will prompt for a project file instead
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub(crate) project: Option<PathBuf>,

    /// The renderer to use. If not given, will use the default renderer for the current platform
    #[arg(long, value_enum)]
    pub(crate) renderer: Option<CliGraphicsBackend>,
}

/// The rendering backend to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub(crate) enum CliGraphicsBackend {
    /// DirectX 12 (Windows only)
    DX12,

    /// Vulkan (Window/Linux)
    Vulkan,

    /// Metal (MacOS only)
    Metal,
}

impl From<CliGraphicsBackend> for graphics::GraphicsBackend {
    fn from(value: CliGraphicsBackend) -> Self {
        match value {
            CliGraphicsBackend::DX12 => graphics::GraphicsBackend::DX12,
            CliGraphicsBackend::Vulkan => graphics::GraphicsBackend::Vulkan,
            CliGraphicsBackend::Metal => graphics::GraphicsBackend::Metal,
        }
    }
}
