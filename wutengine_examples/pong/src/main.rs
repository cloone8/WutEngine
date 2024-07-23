use wutengine::{
    command::Command,
    core::{
        system::{System, SystemPhase},
        windowing::WindowIdentifier,
    },
    plugin::EnginePlugin,
    renderer::OpenGLRenderer,
    world::World,
    EngineCommand, RuntimeInitializer, SystemFunction,
};

fn main() {
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
    commands.window().open(WindowIdentifier::new("pong"));
}
