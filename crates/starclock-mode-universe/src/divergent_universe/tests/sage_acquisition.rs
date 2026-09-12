//! Sage acquisition effects; elite/aberration victory and public upgrades remain separate.

use starclock_activity::{
    ActivityMasterSeed, ActivityOptionId, ActivityTerminalOutcome, GraphActivity,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingCategory,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

use super::{instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseCurrencyCommand, DivergentUniverseCurrencyKind, DivergentUniverseFlowInstance,
    DivergentUniverseOfferedSelection, curio_runtime::DivergentUniverseCurioRuntimeError,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    verify_divergent_universe_replay,
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

fn start(flow: &DivergentUniverseFlowInstance, seed: u64) -> GraphActivity {
    flow.start(instance(1), ActivityMasterSeed::from_u64(seed))
        .unwrap()
        .into_activity()
}

#[test]
fn sage_all_three_forms_grant_exact_rarity_counts_and_reconstruct_atomically() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut first = start(&flow, 24101);
        let mut fresh = start(&flow, 24101);
        for activity in [&mut first, &mut fresh] {
            let hash = activity.state_hash();
            for target in ["9193", "9194"] {
                assert_eq!(
                    curios.acquire_accepted_state(activity, hash, &state(target)),
                    Err(DivergentUniverseCurioRuntimeError::EvolutionOnly)
                );
            }
            let draws = reward_draws(activity);
            curios
                .acquire_accepted_state(activity, hash, &state("9192"))
                .unwrap();
            for (from, to) in [("9192", "9193"), ("9193", "9194")] {
                let hash = activity.state_hash();
                curios
                    .evolve_accepted_state(activity, hash, &state(from), &state(to))
                    .unwrap();
                let bytes = activity.canonical_state_bytes();
                let hash = activity.state_hash();
                assert!(
                    curios
                        .evolve_accepted_state(activity, hash, &state(from), &state(to))
                        .is_err()
                );
                assert_eq!(activity.canonical_state_bytes(), bytes);
            }
            assert_eq!(reward_draws(activity) - draws, 6);
            let owned = blessings.owned(activity).unwrap();
            for (category, count) in [
                (DivergentUniverseBlessingCategory::Common, 1),
                (DivergentUniverseBlessingCategory::Rare, 2),
                (DivergentUniverseBlessingCategory::Legendary, 3),
            ] {
                assert_eq!(
                    owned
                        .iter()
                        .filter(|held| blessings
                            .blessings()
                            .iter()
                            .any(|definition| definition.id() == held.blessing()
                                && definition.category() == category))
                        .count(),
                    count
                );
            }
            let owned = curios.owned(activity).unwrap();
            assert_eq!(owned.len(), 1);
            assert_eq!(owned[0].state(), &state("9194"));
            assert_eq!(owned[0].curio().as_str(), "divergent-universe.curio.9154");
            assert_eq!(
                curios
                    .states()
                    .iter()
                    .find(|definition| definition.id() == &state("9194"))
                    .unwrap()
                    .curio(),
                None
            );
        }
        assert_eq!(first.canonical_state_bytes(), fresh.canonical_state_bytes());
    }
}

#[test]
fn sage_and_wax_overlapping_pools_reserve_a_complete_distinct_assignment() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let common = blessings
        .blessings()
        .iter()
        .find(|blessing| {
            blessing.path().as_str() == "126"
                && blessing.category() == DivergentUniverseBlessingCategory::Common
        })
        .unwrap()
        .id();
    let rare = blessings
        .blessings()
        .iter()
        .find(|blessing| {
            blessing.path().as_str() == "126"
                && blessing.category() == DivergentUniverseBlessingCategory::Rare
        })
        .unwrap()
        .id();
    for viable in [true, false] {
        for seed in 0..8 {
            let preload = blessings
                .blessings()
                .iter()
                .filter(|blessing| blessing.id() != common && (!viable || blessing.id() != rare))
                .map(|blessing| blessing.id().clone())
                .collect::<Vec<_>>();
            let mut first = start(&flow, seed);
            let mut fresh = start(&flow, seed);
            for (activity, states) in [
                (&mut first, [state("9043"), state("9192")]),
                (&mut fresh, [state("9192"), state("9043")]),
            ] {
                blessings
                    .acquire_accepted_identities(activity, activity.state_hash(), &preload)
                    .unwrap();
                let bytes = activity.canonical_state_bytes();
                let draws = reward_draws(activity);
                let result =
                    curios.acquire_accepted_states(activity, activity.state_hash(), &states);
                if viable {
                    result.unwrap();
                    assert_eq!(
                        blessings.owned(activity).unwrap().len(),
                        blessings.blessings().len()
                    );
                    assert_eq!(reward_draws(activity) - draws, 2);
                } else {
                    assert_eq!(
                        result,
                        Err(DivergentUniverseCurioRuntimeError::NoLegalCandidate)
                    );
                    assert_eq!(activity.canonical_state_bytes(), bytes);
                    assert_eq!(reward_draws(activity), draws);
                }
            }
            assert_eq!(first.canonical_state_bytes(), fresh.canonical_state_bytes());
        }
    }
}

#[test]
fn sage_upgrade_exhaustion_keeps_predecessor_and_late_fragment_overflow_restores_blessings() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let mut activity = start(&flow, 24102);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state("9192"))
        .unwrap();
    let rare = blessings
        .blessings()
        .iter()
        .filter(|blessing| blessing.category() == DivergentUniverseBlessingCategory::Rare)
        .skip(1)
        .map(|blessing| blessing.id().clone())
        .collect::<Vec<_>>();
    let hash = activity.state_hash();
    blessings
        .acquire_accepted_identities(&mut activity, hash, &rare)
        .unwrap();
    let bytes = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    assert_eq!(
        curios.evolve_accepted_state(&mut activity, hash, &state("9192"), &state("9193")),
        Err(DivergentUniverseCurioRuntimeError::NoLegalCandidate)
    );
    assert_eq!(activity.canonical_state_bytes(), bytes);
    assert_eq!(reward_draws(&activity), draws);
    let mut activity = start(&flow, 24103);
    let currency = DivergentUniverseCurrencyKind::CosmicFragment;
    let rule = flow.economy().currency(currency).gain_rules()[0];
    let hash = activity.state_hash();
    flow.apply_currency_command(
        &mut activity,
        hash,
        DivergentUniverseCurrencyCommand::Credit {
            currency,
            rule,
            amount: i64::MAX as u64,
        },
    )
    .unwrap();
    let bytes = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    for _ in 0..2 {
        let hash = activity.state_hash();
        assert!(
            curios
                .acquire_accepted_states(&mut activity, hash, &[state("9192"), state("9195")])
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        assert_eq!(reward_draws(&activity), draws);
    }
}

#[test]
fn sage_base_is_obtained_through_public_reward_and_fresh_selection_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut found = false;
        for seed in 0..512 {
            let mut activity = start(&flow, seed);
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
            if !curios
                .owned(&activity)
                .unwrap()
                .iter()
                .any(|owned| owned.state() == &state("9192"))
            {
                continue;
            }
            assert!(
                !fixture
                    .factory()
                    .blessing_runtime()
                    .unwrap()
                    .owned(&activity)
                    .unwrap()
                    .is_empty()
            );
            let mut steps = vec![initial, event];
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < 12);
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
            let recorded =
                record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                    .unwrap();
            let report = verify_divergent_universe_replay(
                &encode_divergent_universe_replay(&recorded).unwrap(),
                &fresh,
            )
            .unwrap();
            assert_eq!(report.terminal(), ActivityTerminalOutcome::Completed);
            assert_eq!(report.battle_count(), 3);
            found = true;
            break;
        }
        assert!(found, "bounded public corpus must acquire Sage's Leaf Robe");
    }
}
