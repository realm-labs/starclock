//! Canonical battle/activity codec and replay-verification boundary.
//!
//! Replay transport observes public domain commands, events and hashes without
//! owning combat or activity mutation.

#![forbid(unsafe_code)]

pub mod activity;
pub mod battle;
pub mod battle_event;
mod battle_event_cause;
pub mod codec;
pub mod component;
pub mod digest;
pub mod entry;
pub mod envelope;
pub mod format;
pub mod nested_battle;
pub mod record;
