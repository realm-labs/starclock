//! Static registry boundary for exceptional battle and activity rule handlers.
//!
//! Registered handlers emit ordinary validated domain operations. They do not
//! mutate authoritative state directly or introduce a dynamic plugin ABI.

#![forbid(unsafe_code)]
