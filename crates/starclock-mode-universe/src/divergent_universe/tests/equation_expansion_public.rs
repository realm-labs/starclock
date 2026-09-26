//! Public paid acquisition and reward-trigger replay, without prepared holdings.

use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineStep, DivergentUniverseCurrencyKind,
    DivergentUniverseFlowInstance,
    decision_rewards::DecisionRewardGrant,
    encode_divergent_universe_replay,
    equation_progress::DivergentUniverseEquationExpansionState,
    record_divergent_universe_transcript,
    state::BATTLE_BLESSING_CANDIDATES_SLOT,
    tests::{
        contribution_keys, currency_balance, domain_choices::advance, instance, reward_draws,
        slot_value,
    },
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityRngContext, ActivityRngLabel,
    ActivityRngStreams, ActivityTerminalOutcome, ActivityValue, GraphActivity,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioCategory;

fn prefix(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    seed: u64,
    initial: usize,
) -> Option<(GraphActivity, Vec<DivergentUniverseBaselineStep>)> {
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(seed))
        .unwrap()
        .into_activity();
    let option = activity.player_view().decision().unwrap().options()[initial]
        .id()
        .get();
    let mut steps = vec![advance(fixture, flow, &mut activity, Some(option))];
    steps.push(advance(fixture, flow, &mut activity, Some(2)));
    let curios = fixture.factory().curio_runtime().unwrap();
    let held = curios.owned(&activity).unwrap();
    if !held
        .iter()
        .any(|held| held.state().as_str() == "divergent-universe.curio-state.9192")
        || !held
            .iter()
            .any(|held| held.state().as_str() == "divergent-universe.curio-state.9195")
    {
        return None;
    }
    let currency = flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    assert!(currency_balance(&activity, currency) >= 100);
    steps.push(advance(fixture, flow, &mut activity, None));
    steps.push(advance(fixture, flow, &mut activity, Some(1)));
    let target = curios
        .states()
        .iter()
        .find(|state| state.id().as_str() == "divergent-universe.curio-state.9074")
        .unwrap()
        .state_key();
    if !activity
        .player_view()
        .decision()
        .unwrap()
        .options()
        .iter()
        .any(|option| option.id().get() == target)
    {
        return None;
    }
    let before = currency_balance(&activity, currency);
    steps.push(advance(fixture, flow, &mut activity, Some(target)));
    assert_eq!(currency_balance(&activity, currency), before - 100);
    let held = curios.owned(&activity).unwrap();
    let purchased = held
        .iter()
        .find(|held| held.state().as_str() == "divergent-universe.curio-state.9074")
        .unwrap();
    assert_eq!((purchased.charges(), purchased.activations()), (3, 0));
    steps.push(advance(fixture, flow, &mut activity, Some(2)));
    Some((activity, steps))
}

fn finish(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    steps: &mut Vec<DivergentUniverseBaselineStep>,
) -> (u16, Option<ActivityDecisionKind>) {
    let progress = fixture.factory().equation_progress_runtime().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let mut remaining = 3;
    let mut trigger_boundary = None;
    let observations = progress.observations(activity).unwrap();
    let recipe = progress
        .recipes()
        .iter()
        .find(|recipe| recipe.equation() == observations[0].equation())
        .unwrap();
    let main = contribution_keys(fixture.factory(), recipe.equation(), recipe.main_path());
    let sub = contribution_keys(
        fixture.factory(),
        recipe.equation(),
        recipe.sub_path().unwrap(),
    );
    while activity.player_view().terminal().is_none() {
        assert!(steps.len() < 32);
        let before_count = blessings.owned(activity).unwrap().len();
        let before_draws = reward_draws(activity);
        let event = flow
            .offered_evolution_event(activity)
            .map(|event| event.key.clone());
        let kind = activity.player_view().decision().unwrap().kind();
        let base_grants = if let Some(event) = &event {
            match event.as_ref() {
                "du.evolution-event.sage.ii" => 2,
                "du.evolution-event.sage.iii" => 3,
                _ => 0,
            }
        } else if kind == ActivityDecisionKind::Encounter {
            match activity.player_view().completed_battle_count() {
                0 => 0,
                1 => 2,
                2 => 3,
                _ => panic!("three battles"),
            }
        } else if kind == ActivityDecisionKind::Reward {
            1
        } else {
            0
        };
        let option = if flow
            .offered_battle_domain_choices(activity)
            .unwrap()
            .is_some()
        {
            Some(2)
        } else if flow.offered_evolution_event(activity).is_some() {
            Some(1)
        } else if activity.player_view().decision().unwrap().kind() == ActivityDecisionKind::Reward
        {
            let view = activity.player_view();
            let ActivityValue::OrderedIdSet(candidates) =
                slot_value(&view, BATTLE_BLESSING_CANDIDATES_SLOT)
            else {
                panic!("candidate slot");
            };
            let observations = progress.observations(activity).unwrap();
            let observation = &observations[0];
            candidates
                .iter()
                .enumerate()
                .max_by_key(|(index, key)| {
                    let helps = (main.contains(key)
                        && observation.main_count() < observation.main_required())
                        || (sub.contains(key)
                            && observation.sub_count() < observation.sub_required());
                    (helps, std::cmp::Reverse(*index))
                })
                .map(|(index, _)| u64::try_from(index + 1).unwrap())
        } else {
            None
        };
        steps.push(advance(fixture, flow, activity, option));
        if activity.player_view().terminal().is_none() {
            let held = curios.owned(activity).unwrap();
            let card = held
                .iter()
                .find(|held| held.state().as_str() == "divergent-universe.curio-state.9074")
                .expect("one monotonically expanding Equation cannot exhaust the card");
            assert!(card.charges() <= remaining);
            let triggers = remaining - card.charges();
            assert_eq!(card.activations(), u32::from(3 - card.charges()));
            assert_eq!(
                blessings.owned(activity).unwrap().len() - before_count,
                base_grants + usize::from(triggers)
            );
            if triggers != 0 {
                assert!(trigger_boundary.replace(kind).is_none());
                assert_eq!(triggers, 1);
                assert_eq!(
                    progress.observations(activity).unwrap()[0].state(),
                    DivergentUniverseEquationExpansionState::Expanded
                );
                // A battle also publishes three candidates; direct grants and
                // a chosen Blessing otherwise draw only their immediate grants.
                let expected_draws = if kind == ActivityDecisionKind::Reward {
                    1
                } else {
                    base_grants
                        + 1
                        + if kind == ActivityDecisionKind::Encounter {
                            3
                        } else {
                            0
                        }
                };
                assert_eq!(
                    reward_draws(activity) - before_draws,
                    u64::try_from(expected_draws).unwrap()
                );
                eprintln!(
                    "public expansion at step {} event={event:?} kind={kind:?}",
                    steps.len()
                );
            }
            remaining = card.charges();
        }
    }
    // Terminal scope disposal is not a Curio trigger or limiting discard.
    assert!(curios.owned(activity).unwrap().is_empty());
    (remaining, trigger_boundary)
}

#[test]
fn expansion_rewards_paid_public_fixed_vectors() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for (family, seed, initial, boundary) in [
        (
            DivergentUniverseRunFamily::Ordinary,
            1000243,
            0,
            ActivityDecisionKind::Service,
        ),
        (
            DivergentUniverseRunFamily::Cyclical,
            328897,
            0,
            ActivityDecisionKind::Encounter,
        ),
    ] {
        let flow = fixture.flow_with_tawot_service(family, 2).unwrap();
        let (mut activity, mut steps) = prefix(&fixture, &flow, seed, initial).unwrap();
        assert_eq!(
            finish(&fixture, &flow, &mut activity, &mut steps),
            (2, Some(boundary))
        );
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        let step_count = steps.len();
        let run =
            record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps).unwrap();
        let verified = verify_divergent_universe_replay(
            &encode_divergent_universe_replay(&run).unwrap(),
            &fresh,
        )
        .unwrap();
        assert_eq!(
            verified.final_state_hash().bytes(),
            activity.state_hash().bytes()
        );
        assert_eq!(verified.battle_count(), 3);
        assert_eq!(
            usize::try_from(verified.action_count()).unwrap(),
            step_count
        );
    }
}

#[test]
#[ignore = "explicit bounded public seed discovery; retain fixed positive vectors in the default suite"]
fn expansion_rewards_search_paid_public_trigger() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow_with_tawot_service(family, 2).unwrap();
        let rewards = fixture.factory().decision_reward_runtime().unwrap();
        let mut template = flow
            .start(instance(1), ActivityMasterSeed::from_u64(0))
            .unwrap()
            .into_activity();
        advance(&fixture, &flow, &mut template, None);
        let view = template.player_view();
        let curios = fixture.factory().curio_runtime().unwrap();
        let mut candidates = Vec::new();
        for curio in curios.curios() {
            if !matches!(
                curio.category(),
                DivergentUniverseCurioCategory::Common | DivergentUniverseCurioCategory::Rare
            ) {
                continue;
            }
            let state = curios
                .states()
                .iter()
                .filter(|state| state.curio() == Some(curio.id()))
                .min_by_key(|state| state.id())
                .unwrap();
            if curios
                .acquisition_rewards_available(&view, state.id())
                .unwrap()
            {
                candidates.push((curio.id().clone(), state.id().clone()));
            }
        }
        candidates.sort_by(|left, right| left.0.cmp(&right.0));
        let definition = flow.definition();
        let identity = definition.identity();
        let graph = definition.graph();
        let graph_digest = graph.digest();
        let entry = graph.entry();
        let section = graph.node(entry).unwrap().section();
        let initial_policy = flow.initial_equation_policy().unwrap();
        let weights = vec![
            1;
            fixture
                .factory()
                .bundle
                .equation_catalog()
                .equations()
                .iter()
                .filter(|equation| equation.category == initial_policy.category)
                .count()
        ];
        let choice = &fixture
            .factory()
            .decision_catalog()
            .occurrences()
            .iter()
            .flat_map(|occurrence| occurrence.choices.iter())
            .find(|choice| choice.id.as_str() == "du.choice.color.blood-red")
            .unwrap()
            .id;
        let mut found = false;
        for seed in 0..2_000_000 {
            if seed % 10_000 == 0 {
                eprintln!("public expansion search: {family:?} seed={seed}");
            }
            // Search-only projection of the initial Reward stream. No generated
            // operations are applied. Every candidate must pass the real prefix.
            let context = ActivityRngContext::new(
                ActivityMasterSeed::from_u64(seed),
                identity.id(),
                identity.definition_digest(),
                identity.config_digest(),
                graph_digest,
                instance(1),
                Some(section),
                Some(entry),
                None,
                0,
            );
            let mut rng = ActivityRngStreams::new(context);
            rng.choose_weighted_without_replacement(
                ActivityRngLabel::Reward,
                23_801,
                &weights,
                initial_policy.offer_width,
            )
            .unwrap();
            let first = usize::try_from(
                rng.choose_index(
                    ActivityRngLabel::Reward,
                    23_611,
                    u32::try_from(candidates.len()).unwrap(),
                )
                .unwrap()
                .unwrap()
                .value(),
            )
            .unwrap();
            let second = usize::try_from(
                rng.choose_index(
                    ActivityRngLabel::Reward,
                    23_611,
                    u32::try_from(candidates.len() - 1).unwrap(),
                )
                .unwrap()
                .unwrap()
                .value(),
            )
            .unwrap();
            let second = second + usize::from(second >= first);
            let states = [&candidates[first].1, &candidates[second].1];
            // This cache is search-only. Check its ordering/sampling against the
            // actual producer; every positive vector still executes all commands.
            if seed < 256 {
                let mut actual_rng = ActivityRngStreams::new(context);
                actual_rng
                    .choose_weighted_without_replacement(
                        ActivityRngLabel::Reward,
                        23_801,
                        &weights,
                        initial_policy.offer_width,
                    )
                    .unwrap();
                let (_, actual) = rewards
                    .generate_choice(&view, choice, &mut actual_rng)
                    .unwrap()
                    .into_parts();
                let DecisionRewardGrant::Curios(actual) = actual else {
                    panic!("Color Curios");
                };
                assert_eq!(states, [&actual[0], &actual[1]]);
            }
            if !["9192", "9195"].iter().all(|raw| {
                states
                    .iter()
                    .any(|state| state.as_str() == format!("divergent-universe.curio-state.{raw}"))
            }) {
                continue;
            }
            eprintln!("Color pair {family:?} seed={seed}");
            for initial in 0..3 {
                let Some((mut activity, mut steps)) = prefix(&fixture, &flow, seed, initial) else {
                    continue;
                };
                let (remaining, boundary) = finish(&fixture, &flow, &mut activity, &mut steps);
                eprintln!(
                    "paid candidate {family:?} seed={seed} initial={initial} remaining={remaining}"
                );
                let required_boundary = match family {
                    DivergentUniverseRunFamily::Ordinary => ActivityDecisionKind::Service,
                    DivergentUniverseRunFamily::Cyclical => ActivityDecisionKind::Encounter,
                };
                if remaining == 3 || boundary != Some(required_boundary) {
                    continue;
                }
                let run =
                    record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                        .unwrap();
                let verified = verify_divergent_universe_replay(
                    &encode_divergent_universe_replay(&run).unwrap(),
                    &fresh,
                )
                .unwrap();
                assert_eq!(
                    verified.final_state_hash().bytes(),
                    activity.state_hash().bytes()
                );
                assert_eq!(verified.battle_count(), 3);
                found = true;
                break;
            }
            if found {
                break;
            }
        }
        assert!(found, "bounded public search did not trigger expansion");
    }
}
