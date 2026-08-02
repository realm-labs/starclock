//! Canonical digest encoding for deterministic Swarm complete runs.

use starclock_activity::{ActivityStateHash, ActivityTerminalOutcome, BattleResultDigest};

use crate::digest::Encoder;

use super::{
    encounter_runtime::EncounterRole,
    seeded_run::{SWARM_DISASTER_SEEDED_RUN_REVISION, SwarmSeededRunStep, SwarmSeededStepKind},
};

pub(super) fn transcript_digest(
    seed: u64,
    terminal: ActivityTerminalOutcome,
    final_state_hash: ActivityStateHash,
    battle_count: u32,
    maximum_disarray_level: i64,
    cross_plane_countdown_carried: bool,
    steps: &[SwarmSeededRunStep],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.seeded-run.v1");
    encoder.text(SWARM_DISASTER_SEEDED_RUN_REVISION);
    encoder.u64(seed);
    encoder.u8(terminal_code(terminal));
    encoder.digest(final_state_hash.bytes());
    encoder.u32(battle_count);
    encoder.i64(maximum_disarray_level);
    encoder.bool(cross_plane_countdown_carried);
    encoder.u32(u32::try_from(steps.len()).expect("seeded steps are bounded"));
    for step in steps {
        encode_step_kind(&mut encoder, step.kind);
        encoder.u32(step.source_node.get());
        encoder.digest(step.state_hash.bytes());
        encoder.optional_digest(step.result_digest.map(BattleResultDigest::bytes));
    }
    encoder.finish()
}

fn encode_step_kind(encoder: &mut Encoder, kind: SwarmSeededStepKind) {
    match kind {
        SwarmSeededStepKind::ProfileEntry => encoder.u8(0),
        SwarmSeededStepKind::AudienceInitialization => encoder.u8(1),
        SwarmSeededStepKind::TrailRunStart => encoder.u8(2),
        SwarmSeededStepKind::CountdownSetup => encoder.u8(3),
        SwarmSeededStepKind::PlaneCreation(plane) => {
            encoder.u8(4);
            encoder.u8(plane);
        }
        SwarmSeededStepKind::DiceRoll => encoder.u8(5),
        SwarmSeededStepKind::Traverse => encoder.u8(6),
        SwarmSeededStepKind::BossSelection(plane) => {
            encoder.u8(7);
            encoder.u8(plane);
        }
        SwarmSeededStepKind::Battle(role) => {
            encoder.u8(8);
            encoder.u8(role_code(role));
        }
    }
}

const fn role_code(role: EncounterRole) -> u8 {
    match role {
        EncounterRole::Combat => 0,
        EncounterRole::Elite => 1,
        EncounterRole::FirstPlaneBoss => 2,
        EncounterRole::SecondPlaneBoss => 3,
        EncounterRole::FinalBoss => 4,
    }
}

const fn terminal_code(terminal: ActivityTerminalOutcome) -> u8 {
    match terminal {
        ActivityTerminalOutcome::Completed => 0,
        ActivityTerminalOutcome::Failed => 1,
        ActivityTerminalOutcome::Abandoned => 2,
        ActivityTerminalOutcome::Faulted => 3,
    }
}
