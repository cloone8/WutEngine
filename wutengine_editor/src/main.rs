//! The WutEngine Editor

#![allow(clippy::missing_docs_in_private_items)]

use wutengine::graphics::windowing::WindowIdentifier;
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::plugins::{self, WutEnginePlugin};
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;
use wutengine::windowing::{FullscreenType, OpenWindowParams};

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.with_log_config(LogConfig {
        runtime: Some(ComponentLogConfig {
            min_level: log::LevelFilter::Debug,
            output: log::LogOutput::StdOut,
        }),
        ..Default::default()
    });

    runtime.with_plugin(WutEngineEditorPlugin::new());

    runtime.run::<OpenGLRenderer>();
}

struct WutEngineEditorPlugin;

impl WutEngineEditorPlugin {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl WutEnginePlugin for WutEngineEditorPlugin {
    fn on_start(&mut self, context: &mut plugins::Context) {
        context.windows.open(OpenWindowParams {
            id: WindowIdentifier::new("Main"),
            title: "WutEngine".to_string(),
            mode: FullscreenType::Windowed,
            ignore_existing: false,
        });
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
