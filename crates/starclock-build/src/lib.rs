//! Character progression and equipment compilation boundary.
//!
//! Build-domain catalogs and selections remain upstream of combat. Successful
//! compilation emits only the generic combat-domain input owned by
//! `starclock-combat`.

#![forbid(unsafe_code)]

pub mod ability;
pub mod catalog;
pub mod compiler;
pub mod digest;
pub mod eidolon;
pub mod id;
pub mod light_cone;
pub mod output;
pub mod patch;
pub mod preset;
pub mod relic_boundary;
pub mod report;
pub mod spec;
pub mod trace;
