//! Pure fixed-point combat calculators. These services never inspect or mutate battle state.

pub mod damage;
pub mod effect;
pub mod hp;
pub mod model;
pub mod shield;
pub mod sustain;
pub mod toughness;

pub(crate) use sustain::{healing, ordinary_damage};
