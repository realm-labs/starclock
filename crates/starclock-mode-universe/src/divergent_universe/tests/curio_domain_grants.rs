//! Entry income must change real balances, not only allowance/catalog markers.

use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioRuntime, DivergentUniverseFlowInstance,
    domain_choices::set_domain,
    economy::DivergentUniverseCurrencyKind,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    state::CURRENCIES_SLOT,
    tests::{
        curio_battle_grants::ready, currency_balance, domain_choices::advance, instance,
        reward_draws,
    },
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel, ActivityTerminalOutcome,
    ActivityValue, GraphActivity,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
fn charges(curios: &DivergentUniverseCurioRuntime, activity: &GraphActivity) -> Option<u16> {
    curios
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == &state("9071"))
        .map(|held| held.charges())
}
fn key(flow: &DivergentUniverseFlowInstance) -> u64 {
    flow.economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key()
}
fn acquire(curios: &DivergentUniverseCurioRuntime, activity: &mut GraphActivity, raw: &str) {
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(activity, hash, &state(raw))
        .unwrap();
}
fn apply(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    if operations.is_empty() {
        return;
    }
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(24100).unwrap(), operations).unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}
fn enter_vector(curios: &DivergentUniverseCurioRuntime, activity: &mut GraphActivity) {
    apply(
        activity,
        curios
            .domain_entry_operations(&activity.player_view())
            .unwrap(),
    );
}
fn set_balance(activity: &mut GraphActivity, key: u64, value: i64) {
    apply(
        activity,
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(value)),
        }],
    );
}
fn at_route(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
) {
    for _ in 0..6 {
        if flow
            .offered_battle_domain_choices(activity)
            .unwrap()
            .is_some()
        {
            return;
        }
        advance(fixture, flow, activity, None);
    }
    panic!("bounded route did not offer domains");
}

#[test]
fn curio_domain_grants_three_entry_vectors_include_final_grant_and_active_bonuses() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for (bonuses, expected) in [
            (vec![], 60),
            (vec!["9055"], 90),
            (vec!["9159"], 78),
            (vec!["9055", "9159"], 108),
        ] {
            let (flow, mut activity) = ready(&fixture, family);
            for raw in bonuses {
                acquire(&curios, &mut activity, raw);
            }
            let balance = currency_balance(&activity, key(&flow));
            acquire(&curios, &mut activity, "9071");
            assert_eq!(charges(&curios, &activity), Some(3));
            assert_eq!(
                currency_balance(&activity, key(&flow)),
                balance,
                "acquisition is not a future entry"
            );
            let before = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                curios
                    .set_accepted_charges(&mut activity, hash, &state("9071"), 4)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            for remaining in (1..=3).rev() {
                // Three domain-operation vectors, not three public route entries.
                assert_eq!(charges(&curios, &activity), Some(remaining));
                assert!(
                    curios
                        .battle_lifetime_operations(&activity.player_view())
                        .unwrap()
                        .is_empty()
                );
                let balance = currency_balance(&activity, key(&flow));
                let draws = reward_draws(&activity);
                enter_vector(&curios, &mut activity);
                assert_eq!(currency_balance(&activity, key(&flow)) - balance, expected);
                assert_eq!(reward_draws(&activity), draws);
            }
            assert_eq!(charges(&curios, &activity), None);
            let after = activity.canonical_state_bytes();
            enter_vector(&curios, &mut activity);
            assert_eq!(activity.canonical_state_bytes(), after);
        }
    }
}

#[test]
fn curio_domain_grants_destruction_zero_replacement_and_reacquisition_vectors() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        acquire(&curios, &mut activity, "9071");
        enter_vector(&curios, &mut activity);
        assert_eq!(charges(&curios, &activity), Some(2));
        let hash = activity.state_hash();
        curios
            .destroy_accepted(&mut activity, hash, &state("9071"))
            .unwrap();
        let before = activity.canonical_state_bytes();
        enter_vector(&curios, &mut activity);
        assert_eq!(activity.canonical_state_bytes(), before);
        let hash = activity.state_hash();
        curios
            .repair_accepted(&mut activity, hash, &state("9071"))
            .unwrap();
        assert_eq!(charges(&curios, &activity), Some(2));
        let balance = currency_balance(&activity, key(&flow));
        enter_vector(&curios, &mut activity);
        assert_eq!(currency_balance(&activity, key(&flow)) - balance, 60);
        for (from, to) in [("9071", "9073"), ("9073", "9071")] {
            let hash = activity.state_hash();
            curios
                .replace_accepted(&mut activity, hash, &state(from), &state(to))
                .unwrap();
        }
        assert_eq!(charges(&curios, &activity), Some(3));
        let hash = activity.state_hash();
        curios
            .set_accepted_charges(&mut activity, hash, &state("9071"), 0)
            .unwrap();
        let balance = currency_balance(&activity, key(&flow));
        enter_vector(&curios, &mut activity);
        assert_eq!(charges(&curios, &activity), None);
        assert_eq!(currency_balance(&activity, key(&flow)), balance);
        acquire(&curios, &mut activity, "9071");
        assert_eq!(charges(&curios, &activity), Some(3));
        assert_eq!(currency_balance(&activity, key(&flow)), balance);
    }
}

#[test]
fn curio_domain_grants_public_choices_reject_bypass_overflow_and_late_failure_atomically() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for ordinal in 1..=3 {
            let (flow, mut activity) = ready(&fixture, family);
            acquire(&curios, &mut activity, "9071");
            let hash = activity.state_hash();
            curios
                .set_accepted_charges(&mut activity, hash, &state("9071"), 1)
                .unwrap();
            at_route(&fixture, &flow, &mut activity);
            assert_eq!(
                charges(&curios, &activity),
                Some(1),
                "battles and graph nodes never consume entries"
            );
            acquire(&curios, &mut activity, "9055");
            let option = ActivityOptionId::new(ordinal).unwrap();
            for balance in [i64::MAX - 59, i64::MAX - 75] {
                // First overflows the base, second only the subsequent +50% bonus.
                set_balance(&mut activity, key(&flow), balance);
                let view = activity.player_view();
                let before = activity.canonical_state_bytes();
                let draws = reward_draws(&activity);
                assert!(
                    flow.choose_battle_domain(
                        &mut activity,
                        view.state_hash(),
                        view.decision().unwrap().id(),
                        option
                    )
                    .is_err()
                );
                assert_eq!(activity.canonical_state_bytes(), before);
                assert_eq!(reward_draws(&activity), draws);
            }
            set_balance(&mut activity, key(&flow), 100);
            let view = activity.player_view();
            let decision = view.decision().unwrap().id();
            let before = activity.canonical_state_bytes();
            let draws = reward_draws(&activity);
            assert!(
                activity
                    .choose_option(view.state_hash(), decision, option)
                    .is_err()
            );
            assert!(
                flow.choose_battle_domain(
                    &mut activity,
                    view.state_hash(),
                    decision,
                    ActivityOptionId::new(4).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert!(
                activity
                    .choose_option_with_generated_prefix(
                        view.state_hash(),
                        decision,
                        option,
                        |view, rng| {
                            let mut operations = curios.domain_entry_operations(view).unwrap();
                            rng.choose_index(ActivityRngLabel::Reward, 24100, 2)
                                .unwrap();
                            operations.push(set_domain(Some(BattleRewardDomain::Elite)));
                            operations.push(ActivityOperation::Require(
                                ActivityCondition::Boolean(ActivityExpression::Literal(
                                    ActivityValue::Boolean(false),
                                )),
                            ));
                            Ok((operations, ()))
                        }
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
            flow.choose_battle_domain(&mut activity, view.state_hash(), decision, option)
                .unwrap();
            assert_eq!(currency_balance(&activity, key(&flow)), 190);
            assert_eq!(charges(&curios, &activity), None);
            let after = activity.canonical_state_bytes();
            assert!(
                flow.choose_battle_domain(&mut activity, view.state_hash(), decision, option)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), after);
        }
    }
}

#[test]
fn curio_domain_grants_paid_public_acquisition_and_future_entries_replay_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let target = curios
        .states()
        .iter()
        .find(|s| s.id() == &state("9071"))
        .unwrap()
        .state_key();
    for family in FAMILIES {
        let flow = fixture.flow_with_tawot_service(family, 2).unwrap();
        let (seed, mut activity, mut steps) = (0..96)
            .find_map(|seed| {
                let mut activity = flow
                    .start(instance(1), ActivityMasterSeed::from_u64(seed))
                    .unwrap()
                    .into_activity();
                let mut steps = Vec::new();
                for _ in 0..3 {
                    steps.push(advance(&fixture, &flow, &mut activity, None));
                }
                steps.push(advance(&fixture, &flow, &mut activity, Some(1)));
                let offered = activity
                    .player_view()
                    .decision()
                    .unwrap()
                    .options()
                    .iter()
                    .any(|option| option.id().get() == target);
                offered.then_some((seed, activity, steps))
            })
            .expect("bounded paid service offers 9071");
        let balance = currency_balance(&activity, key(&flow));
        steps.push(advance(&fixture, &flow, &mut activity, Some(target)));
        assert_eq!(currency_balance(&activity, key(&flow)), balance - 100);
        assert_eq!(charges(&curios, &activity), Some(3));
        steps.push(advance(&fixture, &flow, &mut activity, Some(2)));
        let mut entries = 0;
        while activity.player_view().terminal().is_none() {
            assert!(steps.len() < 24);
            assert_eq!(charges(&curios, &activity), Some(3 - entries));
            let route = flow
                .offered_battle_domain_choices(&activity)
                .unwrap()
                .is_some();
            let balance = currency_balance(&activity, key(&flow));
            steps.push(advance(
                &fixture,
                &flow,
                &mut activity,
                if route {
                    Some(2 + u64::from(entries))
                } else {
                    None
                },
            ));
            if route {
                entries += 1;
                assert_eq!(currency_balance(&activity, key(&flow)) - balance, 60);
                assert_eq!(charges(&curios, &activity), Some(3 - entries));
            }
        }
        assert_eq!(
            entries, 2,
            "current route is not three-domain public exhaustion"
        );
        assert_eq!(activity.player_view().completed_battle_count(), 3);
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(
            charges(&curios, &activity),
            None,
            "run finalization clears remaining inventory"
        );
        let run =
            record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps).unwrap();
        let bytes = encode_divergent_universe_replay(&run).unwrap();
        let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
        assert_eq!(
            verified.final_state_hash().bytes(),
            activity.state_hash().bytes()
        );
        assert_eq!(verified.battle_count(), 3);
    }
}
