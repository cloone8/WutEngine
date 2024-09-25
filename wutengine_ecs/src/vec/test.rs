use std::cell::Cell;
use std::rc::Rc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::*;

struct DropCounter<T> {
    val: T,
    dropped_count: Rc<Cell<usize>>,
}

impl<T> Drop for DropCounter<T> {
    fn drop(&mut self) {
        self.dropped_count.set(self.dropped_count.take() + 1);
    }
}

#[test]
fn test_push() {
    let mut vec = Vec::<u32>::new();
    let mut anyvec = AnyVec::new::<u32>();

    (0..u16::MAX).for_each(|elem| {
        vec.push(elem as u32);
        anyvec.push(elem as u32);
    });

    let vec_slice = vec.as_slice();
    let anyvec_slice = anyvec.as_slice::<u32>();

    for i in 0..u16::MAX {
        assert_eq!(vec_slice[i as usize], anyvec_slice[i as usize]);
    }
}

#[test]
fn test_push_pop() {
    let mut vec = Vec::<u32>::new();
    let mut anyvec = AnyVec::new::<u32>();

    (0..u16::MAX).for_each(|elem| {
        vec.push(elem as u32);
        anyvec.push(elem as u32);
    });

    let mut iter: usize = 0;
    loop {
        let vec_pop = vec.pop();
        let anyvec_pop = anyvec.pop::<u32>();

        assert_eq!(vec_pop, anyvec_pop, "Failed at iter {}", iter);

        if vec_pop.is_none() {
            break;
        }

        iter += 1;
    }
}

#[test]
fn test_len_empty_clear() {
    let mut anyvec = AnyVec::new::<u32>();

    assert!(anyvec.is_empty());

    (0..u16::MAX).for_each(|elem| {
        assert_eq!(elem as usize, anyvec.len());

        anyvec.push(elem as u32);
    });

    assert_eq!(u16::MAX as usize, anyvec.len());

    (0..(u16::MAX / 2)).for_each(|elem| {
        let expected_size = (u16::MAX - elem) as usize;

        assert_eq!(expected_size, anyvec.len());

        _ = anyvec.pop::<u32>();
    });

    anyvec.clear();

    assert_eq!(0, anyvec.len());
    assert!(anyvec.is_empty());
}

#[test]
fn test_drops() {
    let mut counters: Vec<Rc<Cell<usize>>> = Vec::new();
    let mut anyvec = AnyVec::new::<DropCounter<u32>>();

    (0..u16::MAX).for_each(|elem| {
        let counter = Rc::new(Cell::new(0));

        anyvec.push(DropCounter {
            val: elem as u32,
            dropped_count: counter.clone(),
        });

        counters.push(counter);
    });

    let anyvec_slice = anyvec.as_slice::<DropCounter<u32>>();

    for i in 0..u16::MAX {
        assert_eq!(i as u32, anyvec_slice[i as usize].val);
    }

    std::mem::drop(anyvec);

    for counter in counters {
        assert_eq!(1, counter.get());
    }
}

#[repr(packed)]
#[derive(Debug, PartialEq, Clone)]
struct WeirdStruct {
    a: i128,
    b: u64,
    c: u16,
    d: i64,
    e: isize,
    f: f64,
    g: f32,
    h: u8,
    i: u128,
}

impl WeirdStruct {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            a: rng.gen(),
            b: rng.gen(),
            c: rng.gen(),
            d: rng.gen(),
            e: rng.gen(),
            f: rng.gen(),
            g: rng.gen(),
            h: rng.gen(),
            i: rng.gen(),
        }
    }
}

#[test]
fn test_rand() {
    let mut rng = SmallRng::seed_from_u64(0xdeadbeef);

    let mut vec = Vec::<WeirdStruct>::new();
    let mut anyvec = AnyVec::new::<WeirdStruct>();

    for _ in 0..((u16::MAX as usize) * 100) {
        let do_push: bool = rng.gen_bool(0.80);

        if do_push {
            let val = WeirdStruct::random(&mut rng);

            vec.push(val.clone());
            anyvec.push::<WeirdStruct>(val);
        } else {
            let do_clear: bool = rng.gen_bool(0.05);

            if do_clear {
                vec.clear();
                anyvec.clear();
            } else {
                assert_eq!(vec.pop(), anyvec.pop::<WeirdStruct>());
            }
        }
    }

    loop {
        let vec_pop = vec.pop();
        let anyvec_pop = anyvec.pop::<WeirdStruct>();

        assert_eq!(vec_pop, anyvec_pop);

        if vec_pop.is_none() {
            break;
        }
    }
}

struct NoSize;

#[test]
fn test_zero_sized() {
    assert_eq!(
        0,
        size_of::<NoSize>(),
        "Test won't be valid because the type is not actually zero-sized"
    );

    let mut anyvec = AnyVec::new::<NoSize>();

    anyvec.push(NoSize);
    anyvec.push(NoSize);
    anyvec.push(NoSize);
    anyvec.push(NoSize);

    assert_eq!(4, anyvec.len());

    for item in anyvec.as_slice::<NoSize>() {
        let as_ptr = item as *const NoSize;

        assert!(as_ptr.is_aligned());
    }

    assert!(anyvec.pop::<NoSize>().is_some());
    assert!(anyvec.pop::<NoSize>().is_some());
    assert!(anyvec.pop::<NoSize>().is_some());
    assert!(anyvec.pop::<NoSize>().is_some());

    assert_eq!(0, anyvec.len());
}

#[test]
#[should_panic]
fn test_type_push_panic() {
    let mut anyvec = AnyVec::new::<u32>();

    anyvec.push::<u64>(0xdead);
}

#[test]
#[should_panic]
fn test_type_pop_panic() {
    let mut anyvec = AnyVec::new::<u32>();

    let _dead = anyvec.pop::<u64>();
}

#[test]
#[should_panic]
fn test_type_slice_panic() {
    let anyvec = AnyVec::new::<u32>();

    anyvec.as_slice::<u64>();
}

#[test]
#[should_panic]
fn test_type_mut_slice_panic() {
    let mut anyvec = AnyVec::new::<u32>();

    anyvec.as_mut_slice::<u64>();
}
