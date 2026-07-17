//! Versioned static registry for exceptional battle and activity rule handlers.
//!
//! Registrations are ordinary immutable Rust values. Battle handlers receive a
//! read-only Rule IR context and return the same typed emissions as authored IR;
//! they never receive mutable battle state or resolver internals.

#![forbid(unsafe_code)]

pub mod model;
pub mod registry;
