//! WutEngine ECS system registration and query helpers

use core::any::TypeId;
use std::collections::HashSet;

/// Helper trait that allows for better runtime scheduling of ECS systems
pub trait Queryable {
    /// Adds the borrows of this query to their corresponding maps
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>);
}

impl<T> Queryable for &T
where
    T: hecs::Component,
{
    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, _exclusive: &mut HashSet<TypeId>) {
        shared.insert(TypeId::of::<T>());
    }
}

impl<T> Queryable for &mut T
where
    T: hecs::Component,
{
    #[inline]
    fn register_borrows(_shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        exclusive.insert(TypeId::of::<T>());
    }
}

impl<T> Queryable for Option<T>
where
    T: Queryable,
{
    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        T::register_borrows(shared, exclusive);
    }
}

impl<L, R> Queryable for hecs::Or<L, R>
where
    L: Queryable,
    R: Queryable,
{
    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        L::register_borrows(shared, exclusive);
        R::register_borrows(shared, exclusive);
    }
}

impl<T> Queryable for hecs::Satisfies<T>
where
    T: Queryable,
{
    #[inline]
    fn register_borrows(_shared: &mut HashSet<TypeId>, _exclusive: &mut HashSet<TypeId>) {}
}

impl<Q, R> Queryable for hecs::With<Q, R>
where
    Q: Queryable,
    R: Queryable,
{
    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        Q::register_borrows(shared, exclusive);
    }
}

impl<Q, R> Queryable for hecs::Without<Q, R>
where
    Q: Queryable,
    R: Queryable,
{
    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        Q::register_borrows(shared, exclusive);
    }
}

/// Generates tuple implementations for [Queryable]
macro_rules! queryable_tuples {
    ($t:ident) => {
        impl<$t> Queryable for ($t,)
        where
            $t: Queryable,
        {
            #[inline]
            fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
                $t::register_borrows(shared, exclusive);
            }
        }
    };

    ($t:ident, $($others:ident),*) => {
        impl<$t, $($others),*> Queryable for ($t, $($others),*)
        where
            $t: Queryable,
            $($others: Queryable),*
        {
            #[inline]
            fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
                $t::register_borrows(shared, exclusive);
                $($others::register_borrows(shared, exclusive));*;
            }
        }

        queryable_tuples!($($others),*);
    };
}

queryable_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

/// Where, in the process of running a single tick, the system is called
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Called once each tick
    Update,

    /// Called once each fixed update. Depends on the configured fixed update time.
    /// Might be any number (or zero) times per frame
    FixedUpdate,
}

pub fn register_system<Q, F>(phase: Phase, sys: F)
where
    Q: crate::hecs::Query + Queryable,
    F: for<'a> Fn(crate::entity::Entity, Q::Item<'a>) + Send + Sync + 'static,
{
    let mut borrows_read = HashSet::new();
    let mut borrows_write = HashSet::new();

    Q::register_borrows(&mut borrows_read, &mut borrows_write);

    // let world = hecs::World::new();

    // let mut borrow = world.query::<(hecs::Entity, Q)>();

    // for item in borrow.iter() {
    //     sys(crate::entity::Entity(item.0), item.1)
    // }
}
