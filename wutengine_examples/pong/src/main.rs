use wutengine::{
    command::Command,
    core::{
        component::{Component, ComponentTypeId, DynComponent},
        system::{System, SystemPhase},
    },
    plugin::EnginePlugin,
    windowing::WindowIdentifier,
    world::World,
    EngineCommand, RuntimeInitializer, SystemFunction,
};

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.add_plugin(Box::new(PongStarterPlugin));
    runtime.add_component_type::<TestComponent>();

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
                    func: SystemFunction::Immutable(init_window),
                }));
            }
        }

        response
    }
}

fn init_window(commands: &mut Command, world: &World) {
    commands.window().open(WindowIdentifier::new("pong"));
    commands
        .entity()
        .spawn_with_components(vec![Box::new(TestComponent {
            name: "Henry".to_string(),
        })]);
}

#[derive(Debug)]
struct TestComponent {
    name: String,
}

impl Drop for TestComponent {
    fn drop(&mut self) {
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        println!("Goodbye from {}", self.name);
        println!("{}", bt);
    }
}

impl DynComponent for TestComponent {
    fn get_dyn_component_id(&self) -> ComponentTypeId {
        Self::get_component_id()
    }
}

impl Component for TestComponent {
    fn get_component_id() -> wutengine::core::component::ComponentTypeId {
        ComponentTypeId::from_int(69)
    }
}
