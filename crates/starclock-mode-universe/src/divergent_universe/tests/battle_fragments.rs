//! Public fixed-domain drops and counterfactual failure/overflow boundaries.

use super::{currency_balance, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseFlowInstance,
    economy::DivergentUniverseCurrencyKind,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    tests::{
        curio_battle_grants::{project, ready, result},
        domain_choices::advance,
    },
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityTerminalOutcome, BattleOutcome, BattleResult,
    ProjectedValue,
};
use starclock_combat::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn key(flow: &DivergentUniverseFlowInstance) -> u64 {
    flow.economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key()
}
fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

#[test]
fn base_battle_fragments_public_domains_credit_and_replay_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow(family).unwrap();
        let seed = 24_060;
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(seed))
            .unwrap()
            .into_activity();
        let mut steps = Vec::new();
        let mut battles = 0;
        let mut domains = 0;
        while activity.player_view().terminal().is_none() {
            assert!(steps.len() < 10);
            let view = activity.player_view();
            let battle = view.decision().unwrap().kind() == ActivityDecisionKind::Encounter;
            let selection = if flow
                .offered_battle_domain_choices(&activity)
                .unwrap()
                .is_some()
            {
                domains += 1;
                Some(domains + 1)
            } else {
                None
            };
            let before = currency_balance(&activity, key(&flow));
            let draws = reward_draws(&activity);
            steps.push(advance(&fixture, &flow, &mut activity, selection));
            if battle {
                assert_eq!(
                    currency_balance(&activity, key(&flow)) - before,
                    [40, 100, 40][battles]
                );
                assert_eq!(reward_draws(&activity) - draws, 3);
                battles += 1;
            }
        }
        assert_eq!((domains, battles, steps.len()), (2, 3, 10));
        assert_eq!(currency_balance(&activity, key(&flow)), 0);
        let recorded =
            record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps).unwrap();
        let verified = verify_divergent_universe_replay(
            &encode_divergent_universe_replay(&recorded).unwrap(),
            &fresh,
        )
        .unwrap();
        assert_eq!(verified.action_count(), 10);
        assert_eq!(verified.battle_count(), 3);
        assert_eq!(
            verified.final_state_hash().bytes(),
            activity.state_hash().bytes()
        );
    }
}

#[test]
fn base_battle_fragments_apply_live_bonuses_even_when_blessings_are_suppressed() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for (modifiers, amount) in [
            (vec!["9055"], 60),
            (vec!["9070"], 60),
            (vec!["9079"], 52),
            (vec!["9159"], 52),
            (vec!["9055", "9070", "9159"], 92),
        ] {
            let (flow, mut activity) = ready(&fixture, family);
            let ids = modifiers.iter().map(|id| state(id)).collect::<Vec<_>>();
            let hash = activity.state_hash();
            curios
                .acquire_accepted_states(&mut activity, hash, &ids)
                .unwrap();
            let before = currency_balance(&activity, key(&flow));
            let draws = reward_draws(&activity);
            let won = result(&fixture, &flow, &mut activity);
            let hash = activity.state_hash();
            let settled = fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, won.clone(), None)
                .unwrap();
            assert_eq!(currency_balance(&activity, key(&flow)) - before, amount);
            let suppressed = modifiers.contains(&"9055");
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
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&flow, &mut activity, hash, won, None)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
        }
        for transition in [0, 1, 2] {
            let (flow, mut activity) = ready(&fixture, family);
            let hash = activity.state_hash();
            curios
                .acquire_accepted_state(&mut activity, hash, &state("9055"))
                .unwrap();
            let hash = activity.state_hash();
            curios
                .destroy_accepted(&mut activity, hash, &state("9055"))
                .unwrap();
            let hash = activity.state_hash();
            if transition == 1 {
                curios
                    .repair_accepted(&mut activity, hash, &state("9055"))
                    .unwrap();
            }
            if transition == 2 {
                curios
                    .replace_accepted(&mut activity, hash, &state("9055"), &state("9001"))
                    .unwrap();
            }
            let before = currency_balance(&activity, key(&flow));
            let won = result(&fixture, &flow, &mut activity);
            let hash = activity.state_hash();
            fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, won, None)
                .unwrap();
            assert_eq!(
                currency_balance(&activity, key(&flow)) - before,
                if transition == 1 { 60 } else { 40 }
            );
        }
    }
}

#[test]
fn base_battle_fragments_do_not_credit_or_draw_on_loss_or_fault() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for (outcome, terminal) in [
            (BattleOutcome::Lost, ActivityTerminalOutcome::Failed),
            (BattleOutcome::Faulted, ActivityTerminalOutcome::Faulted),
        ] {
            let (flow, mut activity) = ready(&fixture, family);
            let before = currency_balance(&activity, key(&flow));
            let draws = reward_draws(&activity);
            let actual = result(&fixture, &flow, &mut activity);
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
            let settled = fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, projected, None)
                .unwrap();
            assert_eq!(currency_balance(&activity, key(&flow)), before);
            assert_eq!(reward_draws(&activity), draws);
            assert!(
                !settled
                    .events()
                    .iter()
                    .any(|event| event.cause().program().get() == 24_060)
            );
            assert_eq!(activity.player_view().terminal(), Some(terminal));
        }
    }
}
