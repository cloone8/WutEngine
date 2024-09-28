use core::any::TypeId;

use crate::vec::Dynamic;
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

    let new = world.create_entity();
    world.add_component_to_entity(new, Dynamic::new(pos));

    world.assert_coherent::<false>();

    let new = world.create_entity();
    world.add_component_to_entity(new, Dynamic::new(vel));
    world.assert_coherent::<false>();

    let new = world.create_entity();
    world.add_component_to_entity(new, Dynamic::new(size));

    world.assert_coherent::<false>();

    unsafe {
        let num_found: usize = world
            .query(|_, found: &Position| {
                assert_eq!(pos, *found);
                1
            })
            .into_iter()
            .sum();

        assert_eq!(1, num_found);

        let num_found = world
            .query(|_, found: &Velocity| {
                assert_eq!(vel, *found);
                1
            })
            .into_iter()
            .sum();

        assert_eq!(1, num_found);

        let num_found = world
            .query(|_, found: &Size| {
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

    let new = world.create_entity();
    world.add_component_to_entity(new, Dynamic::new(pos1));

    world.assert_coherent::<false>();

    let new = world.create_entity();
    world.add_component_to_entity(new, Dynamic::new(pos2));

    world.assert_coherent::<false>();

    let new = world.create_entity();
    world.add_component_to_entity(new, Dynamic::new(size));

    world.assert_coherent::<false>();

    unsafe {
        let found = world.query(|_, found: &Position| *found);

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
        let num_found = world.query(|_, found: &Size| {
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

    let a = world.create_entity();
    world.add_component_to_entity(a, Dynamic::new(Position { x: 0.0, y: 0.1 }));

    world.assert_coherent::<false>();

    let b = world.create_entity();
    world.add_component_to_entity(b, Dynamic::new(Velocity { x: 0.0, y: 0.1 }));

    world.assert_coherent::<false>();

    let c = world.create_entity();
    world.add_component_to_entity(c, Dynamic::new(Size { x: 0.0 }));

    world.assert_coherent::<false>();

    world.remove_entity(a);
    world.assert_coherent::<false>();

    world.remove_entity(c);
    world.assert_coherent::<false>();

    world.remove_entity(b);
    world.assert_coherent::<false>();

    unsafe {
        world.query(|_, p: &Position| {
            panic!("Found unexpected position {:?}", p);
        });

        world.query(|_, p: &Velocity| {
            panic!("Found unexpected velocity {:?}", p);
        });

        world.query(|_, p: &Size| {
            panic!("Found unexpected size {:?}", p);
        });
    }
}

#[test]
fn create_and_remove_multiple_same_component() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    let a = world.create_entity();
    world.add_component_to_entity(a, Dynamic::new(Position { x: 0.0, y: 0.1 }));

    world.assert_coherent::<false>();

    let b = world.create_entity();
    world.add_component_to_entity(b, Dynamic::new(Position { x: 0.0, y: 0.1 }));

    world.assert_coherent::<false>();

    let c = world.create_entity();
    world.add_component_to_entity(c, Dynamic::new(Size { x: 0.0 }));

    world.assert_coherent::<false>();

    world.remove_entity(a);
    world.assert_coherent::<false>();

    world.remove_entity(c);
    world.assert_coherent::<false>();

    world.remove_entity(b);
    world.assert_coherent::<false>();

    unsafe {
        world.query(|_, p: &Position| {
            panic!("Found unexpected position {:?}", p);
        });

        world.query(|_, p: &Size| {
            panic!("Found unexpected size {:?}", p);
        });
    }
}

#[test]
fn create_entity_and_add_components() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    let entity = world.create_entity();
    world.add_component_to_entity(entity, Dynamic::new(Position { x: 0.0, y: 0.1 }));

    world.assert_coherent::<false>();

    unsafe {
        let res = world.query(|id, _p: &Position| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.add_component_to_entity(entity, Dynamic::new(Size { x: 5.0 }));

    unsafe {
        let res = world.query(|id, _p: &Position| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());

        let res = world.query(|id, _s: &Size| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());

        let res = world.query(|id, _components: (&Position, &Size)| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());

        let res = world.query(|id, _components_inv: (&Size, &Position)| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }
}

#[test]
fn create_entity_and_remove_single_components() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    let entity = world.create_entity();
    world.add_component_to_entity(entity, Dynamic::new(Position { x: 0.0, y: 0.1 }));

    world.assert_coherent::<false>();

    unsafe {
        let res = world.query(|id, _p: &Position| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.add_component_to_entity(entity, Dynamic::new(Size { x: 5.0 }));
    world.add_component_to_entity(entity, Dynamic::new(Velocity { x: 6.0, y: 7.0 }));

    unsafe {
        let res = world.query(|id, _components_inv: (&Position, &Size, &Velocity)| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.remove_components_from_entity(entity, &[TypeId::of::<Size>()]);

    unsafe {
        world.query(|id, _components_inv: (&Position, &Size, &Velocity)| {
            panic!("Found invalid entity: {:?}", id);
        });

        let res = world.query(|id, _components_inv: (&Position, &Velocity)| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.remove_components_from_entity(entity, &[TypeId::of::<Position>()]);

    unsafe {
        world.query(|id, _components_inv: (&Position, &Size, &Velocity)| {
            panic!("Found invalid entity: {:?}", id);
        });

        world.query(|id, _components_inv: (&Position, &Velocity)| {
            panic!("Found invalid entity: {:?}", id);
        });

        let res = world.query(|id, _components_inv: &Velocity| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.remove_components_from_entity(entity, &[TypeId::of::<Velocity>()]);

    unsafe {
        world.query(|id, _components_inv: &Velocity| {
            panic!("Found invalid entity: {:?}", id);
        });
    }
}

#[test]
fn create_entity_and_remove_multiple_components() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    let entity = world.create_entity();
    world.add_component_to_entity(entity, Dynamic::new(Position { x: 0.0, y: 0.1 }));

    world.assert_coherent::<false>();

    unsafe {
        let res = world.query(|id, _p: &Position| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.add_component_to_entity(entity, Dynamic::new(Size { x: 5.0 }));
    world.add_component_to_entity(entity, Dynamic::new(Velocity { x: 6.0, y: 7.0 }));

    unsafe {
        let res = world.query(|id, _components_inv: (&Position, &Size, &Velocity)| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.remove_components_from_entity(entity, &[TypeId::of::<Size>(), TypeId::of::<Velocity>()]);

    unsafe {
        world.query(|id, _components_inv: (&Position, &Size, &Velocity)| {
            panic!("Found invalid entity: {:?}", id);
        });

        world.query(|id, _components_inv: &Size| {
            panic!("Found invalid entity: {:?}", id);
        });

        world.query(|id, _components_inv: &Velocity| {
            panic!("Found invalid entity: {:?}", id);
        });

        let res = world.query(|id, _components_inv: &Position| {
            assert_eq!(entity, id);
        });

        assert_eq!(1, res.len());
    }

    world.remove_components_from_entity(entity, &[TypeId::of::<Position>()]);

    unsafe {
        world.query(|id, _components_inv: &Position| {
            panic!("Found invalid entity: {:?}", id);
        });
    }
}

#[test]
fn query_unknown_type() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    unsafe {
        let result: Vec<()> = world.query(|id, _components: &Position| {
            panic!("World should be empty, found entity {:?} instead", id);
        });

        assert_eq!(0, result.len());
    }

    let entity = world.create_entity();
    world.add_component_to_entity(entity, Dynamic::new(Size { x: 5.0 }));
    world.add_component_to_entity(entity, Dynamic::new(Velocity { x: 6.0, y: 7.0 }));

    unsafe {
        let result: Vec<()> = world.query(|id, _components: (&Size, &Position, &Velocity)| {
            panic!("Found invalid entity {:?}", id);
        });

        assert_eq!(0, result.len());
    }
}

#[test]
fn query_optional() {
    let mut world = World::new();
    world.assert_coherent::<false>();

    let entity_a = world.create_entity();
    world.add_component_to_entity(entity_a, Dynamic::new(Size { x: 5.0 }));

    let entity_b = world.create_entity();
    world.add_component_to_entity(entity_b, Dynamic::new(Size { x: 5.0 }));
    world.add_component_to_entity(entity_b, Dynamic::new(Velocity { x: 6.0, y: 7.0 }));

    let entity_c = world.create_entity();
    world.add_component_to_entity(entity_c, Dynamic::new(Size { x: 5.0 }));
    world.add_component_to_entity(entity_c, Dynamic::new(Velocity { x: 6.0, y: 7.0 }));

    let entity_d = world.create_entity();
    world.add_component_to_entity(entity_d, Dynamic::new(Size { x: 5.0 }));
    world.add_component_to_entity(entity_d, Dynamic::new(Position { x: 0.0, y: 0.1 }));

    unsafe {
        let result: Vec<()> = world.query(|_id, _components: &Size| {});

        assert_eq!(4, result.len());
    }

    unsafe {
        let result: Vec<()> = world.query(|id, components: (&Size, Option<&Velocity>)| {
            let (_size, vel) = components;

            if vel.is_some() {
                assert!(id == entity_b || id == entity_c);
            } else {
                assert!(id == entity_a || id == entity_d);
            }
        });

        assert_eq!(4, result.len());
    }

    unsafe {
        let result: Vec<()> = world.query(|id, components: (&Size, Option<&Position>)| {
            let (_size, pos) = components;

            if pos.is_some() {
                assert_eq!(entity_d, id);
            } else {
                assert_ne!(entity_d, id);
            }
        });

        assert_eq!(4, result.len());
    }

    unsafe {
        let result: Vec<()> = world.query(
            |id, components: (&Size, Option<&Position>, Option<&Velocity>)| {
                let (_size, pos, vel) = components;
                if id == entity_a {
                    assert!(pos.is_none());
                    assert!(vel.is_none());
                } else if id == entity_b || id == entity_c {
                    assert!(pos.is_none());
                    assert!(vel.is_some());
                } else if id == entity_d {
                    assert!(pos.is_some());
                    assert!(vel.is_none());
                } else {
                    panic!("Unknown entity: {}", id);
                }
            },
        );

        assert_eq!(4, result.len());
    }
}
