use crate::battle_materialization::UniverseBattleRoster;
use starclock_activity::{ActivityInstanceId, ActivityTerminalOutcome};
use starclock_replay::{
    codec::{CanonicalEncode, CanonicalSink, Encoder},
    component::ConfigurationComponentSet,
    digest::Sha256Sink,
    format::{ReplayHeader, decode_replay, encode_replay},
    record::{RecordKind, RecordRef},
};

use crate::swarm_disaster_components::swarm_disaster_component_set;

use super::{
    replay::{
        SwarmReplayDivergenceKind, encode_complete_swarm_replay,
        verify_complete_swarm_replay,
    },
    seeded_run::{SwarmSeededBoundary, SwarmSeededRunRequest},
};

#[test]
fn component_replay_reexecutes_real_battles_and_reports_every_first_boundary() {
    let (instance, roster) = super::seeded_run_tests::representative_runtime();
    let request = request();
    let component_set = components(&instance, 0x44);
    assert_eq!(
        hex(component_set.root().bytes()),
        "d92b016d97686e7a8286aa9ce7924bedf064e31692ea7e4ac37ed720b6656a54"
    );
    let bytes = encode_complete_swarm_replay(
        &instance,
        request.seed,
        request.identity,
        request.activity_instance,
        &roster,
        component_set.clone(),
    )
    .unwrap();
    assert_eq!(
        encode_complete_swarm_replay(
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
    let verified = verify_complete_swarm_replay(
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
        "caabb6cdfdcc827b592ce1fe3576600d84e06c0a68adbafb1bf663bd83b820db"
    );

    let mut replay_digest = Sha256Sink::new();
    replay_digest.write(&bytes);
    let replay_digest = hex(replay_digest.finalize().bytes());
    let replay = decode_replay(&bytes).unwrap();
    assert_eq!(verified.action_count(), 48);
    assert_eq!(verified.battle_count(), 12);
    assert_eq!(verified.battle_command_count(), 72);
    assert_eq!(bytes.len(), 76_215);
    assert_eq!(replay.records().len(), 264);
    assert_eq!(
        replay_digest,
        "a909811127e983e2570fec815773b4d316d7d979ab785e7c36b8867e54326a9b"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedActivityCommand]),
        "c15a034dfc8209dfd88c964ec405b2ab417cd1469ccf7ffedc468df98c3e196e"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedBattleCommand]),
        "43af8a15aea84a3cb5ad504fc5ec65bc554645e43b4bf4b560f2bb8ac27d3a86"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedBattleState]),
        "57e25a03413a24122e930fa4cb3141f2fa0bdb8606cf5e1c4105e389c240c58d"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedActivityState]),
        "2a092c50d7cb0cedb44d3ca4e8d0d136785717413becaff13ff818174e448a85"
    );

    assert_divergence(
        &bytes,
        &fresh_instance,
        request,
        &fresh_roster,
        &components(&fresh_instance, 0x45),
        SwarmReplayDivergenceKind::Component,
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
        combat.digest().bytes(),
        [0x33; 32],
        super::battle_materialization_tests::activity_identity()
            .definition_digest()
            .bytes(),
        instance.battle_catalog.digest(),
        instance.graph_definition().digest().bytes(),
        ("swarm-disaster-seeded-controller", [controller; 32]),
    )
    .unwrap()
}

fn assert_divergence(
    bytes: &[u8],
    instance: &super::SwarmDisasterRuntimeInstance,
    request: SwarmSeededRunRequest,
    roster: &UniverseBattleRoster,
    components: &ConfigurationComponentSet,
    expected: SwarmReplayDivergenceKind,
) {
    let error = verify_complete_swarm_replay(
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
    let replay = decode_replay(bytes).unwrap();
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

fn encode_payloads(header: ReplayHeader, payloads: &[(RecordKind, Vec<u8>)]) -> Vec<u8> {
    let records = payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| RecordRef::new(*kind, index as u64, payload).unwrap())
        .collect::<Vec<_>>();
    encode_replay(&header, &records, Vec::new()).unwrap()
}

fn record_digest(bytes: &[u8], kinds: &[RecordKind]) -> String {
    let replay = decode_replay(bytes).unwrap();
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
