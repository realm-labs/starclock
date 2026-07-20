//! Character progression and equipment compilation boundary.
//!
//! Build-domain catalogs and selections remain upstream of combat. Successful
//! compilation emits only the generic combat-domain input owned by
//! `starclock-combat`.

#![forbid(unsafe_code)]

pub mod catalog;
pub mod compiler;
pub mod output;
pub mod report;
pub mod spec;
