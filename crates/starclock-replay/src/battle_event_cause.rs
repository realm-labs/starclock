//! Versioned cause-chain encoding for battle-event payloads.

use starclock_combat::{Cause, CauseActor};

use crate::{
    battle_event::BATTLE_EVENT_PAYLOAD_VERSION_V1,
    codec::{CanonicalSink, Encoder},
};

pub(crate) fn encode_cause<S: CanonicalSink>(encoder: &mut Encoder<S>, cause: Cause, version: u16) {
    optional_u64(encoder, cause.parent_event().map(|value| value.get()));
    encoder.u64(cause.root_command().get());
    optional_u64(encoder, cause.action().map(|value| value.get()));
    optional_u64(encoder, cause.phase().map(|value| value.get()));
    optional_u64(encoder, cause.hit().map(|value| value.get()));
    optional_u64(encoder, cause.owner().map(|value| value.get()));
    match cause.actor() {
        None => encoder.u8(0),
        Some(CauseActor::Unit(value)) => {
            encoder.u8(1);
            encoder.u64(value.get());
        }
        Some(CauseActor::TimelineActor(value)) => {
            encoder.u8(2);
            encoder.u64(value.get());
        }
    }
    optional_u64(encoder, cause.applier().map(|value| value.get()));
    optional_u32(encoder, cause.source_definition().map(|value| value.get()));
    optional_u64(encoder, cause.primary_target().map(|value| value.get()));
    if version == BATTLE_EVENT_PAYLOAD_VERSION_V1 {
        // Released v1 reserved this unwritten field. Preserve its exact zero
        // option byte for historical replay verification.
        optional_u32(encoder, None);
    }
}

fn optional_u32<S: CanonicalSink>(encoder: &mut Encoder<S>, value: Option<u32>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        encoder.u32(value);
    }
}

fn optional_u64<S: CanonicalSink>(encoder: &mut Encoder<S>, value: Option<u64>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        encoder.u64(value);
    }
}
