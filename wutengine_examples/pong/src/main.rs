use wutengine::{
    core::{System, SystemFunction, SystemPhase, World},
    plugin::EnginePlugin,
    EngineCommand, RuntimeInitializer,
};

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.add_plugin(Box::new(PongStarterPlugin));

    runtime.run().unwrap();
}

struct PongStarterPlugin;

impl EnginePlugin for PongStarterPlugin {
    fn on_event(&mut self, event: &wutengine::EngineEvent) -> Vec<EngineCommand> {
        let mut response = Vec::new();

        match event {
            wutengine::EngineEvent::RuntimeStart => {
                response.push(EngineCommand::AddSystem(System {
                    phase: SystemPhase::RuntimeStart,
                    func: SystemFunction::Immutable(hello_system),
                }))
            }
        }

        response
    }
}

fn hello_system(world: &World) {
    println!("Hello world!");
}
