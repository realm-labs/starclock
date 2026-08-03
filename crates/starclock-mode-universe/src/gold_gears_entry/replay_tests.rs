use crate::battle_materialization::UniverseBattleRoster;
use starclock_activity::{ActivityInstanceId, ActivityTerminalOutcome};
use starclock_replay::{
    codec::{CanonicalEncode, CanonicalSink, Encoder},
    component::ConfigurationComponentSet,
    digest::Sha256Sink,
    format::{ReplayHeader, decode_replay, encode_replay},
    record::{RecordKind, RecordRef},
};

use crate::{
    gold_gears_components::gold_and_gears_component_set,
    gold_gears_identity::GoldAndGearsCatalogIdentity,
};

use super::{
    GoldAndGearsReplayDivergenceKind, GoldAndGearsRuntimeFactory,
    GoldAndGearsSeededRunRequest, encode_gold_and_gears_replay, gold_and_gears_replay_header,
    record_gold_and_gears_run, verify_gold_and_gears_replay,
    battle_materialization_tests::{activity_identity, seeded_matrix_roster},
};

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

#[test]
fn component_replay_reexecutes_real_battles_and_reports_every_first_boundary() {
    let primary = super::tests::shared_factory();
    let instance = replay_instance(primary);
    let roster = seeded_matrix_roster(&instance);
    let request = GoldAndGearsSeededRunRequest::new(
        14_901,
        activity_identity(),
        ActivityInstanceId::new(1).unwrap(),
    );
    let component_set = components(&instance, 0x44);
    assert_eq!(
        hex(component_set.root().bytes()),
        "ff277ff1a78467a2dd9188fff67739f22d1fb0df99cc044459ec200b2c965724"
    );
    let recorded = record_gold_and_gears_run(&instance, request, &roster).unwrap();
    let header = gold_and_gears_replay_header(component_set.clone(), request, &roster).unwrap();
    let bytes = encode_gold_and_gears_replay(&header, &recorded).unwrap();
    assert_eq!(encode_gold_and_gears_replay(&header, &recorded).unwrap(), bytes);

    let fresh = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let fresh_instance = replay_instance(&fresh);
    let fresh_roster = seeded_matrix_roster(&fresh_instance);
    let verified = verify_gold_and_gears_replay(
        &bytes,
        &fresh_instance,
        request,
        &fresh_roster,
        &component_set,
    )
    .unwrap();
    assert_eq!(verified.terminal(), ActivityTerminalOutcome::Completed);
    assert_eq!(verified.battle_count(), recorded.report().battle_count());
    assert_eq!(verified.action_count() as usize, recorded.action_count());
    assert!(verified.battle_command_count() > 0);
    assert_eq!(
        verified.final_state_hash().bytes(),
        recorded.report().final_state_hash().bytes()
    );
    let mut replay_digest = Sha256Sink::new();
    replay_digest.write(&bytes);
    let replay_digest = hex(replay_digest.finalize().bytes());
    assert_eq!(verified.action_count(), 62);
    assert_eq!(verified.battle_count(), 17);
    assert_eq!(verified.battle_command_count(), 93);
    assert_eq!(bytes.len(), 95_551);
    let replay = decode_replay(&bytes).unwrap();
    assert_eq!(
        replay_digest,
        "f97494350366e8da2378bee8dabe8ec698f6dfeef7d02770c6feb7714fd996d0"
    );
    assert_eq!(replay.records().len(), 344);
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedActivityCommand]),
        "56289e5507856fd9138112f0911382056dfc076648410967157a095e92bb19db"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::AcceptedBattleCommand]),
        "5d65745b52d2bad3ad1d3dfb9d4fe3748a0ecf426dc4f71e6039ad949fc79948"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedBattleState]),
        "a917ca9e0277ba895e73983dc3be07f38c6cb35c3d8288c801771ab584fe6e21"
    );
    assert_eq!(
        record_digest(&bytes, &[RecordKind::ExpectedActivityState]),
        "60430a58f16f061b5bf20676ac019e6f3efe74cafd97cb2cb5731db75ae2a2a1"
    );

    assert_divergence(
        &bytes,
        &fresh_instance,
        request,
        &fresh_roster,
        &components(&fresh_instance, 0x45),
        GoldAndGearsReplayDivergenceKind::Component,
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
        GoldAndGearsReplayDivergenceKind::Assembly,
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
        GoldAndGearsReplayDivergenceKind::ActivityCommand,
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
        GoldAndGearsReplayDivergenceKind::BattleCommand,
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
        GoldAndGearsReplayDivergenceKind::Event,
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
        GoldAndGearsReplayDivergenceKind::BattleState,
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
        GoldAndGearsReplayDivergenceKind::BattleResult,
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
        GoldAndGearsReplayDivergenceKind::ActivityState,
    );
}

fn replay_instance(factory: &GoldAndGearsRuntimeFactory) -> super::GoldAndGearsRuntimeInstance {
    let dice = &factory.unique.dice[0];
    factory
        .compile_entry(super::tests::battle_entry(
            factory,
            "gold-gears.area.401",
            "universe.path.abundance",
            dice,
        ))
        .unwrap()
}

fn components(
    instance: &super::GoldAndGearsRuntimeInstance,
    controller: u8,
) -> ConfigurationComponentSet {
    let identity = GoldAndGearsCatalogIdentity::load(BUNDLE).unwrap();
    let combat = instance.battle_catalog.combat();
    gold_and_gears_component_set(
        &identity,
        combat.digest().bytes(),
        [0x33; 32],
        activity_identity().definition_digest().bytes(),
        instance.battle_catalog.digest(),
        instance.graph_definition().digest().bytes(),
        (
            "gold-and-gears-seeded-controller",
            [controller; 32],
        ),
    )
    .unwrap()
}

fn assert_divergence(
    bytes: &[u8],
    instance: &super::GoldAndGearsRuntimeInstance,
    request: GoldAndGearsSeededRunRequest,
    roster: &UniverseBattleRoster,
    components: &ConfigurationComponentSet,
    expected: GoldAndGearsReplayDivergenceKind,
) {
    let error = verify_gold_and_gears_replay(bytes, instance, request, roster, components)
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
