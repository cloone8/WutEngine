//! The WutEngine Editor

use wutengine::builtins::components::ui::ScreenSpaceUICanvas;
use wutengine::gameobject::GameObject;
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::plugins::{self, WutEnginePlugin};
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;
use wutengine::ui::UIPlugin;
use wutengine::windowing::WindowIdentifier;
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

    runtime.with_plugin(UIPlugin::default());
    runtime.with_plugin(WutEngineEditorPlugin::new());

    runtime.run::<OpenGLRenderer>();
}

/// The main editor starter plugin for the WutEngine Editor
#[derive(Debug)]
struct WutEngineEditorPlugin;

impl WutEngineEditorPlugin {
    /// A new empty WutEngine editor plugin
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl WutEnginePlugin for WutEngineEditorPlugin {
    fn on_start(&mut self, context: &mut plugins::Context) {
        let main_window_id = WindowIdentifier::new("main");

        context.windows.open(OpenWindowParams {
            id: main_window_id.clone(),
            title: "WutEngine Editor".to_string(),
            mode: FullscreenType::Windowed,
            ignore_existing: false,
        });

        let mut main_ui = GameObject::new(Some("Main UI"));

        main_ui.add_component(ScreenSpaceUICanvas {
            window: main_window_id,
        });

        context.engine.spawn_gameobject(main_ui);
    }
}
