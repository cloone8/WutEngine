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

    unsafe {
        let num_found: usize = world
            .query(|found: &Position| {
                assert_eq!(pos, *found);
                1
            })
            .into_iter()
            .sum();

        assert_eq!(1, num_found);

        let num_found = world
            .query(|found: &Velocity| {
                assert_eq!(vel, *found);
                1
            })
            .into_iter()
            .sum();

        assert_eq!(1, num_found);

        let num_found = world
            .query(|found: &Size| {
                assert_eq!(size, *found);
                1
            })
            .into_iter()
            .sum();

        assert_eq!(1, num_found);
    }
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

    unsafe {
        let found = world.query(|found: &Position| *found);

        let mut found1 = false;
        let mut found2 = false;

        for foundpos in found {
            if foundpos == pos1 {
                assert!(!found1);
                found1 = true;
            } else if foundpos == pos2 {
                assert!(!found2);
                found2 = true;
            } else {
                panic!("Got unknown position: {:?}", foundpos);
            }
        }
        let num_found = world.query(|found: &Size| {
            assert_eq!(size, *found);
            1
        });

        assert_eq!(1, num_found.into_iter().sum());
    }
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

    unsafe {
        world.query(|p: &Position| {
            panic!("Found unexpected position {:?}", p);
        });

        world.query(|p: &Velocity| {
            panic!("Found unexpected velocity {:?}", p);
        });

        world.query(|p: &Size| {
            panic!("Found unexpected size {:?}", p);
        });
    }
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

    unsafe {
        world.query(|p: &Position| {
            panic!("Found unexpected position {:?}", p);
        });

        dbg!(b);

        world.query(|p: &Size| {
            panic!("Found unexpected size {:?}", p);
        });
    }
}
