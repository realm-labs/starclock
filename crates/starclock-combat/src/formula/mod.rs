//! Pure fixed-point combat calculators. These services never inspect or mutate battle state.

mod sustain;

pub(crate) use sustain::{healing, ordinary_damage};
