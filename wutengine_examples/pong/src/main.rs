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
                    func: SystemFunction::Immutable(init_system),
                }));
            }
        }

        response
    }
}

#[derive(Debug)]
struct TestComponent {
    lmao: String,
}

impl DynComponent for TestComponent {
    fn get_dyn_component_id(&self) -> ComponentTypeId {
        TestComponent::COMPONENT_ID
    }
}

impl Component for TestComponent {
    const COMPONENT_ID: wutengine::core::component::ComponentTypeId =
        ComponentTypeId::from_int(231);
}

fn init_system(commands: &mut Command, world: &World) {
    commands.window().open(WindowIdentifier::new("pong"));

    let result = world.query::<&TestComponent>();

    let a = result[0].1;

    println!("{}", a.lmao);
}
