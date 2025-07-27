//! Module for atomic float types, implemented through standard atomics

use core::sync::atomic::{AtomicU32, Ordering};

/// Fake-atomic float. Works by treating itself as an [AtomicU32], and then casting
/// the raw float value with [f32::to_bits]
#[derive(Debug)]
pub(crate) struct AtomicF32(AtomicU32);

impl AtomicF32 {
    #[inline(always)]
    const fn fail_ordering(ordering: Ordering) -> Ordering {
        match ordering {
            Ordering::Relaxed => Ordering::Relaxed,
            Ordering::Release => Ordering::Relaxed,
            Ordering::Acquire => Ordering::Acquire,
            Ordering::AcqRel => Ordering::Acquire,
            Ordering::SeqCst => Ordering::SeqCst,
            _ => panic!("Unknown ordering"),
        }
    }

    /// Creates a new [AtomicF32]
    #[inline(always)]
    pub(crate) const fn new(x: f32) -> Self {
        AtomicF32(AtomicU32::new(x.to_bits()))
    }

    /// Same as [AtomicU32::load]
    #[inline(always)]
    pub(crate) fn load(&self, ordering: Ordering) -> f32 {
        f32::from_bits(self.0.load(ordering))
    }

    /// Same as [AtomicU32::store]
    #[inline(always)]
    pub(crate) fn store(&self, x: f32, ordering: Ordering) {
        self.0.store(x.to_bits(), ordering);
    }

    /// Mostly the same as [AtomicU32::fetch_add], but implemented using
    /// [AtomicU32::update]
    #[inline(always)]
    pub(crate) fn fetch_add(&self, x: f32, ordering: Ordering) -> f32 {
        let prev = self
            .0
            .fetch_update(ordering, Self::fail_ordering(ordering), |val| {
                let val = f32::from_bits(val);
                Some((val + x).to_bits())
            })
            .unwrap();

        f32::from_bits(prev)
    }
}
