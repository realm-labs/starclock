use starclock_activity::{
    ActivityBattleInPlaceSettlementError, ActivityBattleResultSubmission,
    ActivityBattleSettlementError, ActivityExpression, ActivityInstanceId, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityTransactionState, ActivityValue,
    AttemptId, BattleResult, BattleResultDigest, BattleSequence, ParticipantBattleState,
    ProjectedValue,
};
use starclock_combat::{Hp, LifeState, PresenceState};

use crate::{
    baseline_runner::NestedBattleExecutor, nested_battle_executor::UniverseNestedBattleExecutor,
};

use super::{
    battle_execution::SWARM_DISASTER_BATTLE_EXECUTION_REVISION,
    battle_materialization_tests::{
        activity_identity, activity_rng, combat_state, commit, instance, roster,
    },
};

#[test]
fn real_nested_battle_executes_and_settles_verified_carry() {
    let instance = instance();
    let mut state = combat_state(&instance);
    let repairing = instance
        .content_runtime
        .curios
        .iter()
        .find(|curio| {
            curio.initial_state == super::content_runtime::CurioState::Repairing
                && curio.repair_after_battles.is_some()
        })
        .map(|curio| curio.id)
        .unwrap();
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(repairing).unwrap(),
    );
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let mut rng = activity_rng(&instance, 0x2006_0601);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let (result, report, settlement) = instance
        .execute_current_battle(
            &mut state,
            &mut rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &roster,
            false,
        )
        .unwrap();

    assert_eq!(
        SWARM_DISASTER_BATTLE_EXECUTION_REVISION,
        "swarm-disaster-nested-battle-execution-v1"
    );
    assert_eq!(result.actual_digest(), result.claimed_digest());
    assert_eq!(
        digest_hex(result.actual_digest().bytes()),
        "d2eada5ea5f179780f4e891a6f9795ccff4242dcd0f08b1825961f767d3565d0"
    );
    assert_eq!(
        digest_hex(settlement.state_hash().bytes())
        ,
        "8f787b578b334622e41db8423f2662ef88c071b6c76ae0f2e10684b0de50e423"
    );
    assert_eq!(report.outcome(), settlement.outcome());
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
        content_counter(&state, super::content_runtime::counter_key(repairing)),
        1
    );
}

#[test]
fn final_boss_choice_decay_and_completion_settle_atomically() {
    let instance = instance();
    let node = instance.plane_ends().last().unwrap();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    let decay = instance
        .compile_boss_decay_selection(
            &state,
            &[
                "swarm-disaster.boss-decay.1",
                "swarm-disaster.boss-decay.25",
            ],
        )
        .unwrap();
    commit(&instance, &mut state, decay);
    commit(
        &instance,
        &mut state,
        instance
            .compile_boss_selection(3, "swarm-disaster.boss-choice.8024010")
            .unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(node, "swarm-disaster.domain.monsterswarmboss", None)
            .unwrap(),
    );
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let mut rng = activity_rng(&instance, 0x2006_0621);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let (_, report, settlement) = instance
        .execute_current_battle(
            &mut state,
            &mut rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &roster,
            false,
        )
        .unwrap();
    assert_eq!(report.outcome(), starclock_activity::BattleOutcome::Won);
    assert_eq!(
        digest_hex(settlement.result_digest().bytes()),
        "0a02bd51658cec20c62bb86ee185abe82d678f73702eb941cf594d7d354a13d8"
    );
    assert_eq!(
        digest_hex(settlement.state_hash().bytes())
        ,
        "1aec0e588dfdcb78aab6f8755fa54b343de5d8db7f1a41660be653707f5d8e6a"
    );
    assert!(!settlement.events().is_empty());
    assert_eq!(
        state.terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Completed)
    );
}

#[test]
fn missing_boss_choice_rolls_back_encounter_rng_and_activity_state() {
    let instance = instance();
    let node = instance.plane_ends().next().unwrap();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(node, "swarm-disaster.domain.monsterboss", None)
            .unwrap(),
    );
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let mut rng = activity_rng(&instance, 0x2006_0622);
    let before_rng = rng.snapshots();
    let before = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    assert!(
        instance
            .start_current_battle(
                &mut state,
                &mut rng,
                before,
                identity,
                activity_instance,
                AttemptId::new(1).unwrap(),
                BattleSequence::new(1).unwrap(),
                &roster,
            )
            .is_err()
    );
    assert_eq!(rng.snapshots(), before_rng);
    assert_eq!(
        state.state_hash(identity, instance.graph_definition(), activity_instance, &rng),
        before
    );
}

#[test]
fn rejected_result_is_byte_identical_and_defeated_carry_can_be_revived() {
    let instance = instance();
    let mut state = combat_state(&instance);
    commit(
        &instance,
        &mut state,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(0x7f98_0001).unwrap(),
            vec![ActivityOperation::AddCounter {
                slot: starclock_activity::ActivitySlotId::new(super::state::RESOURCES).unwrap(),
                key: 1,
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(100)),
            }],
        )
        .unwrap(),
    );
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let mut rng = activity_rng(&instance, 0x2006_0611);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let start = instance
        .start_current_battle(
            &mut state,
            &mut rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &roster,
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

    let revival = instance
        .compile_service_revival(
            "swarm-disaster.service.universe-service-reviver",
            defeated,
            0,
        )
        .unwrap();
    commit(&instance, &mut state, revival);
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
fn lost_nested_result_enters_generic_failed_terminal() {
    let instance = instance();
    let mut state = combat_state(&instance);
    let roster = roster(&instance);
    let identity = activity_identity();
    let activity_instance = ActivityInstanceId::new(1).unwrap();
    let mut rng = activity_rng(&instance, 0x2006_0631);
    let expected = state.state_hash(identity, instance.graph_definition(), activity_instance, &rng);
    let start = instance
        .start_current_battle(
            &mut state,
            &mut rng,
            expected,
            identity,
            activity_instance,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &roster,
        )
        .unwrap();
    let mut executor = UniverseNestedBattleExecutor::new(start.combat_catalog().clone());
    let result = executor.execute(start.handoff()).unwrap();
    let values = result
        .values()
        .iter()
        .map(|value| match value {
            ProjectedValue::Outcome(_) => {
                ProjectedValue::Outcome(starclock_activity::BattleOutcome::Lost)
            }
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

fn content_counter(state: &ActivityTransactionState, key: u64) -> i64 {
    match state.slot(starclock_activity::ActivitySlotId::new(super::state::CONTENT).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        _ => 0,
    }
}

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
