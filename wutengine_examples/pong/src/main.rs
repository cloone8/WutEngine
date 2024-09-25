use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.with_log_config(LogConfig {
        runtime: Some(ComponentLogConfig {
            min_level: log::LevelFilter::Info,
            output: log::LogOutput::StdOut,
        }),
        ..Default::default()
    });

    runtime.run::<OpenGLRenderer>().unwrap();
}
