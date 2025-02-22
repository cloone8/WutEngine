//! Random number generation
//!
//! For simple usecases, see [self::simple]. Otherwise, for the full re-export of the [rand] crate, see [self::rand]

pub use rand;

pub mod simple {
    //! Simple random number generation functions
    //!
    //! For advanced random number generation, use the re-export of the [rand] crate through [super::rand].

    use rand::prelude::SmallRng;
    use rand::{Rng, SeedableRng};

    /// Returns either -1.0 or 1.0 randomly
    pub fn sign() -> f32 {
        let mut rng = SmallRng::from_os_rng();

        if rng.random_bool(0.5) { 1.0 } else { -1.0 }
    }
}
