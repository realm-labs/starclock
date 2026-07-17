//! Generic cross-battle activity orchestration boundary.
//!
//! Activities own flow, scoped state, participant locks and declared battle
//! result projections while treating resolved battle input as opaque handoff
//! data.

#![forbid(unsafe_code)]
