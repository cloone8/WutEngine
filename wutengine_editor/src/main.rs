use wutengine::command::Command;
use wutengine::core::{EntityId, SystemPhase};
use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.with_system::<test_system>(SystemPhase::Update);

    runtime.run::<OpenGLRenderer>();
}

struct TestComponentA {}

impl TestComponentA {
    pub fn hello(&self, entity: EntityId) {
        println!("Hello from {}", entity);
    }
}

struct TestComponentB {}

impl TestComponentB {
    pub fn hello_mut(&mut self, entity: EntityId) {
        println!("Hello mut from {}", entity);
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
