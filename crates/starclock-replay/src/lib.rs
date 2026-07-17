//! Canonical battle/activity codec and replay-verification boundary.
//!
//! Replay transport observes public domain commands, events and hashes without
//! owning combat or activity mutation.

#![forbid(unsafe_code)]
