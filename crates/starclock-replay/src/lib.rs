//! Canonical battle/activity codec and replay-verification boundary.
//!
//! Replay transport observes public domain commands, events and hashes without
//! owning combat or activity mutation.

#![forbid(unsafe_code)]

pub mod activity;
pub mod battle;
pub mod battle_event;
pub mod codec;
pub mod component;
pub mod digest;
pub mod format;
pub mod format_v2;
pub mod format_v3;
pub mod nested_battle;
pub mod record;
