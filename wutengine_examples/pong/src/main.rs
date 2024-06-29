use wutengine::{renderer::HeadlessRenderer, world::World, WutEngine};

fn main() {
    let logconfig = simplelog::ConfigBuilder::new()
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set logger time offset to local")
        .build();

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Debug,
        logconfig,
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .expect("Could not initialize logger");

    let world = World {};

    let engine = WutEngine::<HeadlessRenderer>::new(1, world);

    engine.run().unwrap();
}
