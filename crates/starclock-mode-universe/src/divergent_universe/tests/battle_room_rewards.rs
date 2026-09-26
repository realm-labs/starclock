//! Authored Boss reward policy on fixed source positions, not original Boss AI.

use super::{Scenario, advance, compile_rooms, prepare, start};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineStep,
    domain_route::DomainRoomComposition,
    economy::DivergentUniverseCurrencyKind,
    state::CURRENCIES_SLOT,
    tests::{curio_battle_grants::project, currency_balance, reward_draws},
};
use starclock_activity::{
    ActivityDecisionKind, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityTerminalOutcome, ActivityValue, BattleOutcome, BattleResult,
    GraphActivity, ProjectedValue,
};
use starclock_combat::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_layout::FixedDomainKind,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn boss_scenario(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> Scenario {
    compile_rooms(fixture, family, &[BattleRewardDomain::Boss], |context| {
        (context.composition == DomainRoomComposition::Fixed(FixedDomainKind::Boss)).then_some(0)
    })
}

fn ready(fixture: &DivergentUniverseBaselineFixture, scenario: &Scenario) -> GraphActivity {
    let mut activity = start(scenario);
    while activity.player_view().decision().unwrap().kind() != ActivityDecisionKind::Encounter {
        advance(fixture, scenario, &mut activity);
    }
    activity
}

fn wallet(scenario: &Scenario) -> u64 {
    scenario
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key()
}

#[test]
fn boss_rewards_execute_each_fixed_plane_position_and_reconstruct_from_fresh_inputs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let scenario = boss_scenario(&fixture, family);
        let fresh = boss_scenario(&fixture, family);
        let mut activity = start(&scenario);
        let mut reconstructed = start(&fresh);
        let mut planes = Vec::new();
        while activity.player_view().terminal().is_none() {
            let encounter = activity.player_view().decision().unwrap().kind()
                == ActivityDecisionKind::Encounter;
            let before = currency_balance(&activity, wallet(&scenario));
            if encounter {
                let room = scenario
                    .rooms
                    .iter()
                    .find(|room| room.context().node(1).unwrap() == activity.current_node())
                    .unwrap();
                assert_eq!(
                    room.context().composition,
                    DomainRoomComposition::Fixed(FixedDomainKind::Boss)
                );
                assert_eq!(
                    (room.context().preset_source.as_ref(), room.context().level),
                    ("1002", 1)
                );
                planes.push(room.context().plane_ordinal);
                assert_eq!(
                    scenario.flow.current_battle_domain(&activity).unwrap(),
                    Some(BattleRewardDomain::Boss)
                );
            }
            let step = advance(&fixture, &scenario, &mut activity);
            assert_eq!(step, advance(&fixture, &fresh, &mut reconstructed));
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
            assert_eq!(activity.debug_view(), reconstructed.debug_view());
            if encounter {
                assert!(matches!(
                    step,
                    Some(DivergentUniverseBaselineStep::Battle {
                        outcome: BattleOutcome::Won,
                        ..
                    })
                ));
                assert_eq!(currency_balance(&activity, wallet(&scenario)) - before, 100);
                assert_eq!(
                    activity.player_view().decision().unwrap().kind(),
                    ActivityDecisionKind::Reward
                );
            }
        }
        assert_eq!(planes, [1, 2, 3]);
        assert_eq!(activity.player_view().completed_battle_count(), 3);
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        // Other positions use explicit probes. This proves the reward policy,
        // never source Boss programs, complete room payloads or release parity.
    }
}

#[test]
fn boss_fragment_credit_applies_live_modifiers_and_suppression_without_reward_rng_or_duplicate_grants()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for (states, amount) in [
            (vec!["9055"], 150),
            (vec!["9070"], 150),
            (vec!["9079"], 130),
            (vec!["9159"], 130),
            (vec!["9055", "9070", "9159"], 230),
        ] {
            let scenario = boss_scenario(&fixture, family);
            let mut activity = ready(&fixture, &scenario);
            let ids = states
                .iter()
                .map(|state| {
                    DivergentUniverseCurioStateId::new(format!(
                        "divergent-universe.curio-state.{state}"
                    ))
                    .unwrap()
                })
                .collect::<Vec<_>>();
            // Controlled trusted acquisition checks the established modifiers;
            // it does not claim these Curios were offered by the probe rooms.
            let hash = activity.state_hash();
            curios
                .acquire_accepted_states(&mut activity, hash, &ids)
                .unwrap();
            let before = currency_balance(&activity, wallet(&scenario));
            let draws = reward_draws(&activity);
            let (result, report) = prepare(&fixture, &scenario, &mut activity)
                .execute()
                .unwrap();
            let hash = activity.state_hash();
            let settled = fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(
                    &scenario.flow,
                    &mut activity,
                    hash,
                    result.clone(),
                    Some(report),
                )
                .unwrap();
            assert_eq!(
                currency_balance(&activity, wallet(&scenario)) - before,
                amount
            );
            let suppressed = states.contains(&"9055");
            assert_eq!(
                reward_draws(&activity) - draws,
                if suppressed { 0 } else { 3 }
            );
            assert_eq!(
                activity.player_view().decision().unwrap().kind(),
                if suppressed {
                    ActivityDecisionKind::Route
                } else {
                    ActivityDecisionKind::Reward
                }
            );
            let base = settled
                .events()
                .iter()
                .position(|event| event.cause().program().get() == 24_060)
                .unwrap();
            let offer = settled
                .events()
                .iter()
                .position(|event| event.cause().program().get() == 23_702)
                .unwrap();
            assert!(base < offer);
            let bytes = activity.canonical_state_bytes();
            let debug = activity.debug_view();
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&scenario.flow, &mut activity, hash, result, None)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(activity.debug_view(), debug);
        }
    }
}

#[test]
fn boss_counterfactual_loss_and_fault_have_no_fragment_or_blessing_rewards() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for (outcome, terminal) in [
            (BattleOutcome::Lost, ActivityTerminalOutcome::Failed),
            (BattleOutcome::Faulted, ActivityTerminalOutcome::Faulted),
        ] {
            let scenario = boss_scenario(&fixture, family);
            let mut activity = ready(&fixture, &scenario);
            let before = currency_balance(&activity, wallet(&scenario));
            let draws = reward_draws(&activity);
            let (actual, _) = prepare(&fixture, &scenario, &mut activity)
                .execute()
                .unwrap();
            let projected = project(&actual, 0, outcome);
            let projected = BattleResult::seal(
                projected.identity(),
                projected
                    .values()
                    .iter()
                    .map(|value| {
                        if matches!(value, ProjectedValue::TerminalFault(_))
                            && outcome == BattleOutcome::Faulted
                        {
                            ProjectedValue::TerminalFault(Some(BattleFault::from_parts(
                                FaultKind::Numeric,
                                FaultBoundary::Command,
                                FaultPolicy::Rollback,
                                1,
                                None,
                            )))
                        } else {
                            value.clone()
                        }
                    })
                    .collect(),
            );
            let hash = activity.state_hash();
            fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&scenario.flow, &mut activity, hash, projected, None)
                .unwrap();
            assert_eq!(activity.player_view().terminal(), Some(terminal));
            assert_eq!(currency_balance(&activity, wallet(&scenario)), before);
            assert_eq!(reward_draws(&activity), draws);
        }
    }
}

#[test]
fn boss_later_curio_overflow_restores_the_successful_base_credit_and_pending_carry() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let scenario = boss_scenario(&fixture, family);
        let mut activity = ready(&fixture, &scenario);
        let hash = activity.state_hash();
        fixture
            .factory()
            .curio_runtime()
            .unwrap()
            .acquire_accepted_state(
                &mut activity,
                hash,
                &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9195").unwrap(),
            )
            .unwrap();
        let boundary = ActivityProgramDefinition::new(
            ActivityProgramId::new(23_921).unwrap(),
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet(&scenario),
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX - 100)),
            }],
        )
        .unwrap();
        activity
            .apply_boundary_program(activity.state_hash(), &boundary)
            .unwrap();
        let (actual, _) = prepare(&fixture, &scenario, &mut activity)
            .execute()
            .unwrap();
        // Counterfactual full-HP carry isolates the later Green credit overflow.
        let full = project(&actual, 4, BattleOutcome::Won);
        let bytes = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        for _ in 0..2 {
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&scenario.flow, &mut activity, hash, full.clone(), None,)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(activity.debug_view(), debug);
            assert_eq!(activity.player_view().completed_battle_count(), 0);
        }
        let hash = activity.state_hash();
        fixture
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(
                &scenario.flow,
                &mut activity,
                hash,
                project(&actual, 0, BattleOutcome::Won),
                None,
            )
            .unwrap();
        assert_eq!(currency_balance(&activity, wallet(&scenario)), i64::MAX);
        assert_eq!(activity.player_view().completed_battle_count(), 1);
    }
}
