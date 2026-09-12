//! Independent weight vectors drive the same shared sampler as real settlement.

use super::{FAMILIES, fight, keys, ready};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseFlowInstance, DivergentUniverseOfferedSelection,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    tests::{instance, reward_draws},
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityMasterSeed, ActivityOptionId, ActivityRngContext, ActivityRngLabel, ActivityRngStreams,
    ActivityTerminalOutcome, GraphActivity,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingCategory,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

fn state(raw: u32) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

fn expected(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &GraphActivity,
    active_paths: &[&str],
) -> Vec<u64> {
    expected_at_seed(fixture, flow, activity, active_paths, 23_701)
}

fn expected_at_seed(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &GraphActivity,
    active_paths: &[&str],
    seed: u64,
) -> Vec<u64> {
    let definition = activity.definition();
    let entry = definition.graph().entry();
    let identity = definition.identity();
    assert_eq!(flow.definition().identity(), identity);
    let context = ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        definition.graph().digest(),
        instance(1),
        Some(definition.graph().node(entry).unwrap().section()),
        Some(entry),
        None,
        0,
    );
    let mut rng = ActivityRngStreams::new(context);
    for _ in 0..reward_draws(activity) {
        rng.choose_index(ActivityRngLabel::Reward, 1, 1).unwrap();
    }
    let runtime = fixture.factory().blessing_runtime().unwrap();
    let owned = runtime.owned(activity).unwrap();
    let mut pool = runtime
        .blessings()
        .iter()
        .filter(|blessing| {
            blessing.category() != DivergentUniverseBlessingCategory::Legendary
                && !owned.iter().any(|item| item.blessing() == blessing.id())
        })
        .collect::<Vec<_>>();
    pool.sort_by(|a, b| a.id().cmp(b.id()));
    let weights = pool
        .iter()
        .map(|blessing| {
            if active_paths.contains(&blessing.path().as_str()) {
                4
            } else {
                1
            }
        })
        .collect::<Vec<_>>();
    let draw = rng
        .choose_weighted_without_replacement(ActivityRngLabel::Reward, 23_701, &weights, 3)
        .unwrap();
    let mut keys = draw
        .iter()
        .map(|index| pool[*index as usize].state_key())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}

#[test]
fn all_eight_waxes_change_real_battle_weights_and_follow_destroy_repair_replace() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let mut differing_from_uniform = 0;
    for family in FAMILIES {
        for (raw, path) in [
            (9043, "126"),
            (9044, "124"),
            (9045, "125"),
            (9046, "121"),
            (9047, "122"),
            (9048, "127"),
            (9049, "128"),
            (9147, "129"),
        ] {
            for transition in 0..4 {
                let (flow, mut activity) = ready(&fixture, family);
                let wax = state(raw);
                let hash = activity.state_hash();
                curios
                    .acquire_accepted_state(&mut activity, hash, &wax)
                    .unwrap();
                if transition == 1 || transition == 2 {
                    let hash = activity.state_hash();
                    curios.destroy_accepted(&mut activity, hash, &wax).unwrap();
                }
                if transition == 2 {
                    let hash = activity.state_hash();
                    curios.repair_accepted(&mut activity, hash, &wax).unwrap();
                }
                if transition == 3 {
                    let hash = activity.state_hash();
                    curios
                        .replace_accepted(&mut activity, hash, &wax, &state(9001))
                        .unwrap();
                }
                let active = transition == 0 || transition == 2;
                let active_paths = [path];
                let wanted = expected(
                    &fixture,
                    &flow,
                    &activity,
                    if active { &active_paths } else { &[] },
                );
                let uniform = expected(&fixture, &flow, &activity, &[]);
                differing_from_uniform += usize::from(wanted != uniform);
                let draws = reward_draws(&activity);
                fight(&fixture, &flow, &mut activity);
                assert_eq!(
                    keys(&activity),
                    wanted,
                    "{family:?} state={raw} transition={transition}"
                );
                assert_eq!(reward_draws(&activity), draws + 3);
                if active {
                    let hash = activity.state_hash();
                    curios.destroy_accepted(&mut activity, hash, &wax).unwrap();
                    assert_eq!(keys(&activity), wanted, "published offer is a snapshot");
                    assert_eq!(reward_draws(&activity), draws + 3);
                }
            }
        }
    }
    assert!(
        differing_from_uniform > 0,
        "fixture must detect an ignored weight program"
    );
}

#[test]
fn multiple_waxes_compose_independent_paths_and_trailblaze_adds_no_offer_weight() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for states in [vec![9043, 9044], vec![9044, 9043], vec![9187]] {
            let (flow, mut activity) = ready(&fixture, family);
            for raw in &states {
                let hash = activity.state_hash();
                curios
                    .acquire_accepted_state(&mut activity, hash, &state(*raw))
                    .unwrap();
            }
            let paths = if states.len() == 2 {
                vec!["126", "124"]
            } else {
                vec![]
            };
            let wanted = expected(&fixture, &flow, &activity, &paths);
            fight(&fixture, &flow, &mut activity);
            assert_eq!(keys(&activity), wanted);
        }
    }
}

#[test]
fn public_occurrence_wax_reward_changes_battle_offer_and_fresh_replay_completes() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow(family).unwrap();
        let mut covered = false;
        for seed in 0..64 {
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
            let first = runner
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
            let owned = fixture
                .factory()
                .curio_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap();
            if !owned.iter().any(|curio| {
                fixture
                    .factory()
                    .decision_catalog()
                    .curio_battle_weights()
                    .iter()
                    .any(|row| &row.state == curio.state())
            }) {
                continue;
            }
            if owned.iter().any(|curio| curio.state() == &state(9055)) {
                continue;
            }
            let paths = fixture
                .factory()
                .decision_catalog()
                .curio_battle_weights()
                .iter()
                .filter(|row| owned.iter().any(|curio| curio.state() == &row.state))
                .map(|row| row.path.as_str())
                .collect::<Vec<_>>();
            let wanted = expected_at_seed(&fixture, &flow, &activity, &paths, seed);
            let uniform = expected_at_seed(&fixture, &flow, &activity, &[], seed);
            if wanted == uniform {
                continue;
            }
            let mut steps = vec![initial, first];
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
            assert_eq!(
                keys(&activity),
                wanted,
                "public event's wax must affect actual offer"
            );
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < policy.max_steps() as usize);
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
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
            let transcript =
                record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                    .unwrap();
            let bytes = encode_divergent_universe_replay(&transcript).unwrap();
            let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
            assert_eq!(verified.terminal(), ActivityTerminalOutcome::Completed);
            assert_eq!(verified.battle_count(), 3);
            assert_eq!(
                verified.final_state_hash().bytes(),
                activity.state_hash().bytes()
            );
            covered = true;
            break;
        }
        assert!(
            covered,
            "bounded public seed corpus must execute a wax reward"
        );
    }
}
