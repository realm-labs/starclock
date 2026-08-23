use std::{collections::BTreeMap, sync::Arc};

use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityTerminalOutcome,
};
use starclock_ai::{
    CurrencyWarsBaselineActivityAction, CurrencyWarsBaselineController,
    CurrencyWarsReplayDivergenceKind, CurrencyWarsReplayGambit, CurrencyWarsReplayIdentity,
    CurrencyWarsReplayRequest, encode_currency_wars_replay, verify_currency_wars_replay,
};
use starclock_data::{
    currency_wars::load_currency_wars_catalog_candidate, load_currency_wars_battle_resources,
};
use starclock_mode_currency_wars::{
    CurrencyWarsBattleAssembler, CurrencyWarsDeployment, CurrencyWarsEntryState,
    CurrencyWarsGambit, CurrencyWarsPosition, CurrencyWarsPositionKind, CurrencyWarsRoleId,
    CurrencyWarsRoleState, CurrencyWarsRoster, CurrencyWarsRun, CurrencyWarsRunDefinition,
    CurrencyWarsRunSetup,
};
use starclock_replay::{
    format::{decode_replay, encode_replay},
    record::{RecordKind, RecordRef},
};

#[test]
fn production_baseline_controller_completes_a_real_standard_run_deterministically() {
    let (mut left, mut left_assembler) = production_run(31, CurrencyWarsGambit::Standard);
    let left_report = CurrencyWarsBaselineController::default()
        .run_to_terminal(&mut left, &mut left_assembler)
        .expect("baseline controller completes the production run");
    assert_eq!(left_report.terminal(), ActivityTerminalOutcome::Completed);
    assert_eq!(left_report.battles().len(), 7);
    assert_eq!(
        left_report.activity_steps() as usize,
        left_report.activity_trace().len()
    );
    assert_eq!(left_report.activity_trace().len(), 14);
    assert!(
        left_report
            .activity_trace()
            .chunks_exact(2)
            .enumerate()
            .all(|(index, pair)| {
                pair[0].action() == CurrencyWarsBaselineActivityAction::EngageEncounter
                    && pair[0].battle_index().is_none()
                    && pair[1].action() == CurrencyWarsBaselineActivityAction::PrepareBattle
                    && pair[1].battle_index() == u32::try_from(index + 1).ok()
            })
    );
    assert!(left_report.battles().iter().all(|battle| {
        !battle.trace().is_empty()
            && battle
                .trace()
                .iter()
                .all(|entry| !entry.events().is_empty())
    }));

    let (mut right, mut right_assembler) = production_run(31, CurrencyWarsGambit::Standard);
    let right_report = CurrencyWarsBaselineController::default()
        .run_to_terminal(&mut right, &mut right_assembler)
        .expect("same seed completes the same production run");
    assert_eq!(left_report, right_report);
}

#[test]
fn currency_wars_replay_binds_nine_components_and_reports_first_divergence() {
    let (mut run, mut assembler) = production_run(33, CurrencyWarsGambit::Standard);
    let report = CurrencyWarsBaselineController::default()
        .run_to_terminal(&mut run, &mut assembler)
        .expect("baseline run completes");
    let request =
        CurrencyWarsReplayRequest::new(801, 1001, CurrencyWarsReplayGambit::Standard, 31_000_501);
    let identity = replay_identity();
    let bytes = encode_currency_wars_replay(request, identity, &report).expect("replay encodes");
    assert_eq!(
        decode_replay(&bytes)
            .expect("replay decodes")
            .header()
            .components()
            .components()
            .len(),
        9
    );
    verify_currency_wars_replay(&bytes, request, identity, &report).expect("fresh replay verifies");

    let changed_identity = replay_identity_with_configuration([9; 32]);
    let error = verify_currency_wars_replay(&bytes, request, changed_identity, &report)
        .expect_err("component drift is rejected");
    assert_eq!(
        replay_divergence_kind(error),
        CurrencyWarsReplayDivergenceKind::Catalog
    );

    for (record, expected) in [
        (
            RecordKind::AcceptedActivityCommand,
            CurrencyWarsReplayDivergenceKind::Activity,
        ),
        (
            RecordKind::NestedBattleStart,
            CurrencyWarsReplayDivergenceKind::BattleAssembly,
        ),
        (
            RecordKind::AcceptedBattleCommand,
            CurrencyWarsReplayDivergenceKind::BattleCommand,
        ),
        (
            RecordKind::NestedBattleEnd,
            CurrencyWarsReplayDivergenceKind::Settlement,
        ),
    ] {
        let corrupted = corrupt_record(&bytes, record);
        let error = verify_currency_wars_replay(&corrupted, request, identity, &report)
            .expect_err("record corruption is rejected");
        assert_eq!(replay_divergence_kind(error), expected);
    }
}

fn corrupt_record(bytes: &[u8], target: RecordKind) -> Vec<u8> {
    let decoded = decode_replay(bytes).expect("replay decodes");
    let mut payloads = decoded
        .records()
        .iter()
        .map(|record| record.payload().to_vec())
        .collect::<Vec<_>>();
    let index = decoded
        .records()
        .iter()
        .position(|record| record.kind() == target)
        .expect("target record exists");
    let byte = payloads[index]
        .last_mut()
        .expect("record payload is non-empty");
    *byte ^= 1;
    let records = decoded
        .records()
        .iter()
        .zip(&payloads)
        .map(|(record, payload)| RecordRef::new(record.kind(), record.sequence(), payload).unwrap())
        .collect::<Vec<_>>();
    encode_replay(decoded.header(), &records, Vec::new()).expect("corrupted replay reframes")
}

fn replay_divergence_kind(
    error: starclock_ai::CurrencyWarsReplayError,
) -> CurrencyWarsReplayDivergenceKind {
    match error {
        starclock_ai::CurrencyWarsReplayError::Diverged(divergence) => divergence.kind(),
        other => panic!("expected replay divergence, got {other:?}"),
    }
}

fn replay_identity() -> CurrencyWarsReplayIdentity {
    let candidate = load_currency_wars_catalog_candidate().expect("production catalog loads");
    let identity = candidate.identity().clone();
    let catalog = candidate.into_catalog();
    let resources =
        load_currency_wars_battle_resources(&catalog).expect("production battle resources load");
    CurrencyWarsReplayIdentity::new(
        identity.schema_digest().bytes(),
        identity.configuration_digest().bytes(),
        identity.content_digest().bytes(),
        resources.digest(),
        resources.combat().digest().bytes(),
    )
}

fn replay_identity_with_configuration(
    configuration_digest: [u8; 32],
) -> CurrencyWarsReplayIdentity {
    let candidate = load_currency_wars_catalog_candidate().expect("production catalog loads");
    let identity = candidate.identity().clone();
    let catalog = candidate.into_catalog();
    let resources =
        load_currency_wars_battle_resources(&catalog).expect("production battle resources load");
    CurrencyWarsReplayIdentity::new(
        identity.schema_digest().bytes(),
        configuration_digest,
        identity.content_digest().bytes(),
        resources.digest(),
        resources.combat().digest().bytes(),
    )
}

#[test]
fn production_baseline_controller_completes_a_real_overclock_run() {
    let (mut run, mut assembler) = production_run(32, CurrencyWarsGambit::Overclock);

    let report = CurrencyWarsBaselineController::default()
        .run_to_terminal(&mut run, &mut assembler)
        .expect("baseline controller completes the production Overclock run");

    assert_eq!(report.terminal(), ActivityTerminalOutcome::Completed);
    assert_eq!(report.battles().len(), 7);
    assert!(
        report
            .battles()
            .iter()
            .all(|battle| !battle.trace().is_empty())
    );
}

fn production_run(
    instance: u64,
    gambit: CurrencyWarsGambit,
) -> (CurrencyWarsRun, CurrencyWarsBattleAssembler) {
    let candidate = load_currency_wars_catalog_candidate().expect("production catalog loads");
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(31).expect("non-zero definition ID"),
        ActivityDefinitionDigest::new(candidate.identity().content_digest().bytes())
            .expect("non-zero content digest"),
        ActivityConfigDigest::new(candidate.identity().configuration_digest().bytes())
            .expect("non-zero configuration digest"),
    );
    let catalog = Arc::new(candidate.into_catalog());
    let route = catalog
        .routes()
        .iter()
        .find(|route| route.stable_key.as_ref() == "currency-wars.area.route.801")
        .expect("short released route exists");
    let difficulty = catalog
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.division_level == 1)
        .expect("initial released difficulty exists");
    let roles = [1301, 1306, 1014, 1015].map(|raw| {
        CurrencyWarsRoleState::new(CurrencyWarsRoleId::new(raw).expect("non-zero role ID"), 1)
            .expect("released one-star role state")
    });
    let roster = CurrencyWarsRoster::new(&catalog, roles.into_iter().map(|role| (role, 1)))
        .expect("released roster is valid");
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        4,
        (1_u8..=4).map(|index| {
            (
                CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, index)
                    .expect("front position is valid"),
                roles[usize::from(index - 1)],
            )
        }),
    )
    .expect("four-role front deployment is valid");
    let definition = Arc::new(
        CurrencyWarsRunDefinition::new(
            identity,
            Arc::clone(&catalog),
            route.id,
            difficulty.source_id,
            gambit,
            CurrencyWarsEntryState::new(21, true, 9),
            CurrencyWarsRunSetup {
                initial_gold: 0,
                initial_team_level: 4,
                initial_experience: 0,
                roster,
                deployment,
                enemy_affix_ids: Box::new([]),
                owned_builds: BTreeMap::new(),
            },
        )
        .expect("production run definition compiles"),
    );
    let run = CurrencyWarsRun::start(
        definition,
        ActivityInstanceId::new(instance).expect("non-zero instance ID"),
        ActivityMasterSeed::from_u64(31_000_501),
    )
    .expect("production run starts");
    let resources = Arc::new(
        load_currency_wars_battle_resources(&catalog).expect("production battle resources load"),
    );
    let assembler =
        CurrencyWarsBattleAssembler::new(resources, 16).expect("bounded assembler is valid");
    (run, assembler)
}
