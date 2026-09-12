//! Exact allowance vectors plus authenticated production route integration.

use super::{currency_balance, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioRuntime, DivergentUniverseFlowInstance,
    domain_choices::set_domain,
    economy::DivergentUniverseCurrencyKind,
    state::{CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT},
    tests::{
        curio_battle_grants::ready, curio_battle_stats::assert_public_speed_passive,
        domain_choices::advance,
    },
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityExpression, ActivityMasterSeed,
    ActivityOperation, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngLabel, ActivityValue, GraphActivity,
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
fn charges(
    curios: &DivergentUniverseCurioRuntime,
    activity: &GraphActivity,
    id: &DivergentUniverseCurioStateId,
) -> Option<u16> {
    curios
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == id)
        .map(|held| held.charges())
}
fn enter_vector(curios: &DivergentUniverseCurioRuntime, activity: &mut GraphActivity) {
    let operations = curios
        .domain_entry_operations(&activity.player_view())
        .unwrap();
    if operations.is_empty() {
        return;
    }
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(24100).unwrap(), operations).unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
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
    panic!("bounded current route must offer a domain");
}

#[test]
fn domain_expiry_exact_allowance_discard_pause_repair_and_reacquire_vectors() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    // Pure entry-operation vectors, not a claim of five domains in the current
    // three-layer route. Public-route integration is tested independently below.
    for (raw, limit) in [("9070", 3), ("9079", 5)] {
        let (_, mut activity) = ready(&fixture, FAMILIES[0]);
        let id = state(raw);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &id)
            .unwrap();
        assert_eq!(charges(&curios, &activity, &id), Some(limit));
        let bytes = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert!(
            curios
                .set_accepted_charges(&mut activity, hash, &id, limit + 1)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        enter_vector(&curios, &mut activity);
        assert_eq!(charges(&curios, &activity, &id), Some(limit - 1));
        let hash = activity.state_hash();
        curios.destroy_accepted(&mut activity, hash, &id).unwrap();
        let destroyed = activity.canonical_state_bytes();
        enter_vector(&curios, &mut activity);
        assert_eq!(activity.canonical_state_bytes(), destroyed);
        let hash = activity.state_hash();
        curios.repair_accepted(&mut activity, hash, &id).unwrap();
        assert_eq!(charges(&curios, &activity, &id), Some(limit - 1));
        for remaining in (1..limit).rev() {
            assert_eq!(charges(&curios, &activity, &id), Some(remaining));
            enter_vector(&curios, &mut activity);
        }
        assert_eq!(charges(&curios, &activity, &id), None);
        let hash = activity.state_hash();
        assert!(curios.repair_accepted(&mut activity, hash, &id).is_err());
        let key = curios
            .states()
            .iter()
            .find(|row| row.id() == &id)
            .unwrap()
            .state_key();
        for slot in [
            CURIO_STATES_SLOT,
            CURIO_CHARGES_SLOT,
            CURIO_ACTIVATIONS_SLOT,
        ] {
            let view = activity.player_view();
            let value = view
                .slots()
                .iter()
                .find(|row| row.id() == slot)
                .unwrap()
                .value();
            assert!(
                matches!(value, ActivityValue::BoundedCounterMap(entries) if !entries.iter().any(|entry| entry.0 == key))
            );
        }
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &id)
            .unwrap();
        assert_eq!(charges(&curios, &activity, &id), Some(limit));
        let other = state(if raw == "9070" { "9079" } else { "9070" });
        let hash = activity.state_hash();
        curios
            .replace_accepted(&mut activity, hash, &id, &other)
            .unwrap();
        assert_eq!(charges(&curios, &activity, &id), None);
        assert_eq!(
            charges(&curios, &activity, &other),
            Some(if raw == "9070" { 5 } else { 3 })
        );
    }
}

#[test]
fn domain_expiry_public_entry_discards_before_reward_and_rejects_repeated_or_bypassed_choices() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for raw in ["9070", "9079"] {
            let id = state(raw);
            let (flow, mut activity) = ready(&fixture, family);
            let hash = activity.state_hash();
            curios
                .acquire_accepted_state(&mut activity, hash, &id)
                .unwrap();
            // Counterfactual one-entry remainder reaches real discard in this
            // short route; exact initial 3/5 allowances have separate vectors.
            let hash = activity.state_hash();
            curios
                .set_accepted_charges(&mut activity, hash, &id, 1)
                .unwrap();
            at_route(&fixture, &flow, &mut activity);
            assert_eq!(
                charges(&curios, &activity, &id),
                Some(1),
                "battle/reward/initializer are not entries"
            );
            let view = activity.player_view();
            let decision = view.decision().unwrap().id();
            let option = ActivityOptionId::new(2).unwrap();
            let bytes = activity.canonical_state_bytes();
            assert!(
                activity
                    .choose_option(view.state_hash(), decision, option)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert!(
                flow.choose_battle_domain(
                    &mut activity,
                    view.state_hash(),
                    decision,
                    ActivityOptionId::new(4).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            // Generated discard and RNG followed by a rejecting operation must
            // restore the old holding, counters, route and random streams.
            let draws = reward_draws(&activity);
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
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(reward_draws(&activity), draws);
            flow.choose_battle_domain(&mut activity, view.state_hash(), decision, option)
                .unwrap();
            assert_eq!(charges(&curios, &activity, &id), None);
            let after = activity.canonical_state_bytes();
            assert!(
                flow.choose_battle_domain(&mut activity, view.state_hash(), decision, option)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), after);
            let key = flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            let balance = currency_balance(&activity, key);
            advance(&fixture, &flow, &mut activity, None);
            assert_eq!(
                currency_balance(&activity, key) - balance,
                100,
                "discarded bonus cannot affect Elite income"
            );
        }
    }
}

#[test]
fn domain_expiry_controlled_acquisition_future_entries_and_fresh_reconstruction() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for (raw, initial) in [("9070", 3), ("9079", 5)] {
            let mut traces = Vec::new();
            for current in [&fixture, &fresh] {
                let curios = current.factory().curio_runtime().unwrap();
                let (flow, mut activity) = ready(current, family);
                let id = state(raw);
                let hash = activity.state_hash();
                curios
                    .acquire_accepted_state(&mut activity, hash, &id)
                    .unwrap();
                let mut hashes = vec![activity.state_hash()];
                let mut entries = 0;
                while activity.player_view().terminal().is_none() {
                    assert!(hashes.len() < 12);
                    assert_eq!(charges(&curios, &activity, &id), Some(initial - entries));
                    let route = flow
                        .offered_battle_domain_choices(&activity)
                        .unwrap()
                        .is_some();
                    advance(current, &flow, &mut activity, None);
                    if route {
                        entries += 1;
                    }
                    hashes.push(activity.state_hash());
                }
                assert_eq!(entries, 2);
                traces.push(hashes);
            }
            assert_eq!(traces[0], traces[1]);
        }
    }
}

#[test]
fn domain_expiry_public_canonical_state_exclusion_and_route_transcript_replay() {
    use crate::divergent_universe::{
        encode_divergent_universe_replay, record_divergent_universe_transcript,
        verify_divergent_universe_replay,
    };
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow(family).unwrap();
        // The existing per-identity minimum-state policy selects 9068, not
        // either reviewed expiry state. Never change that pool for this test.
        let owner = curios
            .states()
            .iter()
            .find(|row| row.id() == &state("9070"))
            .unwrap()
            .curio()
            .unwrap();
        let canonical = curios
            .states()
            .iter()
            .filter(|row| row.curio() == Some(owner))
            .min_by(|left, right| left.id().cmp(right.id()))
            .unwrap();
        assert_eq!(canonical.id(), &state("9068"));
        let (seed, mut activity, mut steps) = (0..2048)
            .find_map(|seed| {
                let mut activity = flow
                    .start(instance(1), ActivityMasterSeed::from_u64(seed))
                    .unwrap()
                    .into_activity();
                let first = advance(&fixture, &flow, &mut activity, None);
                let color = advance(&fixture, &flow, &mut activity, Some(2));
                let owned = curios.owned(&activity).unwrap();
                assert!(
                    !owned
                        .iter()
                        .any(|held| [state("9070"), state("9079")].contains(held.state()))
                );
                owned
                    .iter()
                    .any(|held| held.state() == canonical.id())
                    .then_some((seed, activity, vec![first, color]))
            })
            .expect("bounded public acquisition corpus includes the canonical state");
        let mut entries = 0;
        let mut battles = 0;
        while activity.player_view().terminal().is_none() {
            assert!(steps.len() < 14);
            let route = flow
                .offered_battle_domain_choices(&activity)
                .unwrap()
                .is_some();
            assert_eq!(
                charges(&curios, &activity, &state("9068")),
                Some(5 - battles),
                "speed state consumes battles, not another state's domain allowance"
            );
            let battle = activity.player_view().decision().unwrap().kind()
                == ActivityDecisionKind::Encounter;
            if battle {
                assert_public_speed_passive(&fixture, &flow, &activity);
            }
            steps.push(advance(&fixture, &flow, &mut activity, None));
            if battle {
                battles += 1;
            }
            if route {
                entries += 1;
            }
        }
        assert_eq!(entries, 2);
        let recorded =
            record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps).unwrap();
        let replay = verify_divergent_universe_replay(
            &encode_divergent_universe_replay(&recorded).unwrap(),
            &fresh,
        )
        .unwrap();
        assert_eq!(
            replay.final_state_hash().bytes(),
            activity.state_hash().bytes()
        );
    }
}
