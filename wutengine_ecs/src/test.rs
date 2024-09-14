use crate::world::World;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Size {
    x: f32,
}

#[test]
fn simple_create() {
    let mut world = World::new();
    let pos = Position { x: 0.0, y: 0.1 };
    let vel = Velocity { x: 0.2, y: 0.3 };
    let size = Size { x: 0.4 };

    world.assert_coherent::<false>();

    world.create_entity(pos);

    world.assert_coherent::<false>();

    world.create_entity(vel);

    world.assert_coherent::<false>();

    world.create_entity(size);

    world.assert_coherent::<false>();

    let positions = world.query::<Position>();
    let velocities = world.query::<Velocity>();
    let sizes = world.query::<Size>();

    assert_eq!(1, positions.len());
    assert_eq!(pos, *positions[0]);

    assert_eq!(1, velocities.len());
    assert_eq!(vel, *velocities[0]);

    assert_eq!(1, sizes.len());
    assert_eq!(size, *sizes[0]);
}

#[test]
fn create_multiple_same_component() {
    let mut world = World::new();

    let pos1 = Position { x: 0.0, y: 0.1 };
    let pos2 = Position { x: 0.2, y: 0.3 };
    let size = Size { x: 0.4 };

    world.assert_coherent::<false>();

    world.create_entity(pos1);

    world.assert_coherent::<false>();

    world.create_entity(pos2);

    world.assert_coherent::<false>();

    world.create_entity(size);

    world.assert_coherent::<false>();

    let positions = world.query::<Position>();
    let sizes = world.query::<Size>();

    assert_eq!(2, positions.len());
    assert!(positions.contains(&&pos1));
    assert!(positions.contains(&&pos2));

    assert_eq!(1, sizes.len());
    assert_eq!(size, *sizes[0]);
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

    let positions = world.query::<Position>();
    let velocities = world.query::<Velocity>();
    let sizes = world.query::<Size>();

    assert_eq!(0, positions.len());
    assert_eq!(0, velocities.len());
    assert_eq!(0, sizes.len());
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

    let positions = world.query::<Position>();
    let sizes = world.query::<Size>();

    assert_eq!(0, positions.len());
    assert_eq!(0, sizes.len());
}
