use starclock_activity::{ActivityInstanceId, ActivityTerminalOutcome};
use starclock_replay::{
    codec::{CanonicalEncode, CanonicalSink, Encoder},
    component::ConfigurationComponentSet,
    digest::Sha256Sink,
    format_v2::{ReplayHeaderV2, decode_replay_v2, encode_replay_v2},
    record::{RecordKind, RecordRef},
};

use crate::swarm_disaster_components::swarm_disaster_component_set;

use super::{
    replay::{
        SwarmReplayDivergenceKind, encode_complete_swarm_replay_v2,
        verify_complete_swarm_replay_v2,
    },
    seeded_run::{
        SWARM_DISASTER_SEEDED_RUN_REVISION, SwarmSeededBoundary, SwarmSeededRunRequest,
    },
};

#[test]
fn component_replay_reexecutes_real_battles_and_reports_every_first_boundary() {
    let (instance, roster) = super::seeded_run_tests::representative_runtime();
    let request = request();
    let component_set = components(&instance, 0x44);
    assert_eq!(
        hex(component_set.root().bytes()),
        "01dce3ee71b2cf1e790d29b4ccc923e57055ea70208160c7fc1cc2940a0d0b22"
    );
    let bytes = encode_complete_swarm_replay_v2(
        &instance,
        request.seed,
        request.identity,
        request.activity_instance,
        &roster,
        component_set.clone(),
    )
    .unwrap();
    assert_eq!(
        encode_complete_swarm_replay_v2(
            &instance,
            request.seed,
            request.identity,
            request.activity_instance,
            &roster,
            component_set.clone(),
        )
        .unwrap(),
        bytes
    );

    let (fresh_instance, fresh_roster) = super::seeded_run_tests::representative_runtime();
    let verified = verify_complete_swarm_replay_v2(
        &bytes,
        &fresh_instance,
        request.seed,
        request.identity,
        request.activity_instance,
        &fresh_roster,
        &component_set,
    )
    .unwrap();
    assert_eq!(verified.terminal(), ActivityTerminalOutcome::Completed);
    assert!(verified.battle_command_count() > 0);
    assert_eq!(
        hex(verified.final_state_hash().bytes()),
        "059710ea6ac74f7ae919a5f066b17fed91e13b249621eaba30e876126a207c11"
    );

    let mut replay_digest = Sha256Sink::new();
    replay_digest.write(&bytes);
    let replay = decode_replay_v2(&bytes).unwrap();
    assert_eq!(verified.action_count(), 48);
    assert_eq!(verified.battle_count(), 12);
    assert_eq!(verified.battle_command_count(), 74);
    assert_eq!(bytes.len(), 88_813);
    assert_eq!(replay.records().len(), 268);
    assert_eq!(
        hex(replay_digest.finalize().bytes()),
        "c627e93fb58e350e7dd2cc0c3d2651ecc1140b705142a5a79628908fb755b259"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedActivityCommand]),
        "182e304fd896c6826a3728041205635ccb8c18777fe7d8cedda405028dc21c74"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedBattleCommand]),
        "46c7b023c09f585f23c5e06bf0229d690927226b10d2fa3ad1cf33cda0cc9127"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedBattleState]),
        "3eea401a501f1f70c30fbabaf655232544913861bee094c7439e45548559107b"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedActivityState]),
        "91eda49f0fab401bb3ecbfe31ab60dbd094f09a18e96acf16251db1e61da2290"
    );

    assert_divergence(
        &bytes,
        &fresh_instance,
        request,
        &fresh_roster,
        &components(&fresh_instance, 0x45),
        SwarmReplayDivergenceKind::Component,
    );
    let catalog = mutate_first(&bytes, RecordKind::AcceptedActivityCommand, |payload| {
        payload[7] ^= 1;
    });
    assert_divergence(
        &catalog,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::Catalog,
    );
    let assembly = mutate_first(&bytes, RecordKind::NestedBattleStart, |payload| {
        *payload.last_mut().unwrap() ^= 1;
    });
    assert_divergence(
        &assembly,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::Assembly,
    );
    let activity_command = mutate_first(
        &bytes,
        RecordKind::AcceptedActivityCommand,
        |payload| *payload.last_mut().unwrap() ^= 1,
    );
    assert_divergence(
        &activity_command,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::ActivityCommand,
    );
    let battle_command = mutate_first(&bytes, RecordKind::AcceptedBattleCommand, |payload| {
        *payload.last_mut().unwrap() ^= 1;
    });
    assert_divergence(
        &battle_command,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::BattleCommand,
    );
    let event = mutate_first_where(
        &bytes,
        RecordKind::ExpectedBattleState,
        |payload| payload.len() > 42,
        |payload| *payload.last_mut().unwrap() ^= 1,
    );
    assert_divergence(
        &event,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::Event,
    );
    let battle_state = mutate_first(&bytes, RecordKind::ExpectedBattleState, |payload| {
        payload[2] ^= 1;
    });
    assert_divergence(
        &battle_state,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::BattleState,
    );
    let battle_result = mutate_first(&bytes, RecordKind::NestedBattleEnd, |payload| {
        *payload.last_mut().unwrap() ^= 1;
    });
    assert_divergence(
        &battle_result,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::BattleResult,
    );
    let activity_state = mutate_first(&bytes, RecordKind::ExpectedActivityState, |payload| {
        payload[0] ^= 1;
    });
    assert_divergence(
        &activity_state,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
        SwarmReplayDivergenceKind::ActivityState,
    );
}

fn request() -> SwarmSeededRunRequest {
    SwarmSeededRunRequest {
        seed: 20_001,
        identity: super::battle_materialization_tests::activity_identity(),
        activity_instance: ActivityInstanceId::new(1).unwrap(),
        config_digest: starclock_activity::ActivityConfigDigest::new([0x6d; 32]).unwrap(),
        boundary: SwarmSeededBoundary::Baseline,
    }
}

fn components(
    instance: &super::SwarmDisasterRuntimeInstance,
    controller: u8,
) -> ConfigurationComponentSet {
    let combat = instance.battle_catalog.combat();
    swarm_disaster_component_set(
        super::tests::BUNDLE,
        (combat.revision().as_str(), combat.digest().bytes()),
        ("swarm-disaster-synthetic-build-v1", [0x33; 32]),
        super::battle_materialization_tests::activity_identity()
            .definition_digest()
            .bytes(),
        instance.battle_catalog.digest(),
        instance.graph_definition().digest().bytes(),
        (
            "swarm-disaster-seeded-controller",
            SWARM_DISASTER_SEEDED_RUN_REVISION,
            [controller; 32],
        ),
    )
    .unwrap()
}

fn assert_divergence(
    bytes: &[u8],
    instance: &super::SwarmDisasterRuntimeInstance,
    request: SwarmSeededRunRequest,
    roster: &crate::battle_materialization::UniverseBattleRoster,
    components: &ConfigurationComponentSet,
    expected: SwarmReplayDivergenceKind,
) {
    let error = verify_complete_swarm_replay_v2(
        bytes,
        instance,
        request.seed,
        request.identity,
        request.activity_instance,
        roster,
        components,
    )
    .expect_err("corrupted replay must fail");
    assert_eq!(error.first_divergence(), Some(expected), "{error:?}");
}

fn mutate_first(bytes: &[u8], kind: RecordKind, mutate: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    mutate_first_where(bytes, kind, |_| true, mutate)
}

fn mutate_first_where(
    bytes: &[u8],
    kind: RecordKind,
    predicate: impl Fn(&[u8]) -> bool,
    mutate: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    let replay = decode_replay_v2(bytes).unwrap();
    let mut payloads = replay
        .records()
        .iter()
        .map(|record| (record.kind(), record.payload().to_vec()))
        .collect::<Vec<_>>();
    let payload = payloads
        .iter_mut()
        .find(|(candidate, payload)| *candidate == kind && predicate(payload))
        .map(|(_, payload)| payload)
        .expect("selected replay record exists");
    mutate(payload);
    encode_payloads(replay.header().clone(), &payloads)
}

fn encode_payloads(header: ReplayHeaderV2, payloads: &[(RecordKind, Vec<u8>)]) -> Vec<u8> {
    let records = payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| RecordRef::new(*kind, index as u64, payload).unwrap())
        .collect::<Vec<_>>();
    encode_replay_v2(&header, &records, Vec::new()).unwrap()
}

fn record_digest(bytes: &[u8], kinds: &[RecordKind]) -> String {
    let replay = decode_replay_v2(bytes).unwrap();
    let mut encoder = Encoder::new(Sha256Sink::new());
    for record in replay
        .records()
        .iter()
        .filter(|record| kinds.contains(&record.kind()))
    {
        record.encode(&mut encoder).unwrap();
    }
    hex(encoder.into_inner().finalize().bytes())
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
