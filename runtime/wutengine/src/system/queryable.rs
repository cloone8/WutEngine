use core::any::TypeId;
use std::collections::HashSet;

/// Helper trait that allows for better runtime scheduling of ECS systems
/// Should not be implemented by hand. Is automatically implemented for all valid types.
pub trait Queryable {
    /// The amount of shared borrows, for preallocation purposes
    const NUM_SHARED_BORROWS: usize;

    /// The amount of exclusive borrows, for preallocation purposes
    const NUM_EXCLUSIVE_BORROWS: usize;

    /// Adds the borrows of this query to their corresponding maps
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>);
}

impl<T> Queryable for &T
where
    T: hecs::Component,
{
    const NUM_SHARED_BORROWS: usize = 1;
    const NUM_EXCLUSIVE_BORROWS: usize = 0;

    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, _exclusive: &mut HashSet<TypeId>) {
        shared.insert(TypeId::of::<T>());
    }
}

impl<T> Queryable for &mut T
where
    T: hecs::Component,
{
    const NUM_SHARED_BORROWS: usize = 0;
    const NUM_EXCLUSIVE_BORROWS: usize = 1;

    #[inline]
    fn register_borrows(_shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        exclusive.insert(TypeId::of::<T>());
    }
}

impl<T> Queryable for Option<T>
where
    T: Queryable,
{
    const NUM_SHARED_BORROWS: usize = T::NUM_SHARED_BORROWS;
    const NUM_EXCLUSIVE_BORROWS: usize = T::NUM_EXCLUSIVE_BORROWS;

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
    const NUM_SHARED_BORROWS: usize = const {
        if L::NUM_SHARED_BORROWS > R::NUM_SHARED_BORROWS {
            L::NUM_SHARED_BORROWS
        } else {
            R::NUM_SHARED_BORROWS
        }
    };

    const NUM_EXCLUSIVE_BORROWS: usize = const {
        if L::NUM_EXCLUSIVE_BORROWS > R::NUM_EXCLUSIVE_BORROWS {
            L::NUM_EXCLUSIVE_BORROWS
        } else {
            R::NUM_EXCLUSIVE_BORROWS
        }
    };

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
    const NUM_SHARED_BORROWS: usize = 0;
    const NUM_EXCLUSIVE_BORROWS: usize = 0;

    #[inline]
    fn register_borrows(_shared: &mut HashSet<TypeId>, _exclusive: &mut HashSet<TypeId>) {}
}

impl<Q, R> Queryable for hecs::With<Q, R>
where
    Q: Queryable,
    R: Queryable,
{
    const NUM_SHARED_BORROWS: usize = Q::NUM_SHARED_BORROWS;
    const NUM_EXCLUSIVE_BORROWS: usize = Q::NUM_EXCLUSIVE_BORROWS;

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
    const NUM_SHARED_BORROWS: usize = Q::NUM_SHARED_BORROWS;
    const NUM_EXCLUSIVE_BORROWS: usize = Q::NUM_EXCLUSIVE_BORROWS;

    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        Q::register_borrows(shared, exclusive);
    }
}

/// Generates tuple implementations for [`Queryable`]
macro_rules! queryable_tuples {
    ($t:ident) => {
        impl<$t> Queryable for ($t,)
        where
            $t: Queryable,
        {
            const NUM_SHARED_BORROWS: usize = $t::NUM_SHARED_BORROWS;
            const NUM_EXCLUSIVE_BORROWS: usize = $t::NUM_EXCLUSIVE_BORROWS;

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
            const NUM_SHARED_BORROWS: usize = $t::NUM_SHARED_BORROWS  $(+ $others::NUM_SHARED_BORROWS)*;
            const NUM_EXCLUSIVE_BORROWS: usize = $t::NUM_EXCLUSIVE_BORROWS  $(+ $others::NUM_EXCLUSIVE_BORROWS)*;

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

#[cfg(test)]
mod test {
    use core::any::TypeId;
    use std::collections::HashSet;

    use hecs::{Or, Satisfies, With, Without};

    use super::Queryable;

    struct CompA;
    struct CompB;
    struct CompC;
    struct CompD;
    struct CompE;
    struct CompF;
    struct CompG;
    struct Dummy;
    struct Invalid;

    const A_TID: TypeId = TypeId::of::<CompA>();
    const B_TID: TypeId = TypeId::of::<CompB>();
    const C_TID: TypeId = TypeId::of::<CompC>();
    const D_TID: TypeId = TypeId::of::<CompD>();
    const E_TID: TypeId = TypeId::of::<CompE>();
    const F_TID: TypeId = TypeId::of::<CompF>();
    const G_TID: TypeId = TypeId::of::<CompG>();
    const DUMMY_TID: TypeId = TypeId::of::<Dummy>();
    const INVALID_TID: TypeId = TypeId::of::<Invalid>();

    #[test]
    fn test_readonly() {
        let mut shared = HashSet::new();
        let mut exclusive = HashSet::new();

        <(&CompA, &CompB, &CompC, &CompD, &CompE, &CompF) as Queryable>::register_borrows(
            &mut shared,
            &mut exclusive,
        );

        assert!(exclusive.is_empty(), "No exclusive borrows present");

        assert_eq!(6, shared.len(), "Incorrect amount of shared borrows");
        assert_eq!(6, shared.len(), "Incorrect amount of shared borrows");

        assert!(shared.contains(&A_TID));
        assert!(shared.contains(&B_TID));
        assert!(shared.contains(&C_TID));
        assert!(shared.contains(&D_TID));
        assert!(shared.contains(&E_TID));
        assert!(shared.contains(&F_TID));
    }

    #[test]
    fn test_writeonly() {
        let mut shared = HashSet::new();
        let mut exclusive = HashSet::new();

        <(
            &mut CompA,
            &mut CompB,
            &mut CompC,
            &mut CompD,
            &mut CompE,
            &mut CompF,
        ) as Queryable>::register_borrows(&mut shared, &mut exclusive);

        assert!(shared.is_empty(), "No shared borrows present");

        assert_eq!(6, exclusive.len(), "Incorrect amount of exclusive borrows");

        assert!(exclusive.contains(&A_TID));
        assert!(exclusive.contains(&B_TID));
        assert!(exclusive.contains(&C_TID));
        assert!(exclusive.contains(&D_TID));
        assert!(exclusive.contains(&E_TID));
        assert!(exclusive.contains(&F_TID));
    }

    #[test]
    fn test_mixed() {
        let mut shared = HashSet::new();
        let mut exclusive = HashSet::new();

        <(
            &mut CompA,
            &CompB,
            &mut CompC,
            &CompD,
            &mut CompE,
            &CompF,
        ) as Queryable>::register_borrows(&mut shared, &mut exclusive);

        assert_eq!(3, shared.len(), "Incorrect amount of shared borrows");
        assert_eq!(3, exclusive.len(), "Incorrect amount of exclusive borrows");

        assert!(exclusive.contains(&A_TID));
        assert!(shared.contains(&B_TID));
        assert!(exclusive.contains(&C_TID));
        assert!(shared.contains(&D_TID));
        assert!(exclusive.contains(&E_TID));
        assert!(shared.contains(&F_TID));
    }

    #[test]
    fn test_nested() {
        let mut shared = HashSet::new();
        let mut exclusive = HashSet::new();

        <(
            Option<&mut CompA>,
            With<&CompB, &Dummy>,
            Without<&mut CompC, &Dummy>,
            Or<&CompD, &mut CompE>,
            Satisfies<&Invalid>,
            (&mut CompF, &CompG),
        ) as Queryable>::register_borrows(&mut shared, &mut exclusive);

        assert_eq!(3, shared.len(), "Incorrect amount of shared borrows");
        assert_eq!(4, exclusive.len(), "Incorrect amount of exclusive borrows");

        // Option<>
        assert!(exclusive.contains(&A_TID));
        assert!(!shared.contains(&A_TID));

        // With<>
        assert!(shared.contains(&B_TID));
        assert!(!shared.contains(&DUMMY_TID));
        assert!(!exclusive.contains(&B_TID));
        assert!(!exclusive.contains(&DUMMY_TID));

        // Without<>
        assert!(!shared.contains(&C_TID));
        assert!(!shared.contains(&DUMMY_TID));
        assert!(exclusive.contains(&C_TID));
        assert!(!exclusive.contains(&DUMMY_TID));

        // Or<>
        assert!(shared.contains(&D_TID));
        assert!(!exclusive.contains(&D_TID));
        assert!(!shared.contains(&E_TID));
        assert!(exclusive.contains(&E_TID));

        // Satisfies<>
        assert!(!shared.contains(&INVALID_TID));
        assert!(!exclusive.contains(&INVALID_TID));

        // Tuple()
        assert!(!shared.contains(&F_TID));
        assert!(exclusive.contains(&F_TID));
        assert!(shared.contains(&G_TID));
        assert!(!exclusive.contains(&G_TID));
    }

    #[test]
    fn test_deep_nested() {
        let mut shared = HashSet::new();
        let mut exclusive = HashSet::new();

        <(
            With<Option<Or<(&CompA, &mut CompB), Option<(&CompC, &mut CompD)>>>, &Dummy>,
        ) as Queryable>::register_borrows(&mut shared, &mut exclusive);

        assert_eq!(2, shared.len());
        assert_eq!(2, exclusive.len());

        assert!(shared.contains(&A_TID));
        assert!(exclusive.contains(&B_TID));
        assert!(shared.contains(&C_TID));
        assert!(exclusive.contains(&D_TID));
        assert!(!shared.contains(&DUMMY_TID));
        assert!(!exclusive.contains(&DUMMY_TID));
    }
}
