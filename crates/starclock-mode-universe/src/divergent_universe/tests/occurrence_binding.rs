//! Current authored event on the production graph, not source-program parity.

use starclock_activity::{
    ActivityDecisionKind, ActivityExpression, ActivityMasterSeed, ActivityOperation,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivitySlotId,
    ActivityTransactionEventKind, ActivityValue, GraphActivity, GraphActivityCommandError,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

use super::curio_acquisition_blessings::acquisition_draws;
use super::{instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseOfferedSelection,
    decision_rewards::DecisionRewardGrant,
    encode_divergent_universe_replay,
    occurrence_binding::OccurrenceExecutionError,
    record_divergent_universe_transcript,
    state::{
        BLESSINGS_SLOT, CURIO_STATES_SLOT, CURRENCIES_SLOT, OCCURRENCE_REWARD_ACCEPTED_SLOT,
        ROOM_CONTENT_ENABLED_SLOT, ROOM_DIALOGUE_PROGRAM_SLOT, ROOM_DOORS_OPEN_SLOT,
        ROOM_FINISHED_SLOT,
    },
    verify_divergent_universe_replay,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn value(activity: &GraphActivity, id: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .unwrap()
        .value()
        .clone()
}

#[test]
fn production_occurrence_grants_before_opening_route_and_rejects_bypass_or_repeat() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow(family).unwrap();
        assert!(flow.initial_occurrence().is_some());
        for ordinal in 1..=3 {
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(23630))
                .unwrap()
                .into_activity();
            super::initial_equations::accept_initial(&flow, &mut activity);
            let before = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            let draws = reward_draws(&activity);
            let node = activity.current_node();
            let offer = activity.player_view().decision().unwrap().clone();
            assert_eq!(offer.kind(), ActivityDecisionKind::Choice);
            assert_eq!(offer.options().len(), 3);
            assert!(matches!(
                value(&activity, ROOM_DIALOGUE_PROGRAM_SLOT),
                ActivityValue::OptionalId(Some(_))
            ));
            assert_eq!(
                value(&activity, ROOM_FINISHED_SLOT),
                ActivityValue::Boolean(false)
            );
            assert_eq!(
                value(&activity, ROOM_DOORS_OPEN_SLOT),
                ActivityValue::Boolean(false)
            );
            let option = ActivityOptionId::new(ordinal).unwrap();
            assert!(activity.choose_option(hash, offer.id(), option).is_err());
            assert_eq!(before, activity.canonical_state_bytes());
            assert_eq!(draws, reward_draws(&activity));
            let result = flow
                .choose_occurrence_option(
                    fixture.factory(),
                    &mut activity,
                    hash,
                    offer.id(),
                    option,
                )
                .unwrap();
            let inventory = match result.value() {
                DecisionRewardGrant::Fragments(amount) => {
                    assert_eq!(*amount, 200);
                    assert_eq!(reward_draws(&activity), draws);
                    let ActivityValue::BoundedCounterMap(values) =
                        value(&activity, CURRENCIES_SLOT)
                    else {
                        panic!("currency map")
                    };
                    assert_eq!(values.len(), 1);
                    assert_eq!(values[0].1, 200);
                    CURRENCIES_SLOT
                }
                DecisionRewardGrant::Curios(states) => {
                    assert_eq!(states.len(), 2);
                    assert_eq!(
                        fixture
                            .factory()
                            .curio_runtime()
                            .unwrap()
                            .owned(&activity)
                            .unwrap()
                            .len(),
                        2
                    );
                    assert_eq!(
                        reward_draws(&activity) - draws,
                        2 + acquisition_draws(fixture.factory(), states)
                    );
                    CURIO_STATES_SLOT
                }
                DecisionRewardGrant::Blessings(ids) => {
                    assert_eq!(ids.len(), 2);
                    assert_eq!(
                        fixture
                            .factory()
                            .blessing_runtime()
                            .unwrap()
                            .owned(&activity)
                            .unwrap()
                            .len(),
                        2
                    );
                    assert_eq!(reward_draws(&activity) - draws, 2);
                    BLESSINGS_SLOT
                }
            };
            let events = result.events();
            let position = |slot| {
                events
                    .iter()
                    .position(|event| match event.kind() {
                        ActivityTransactionEventKind::SlotChanged(changed)
                        | ActivityTransactionEventKind::CounterChanged { slot: changed, .. } => {
                            *changed == slot
                        }
                        _ => false,
                    })
                    .unwrap()
            };
            assert!(position(inventory) < position(ROOM_FINISHED_SLOT));
            assert!(position(ROOM_FINISHED_SLOT) < position(ROOM_DOORS_OPEN_SLOT));
            assert_eq!(activity.current_node(), node);
            assert_eq!(
                value(&activity, ROOM_FINISHED_SLOT),
                ActivityValue::Boolean(true)
            );
            assert_eq!(
                value(&activity, ROOM_DOORS_OPEN_SLOT),
                ActivityValue::Boolean(true)
            );
            assert_eq!(
                value(&activity, OCCURRENCE_REWARD_ACCEPTED_SLOT),
                ActivityValue::OptionalId(None)
            );
            let next = activity.player_view().decision().unwrap().clone();
            assert_eq!(next.kind(), ActivityDecisionKind::Encounter);
            assert_ne!(next.id(), offer.id());
            let accepted = activity.canonical_state_bytes();
            assert!(matches!(
                flow.choose_occurrence_option(
                    fixture.factory(),
                    &mut activity,
                    hash,
                    offer.id(),
                    option
                ),
                Err(OccurrenceExecutionError::Activity(
                    GraphActivityCommandError::StaleStateHash
                ))
            ));
            let current = activity.state_hash();
            assert!(matches!(
                flow.choose_occurrence_option(
                    fixture.factory(),
                    &mut activity,
                    current,
                    offer.id(),
                    option
                ),
                Err(OccurrenceExecutionError::NotOffered)
            ));
            assert_eq!(activity.canonical_state_bytes(), accepted);
        }
    }
}

#[test]
fn production_occurrence_all_choices_finish_real_battle_and_verify_selected_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let policy = fixture.policy().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    for family in FAMILIES {
        for ordinal in 1..=3 {
            let flow = fixture.flow(family).unwrap();
            let seed = 23631;
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(seed))
                .unwrap()
                .into_activity();
            let initial = runner
                .advance(
                    fixture.factory(),
                    &flow,
                    &mut activity,
                    fixture.core(),
                    &policy,
                )
                .unwrap();
            let decision = activity.player_view().decision().unwrap().id();
            let mut steps = vec![
                initial,
                runner
                    .advance_selected(
                        fixture.factory(),
                        &flow,
                        &mut activity,
                        fixture.core(),
                        &policy,
                        DivergentUniverseOfferedSelection::new(
                            decision,
                            ActivityOptionId::new(ordinal).unwrap(),
                        ),
                    )
                    .unwrap(),
            ];
            let mut reward_choices = 0_u32;
            let mut treasure_choices = 0_u32;
            let mut domain_choices = 0_u32;
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < policy.max_steps() as usize);
                reward_choices += u32::from(
                    activity.player_view().decision().unwrap().kind()
                        == ActivityDecisionKind::Reward,
                );
                treasure_choices += u32::from(flow.offered_evolution_event(&activity).is_some());
                domain_choices += u32::from(
                    flow.offered_battle_domain_choices(&activity)
                        .unwrap()
                        .is_some(),
                );
                steps.push(
                    runner
                        .advance(
                            fixture.factory(),
                            &flow,
                            &mut activity,
                            fixture.core(),
                            &policy,
                        )
                        .unwrap(),
                );
            }
            assert_eq!(domain_choices, 2);
            assert!(reward_choices <= 3 && treasure_choices <= 4);
            // Curio rewards can suppress later offers or produce Treasure
            // dialogues. Count these real decisions rather than assuming the
            // configuration-dependent seed always yields the ten-action path.
            let expected_actions = 7 + reward_choices + treasure_choices;
            assert_eq!(steps.len(), expected_actions as usize);
            let recorded =
                record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                    .unwrap();
            let bytes = encode_divergent_universe_replay(&recorded).unwrap();
            let replay = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
            assert_eq!(replay.action_count(), expected_actions);
            assert_eq!(
                replay.terminal(),
                starclock_activity::ActivityTerminalOutcome::Completed
            );
            assert_eq!(replay.battle_count(), 3);
            assert!(replay.battle_command_count() > 0);
            assert_eq!(
                replay.final_state_hash().bytes(),
                activity.state_hash().bytes()
            );
        }
    }
}

#[test]
fn production_occurrence_late_content_rejection_rolls_back_reward_and_route_gate_is_live() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for ordinal in [2, 3] {
            let flow = fixture.flow(family).unwrap();
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(23633))
                .unwrap()
                .into_activity();
            super::initial_equations::accept_initial(&flow, &mut activity);
            // A trusted content executor can disable its node after offering.
            // The authored Require runs after reward generation, so rejection
            // must restore both inventories and all random draws.
            set_flag(&mut activity, ROOM_CONTENT_ENABLED_SLOT, false);
            let before = activity.canonical_state_bytes();
            let draws = reward_draws(&activity);
            let hash = activity.state_hash();
            let offer = activity.player_view().decision().unwrap().clone();
            assert!(matches!(
                flow.choose_occurrence_option(
                    fixture.factory(),
                    &mut activity,
                    hash,
                    offer.id(),
                    ActivityOptionId::new(ordinal).unwrap()
                ),
                Err(OccurrenceExecutionError::Activity(
                    GraphActivityCommandError::Rejected(_)
                ))
            ));
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
            set_flag(&mut activity, ROOM_CONTENT_ENABLED_SLOT, true);
            let hash = activity.state_hash();
            flow.choose_occurrence_option(
                fixture.factory(),
                &mut activity,
                hash,
                offer.id(),
                ActivityOptionId::new(ordinal).unwrap(),
            )
            .unwrap();
            let route = activity.player_view().decision().unwrap().clone();
            set_flag(&mut activity, ROOM_DOORS_OPEN_SLOT, false);
            let before = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                activity
                    .choose_option(hash, route.id(), route.options()[0].id())
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
}

fn set_flag(activity: &mut GraphActivity, slot: ActivitySlotId, value: bool) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(23633).unwrap(),
        vec![ActivityOperation::SetSlot {
            slot,
            value: ActivityExpression::Literal(ActivityValue::Boolean(value)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}
