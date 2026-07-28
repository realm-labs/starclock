use std::{num::NonZeroUsize, sync::Arc};

use starclock_activity::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityPreparationBoundary, BattleOutcome,
    BattleResult, EventDigest, ParticipantBattleState, ParticipantId, ProjectedValue,
};
use starclock_combat::{Battle, BattleStateHash, Energy, Hp, LifeState, PresenceState, TeamSide};
use starclock_mode_universe::{
    baseline_runner::{StandardUniverseBaselinePolicy, StandardUniverseBaselineRunner},
    battle_technique::UniverseBattleTechniqueDefinition,
    dynamic_battle_assembler::{
        BattleAssemblyBudget, StandardUniverseBattleAssembler, StandardUniverseDynamicBattleError,
    },
    nested_battle_executor::UniverseNestedBattleExecutor,
    production_runtime::{StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory},
    universe_replay_v3::{
        ReplayV3DivergenceKind, encode_standard_universe_trace_v3, record_baseline_run_v3,
        standard_universe_header_v3, verify_standard_universe_replay_v3_dynamic,
    },
};
use starclock_replay::record::RecordKind;

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn activity_and_assembler(
    seed: u64,
) -> (
    starclock_mode_universe::runtime::StandardUniverseActivity,
    Arc<StandardUniverseBattleAssembler>,
) {
    let factory = StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE).unwrap();
    let instance = factory
        .start(
            1,
            0,
            seed,
            StandardUniverseControllerIdentity {
                id: "dynamic-assembly-test",
                revision: "dynamic-assembly-test-v1",
                digest: [0x62; 32],
            },
        )
        .unwrap();
    let assembler = Arc::clone(instance.battle_assembler());
    let (_, activity, _, _, _) = instance.into_dynamic_parts();
    (activity, assembler)
}

fn drive_to_pending(activity: &mut starclock_mode_universe::runtime::StandardUniverseActivity) {
    for _ in 0..128 {
        let view = activity.view();
        let decision = view.decision().expect("nonterminal Activity decision");
        match decision.kind() {
            ActivityDecisionKind::Encounter => {
                let option = decision.options()[0].id();
                let prepared = activity
                    .engage_encounter(view.state_hash(), decision.id(), option, 5)
                    .unwrap();
                if prepared.boundary() == ActivityPreparationBoundary::Decision {
                    let preparation = activity.preparation_view().unwrap();
                    let normal = preparation.options()[0].id();
                    assert_eq!(
                        activity
                            .choose_preparation_option(activity.view().state_hash(), normal)
                            .unwrap(),
                        ActivityPreparationBoundary::BattleReady
                    );
                }
                return;
            }
            ActivityDecisionKind::ExternalOutcome => {
                activity
                    .submit_external_outcome(
                        view.state_hash(),
                        decision.id(),
                        ActivityExternalOutcomeId::new(decision.options()[0].id().get()).unwrap(),
                    )
                    .unwrap();
            }
            ActivityDecisionKind::Choice
            | ActivityDecisionKind::Reward
            | ActivityDecisionKind::Route => {
                activity
                    .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                    .unwrap();
            }
            other => panic!("unexpected decision before first encounter: {other:?}"),
        }
    }
    panic!("first encounter was not reached within the test budget");
}

#[test]
fn pending_encounter_is_assembled_and_sealed_from_one_current_snapshot() {
    let (mut activity, assembler) = activity_and_assembler(0x6023);
    let initial_bytes = activity.graph().canonical_state_bytes();
    assert!(matches!(
        assembler.start_pending_battle(&mut activity),
        Err(StandardUniverseDynamicBattleError::MissingPendingBattle)
    ));
    assert_eq!(activity.graph().canonical_state_bytes(), initial_bytes);

    drive_to_pending(&mut activity);
    let pending = activity
        .view()
        .pending_battle()
        .expect("encounter preparation produced a pending placeholder")
        .clone();
    let snapshot = activity.battle_start_snapshot().unwrap();
    assert!(
        !assembler
            .resolve_snapshot(&snapshot, None)
            .unwrap()
            .cache_hit()
    );
    assert!(
        assembler
            .resolve_snapshot(&snapshot, None)
            .unwrap()
            .cache_hit()
    );
    let before_start_hash = activity.view().state_hash();
    let started = assembler.start_pending_battle(&mut activity).unwrap();
    let handoff = started.handoff();

    assert!(started.cache_hit());
    assert_eq!(started.assembly_key().contributions(), snapshot.digest());
    assert_eq!(started.assembly_key().carry(), snapshot.carry_digest());
    assert_ne!(
        handoff.identity().assembly_digest(),
        pending.assembly_digest()
    );
    assert_eq!(
        handoff.identity().combat_input_digest(),
        handoff.battle_spec().combat_input_digest()
    );
    assert_eq!(
        handoff.identity().assembly_digest(),
        handoff.battle_spec().assembly_digest()
    );
    assert_ne!(activity.view().state_hash(), before_start_hash);
    assert!(
        handoff
            .contract_digest()
            .bytes()
            .iter()
            .any(|byte| *byte != 0)
    );
    assert!(assembler.cache_metrics().misses() >= 1);
    assert!(assembler.cache_entry_count() >= 2);

    let battle = Battle::create(
        Arc::clone(started.combat_catalog()),
        handoff.battle_spec().clone(),
        handoff.identity().seed(),
    )
    .expect("dynamically assembled handoff is executable");
    assert_eq!(
        battle.view().identity().combat_input_digest(),
        handoff.identity().combat_input_digest()
    );
}

#[test]
fn stale_invalid_and_budget_failures_preserve_state_and_retry_cleanly() {
    let (mut activity, assembler) = activity_and_assembler(0x6024);
    let initial = activity.view();
    activity
        .choose_option(
            initial.state_hash(),
            initial.decision().unwrap().id(),
            initial.decision().unwrap().options()[0].id(),
        )
        .unwrap();
    let stale_snapshot = activity.battle_start_snapshot().unwrap();
    drive_to_pending(&mut activity);
    let pending_bytes = activity.graph().canonical_state_bytes();

    assert!(matches!(
        assembler.start_pending_battle_from_snapshot(&mut activity, stale_snapshot),
        Err(StandardUniverseDynamicBattleError::StaleSnapshot)
    ));
    assert_eq!(activity.graph().canonical_state_bytes(), pending_bytes);

    let current = activity.battle_start_snapshot().unwrap();
    let invalid_technique = UniverseBattleTechniqueDefinition::new(
        starclock_activity::ActivityOptionId::new(0x6024).unwrap(),
        starclock_activity::ParticipantId::new(1).unwrap(),
        starclock_combat::AbilityId::new(1).unwrap(),
        1,
        starclock_activity::TechniqueEngagement::Engage,
    )
    .unwrap();
    let entries_before_invalid = assembler.cache_entry_count();
    assert!(matches!(
        assembler.resolve_snapshot(&current, Some(invalid_technique)),
        Err(StandardUniverseDynamicBattleError::MissingTechnique)
    ));
    assert_eq!(assembler.cache_entry_count(), entries_before_invalid);
    assert_eq!(activity.graph().canonical_state_bytes(), pending_bytes);

    let constrained = assembler
        .fork_with_policy(
            NonZeroUsize::new(2).unwrap(),
            BattleAssemblyBudget::new(1_024, 64, 8, 0),
        )
        .unwrap();
    assert!(matches!(
        constrained.start_pending_battle(&mut activity),
        Err(StandardUniverseDynamicBattleError::BudgetExceeded)
    ));
    assert_eq!(activity.graph().canonical_state_bytes(), pending_bytes);

    let started = assembler.start_pending_battle(&mut activity).unwrap();
    assert_eq!(
        started.handoff().identity().combat_input_digest(),
        started.handoff().battle_spec().combat_input_digest()
    );
}

#[test]
fn bounded_dynamic_cache_hits_and_evicts_exact_activity_snapshots() {
    let (mut activity, assembler) = activity_and_assembler(0x6025);
    let bounded = assembler
        .fork_with_policy(
            NonZeroUsize::new(2).unwrap(),
            BattleAssemblyBudget::default(),
        )
        .unwrap();
    let mut resolved_keys = Vec::new();
    for _ in 0..128 {
        if let Ok(snapshot) = activity.battle_start_snapshot() {
            let resolved = bounded.resolve_snapshot(&snapshot, None).unwrap();
            resolved_keys.push(resolved.assembly_key());
            if resolved_keys.len() == 3 {
                break;
            }
        }
        let view = activity.view();
        let decision = view.decision().unwrap();
        if decision.kind() == ActivityDecisionKind::Encounter {
            let option = decision.options()[0].id();
            activity
                .engage_encounter(view.state_hash(), decision.id(), option, 5)
                .unwrap();
        } else if decision.kind() == ActivityDecisionKind::ExternalOutcome {
            activity
                .submit_external_outcome(
                    view.state_hash(),
                    decision.id(),
                    ActivityExternalOutcomeId::new(decision.options()[0].id().get()).unwrap(),
                )
                .unwrap();
        } else {
            activity
                .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                .unwrap();
        }
    }
    assert_eq!(resolved_keys.len(), 3);
    assert!(resolved_keys.windows(2).all(|pair| pair[0] != pair[1]));
    assert_eq!(bounded.cache_entry_count(), 2);
    assert!(bounded.cache_metrics().evictions() >= 2);
    assert!(
        bounded
            .resolve_snapshot(&activity.battle_start_snapshot().unwrap(), None)
            .unwrap()
            .cache_hit()
    );
}

#[test]
fn settled_carry_is_reassembled_into_the_next_real_battle() {
    let (mut activity, assembler) = activity_and_assembler(1);
    drive_to_pending(&mut activity);
    let first = assembler.start_pending_battle(&mut activity).unwrap();
    let first_input = first.handoff().identity().combat_input_digest();
    activity
        .submit_pending_battle_result(
            activity.view().state_hash(),
            damaged_win(first.handoff().identity()),
        )
        .unwrap();
    assert_eq!(activity.view().participant_carry().len(), 4);

    drive_to_pending(&mut activity);
    let snapshot = activity.battle_start_snapshot().unwrap();
    assert_eq!(
        snapshot.participant_carry(),
        activity.view().participant_carry()
    );
    let second = assembler.start_pending_battle(&mut activity).unwrap();
    assert_ne!(
        second.handoff().identity().combat_input_digest(),
        first_input
    );
    let players = second
        .handoff()
        .battle_spec()
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Player)
        .collect::<Vec<_>>();
    assert_eq!(players.len(), 4);
    for (index, participant) in players.iter().enumerate() {
        let initial = participant
            .initial_state()
            .expect("the second battle embeds exact Activity carry");
        assert_eq!(
            initial.current_hp(),
            Hp::new(70_000 + i64::try_from(index).unwrap()).unwrap()
        );
        assert_eq!(
            initial.current_energy(),
            Energy::from_scaled(40_000_000 + i64::try_from(index).unwrap()).unwrap()
        );
    }
    Battle::create(
        Arc::clone(second.combat_catalog()),
        second.handoff().battle_spec().clone(),
        second.handoff().identity().seed(),
    )
    .expect("the carry-adjusted second battle is executable");
}

fn damaged_win(identity: starclock_activity::BattleResultIdentity) -> BattleResult {
    let mut values = vec![
        ProjectedValue::Outcome(BattleOutcome::Won),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x73; 32])),
        ProjectedValue::EventDigest(EventDigest::new([0x74; 32]).unwrap()),
        ProjectedValue::TerminalFault(None),
    ];
    values.extend((0_u32..4).map(|index| {
        ProjectedValue::ParticipantState(
            ParticipantBattleState::new(
                ParticipantId::new(index + 1).unwrap(),
                Hp::new(70_000 + i64::from(index)).unwrap(),
                Hp::new(100_000).unwrap(),
                Energy::from_scaled(40_000_000 + i64::from(index)).unwrap(),
                Energy::from_scaled(100_000_000).unwrap(),
                LifeState::Alive,
                PresenceState::Present,
            )
            .unwrap(),
        )
    }));
    BattleResult::seal(identity, values)
}

#[test]
fn production_baseline_records_and_verifies_dynamic_replay_v3() {
    let factory = StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE).unwrap();
    let controller = StandardUniverseControllerIdentity {
        id: "dynamic-replay-test",
        revision: StandardUniverseBaselineRunner::REVISION,
        digest: [0x63; 32],
    };
    let instance = factory.start(1, 0, 0x6027, controller).unwrap();
    let profile_id = instance.profile_id().to_owned();
    let components = instance.components().clone();
    let compatibility = instance.compatibility().clone();
    let assembler = Arc::clone(instance.battle_assembler());
    let (_, mut activity, _, _, _) = instance.into_dynamic_parts();
    let header = standard_universe_header_v3(
        compatibility.clone(),
        components.clone(),
        0x6027,
        &activity,
        &profile_id,
    )
    .unwrap();
    let mut executor = UniverseNestedBattleExecutor::dynamic();
    let recorded = record_baseline_run_v3(
        &mut activity,
        &StandardUniverseBaselinePolicy::default(),
        &assembler,
        &mut executor,
    )
    .unwrap();
    let replay = encode_standard_universe_trace_v3(&header, &recorded).unwrap();
    assert!(
        starclock_replay::format_v3::decode_replay_v3(&replay).is_ok(),
        "new production recordings use replay v3"
    );

    let fresh = factory.start(1, 0, 0x6027, controller).unwrap();
    let fresh_assembler = Arc::clone(fresh.battle_assembler());
    let (_, fresh_activity, _, _, _) = fresh.into_dynamic_parts();
    let verified = verify_standard_universe_replay_v3_dynamic(
        &replay,
        fresh_activity,
        &fresh_assembler,
        &components,
        &compatibility,
        &profile_id,
    )
    .unwrap();
    assert_eq!(verified.battle_count(), recorded.battles().len() as u32);
    assert_eq!(
        verified.final_state_hash().bytes(),
        recorded.report().final_state_hash().bytes()
    );
}

#[test]
fn dynamic_replay_reconstructs_each_snapshot_and_reports_first_divergence() {
    const SEED: u64 = 1;
    let controller = StandardUniverseControllerIdentity {
        id: "dynamic-replay-corruption-test",
        revision: StandardUniverseBaselineRunner::REVISION,
        digest: [0x64; 32],
    };
    let recording_factory =
        StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE).unwrap();
    let instance = recording_factory.start(1, 0, SEED, controller).unwrap();
    let profile_id = instance.profile_id().to_owned();
    let components = instance.components().clone();
    let compatibility = instance.compatibility().clone();
    let assembler = Arc::clone(instance.battle_assembler());
    let (_, mut activity, _, _, _) = instance.into_dynamic_parts();
    let header = standard_universe_header_v3(
        compatibility.clone(),
        components.clone(),
        SEED,
        &activity,
        &profile_id,
    )
    .unwrap();
    let mut executor = UniverseNestedBattleExecutor::dynamic();
    let recorded = record_baseline_run_v3(
        &mut activity,
        &StandardUniverseBaselinePolicy::default(),
        &assembler,
        &mut executor,
    )
    .unwrap();
    let replay = encode_standard_universe_trace_v3(&header, &recorded).unwrap();

    let verify = |bytes: &[u8]| {
        let factory = StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE).unwrap();
        let fresh = factory.start(1, 0, SEED, controller).unwrap();
        let assembler = Arc::clone(fresh.battle_assembler());
        let (_, activity, _, _, _) = fresh.into_dynamic_parts();
        let before = assembler.cache_metrics();
        let result = verify_standard_universe_replay_v3_dynamic(
            bytes,
            activity,
            &assembler,
            &components,
            &compatibility,
            &profile_id,
        );
        (result, before, assembler.cache_metrics())
    };

    let (verified, before, after) = verify(&replay);
    assert_eq!(
        after.hits() + after.misses() - before.hits() - before.misses(),
        recorded.battles().len() as u64,
        "verification must resolve one current Activity snapshot per battle"
    );
    assert_eq!(
        verified.unwrap().battle_count() as usize,
        recorded.battles().len()
    );

    let divergence = |bytes: &[u8]| verify(bytes).0.unwrap_err().first_divergence();
    let start_payload = v3_payload_offset(&replay, RecordKind::NestedBattleStart, 0);
    let revision_length = u32::from_le_bytes(
        replay[start_payload + 34..start_payload + 38]
            .try_into()
            .unwrap(),
    ) as usize;
    let identity = start_payload + 38 + revision_length;
    let combat_input = identity + 8 + 16 + 96;
    let assembly = combat_input + 32;
    let state_payload = v3_payload_offset(&replay, RecordKind::ExpectedBattleState, 0);

    let mut component_corrupt = replay.clone();
    component_corrupt[start_payload + 2] ^= 0x80;
    component_corrupt[assembly] ^= 0x80;
    assert_eq!(
        divergence(&component_corrupt),
        Some(ReplayV3DivergenceKind::Component)
    );

    let mut assembly_corrupt = replay.clone();
    assembly_corrupt[assembly] ^= 0x80;
    assert_eq!(
        divergence(&assembly_corrupt),
        Some(ReplayV3DivergenceKind::Assembly)
    );

    let mut combat_corrupt = replay.clone();
    combat_corrupt[combat_input] ^= 0x80;
    assert_eq!(
        divergence(&combat_corrupt),
        Some(ReplayV3DivergenceKind::CombatInput)
    );

    let mut command_corrupt = replay.clone();
    let command_payload = v3_payload_offset(&command_corrupt, RecordKind::AcceptedBattleCommand, 0);
    command_corrupt[command_payload + 2] ^= 0xff;
    assert_eq!(
        divergence(&command_corrupt),
        Some(ReplayV3DivergenceKind::Command)
    );

    let mut event_corrupt = replay.clone();
    event_corrupt[state_payload + 42] ^= 0x80;
    assert_eq!(
        divergence(&event_corrupt),
        Some(ReplayV3DivergenceKind::Event)
    );

    let mut state_corrupt = replay.clone();
    state_corrupt[state_payload + 2] ^= 0x80;
    assert_eq!(
        divergence(&state_corrupt),
        Some(ReplayV3DivergenceKind::State)
    );

    let mut result_corrupt = replay.clone();
    let (result_payload, result_length) =
        v3_payload_range(&result_corrupt, RecordKind::NestedBattleEnd, 0);
    result_corrupt[result_payload + result_length - 1] ^= 0x80;
    assert_eq!(
        divergence(&result_corrupt),
        Some(ReplayV3DivergenceKind::Result)
    );

    let mut activity_corrupt = replay;
    let activity_state = v3_payload_offset(&activity_corrupt, RecordKind::ExpectedActivityState, 0);
    activity_corrupt[activity_state] ^= 0x80;
    assert_eq!(
        divergence(&activity_corrupt),
        Some(ReplayV3DivergenceKind::Activity)
    );
}

fn v3_payload_offset(bytes: &[u8], kind: RecordKind, ordinal: usize) -> usize {
    let decoded = starclock_replay::format_v3::decode_replay_v3(bytes).unwrap();
    let payload = decoded
        .records()
        .iter()
        .filter(|record| record.kind() == kind)
        .nth(ordinal)
        .unwrap_or_else(|| panic!("missing replay-v3 record {kind:?} at ordinal {ordinal}"))
        .payload();
    payload.as_ptr() as usize - bytes.as_ptr() as usize
}

fn v3_payload_range(bytes: &[u8], kind: RecordKind, ordinal: usize) -> (usize, usize) {
    let decoded = starclock_replay::format_v3::decode_replay_v3(bytes).unwrap();
    let payload = decoded
        .records()
        .iter()
        .filter(|record| record.kind() == kind)
        .nth(ordinal)
        .unwrap()
        .payload();
    (
        payload.as_ptr() as usize - bytes.as_ptr() as usize,
        payload.len(),
    )
}
