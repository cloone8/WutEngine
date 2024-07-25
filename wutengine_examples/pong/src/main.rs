use wutengine::{
    command::{Command, FullscreenType, OpenWindowParams},
    components::camera::Camera,
    core::{System, SystemPhase},
    graphics::{color::Color, windowing::WindowIdentifier},
    plugin::EnginePlugin,
    renderer::OpenGLRenderer,
    world::World,
    EngineCommand, RuntimeInitializer, SystemFunction,
};

fn main() {
    let logconfig = simplelog::ConfigBuilder::new()
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set logger time offset to local")
        .build();

    simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        logconfig,
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .expect("Could not initialize logger");

    log::info!("Starting Pong");

    let mut runtime = RuntimeInitializer::new();

    runtime.add_plugin(Box::new(PongStarterPlugin));
    runtime.run::<OpenGLRenderer>().unwrap();
}

struct PongStarterPlugin;

impl EnginePlugin for PongStarterPlugin {
    fn on_event(&mut self, event: &wutengine::EngineEvent) -> Vec<EngineCommand> {
        let mut response = Vec::new();

        match event {
            wutengine::EngineEvent::RuntimeStart => {
                response.push(EngineCommand::AddSystem(System {
                    phase: SystemPhase::RuntimeStart,
                    func: SystemFunction::Immutable(init_system),
                }));
            }
        }

        response
    }
}

fn init_system(commands: &mut Command, world: &World) {
    commands.window().open(OpenWindowParams {
        id: WindowIdentifier::new("pong"),
        title: "Pong - WutEngine Example".to_owned(),
        ignore_existing: false,
        mode: FullscreenType::Windowed,
    });

    let camera = Camera {
        display: WindowIdentifier::new("pong"),
        clear_color: Color::rgb(0.2, 0.3, 0.3),
    };

    commands
        .entity()
        .spawn_with_components(vec![Box::new(camera)]);
}
