//! Deterministic, engine-agnostic ownership boundary for exactly one battle.
//!
//! This crate accepts only generic resolved combat input. Build selections,
//! generated data rows, activities, modes, controllers, replay transport and
//! engines remain outside this dependency root.

#![forbid(unsafe_code)]
