//! Deterministic controller boundary for combat decisions.
//!
//! Controllers consume immutable views and offered commands; they never gain
//! mutable access to a battle aggregate.

#![forbid(unsafe_code)]
