use crate::{digest::Encoder, id::EncounterMemberId};

use super::{CompiledOccurrenceProgram, OCCURRENCE_INTERACTION_RUNTIME_REVISION};

pub(super) fn runtime_catalog(programs: &[CompiledOccurrenceProgram]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-occurrence-interaction-runtime-v1");
    encoder.text(OCCURRENCE_INTERACTION_RUNTIME_REVISION);
    encoder.u32(programs.len() as u32);
    for program in programs {
        encoder.u32(program.choice.get());
        encoder.u32(program.battle_member.map_or(0, EncounterMemberId::get));
        encoder.u32(program.payload.len() as u32);
        for byte in &program.payload {
            encoder.u8(*byte);
        }
        encoder.u32(program.random_candidate_count.unwrap_or(0));
        encoder.u32(u32::from(program.immediate_operations));
        encoder.u32(u32::from(program.deferred_operations));
        encoder.u32(program.external_results.len() as u32);
        for result in &program.external_results {
            encoder.u64(result.content);
            encoder.u32(result.payload.len() as u32);
            for byte in &result.payload {
                encoder.u8(*byte);
            }
            encoder.u32(u32::from(result.immediate_operations));
            encoder.u32(u32::from(result.deferred_operations));
        }
    }
    encoder.finish()
}
