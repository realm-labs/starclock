//! Checked six-decimal authoritative numeric domain.
//!
//! Only this module names the pinned fixed-point backend. Public construction,
//! arithmetic and inspection use fixed-width integers and explicit rounding.

pub mod domain;
pub mod rounding;
pub mod scalar;

#[cfg(test)]
mod tests;
