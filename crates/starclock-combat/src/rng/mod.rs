//! Versioned deterministic RNG, stream derivation and integer-only mappings.

pub mod derive;
pub mod engine;
mod mapping;
pub mod types;

/// Replay-sensitive generator and integer-mapping compatibility revision.
pub const RNG_ALGORITHM_REVISION: &str = "chacha8-rand-0.10.2-intmap-v1";

#[cfg(test)]
mod tests;
