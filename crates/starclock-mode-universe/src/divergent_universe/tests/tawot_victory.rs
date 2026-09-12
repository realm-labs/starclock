//! Positive allowance affects actual victory inventory, not a lifecycle marker.

use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioRuntime,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    tests::{
        curio_battle_grants::{project, ready, result},
        domain_choices::advance,
        instance, reward_draws,
        sage_victory::{leave_pool, owned, settle_domain},
    },
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityMasterSeed, ActivityProgramDefinition, ActivityProgramId, ActivityTerminalOutcome,
    BattleOutcome, BattleResult, GraphActivity, ProjectedValue,
};
use starclock_combat::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingCategory,
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
fn acquire(curios: &DivergentUniverseCurioRuntime, activity: &mut GraphActivity, raw: &str) {
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(activity, hash, &state(raw))
        .unwrap();
}
fn charges(curios: &DivergentUniverseCurioRuntime, activity: &GraphActivity) -> Option<u16> {
    curios
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == &state("9072"))
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

#[test]
fn tawot_victory_positive_allowance_lifecycle_and_nonvictory_controls() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for (mode, outcome, count) in [
            ("active", BattleOutcome::Won, 1),
            ("destroyed", BattleOutcome::Won, 0),
            ("repaired", BattleOutcome::Won, 1),
            ("zero", BattleOutcome::Won, 0),
            ("suppressed", BattleOutcome::Won, 0),
            ("replaced", BattleOutcome::Won, 0),
            ("active", BattleOutcome::Lost, 0),
            ("active", BattleOutcome::Faulted, 0),
        ] {
            let (flow, mut activity) = ready(&fixture, family);
            acquire(&curios, &mut activity, "9072");
            assert_eq!(charges(&curios, &activity), Some(3));
            if matches!(mode, "destroyed" | "repaired") {
                let hash = activity.state_hash();
                curios
                    .destroy_accepted(&mut activity, hash, &state("9072"))
                    .unwrap();
                if mode == "repaired" {
                    let hash = activity.state_hash();
                    curios
                        .repair_accepted(&mut activity, hash, &state("9072"))
                        .unwrap();
                }
            }
            if mode == "zero" {
                let hash = activity.state_hash();
                curios
                    .set_accepted_charges(&mut activity, hash, &state("9072"), 0)
                    .unwrap();
            }
            if mode == "suppressed" {
                acquire(&curios, &mut activity, "9055");
            }
            if mode == "replaced" {
                let hash = activity.state_hash();
                curios
                    .replace_accepted(&mut activity, hash, &state("9072"), &state("9073"))
                    .unwrap();
            }
            let before = owned(&fixture, &activity);
            let allowance = charges(&curios, &activity);
            let actual = result(&fixture, &flow, &mut activity);
            // Nonvictory projections test the verified result contract, not natural losses.
            let projected = project(&actual, 4, outcome);
            let projected = BattleResult::seal(
                projected.identity(),
                projected
                    .values()
                    .iter()
                    .map(|value| {
                        if outcome == BattleOutcome::Faulted
                            && matches!(value, ProjectedValue::TerminalFault(_))
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
            let draws = reward_draws(&activity);
            let hash = activity.state_hash();
            fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, projected.clone(), None)
                .unwrap();
            assert_eq!(
                owned(&fixture, &activity).len() - before.len(),
                count,
                "{mode}/{outcome:?}"
            );
            assert_eq!(
                charges(&curios, &activity),
                allowance,
                "battle result is not a domain entry"
            );
            assert_eq!(
                reward_draws(&activity) - draws,
                if outcome == BattleOutcome::Won && mode != "suppressed" {
                    3 + u64::try_from(count).unwrap()
                } else {
                    0
                }
            );
            let after = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&flow, &mut activity, hash, projected, None)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), after);
        }
    }
}

#[test]
fn tawot_victory_domain_expiry_pauses_refills_and_removes_reward_before_next_battle() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        acquire(&curios, &mut activity, "9072");
        let hash = activity.state_hash();
        curios
            .destroy_accepted(&mut activity, hash, &state("9072"))
            .unwrap();
        let before = activity.canonical_state_bytes();
        enter_vector(&curios, &mut activity);
        assert_eq!(activity.canonical_state_bytes(), before);
        let hash = activity.state_hash();
        curios
            .repair_accepted(&mut activity, hash, &state("9072"))
            .unwrap();
        for remaining in (1..=3).rev() {
            // Separate three-entry operation vector; public route has two future entries.
            assert_eq!(charges(&curios, &activity), Some(remaining));
            assert!(
                curios
                    .battle_lifetime_operations(&activity.player_view())
                    .unwrap()
                    .is_empty()
            );
            enter_vector(&curios, &mut activity);
        }
        assert_eq!(charges(&curios, &activity), None);
        let before = owned(&fixture, &activity);
        let battle = result(&fixture, &flow, &mut activity);
        let hash = activity.state_hash();
        fixture
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(&flow, &mut activity, hash, battle, None)
            .unwrap();
        assert_eq!(owned(&fixture, &activity), before);
        acquire(&curios, &mut activity, "9072");
        assert_eq!(charges(&curios, &activity), Some(3));
        for (from, to) in [("9072", "9073"), ("9073", "9072")] {
            let hash = activity.state_hash();
            curios
                .replace_accepted(&mut activity, hash, &state(from), &state(to))
                .unwrap();
        }
        assert_eq!(charges(&curios, &activity), Some(3));
    }
}

#[test]
fn tawot_victory_policy_rarities_exhaustion_and_late_rollback_use_actual_inventory() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for (category, remaining) in [
        (DivergentUniverseBlessingCategory::Common, 2),
        (DivergentUniverseBlessingCategory::Rare, 2),
        (DivergentUniverseBlessingCategory::Legendary, 2),
        (DivergentUniverseBlessingCategory::Legendary, 0),
    ] {
        let (flow, mut activity) = ready(&fixture, FAMILIES[0]);
        acquire(&curios, &mut activity, "9072");
        let pool = leave_pool(&fixture, &mut activity, category, remaining);
        let before = owned(&fixture, &activity);
        let battle = result(&fixture, &flow, &mut activity);
        let bytes = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        assert!(
            settle_domain(
                &flow,
                &mut activity,
                battle.clone(),
                BattleRewardDomain::Combat,
                true
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        assert_eq!(reward_draws(&activity), draws);
        assert_eq!(charges(&curios, &activity), Some(3));
        settle_domain(
            &flow,
            &mut activity,
            battle,
            BattleRewardDomain::Combat,
            false,
        )
        .unwrap();
        let after = owned(&fixture, &activity);
        let gained = after
            .iter()
            .filter(|id| !before.contains(id))
            .collect::<Vec<_>>();
        assert_eq!(gained.len(), usize::from(remaining > 0));
        assert!(gained.iter().all(|id| pool.contains(id)));
        if category != DivergentUniverseBlessingCategory::Legendary {
            let view = activity.player_view();
            let offer = view.decision().unwrap();
            assert_eq!(
                offer.options().len(),
                1,
                "immediate grant is excluded from normal offer"
            );
            flow.choose_battle_blessing(
                &mut activity,
                view.state_hash(),
                offer.id(),
                offer.options()[0].id(),
            )
            .unwrap();
            assert_eq!(owned(&fixture, &activity).len(), after.len() + 1);
        } else {
            assert_eq!(reward_draws(&activity) - draws, u64::from(remaining > 0));
        }
    }
}

#[test]
fn tawot_victory_all_bound_domains_and_sage_rewards_share_distinct_sampling() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for domain in [
        BattleRewardDomain::Combat,
        BattleRewardDomain::Elite,
        BattleRewardDomain::Aberration,
        BattleRewardDomain::Boss,
    ] {
        let (flow, mut activity) = ready(&fixture, FAMILIES[1]);
        acquire(&curios, &mut activity, "9072");
        acquire(&curios, &mut activity, "9192");
        let pool = leave_pool(
            &fixture,
            &mut activity,
            DivergentUniverseBlessingCategory::Legendary,
            2,
        );
        let before = owned(&fixture, &activity);
        let battle = result(&fixture, &flow, &mut activity);
        let draws = reward_draws(&activity);
        // Domain substitution is only a policy vector, not original Boss placement.
        settle_domain(&flow, &mut activity, battle, domain, false).unwrap();
        let after = owned(&fixture, &activity);
        let gained = after
            .iter()
            .filter(|id| !before.contains(id))
            .collect::<Vec<_>>();
        let expected = if matches!(
            domain,
            BattleRewardDomain::Elite | BattleRewardDomain::Aberration
        ) {
            2
        } else {
            1
        };
        assert_eq!(gained.len(), expected);
        assert!(gained.iter().all(|id| pool.contains(id)));
        assert_eq!(
            reward_draws(&activity) - draws,
            u64::try_from(expected).unwrap()
        );
    }
}

#[test]
fn tawot_victory_paid_public_acquisition_grants_each_battle_and_replays_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let target = curios
        .states()
        .iter()
        .find(|s| s.id() == &state("9072"))
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
            .expect("bounded paid service offers 9072");
        let before = owned(&fixture, &activity);
        steps.push(advance(&fixture, &flow, &mut activity, Some(target)));
        assert_eq!(
            owned(&fixture, &activity),
            before,
            "acquisition is not victory"
        );
        steps.push(advance(&fixture, &flow, &mut activity, Some(2)));
        let mut entries = 0;
        let mut grants = 0;
        while activity.player_view().terminal().is_none() {
            assert!(steps.len() < 24);
            assert_eq!(charges(&curios, &activity), Some(3 - entries));
            let route = flow
                .offered_battle_domain_choices(&activity)
                .unwrap()
                .is_some();
            let battle_count = activity.player_view().completed_battle_count();
            let before = owned(&fixture, &activity);
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
            }
            if activity.player_view().completed_battle_count() > battle_count {
                let after = owned(&fixture, &activity);
                assert_eq!(
                    after.len() - before.len(),
                    1,
                    "one intrinsic grant per actual victory"
                );
                grants += 1;
            }
        }
        assert_eq!(entries, 2);
        assert_eq!(grants, 3);
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
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
