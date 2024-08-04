use itertools::Itertools;

use super::*;

#[test]
fn test_subset() {
    let full_set = &[
        TypeId::of::<u8>(),
        TypeId::of::<u16>(),
        TypeId::of::<u32>(),
        TypeId::of::<u64>(),
        TypeId::of::<i8>(),
        TypeId::of::<i16>(),
        TypeId::of::<i32>(),
        TypeId::of::<i64>(),
    ];

    let full = Archetype::from_iter(full_set);
    let all_valid: Vec<Archetype> = full_set
        .iter()
        .powerset()
        .filter(|tids| !tids.is_empty())
        .map(Archetype::from_iter)
        .collect();

    for valid in &all_valid {
        assert!(
            valid.is_subset_of(&full),
            "Subset {:#?} not detected as a subset of {:#?}",
            valid,
            full
        );
    }

    let all_invalid: Vec<Archetype> = all_valid
        .into_iter()
        .map(|valid| valid.add_type::<u128>())
        .collect();

    for invalid in &all_invalid {
        assert!(
            !invalid.is_subset_of(&full),
            "Subset {:#?} wrongly detected as a subset of {:#?}",
            invalid,
            full
        );
    }
}

#[test]
fn test_shuffled() {
    let full_set = &[
        TypeId::of::<u8>(),
        TypeId::of::<u16>(),
        TypeId::of::<u32>(),
        TypeId::of::<u64>(),
        TypeId::of::<i8>(),
        TypeId::of::<i16>(),
        TypeId::of::<i32>(),
        TypeId::of::<i64>(),
    ];

    let full = Archetype::from_iter(full_set);

    let permutations: Vec<Archetype> = full_set
        .iter()
        .permutations(full_set.len())
        .map(Archetype::from_iter)
        .collect();

    for permutation in permutations {
        assert_eq!(
            full, permutation,
            "Permutation {:#?} not detected as equal to {:#?}",
            permutation, full
        );
    }
}
