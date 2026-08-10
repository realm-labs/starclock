use starclock_activity::{
    ActivityBattleInPlaceSettlementError, ActivityBattleResultSubmission,
    ActivityBattleSettlementError, ActivityCause, ActivityInstanceId, ActivityTransactionOutcome,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivitySlotId,
    ActivityTransactionState, ActivityValue, AttemptId, BattleResult, BattleResultDigest,
    BattleSequence, ParticipantBattleState, ProjectedValue,
};
use starclock_combat::{Hp, LifeState, PresenceState};

use crate::{
    baseline_runner::NestedBattleExecutor, nested_battle_executor::UniverseNestedBattleExecutor,
};

use super::{
    GoldAndGearsBattleAssemblyContext,
    battle_materialization_tests::{
        activity_identity, activity_rng, commit, roster, selected_combat,
    },
    curio_types::GoldAndGearsCurioState,
    progression_runtime::GoldAndGearsExtrapolationContext,
    service_adventure_types::GoldAndGearsServiceKind,
};
use super::{tests, state_layout};

#[test]
fn real_nested_battle_executes_and_settles_verified_carry() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let (mut state, selection) = selected_combat(&instance, 0x1406_0301);
    let repairing = instance
        .curio_definitions()
        .iter()
        .find(|definition| definition.initial_state() == GoldAndGearsCurioState::Repairing)
        .unwrap()
        .id();
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(repairing).unwrap(),
    );
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let rng = activity_rng(&instance, 0x1406_0302);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let start = instance
        .start_current_battle(
            &mut state,
            &rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &selection,
            &roster,
            &GoldAndGearsBattleAssemblyContext::new(Vec::new(), false),
        )
        .unwrap();

    assert_eq!(
        start.handoff().battle_spec().combat_input_digest(),
        start.handoff().identity().combat_input_digest()
    );
    let execution = instance
        .execute_started_battle(&mut state, &rng, identity, activity_instance, &start)
        .unwrap();
    assert_eq!(
        digest_hex(execution.result().actual_digest().bytes()),
        "4ad77516a0abec581a192522ab36f788ae4f7a273d61a9649fad972c5f0d5b14"
    );
    assert_eq!(
        digest_hex(execution.settlement().state_hash().bytes()),
        "a2a7f2bfdab0d8239787da2950a3e746760d9dc42219498a085c6b9b87a5682a"
    );

    assert_eq!(execution.report().outcome(), execution.settlement().outcome());
    assert_eq!(execution.result().actual_digest(), execution.result().claimed_digest());
    assert_eq!(
        state
            .player_view(identity, instance.graph_definition(), activity_instance, &rng)
            .completed_battle_count(),
        1
    );
    assert_eq!(
        state
            .player_view(identity, instance.graph_definition(), activity_instance, &rng)
            .participant_carry()
            .len(),
        4
    );
    assert!(state.current_battle_attempt_is_settled());
    assert_eq!(
        lifecycle_counter(
            &state,
            state_layout::CONTENT_CURIO_CHARGE_BASE + u64::from(repairing.get()),
        ),
        1
    );
    assert!(instance
        .materialize_current_battle(
            &state,
            &selection,
            &roster,
            &GoldAndGearsBattleAssemblyContext::new(Vec::new(), false),
        )
        .is_err());
    let edge = instance
        .graph_definition()
        .outgoing(state.current_node())
        .next()
        .unwrap()
        .id();
    let traverse = ActivityProgramDefinition::new(
        ActivityProgramId::new(0x7f75_0001).unwrap(),
        vec![ActivityOperation::Traverse(edge)],
    )
    .unwrap();
    commit(&instance, &mut state, traverse);
    assert!(!state.current_battle_attempt_is_settled());
}

fn lifecycle_counter(state: &ActivityTransactionState, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(state_layout::CONTENT_LIFECYCLE_SLOT).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        _ => 0,
    }
}

#[test]
fn final_boss_choice_and_extrapolation_execute_before_atomic_plane_completion() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let node = instance.plane_ends().nth(2).unwrap();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    let boss = "gold-gears.boss-choice.8024011";
    commit(
        &instance,
        &mut state,
        instance.compile_boss_selection(3, boss).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(node, "gold-gears.domain.monsterboss", None)
            .unwrap(),
    );
    let mut selection_rng = activity_rng(&instance, 0x1406_0321);
    let selection = instance
        .select_current_encounter(&state, &mut selection_rng)
        .unwrap();
    assert!(selection.waves().iter().flat_map(|wave| wave.slots()).any(
        |slot| slot.boss_choices().any(|candidate| candidate == boss)
    ));
    let extrapolation = instance
        .compile_resonance_extrapolation(
            GoldAndGearsExtrapolationContext::new(3, true, "universe.path.abundance"),
            &mut selection_rng,
        )
        .unwrap();
    let context = GoldAndGearsBattleAssemblyContext::new(Vec::new(), false)
        .with_extrapolation(extrapolation.clone());
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let rng = activity_rng(&instance, 0x1406_0322);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let start = instance
        .start_current_battle(
            &mut state,
            &rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &selection,
            &roster,
            &context,
        )
        .unwrap();
    assert_eq!(
        start.contribution_digest(),
        instance
            .compile_battle_snapshot(&state, &context)
            .unwrap()
            .summary
            .digest()
    );
    let execution = instance
        .execute_started_battle(&mut state, &rng, identity, activity_instance, &start)
        .unwrap();
    assert_eq!(
        digest_hex(execution.result().actual_digest().bytes()),
        "c39d2b897d1dce5ef74143a78ef218a8eedefaeafda519c1f0ce302d4e1bebe3"
    );
    assert_eq!(
        digest_hex(execution.settlement().state_hash().bytes()),
        "daae421ef23e21f7c6fb5ee2415e3ad8c0ab021a82d1462c8e2c9372c5730fa0"
    );
    assert_eq!(execution.report().outcome(), starclock_activity::BattleOutcome::Won);
    assert!(!execution.post_battle_events().is_empty());
    assert_eq!(
        state.terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Completed)
    );
}

#[test]
fn rejected_result_is_byte_identical_and_defeat_can_be_revived() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let (mut state, selection) = selected_combat(&instance, 0x1406_0311);
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let rng = activity_rng(&instance, 0x1406_0312);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let start = instance
        .start_current_battle(
            &mut state,
            &rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &selection,
            &roster,
            &GoldAndGearsBattleAssemblyContext::new(Vec::new(), false),
        )
        .unwrap();
    let mut executor = UniverseNestedBattleExecutor::new(start.combat_catalog().clone());
    let result = executor.execute(start.handoff()).unwrap();
    let before = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let invalid = BattleResult::new(
        result.identity(),
        result.values().to_vec(),
        BattleResultDigest::new([0xee; 32]).unwrap(),
    );
    assert_eq!(
        state.submit_pending_battle_result_in_place(
            identity,
            instance.graph_definition(),
            activity_instance,
            &rng,
            ActivityBattleResultSubmission::new(before, invalid),
            None,
        ),
        Err(ActivityBattleInPlaceSettlementError::Settlement(
            ActivityBattleSettlementError::ResultDigestMismatch
        ))
    );
    assert_eq!(
        state.state_hash(identity, instance.graph_definition(), activity_instance, &rng),
        before
    );

    let defeated = roster.entries()[0].participant();
    let values = result
        .values()
        .iter()
        .map(|value| match value {
            ProjectedValue::ParticipantState(participant)
                if participant.participant() == defeated =>
            {
                ProjectedValue::ParticipantState(
                    ParticipantBattleState::new(
                        defeated,
                        Hp::new(0).unwrap(),
                        participant.maximum_hp(),
                        participant.current_energy(),
                        participant.maximum_energy(),
                        LifeState::Defeated,
                        PresenceState::Present,
                    )
                    .unwrap(),
                )
            }
            value => value.clone(),
        })
        .collect();
    let defeated_result = BattleResult::seal(result.identity(), values);
    state
        .submit_pending_battle_result_in_place(
            identity,
            instance.graph_definition(),
            activity_instance,
            &rng,
            ActivityBattleResultSubmission::new(before, defeated_result),
            None,
        )
        .unwrap();
    let carry = state
        .player_view(identity, instance.graph_definition(), activity_instance, &rng)
        .participant_carry()
        .iter()
        .find(|carry| carry.participant() == defeated)
        .copied()
        .unwrap();
    assert_eq!(carry.life(), LifeState::Defeated);
    assert_eq!(carry.presence(), PresenceState::Departed);

    let reviver = instance
        .service_definitions()
        .iter()
        .find(|service| service.kind() == GoldAndGearsServiceKind::Reviver)
        .unwrap();
    let program = instance
        .compile_service_revival(reviver.stable_key(), defeated, 0)
        .unwrap();
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
    let revived = state
        .player_view(identity, instance.graph_definition(), activity_instance, &rng)
        .participant_carry()
        .iter()
        .find(|carry| carry.participant() == defeated)
        .copied()
        .unwrap();
    assert_eq!(revived.life(), LifeState::Alive);
    assert_eq!(revived.presence(), PresenceState::Present);
    assert!(revived.current_hp().get() > 0);
}

#[test]
fn lost_nested_result_enters_the_generic_failed_terminal_without_a_graph_edge() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let (mut state, selection) = selected_combat(&instance, 0x1406_0331);
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let rng = activity_rng(&instance, 0x1406_0332);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let start = instance
        .start_current_battle(
            &mut state,
            &rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &selection,
            &roster,
            &GoldAndGearsBattleAssemblyContext::new(Vec::new(), false),
        )
        .unwrap();
    let mut executor = UniverseNestedBattleExecutor::new(start.combat_catalog().clone());
    let result = executor.execute(start.handoff()).unwrap();
    let values = result
        .values()
        .iter()
        .map(|value| match value {
            ProjectedValue::Outcome(_) => ProjectedValue::Outcome(starclock_activity::BattleOutcome::Lost),
            value => value.clone(),
        })
        .collect();
    let lost = BattleResult::seal(result.identity(), values);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let settlement = state
        .submit_pending_battle_result_in_place(
            identity,
            instance.graph_definition(),
            activity_instance,
            &rng,
            ActivityBattleResultSubmission::new(expected, lost),
            None,
        )
        .unwrap();
    assert_eq!(
        settlement.terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Failed)
    );
    assert_eq!(state.terminal(), settlement.terminal());
}

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
