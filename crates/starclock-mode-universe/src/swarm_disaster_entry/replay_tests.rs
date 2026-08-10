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
use super::{seeded_run_tests, battle_materialization_tests, SwarmDisasterRuntimeInstance, tests};

#[test]
fn component_replay_reexecutes_real_battles_and_reports_every_first_boundary() {
    let (instance, roster) = seeded_run_tests::representative_runtime();
    let request = request();
    let component_set = components(&instance, 0x44);
    assert_eq!(
        hex(component_set.root().bytes()),
        "f84d04bbda60990513518141ba51859d2c50c934541124720aab0f4627ec7c84"
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

    let (fresh_instance, fresh_roster) = seeded_run_tests::representative_runtime();
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
        "f378d5a204374c531128db37c835069dcdc373b7839850978e24313f935be065"
    );

    let mut replay_digest = Sha256Sink::new();
    replay_digest.write(&bytes);
    let replay_digest = hex(replay_digest.finalize().bytes());
    let replay = decode_replay(&bytes).unwrap();
    assert_eq!(verified.action_count(), 48);
    assert_eq!(verified.battle_count(), 12);
    assert_eq!(verified.battle_command_count(), 60);
    assert_eq!(bytes.len(), 74_727);
    assert_eq!(replay.records().len(), 240);
    assert_eq!(
        replay_digest,
        "12aa4bf49ca80dfe46294fab1fec1ac0c49be24aa0f21b5a6763b4b70ece94d2"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedActivityCommand]),
        "f0e0cc1810eb6dfb21859d90faa3e69d3855c4241144d58aece306955736bbe7"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedBattleCommand]),
        "3dcedce8bbe994fe724ffbd28eb31845420506d97413f4a03f88b97bd970880a"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedBattleState]),
        "9a1b21ef9c5fb210d09c89ee79cabc19ad936269fe9580fe998623cb06ea408c"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedActivityState]),
        "eb08fe0f099b8b6520dfa2cf96165037fea80780321d84139afb90621c6bf1c5"
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
        identity: battle_materialization_tests::activity_identity(),
        activity_instance: ActivityInstanceId::new(1).unwrap(),
        config_digest: starclock_activity::ActivityConfigDigest::new([0x6d; 32]).unwrap(),
        boundary: SwarmSeededBoundary::Baseline,
    }
}

fn components(
    instance: &SwarmDisasterRuntimeInstance,
    controller: u8,
) -> ConfigurationComponentSet {
    let combat = instance.battle_catalog.combat();
    swarm_disaster_component_set(
        tests::BUNDLE,
        combat.digest().bytes(),
        [0x33; 32],
        battle_materialization_tests::activity_identity()
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
    instance: &SwarmDisasterRuntimeInstance,
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
