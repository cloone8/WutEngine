use crate::world::World;

#[derive(Debug, Clone, Copy, Default)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct Size {
    x: f32,
}

#[test]
fn simple_create() {
    let mut world = World::new();

    world.assert_coherent::<false>();

    world.create_entity(Position { x: 0.0, y: 0.1 });

    world.assert_coherent::<false>();

    world.create_entity(Velocity { x: 0.0, y: 0.1 });

    world.assert_coherent::<false>();

    world.create_entity(Size { x: 0.0 });

    world.assert_coherent::<false>();
}

#[test]
fn create_multiple_same_component() {
    let mut world = World::new();

    world.assert_coherent::<false>();

    world.create_entity(Position { x: 0.0, y: 0.1 });

    world.assert_coherent::<false>();

    world.create_entity(Position { x: 0.2, y: 0.3 });

    world.assert_coherent::<false>();

    world.create_entity(Size { x: 0.4 });

    world.assert_coherent::<false>();
}

#[test]
fn create_and_remove_single() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    let a = world.create_entity(Position { x: 0.0, y: 0.1 });
    world.assert_coherent::<false>();

    let b = world.create_entity(Velocity { x: 0.0, y: 0.1 });
    world.assert_coherent::<false>();

    let c = world.create_entity(Size { x: 0.0 });
    world.assert_coherent::<false>();

    world.remove_entity(a);
    world.assert_coherent::<false>();

    world.remove_entity(c);
    world.assert_coherent::<false>();

    world.remove_entity(b);
    world.assert_coherent::<false>();
}

#[test]
fn create_and_remove_multiple_same_component() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    let a = world.create_entity(Position { x: 0.0, y: 0.1 });
    world.assert_coherent::<false>();

    let b = world.create_entity(Position { x: 0.0, y: 0.1 });
    world.assert_coherent::<false>();

    let c = world.create_entity(Size { x: 0.0 });
    world.assert_coherent::<false>();

    world.remove_entity(a);
    world.assert_coherent::<false>();

    world.remove_entity(c);
    world.assert_coherent::<false>();

    world.remove_entity(b);
    world.assert_coherent::<false>();
}
