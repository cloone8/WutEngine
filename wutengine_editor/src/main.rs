use wutengine::{
    command::{FullscreenType, OpenWindowParams},
    graphics::windowing::WindowIdentifier,
    plugin::EnginePlugin,
    renderer::OpenGLRenderer,
    EngineCommand, EngineEvent, RuntimeInitializer,
};

fn main() {
    let logconfig = simplelog::ConfigBuilder::new()
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set logger time offset to local")
        .build();

    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        logconfig,
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .expect("Could not initialize logger");

    log::info!("Starting editor");

    let mut runtime = RuntimeInitializer::new();

    runtime.add_plugin::<WutEngineEditorPlugin>();
    runtime.run::<OpenGLRenderer>().unwrap();
}

struct WutEngineEditorPlugin;

impl EnginePlugin for WutEngineEditorPlugin {
    fn build() -> Self
    where
        Self: Sized,
    {
        WutEngineEditorPlugin
    }

    fn on_event(&mut self, event: &EngineEvent) -> Vec<EngineCommand> {
        match event {
            EngineEvent::RuntimeStart => vec![EngineCommand::OpenWindow(OpenWindowParams {
                id: WindowIdentifier::new("main"),
                title: "WutEngine Editor".to_owned(),
                mode: FullscreenType::Windowed,
                ignore_existing: false,
            })],
        }
    }
}
