//! Real handoffs with counterfactual result vectors isolate victory/HP rules.

use super::{currency_balance, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBattleAssemblyPolicy, DivergentUniverseFlowInstance,
    DivergentUniverseOfferedSelection, curio_runtime::DivergentUniverseCurioLifecycleState,
    economy::DivergentUniverseCurrencyKind, encode_divergent_universe_replay,
    record_divergent_universe_transcript, state::CURRENCIES_SLOT, verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityTerminalOutcome, ActivityValue,
    AttemptId, BattleOutcome, BattleResult, BattleSequence, GraphActivity, ParticipantBattleState,
    ProjectedValue,
};
use starclock_combat::{Hp, LifeState, PresenceState};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
pub(super) fn ready(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> (DivergentUniverseFlowInstance, GraphActivity) {
    let flow = fixture.flow(family).unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(23_921))
        .unwrap()
        .into_activity();
    for _ in 0..2 {
        DivergentUniverseBaselineRunner::default()
            .advance(
                fixture.factory(),
                &flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().unwrap(),
            )
            .unwrap();
    }
    (flow, activity)
}
fn key(flow: &DivergentUniverseFlowInstance) -> u64 {
    flow.economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key()
}
pub(super) fn result(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
) -> BattleResult {
    let factory = fixture.factory();
    let hash = activity.state_hash();
    let contribution = factory
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(flow, activity)
        .unwrap();
    let (group, stage) = flow.offered_encounter(activity).unwrap().unwrap();
    let encounter = factory
        .encounter_reachability_runtime()
        .unwrap()
        .select_stage_candidate(activity, hash, group, stage)
        .unwrap();
    let assembled = factory.battle_assembly_runtime().materialize_current_battle(flow, activity, fixture.core(), &contribution, &encounter,
        DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy).unwrap();
    let start = factory
        .battle_settlement_runtime()
        .start_current_battle(
            flow,
            activity,
            hash,
            AttemptId::new(1).unwrap(),
            BattleSequence::new(1).unwrap(),
            &assembled,
        )
        .unwrap();
    start.execute().unwrap().0
}

// Counterfactual, contract-valid projections are not claimed as natural combat
// outcomes. Maximum resources and participant identities remain battle-bound.
pub(super) fn project(
    original: &BattleResult,
    eligible: u32,
    outcome: BattleOutcome,
) -> BattleResult {
    BattleResult::seal(
        original.identity(),
        original
            .values()
            .iter()
            .map(|value| match value {
                ProjectedValue::Outcome(_) => ProjectedValue::Outcome(outcome),
                ProjectedValue::ParticipantState(value) => {
                    let index = value.participant().get();
                    let (hp, life, presence) = if index <= eligible {
                        (value.maximum_hp(), LifeState::Alive, PresenceState::Present)
                    } else {
                        match index % 3 {
                            0 => (
                                Hp::new(0).unwrap(),
                                LifeState::Defeated,
                                PresenceState::Present,
                            ),
                            1 => (
                                Hp::new(value.maximum_hp().get() - 1).unwrap(),
                                LifeState::Alive,
                                PresenceState::Present,
                            ),
                            _ => (
                                value.maximum_hp(),
                                LifeState::Alive,
                                PresenceState::Reserved,
                            ),
                        }
                    };
                    ProjectedValue::ParticipantState(
                        ParticipantBattleState::new(
                            value.participant(),
                            hp,
                            value.maximum_hp(),
                            value.current_energy(),
                            value.maximum_energy(),
                            life,
                            presence,
                        )
                        .unwrap(),
                    )
                }
                value => value.clone(),
            })
            .collect(),
    )
}

#[test]
fn green_battle_grant_counts_only_full_hp_present_living_roster_and_aggregates_bonuses() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for (count, modifier, expected) in [
            (0, None, 0),
            (1, None, 8),
            (2, None, 16),
            (3, None, 24),
            (4, None, 32),
            (4, Some("9159"), 41),
            (4, Some("9055"), 48),
        ] {
            let (flow, mut activity) = ready(&fixture, family);
            let hash = activity.state_hash();
            curios
                .acquire_accepted_state(&mut activity, hash, &state("9195"))
                .unwrap();
            if let Some(raw) = modifier {
                let hash = activity.state_hash();
                curios
                    .acquire_accepted_state(&mut activity, hash, &state(raw))
                    .unwrap();
            }
            let before = currency_balance(&activity, key(&flow));
            let actual = result(&fixture, &flow, &mut activity);
            let projected = project(&actual, count, BattleOutcome::Won);
            let draws = reward_draws(&activity);
            let hash = activity.state_hash();
            let resolution = fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, projected.clone(), None)
                .unwrap();
            let offer_event = resolution
                .events()
                .iter()
                .position(|event| event.cause().program().get() == 23_702)
                .expect("normal reward stage always writes its candidate set");
            let grant_events = resolution
                .events()
                .iter()
                .enumerate()
                .filter(|(_, event)| event.cause().program().get() == 23_700)
                .collect::<Vec<_>>();
            assert_eq!(grant_events.is_empty(), expected == 0);
            assert!(grant_events.iter().all(|(index, event)| {
                *index < offer_event
                    && event.cause().command_sequence()
                        < resolution.events()[offer_event].cause().command_sequence()
            }));
            let base_drop = match modifier {
                Some("9055") => 60,
                Some("9159") => 52,
                _ => 40,
            };
            assert_eq!(
                currency_balance(&activity, key(&flow)) - before,
                expected + base_drop
            );
            if modifier == Some("9055") {
                assert_eq!(reward_draws(&activity), draws);
            } else {
                assert_eq!(reward_draws(&activity) - draws, 3);
            }
            let bytes = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&flow, &mut activity, hash, projected, None)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
        }
    }
}

#[test]
fn evolved_green_battle_grants_use_only_current_form_and_stop_when_destroyed() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for (target, amount) in [("9196", 16), ("9197", 32)] {
            for destroyed in [false, true] {
                let (flow, mut activity) = ready(&fixture, family);
                let hash = activity.state_hash();
                curios
                    .acquire_accepted_state(&mut activity, hash, &state("9195"))
                    .unwrap();
                let hash = activity.state_hash();
                curios
                    .evolve_accepted_state(&mut activity, hash, &state("9195"), &state("9196"))
                    .unwrap();
                if target == "9197" {
                    let hash = activity.state_hash();
                    curios
                        .evolve_accepted_state(&mut activity, hash, &state("9196"), &state("9197"))
                        .unwrap();
                }
                if destroyed {
                    let hash = activity.state_hash();
                    curios
                        .destroy_accepted(&mut activity, hash, &state(target))
                        .unwrap();
                }
                let actual = result(&fixture, &flow, &mut activity);
                let projected = project(&actual, 4, BattleOutcome::Won);
                let before = currency_balance(&activity, key(&flow));
                let hash = activity.state_hash();
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&flow, &mut activity, hash, projected, None)
                    .unwrap();
                assert_eq!(
                    currency_balance(&activity, key(&flow)) - before,
                    40 + if destroyed { 0 } else { 4 * amount }
                );
            }
        }
    }
}

#[test]
fn green_battle_grant_obeys_lifecycle_and_loss_gates() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for (lifecycle, outcome, expected) in [
        (0, BattleOutcome::Won, 0),
        (1, BattleOutcome::Won, 0),
        (2, BattleOutcome::Won, 32),
        (3, BattleOutcome::Won, 0),
        (2, BattleOutcome::Lost, 0),
    ] {
        let (flow, mut activity) = ready(&fixture, DivergentUniverseRunFamily::Ordinary);
        if lifecycle != 0 {
            let hash = activity.state_hash();
            curios
                .acquire_accepted_state(&mut activity, hash, &state("9195"))
                .unwrap();
            if lifecycle == 3 {
                let hash = activity.state_hash();
                curios
                    .replace_accepted(&mut activity, hash, &state("9195"), &state("9028"))
                    .unwrap();
            } else {
                let hash = activity.state_hash();
                curios
                    .destroy_accepted(&mut activity, hash, &state("9195"))
                    .unwrap();
                if lifecycle == 2 {
                    let hash = activity.state_hash();
                    curios
                        .repair_accepted(&mut activity, hash, &state("9195"))
                        .unwrap();
                }
            }
        }
        let before = currency_balance(&activity, key(&flow));
        let actual = result(&fixture, &flow, &mut activity);
        let projected = project(&actual, 4, outcome);
        let draws = reward_draws(&activity);
        let hash = activity.state_hash();
        fixture
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(&flow, &mut activity, hash, projected, None)
            .unwrap();
        assert_eq!(
            currency_balance(&activity, key(&flow)) - before,
            expected + if outcome == BattleOutcome::Won { 40 } else { 0 }
        );
        if outcome == BattleOutcome::Lost {
            assert_eq!(reward_draws(&activity), draws);
        }
    }
}

#[test]
fn green_battle_grant_overflow_restores_pending_result_carry_and_blessing_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = ready(&fixture, DivergentUniverseRunFamily::Ordinary);
    let hash = activity.state_hash();
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .acquire_accepted_state(&mut activity, hash, &state("9195"))
        .unwrap();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(23_921).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key: key(&flow),
            // The base 40-credit stage succeeds; only the later Green reward
            // overflows. Both stages and the pending result must roll back.
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX - 40)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let actual = result(&fixture, &flow, &mut activity);
    let full = project(&actual, 4, BattleOutcome::Won);
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    for _ in 0..2 {
        let hash = activity.state_hash();
        assert!(
            fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, full.clone(), None)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
        assert_eq!(activity.player_view().completed_battle_count(), 0);
    }
    let hash = activity.state_hash();
    fixture
        .factory()
        .battle_settlement_runtime()
        .settle_started_result(
            &flow,
            &mut activity,
            hash,
            project(&actual, 0, BattleOutcome::Won),
            None,
        )
        .unwrap();
    assert_eq!(currency_balance(&activity, key(&flow)), i64::MAX);
    assert_eq!(activity.player_view().completed_battle_count(), 1);
}

#[test]
fn green_battle_grant_executes_after_public_acquisition_and_fresh_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let excluded = ["9055", "9070", "9079", "9159"].map(state);
    // Discover a positive real-battle case through a bounded public corpus.
    // Configuration identity changes draws; never preload the Curio or fake HP.
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut found = false;
        for seed in 0..4096 {
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
            let event = runner
                .advance_selected(
                    fixture.factory(),
                    &flow,
                    &mut activity,
                    fixture.core(),
                    &policy,
                    DivergentUniverseOfferedSelection::new(
                        decision,
                        ActivityOptionId::new(2).unwrap(),
                    ),
                )
                .unwrap();
            let owned = curios.owned(&activity).unwrap();
            if !owned.iter().any(|curio| curio.state() == &state("9195"))
                || owned.iter().any(|curio| excluded.contains(curio.state()))
            {
                continue;
            }
            let mut steps = vec![initial, event];
            let mut positive_reward = false;
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < 12);
                let before = currency_balance(&activity, key(&flow));
                let is_battle = matches!(flow.offered_encounter(&activity), Ok(Some(_)));
                let owned = curios.owned(&activity).unwrap();
                let bonuses = fixture
                    .factory()
                    .decision_catalog()
                    .curio_fragment_gains()
                    .iter()
                    .filter(|bonus| {
                        owned.iter().any(|owned| {
                            owned.state() == &bonus.state
                                && owned.lifecycle() == DivergentUniverseCurioLifecycleState::Active
                        })
                    })
                    .collect::<Vec<_>>();
                if flow.offered_evolution_event(&activity).is_some() {
                    // Keep the base form for this dedicated victory-grant
                    // producer test. Full upgrades have separate public tests.
                    let decision = activity.player_view().decision().unwrap().id();
                    steps.push(
                        runner
                            .advance_selected(
                                fixture.factory(),
                                &flow,
                                &mut activity,
                                fixture.core(),
                                &policy,
                                DivergentUniverseOfferedSelection::new(
                                    decision,
                                    ActivityOptionId::new(2).unwrap(),
                                ),
                            )
                            .unwrap(),
                    );
                    continue;
                }
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
                if is_battle {
                    let count = activity
                        .player_view()
                        .participant_carry()
                        .iter()
                        .filter(|participant| {
                            participant.life() == LifeState::Alive
                                && participant.presence() == PresenceState::Present
                                && participant.current_hp() == participant.maximum_hp()
                        })
                        .count();
                    let base = 8 * i64::try_from(count).unwrap();
                    let bonus = bonuses
                        .iter()
                        .map(|bonus| {
                            base * i64::from(bonus.numerator) / i64::from(bonus.denominator)
                        })
                        .sum::<i64>();
                    let battle_base = 40;
                    let battle_bonus = bonuses
                        .iter()
                        .map(|bonus| {
                            battle_base * i64::from(bonus.numerator) / i64::from(bonus.denominator)
                        })
                        .sum::<i64>();
                    if activity.player_view().terminal().is_some() {
                        // A keep-event reward can acquire the suppressor. The
                        // final battle then skips its offer and clears run
                        // currency in the same transaction. Its post-finalize
                        // balance is not an intermediate grant observation.
                        assert_eq!(currency_balance(&activity, key(&flow)), 0);
                    } else {
                        assert_eq!(
                            currency_balance(&activity, key(&flow)) - before,
                            base + bonus + battle_base + battle_bonus
                        );
                        positive_reward |= count > 0;
                    }
                }
            }
            if !positive_reward {
                continue;
            }
            let recorded =
                record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                    .unwrap();
            let bytes = encode_divergent_universe_replay(&recorded).unwrap();
            let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
            assert!((9..=11).contains(&verified.action_count()));
            assert_eq!(verified.terminal(), ActivityTerminalOutcome::Completed);
            found = true;
            break;
        }
        assert!(
            found,
            "bounded public corpus must acquire the Curio and earn a positive real-battle grant"
        );
    }
}
