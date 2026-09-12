use super::{
    DivergentUniverseBaselineError, DivergentUniverseBaselineFixture,
    DivergentUniverseBaselinePolicy, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    DivergentUniverseReplayDivergenceKind,
    encode_divergent_universe_replay, record_divergent_universe_run,
    verify_divergent_universe_replay,
};
use starclock_replay::{
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::ComponentDigest,
    format::{ReplayHeader, decode_replay, encode_replay},
    record::{RecordKind, RecordRef},
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct BaselineReceipt {
    family: DivergentUniverseRunFamily,
    terminal: ActivityTerminalOutcome,
    final_state: ActivityStateHash,
    battle_result: starclock_activity::BattleResultDigest,
    battle_final_state: starclock_combat::BattleStateHash,
    activity_steps: usize,
    battle_commands: usize,
}

#[test]
fn baseline_controller_completes_replay_equal_ordinary_and_cyclical_runs_through_real_battles() {
    for (area, family) in [
        ("401", DivergentUniverseRunFamily::Ordinary),
        ("20401", DivergentUniverseRunFamily::Cyclical),
    ] {
        let first = execute_baseline(area, 0x7e26);
        let replay = execute_baseline(area, 0x7e26);
        assert_eq!(first, replay);
        assert_eq!(first.family, family);
        assert_eq!(first.terminal, ActivityTerminalOutcome::Completed);
        assert!(first.activity_steps >= 3);
        assert!(first.battle_commands > 0);
    }
}

#[test]
fn baseline_controller_records_only_scored_offers_and_checked_battle_commands() {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = baseline_flow(&factory, "401", participants, mapping);
    let mut activity = flow
        .start(instance(0x7e27), ActivityMasterSeed::from_u64(91))
        .expect("baseline activity")
        .into_activity();
    let report = DivergentUniverseBaselineRunner::default()
        .run_to_terminal(
            &factory,
            &flow,
            &mut activity,
            &core,
            &baseline_policy(32),
        )
        .expect("complete baseline run");
    for step in report.steps() {
        match step {
            DivergentUniverseBaselineStep::ActivityDecision { decision, .. }
            | DivergentUniverseBaselineStep::Battle { decision, .. } => {
                assert!(decision
                    .scores()
                    .iter()
                    .any(|score| score.option() == decision.option()));
            }
        }
        if let DivergentUniverseBaselineStep::Battle { execution, .. } = step {
            assert!(!execution.trace().is_empty());
            assert!(execution.terminal_fault().is_none());
        }
    }
}

#[test]
fn baseline_route_and_mismatched_activity_reject_without_mutation() {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let no_mapping = DivergentUniverseEntry::new(
        area_id("401"),
        difficulty_id("3011"),
        Arc::clone(&participants),
        input_snapshot(&participants),
        Vec::new(),
    )
    .expect("entry")
    .with_runtime_battle_route();
    assert!(matches!(
        factory.compile(no_mapping),
        Err(DivergentUniverseEntryFlowError::BattleRouteRequiresMapping)
    ));

    let ordinary = baseline_flow(
        &factory,
        "401",
        Arc::clone(&participants),
        Arc::clone(&mapping),
    );
    let cyclical = baseline_flow(&factory, "20401", participants, mapping);
    let mut activity = ordinary
        .start(instance(0x7e28), ActivityMasterSeed::from_u64(92))
        .expect("ordinary activity")
        .into_activity();
    let before = activity.canonical_state_bytes();
    assert!(matches!(
        DivergentUniverseBaselineRunner::default().run_to_terminal(
            &factory,
            &cyclical,
            &mut activity,
            &core,
            &baseline_policy(32),
        ),
        Err(DivergentUniverseBaselineError::DefinitionMismatch)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn production_baseline_fixture_runs_both_families_and_binds_nine_components() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    for (ordinal, family) in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ]
    .into_iter()
    .enumerate()
    {
        let flow = fixture.flow(family).expect("fixture flow");
        assert_eq!(
            fixture
                .components(&flow)
                .expect("component set")
                .components()
                .len(),
            9
        );
        let mut activity = flow
            .start(
                instance(u64::try_from(ordinal + 1).expect("fixture ordinal") + 0x7e29),
                ActivityMasterSeed::from_u64(93),
            )
            .expect("fixture activity")
            .into_activity();
        let report = DivergentUniverseBaselineRunner::default()
            .run_to_terminal(
                fixture.factory(),
                &flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().expect("fixture policy"),
            )
            .expect("fixture complete run");
        assert_eq!(report.terminal(), ActivityTerminalOutcome::Completed);
        assert_eq!(report.completed_battles(), flow.layers().len() as u32);
    }
}

#[test]
fn baseline_replay_round_trips_both_families_and_rejects_corruption() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let recorded = record_divergent_universe_run(&fixture, family, 94)
            .expect("record complete run");
        let bytes = encode_divergent_universe_replay(&recorded).expect("encode replay");
        assert_eq!(
            bytes,
            encode_divergent_universe_replay(
                &record_divergent_universe_run(&fixture, family, 94)
                    .expect("record deterministic replay")
            )
            .expect("encode deterministic replay")
        );
        let verified = verify_divergent_universe_replay(&bytes, &fixture)
            .expect("fresh replay verification");
        assert_eq!(verified.run_family(), family);
        assert_eq!(verified.terminal(), ActivityTerminalOutcome::Completed);
        assert_eq!(verified.battle_count(), 3);
        assert!(verified.battle_command_count() > 0);

        let mut corrupt = bytes;
        let last = corrupt.last_mut().expect("non-empty replay");
        *last ^= 1;
        assert!(verify_divergent_universe_replay(&corrupt, &fixture).is_err());
    }
}

#[test]
fn baseline_replay_reports_component_addressed_first_divergence() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let recorded = record_divergent_universe_run(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        95,
    )
    .expect("record complete run");
    let bytes = encode_divergent_universe_replay(&recorded).expect("encode replay");

    for (component_kind, divergence_kind) in [
        (
            ConfigurationComponentKind::CombatCatalog,
            DivergentUniverseReplayDivergenceKind::Catalog,
        ),
        (
            ConfigurationComponentKind::ActivityCore,
            DivergentUniverseReplayDivergenceKind::Activity,
        ),
        (
            ConfigurationComponentKind::ModeProfile,
            DivergentUniverseReplayDivergenceKind::Mapping,
        ),
        (
            ConfigurationComponentKind::EncounterOverlay,
            DivergentUniverseReplayDivergenceKind::BattleAssembly,
        ),
    ] {
        let corrupted = mutate_divergent_component(&bytes, component_kind);
        let error = verify_divergent_universe_replay(&corrupted, &fixture)
            .expect_err("component divergence must fail");
        assert_eq!(error.first_divergence(), Some(divergence_kind));
        assert_eq!(error.record_index(), Some(0));
        let component = error
            .component_divergence()
            .expect("component address is retained");
        assert_eq!(
            component
                .expected
                .as_ref()
                .expect("expected component")
                .kind(),
            component_kind
        );
    }
}

#[test]
fn baseline_replay_reports_every_runtime_boundary_at_the_first_record() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let recorded = record_divergent_universe_run(
        &fixture,
        DivergentUniverseRunFamily::Cyclical,
        96,
    )
    .expect("record complete run");
    let bytes = encode_divergent_universe_replay(&recorded).expect("encode replay");

    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::AcceptedActivityCommand,
        DivergentUniverseReplayDivergenceKind::ActivityCommand,
        |payload| *payload.last_mut().expect("activity command payload") ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::NestedBattleStart,
        DivergentUniverseReplayDivergenceKind::Mapping,
        |payload| payload[88] ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::NestedBattleStart,
        DivergentUniverseReplayDivergenceKind::ContributionSnapshot,
        |payload| payload[120] ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::NestedBattleStart,
        DivergentUniverseReplayDivergenceKind::BattleAssembly,
        |payload| payload[152] ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::AcceptedBattleCommand,
        DivergentUniverseReplayDivergenceKind::BattleCommand,
        |payload| *payload.last_mut().expect("battle command payload") ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::ExpectedBattleState,
        DivergentUniverseReplayDivergenceKind::BattleState,
        |payload| payload[0] ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::ExpectedBattleState,
        DivergentUniverseReplayDivergenceKind::BattleEvent,
        |payload| *payload.last_mut().expect("battle event payload") ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::NestedBattleEnd,
        DivergentUniverseReplayDivergenceKind::Settlement,
        |payload| payload[0] ^= 1,
    );
    assert_divergent_record_divergence(
        &fixture,
        &bytes,
        RecordKind::ExpectedActivityState,
        DivergentUniverseReplayDivergenceKind::ActivityState,
        |payload| payload[0] ^= 1,
    );
}

#[test]
fn baseline_replay_rejects_malformed_or_unoffered_activity_selection() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let recorded = record_divergent_universe_run(&fixture, DivergentUniverseRunFamily::Ordinary, 97).unwrap();
    let bytes = encode_divergent_universe_replay(&recorded).unwrap();
    for mutation in 0..8 {
        assert_divergent_record_divergence(
            &fixture, &bytes, RecordKind::AcceptedActivityCommand,
            DivergentUniverseReplayDivergenceKind::ActivityCommand,
            |payload| match mutation {
                0 => payload[0] ^= 1,
                1 => payload[4] = 255,
                2 => payload[5..13].copy_from_slice(&0_u64.to_le_bytes()),
                3 => payload[13..21].copy_from_slice(&999_u64.to_le_bytes()),
                4 => payload[21..25].copy_from_slice(&u32::MAX.to_le_bytes()),
                5 => payload.truncate(24),
                6 => payload.push(0),
                _ => payload[5..13].copy_from_slice(&999_u64.to_le_bytes()),
            },
        );
    }
    // A valid different choice is executable, but cannot reuse the old state
    // hash. Validation must report that earliest state divergence, not silently
    // substitute the default controller's choice or fail at a later battle.
    let replay = decode_replay(&bytes).unwrap();
    let mut payloads = replay.records().iter().map(|record| (record.kind(), record.payload().to_vec())).collect::<Vec<_>>();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let activity = flow.start(instance(1), ActivityMasterSeed::from_u64(97)).unwrap().into_activity();
    let view = activity.player_view();
    let options = view.decision().unwrap().options();
    let original = u64::from_le_bytes(payloads[2].1[13..21].try_into().unwrap());
    let different = options.iter().find(|option| option.id().get() != original).unwrap().id().get();
    payloads[2].1[13..21].copy_from_slice(&different.to_le_bytes());
    let changed = encode_payload_pairs(replay.header().clone(), &payloads);
    let error = verify_divergent_universe_replay(&changed, &fixture).unwrap_err();
    assert_eq!(error.first_divergence(), Some(DivergentUniverseReplayDivergenceKind::ActivityState));
    assert_eq!(error.record_index(), Some(3));
}

fn mutate_divergent_component(
    bytes: &[u8],
    target: ConfigurationComponentKind,
) -> Vec<u8> {
    let replay = decode_replay(bytes).expect("decode replay");
    let components = replay
        .header()
        .components()
        .components()
        .iter()
        .map(|component| {
            let mut digest = component.digest().bytes();
            if component.kind() == target {
                digest[0] ^= 1;
            }
            ConfigurationComponentIdentity::new(
                component.kind(),
                component.id(),
                ComponentDigest::new(digest),
            )
            .expect("valid mutated component")
        })
        .collect();
    let components = ConfigurationComponentSet::new(components).expect("canonical components");
    let header = ReplayHeader::new(
        replay.header().environment().clone(),
        components,
        replay.header().master_seed(),
        replay.header().entry().clone(),
        replay.header().record_count(),
    )
    .expect("valid mutated header");
    encode_divergent_payloads(&header, &replay)
}

fn assert_divergent_record_divergence(
    fixture: &DivergentUniverseBaselineFixture,
    bytes: &[u8],
    kind: RecordKind,
    expected: DivergentUniverseReplayDivergenceKind,
    mutate: impl FnOnce(&mut Vec<u8>),
) {
    let replay = decode_replay(bytes).expect("decode replay");
    let mut payloads = replay
        .records()
        .iter()
        .map(|record| (record.kind(), record.payload().to_vec()))
        .collect::<Vec<_>>();
    let (index, payload) = payloads
        .iter_mut()
        .enumerate()
        .skip(2)
        .find(|(_, (candidate, _))| *candidate == kind)
        .map(|(index, (_, payload))| (index, payload))
        .expect("selected record exists");
    mutate(payload);
    let corrupted = encode_payload_pairs(replay.header().clone(), &payloads);
    let error = verify_divergent_universe_replay(&corrupted, fixture)
        .expect_err("record divergence must fail");
    assert_eq!(error.first_divergence(), Some(expected), "{error:?}");
    assert_eq!(error.record_index(), u32::try_from(index).ok());
    assert!(error.component_divergence().is_none());
}

fn encode_divergent_payloads(
    header: &ReplayHeader,
    replay: &starclock_replay::format::DecodedReplay<'_>,
) -> Vec<u8> {
    let payloads = replay
        .records()
        .iter()
        .map(|record| (record.kind(), record.payload().to_vec()))
        .collect::<Vec<_>>();
    encode_payload_pairs(header.clone(), &payloads)
}

fn encode_payload_pairs(header: ReplayHeader, payloads: &[(RecordKind, Vec<u8>)]) -> Vec<u8> {
    let records = payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| {
            RecordRef::new(*kind, index as u64, payload).expect("valid record")
        })
        .collect::<Vec<_>>();
    encode_replay(&header, &records, Vec::new()).expect("encode mutated replay")
}

fn execute_baseline(area: &str, seed: u64) -> BaselineReceipt {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = baseline_flow(&factory, area, participants, mapping);
    let mut activity = flow
        .start(instance(seed), ActivityMasterSeed::from_u64(seed))
        .expect("baseline activity")
        .into_activity();
    let report = DivergentUniverseBaselineRunner::default()
        .run_to_terminal(
            &factory,
            &flow,
            &mut activity,
            &core,
            &baseline_policy(32),
        )
        .expect("complete baseline run");
    let (battle_result, battle_final_state, battle_commands) = report
        .steps()
        .iter()
        .find_map(|step| match step {
            DivergentUniverseBaselineStep::Battle {
                result_digest,
                execution,
                ..
            } => Some((
                *result_digest,
                execution.final_state_hash(),
                execution.trace().len(),
            )),
            DivergentUniverseBaselineStep::ActivityDecision { .. } => None,
        })
        .expect("real battle step");
    BaselineReceipt {
        family: report.run_family(),
        terminal: report.terminal(),
        final_state: report.final_state_hash(),
        battle_result,
        battle_final_state,
        activity_steps: report.steps().len(),
        battle_commands,
    }
}

fn baseline_flow(
    factory: &DivergentUniverseRuntimeFactory,
    area: &str,
    participants: Arc<ParticipantLock>,
    mapping: Arc<super::mapping::DivergentUniverseMappingSnapshot>,
) -> DivergentUniverseFlowInstance {
    let mut entry = DivergentUniverseEntry::new(
        area_id(area),
        difficulty_id("3011"),
        Arc::clone(&participants),
        input_snapshot(&participants),
        Vec::new(),
    )
    .expect("entry")
    .with_mapping_snapshot(mapping)
    .with_runtime_battle_route();
    if area.starts_with("204") {
        entry = entry.with_cyclical_refresh(refresh(1, area));
    }
    factory.compile(entry).expect("baseline flow")
}

fn baseline_policy(max_steps: u32) -> DivergentUniverseBaselinePolicy {
    DivergentUniverseBaselinePolicy::new(
        crate::baseline_controller::ActivityBaselineHints::default(),
        starclock_data::divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId::new(
            "divergent-universe.encounter-group.300202",
        )
        .expect("encounter group"),
        "83002081",
        max_steps,
    )
    .expect("baseline policy")
}
