//! Module for atomic float types, implemented through standard atomics

/// Generates a fake atomic floating point type, implemented using atomic operations on unsigned integers
/// of the same size
macro_rules! atomic_float {
    ($name:ident, $inner:ty, $float:ty) => {
        /// Fake-atomic float. Works by treating itself as an [$inner], and then casting
        /// the raw float value with [$float::to_bits]
        #[derive(Debug)]
        pub(crate) struct $name($inner);

        impl $name {
            #![allow(unused, reason = "auto-generated code")]

            #[inline(always)]
            const fn fail_ordering(
                ordering: core::sync::atomic::Ordering,
            ) -> core::sync::atomic::Ordering {
                use core::sync::atomic::Ordering;

                match ordering {
                    Ordering::Relaxed => Ordering::Relaxed,
                    Ordering::Release => Ordering::Relaxed,
                    Ordering::Acquire => Ordering::Acquire,
                    Ordering::AcqRel => Ordering::Acquire,
                    Ordering::SeqCst => Ordering::SeqCst,
                    _ => panic!("Unknown ordering"),
                }
            }

            /// Creates a new [$name]
            #[inline(always)]
            pub(crate) const fn new(x: $float) -> Self {
                $name(<$inner>::new(x.to_bits()))
            }

            /// Same as [$name::load]
            #[inline(always)]
            pub(crate) fn load(&self, ordering: core::sync::atomic::Ordering) -> $float {
                <$float>::from_bits(self.0.load(ordering))
            }

            /// Same as [$name::store]
            #[inline(always)]
            pub(crate) fn store(&self, x: $float, ordering: core::sync::atomic::Ordering) {
                self.0.store(x.to_bits(), ordering);
            }

            /// Mostly the same as [$inner::fetch_add], but implemented using
            /// [$inner::update]
            #[inline(always)]
            pub(crate) fn fetch_add(
                &self,
                x: $float,
                ordering: core::sync::atomic::Ordering,
            ) -> $float {
                let prev = self
                    .0
                    .fetch_update(ordering, Self::fail_ordering(ordering), |val| {
                        let val = <$float>::from_bits(val);
                        Some((val + x).to_bits())
                    })
                    .unwrap();

                <$float>::from_bits(prev)
            }
        }
    };
}

atomic_float!(AtomicF32, core::sync::atomic::AtomicU32, f32);
atomic_float!(AtomicF64, core::sync::atomic::AtomicU64, f64);
