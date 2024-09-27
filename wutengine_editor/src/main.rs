use wutengine::command::Command;
use wutengine::core::{Component, EntityId, SystemPhase};
use wutengine::plugins::WutEnginePlugin;
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.with_plugin(WutEngineEditorPlugin {});
    runtime.with_system::<test_system>(SystemPhase::Update);

    runtime.run::<OpenGLRenderer>();
}

struct WutEngineEditorPlugin;

impl WutEnginePlugin for WutEngineEditorPlugin {
    fn on_start(&mut self, commands: &mut Command) {
        commands
            .entity()
            .spawn()
            .with_component(TestComponentA {
                name: "CoolTestComponent".to_string(),
            })
            .with_component(TestComponentB { number: 69.69 })
            .build();
    }
}

struct TestComponentA {
    name: String,
}

impl Component for TestComponentA {}

impl TestComponentA {
    pub fn hello(&self, entity: EntityId) {
        println!("Hello from {}: {}", entity, self.name);
    }
}

struct TestComponentB {
    number: f64,
}

impl Component for TestComponentB {}

impl TestComponentB {
    pub fn hello_mut(&mut self, entity: EntityId) {
        println!("Hello mut from {}: {}", entity, self.number);
    }
}

#[wutengine::macros::system]
fn test_system(
    commands: &mut Command,
    entity: EntityId,
    a: &TestComponentA,
    b: &mut TestComponentB,
) {
    a.hello(entity);
    b.hello_mut(entity);

    commands.merge_with(Command::NONE);
}
