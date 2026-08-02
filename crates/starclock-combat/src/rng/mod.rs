//! Deterministic RNG, stream derivation and integer-only mappings.

pub mod derive;
pub mod engine;
mod mapping;
pub mod types;

#[cfg(test)]
mod tests;
