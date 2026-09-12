//! Real battle result contracts with explicitly counterfactual domain context.
//! Positive fixtures do not claim original Elite/Aberration placement is present.

use super::reward_draws;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseFlowInstance,
    tests::curio_battle_grants::{project, ready, result},
};
use starclock_activity::{
    ActivityProgramId, BattleOutcome, BattleResult, GraphActivity, GraphActivityBattleError,
    GraphActivityBattleResolution, GraphActivityCommandError, ProjectedValue,
};
use starclock_data::{
    divergent_universe_blessing_catalog::{
        DivergentUniverseBlessingCategory, DivergentUniverseBlessingId,
    },
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

fn acquire(fixture: &DivergentUniverseBaselineFixture, activity: &mut GraphActivity, target: &str) {
    let runtime = fixture.factory().curio_runtime().unwrap();
    runtime
        .acquire_accepted_state(activity, activity.state_hash(), &state("9192"))
        .unwrap();
    if target != "9192" {
        runtime
            .evolve_accepted_state(
                activity,
                activity.state_hash(),
                &state("9192"),
                &state("9193"),
            )
            .unwrap();
    }
    if target == "9194" {
        runtime
            .evolve_accepted_state(
                activity,
                activity.state_hash(),
                &state("9193"),
                &state("9194"),
            )
            .unwrap();
    }
}

pub(super) fn owned(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &GraphActivity,
) -> Vec<DivergentUniverseBlessingId> {
    fixture
        .factory()
        .blessing_runtime()
        .unwrap()
        .owned(activity)
        .unwrap()
        .iter()
        .map(|held| held.blessing().clone())
        .collect()
}

// Calls the production generators at the real shared post-carry boundary, but
// explicitly substitutes a domain only here. No production label override exists.
pub(super) fn settle_domain(
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    result: BattleResult,
    domain: BattleRewardDomain,
    reject_after_grant: bool,
) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
    let grant = ActivityProgramId::new(24_050).unwrap();
    let offer = ActivityProgramId::new(23_702).unwrap();
    activity.submit_pending_battle_result_with_generated_boundary(
        activity.state_hash(),
        result,
        &[grant, offer],
        |program, view, settlement, rng| {
            if settlement.outcome() != BattleOutcome::Won {
                return Ok(Vec::new());
            }
            assert_eq!(
                flow.battle_reward_domain(view),
                Some(BattleRewardDomain::Combat)
            );
            if program == grant {
                flow.curio_victory_blessings
                    .as_ref()
                    .unwrap()
                    .generate(view, domain, rng)
            } else if reject_after_grant {
                // Fail only after the first stage's grants and draws have applied.
                Err(GraphActivityCommandError::DecisionNotOffered)
            } else {
                flow.battle_blessings.as_ref().unwrap().generate(view, rng)
            }
        },
    )
}

#[test]
fn sage_victory_all_forms_are_domain_gated_and_reconstruct_after_actual_battles() {
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for (target, count) in [("9192", 1), ("9193", 2), ("9194", 3)] {
            for domain in [
                BattleRewardDomain::Combat,
                BattleRewardDomain::Elite,
                BattleRewardDomain::Aberration,
                BattleRewardDomain::Boss,
            ] {
                let run = || {
                    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
                    let (flow, mut activity) = ready(&fixture, family);
                    acquire(&fixture, &mut activity, target);
                    let before = owned(&fixture, &activity);
                    let battle = result(&fixture, &flow, &mut activity);
                    assert!(
                        battle
                            .values()
                            .contains(&ProjectedValue::Outcome(BattleOutcome::Won))
                    );
                    let draws = reward_draws(&activity);
                    settle_domain(&flow, &mut activity, battle.clone(), domain, false).unwrap();
                    let eligible = matches!(
                        domain,
                        BattleRewardDomain::Elite | BattleRewardDomain::Aberration
                    );
                    let granted = if eligible { count } else { 0 };
                    let after = owned(&fixture, &activity);
                    assert_eq!(after.len() - before.len(), granted);
                    assert!(before.iter().all(|id| after.contains(id)));
                    assert_eq!(
                        reward_draws(&activity) - draws,
                        u64::try_from(granted).unwrap() + 3
                    );
                    let bytes = activity.canonical_state_bytes();
                    assert!(settle_domain(&flow, &mut activity, battle, domain, false).is_err());
                    assert_eq!(bytes, activity.canonical_state_bytes());
                    bytes
                };
                assert_eq!(run(), run());
            }
        }
    }
}

// Preserve only the requested unowned reward pool; this is a controlled inventory
// fixture, never a claim that the baseline naturally collected all other Blessings.
pub(super) fn leave_pool(
    fixture: &DivergentUniverseBaselineFixture,
    activity: &mut GraphActivity,
    category: DivergentUniverseBlessingCategory,
    count: usize,
) -> Vec<DivergentUniverseBlessingId> {
    let runtime = fixture.factory().blessing_runtime().unwrap();
    let held = owned(fixture, activity);
    let mut pool = runtime
        .blessings()
        .iter()
        .filter(|definition| definition.category() == category && !held.contains(definition.id()))
        .map(|definition| definition.id().clone())
        .collect::<Vec<_>>();
    pool.sort();
    pool.truncate(count);
    assert_eq!(pool.len(), count);
    let fill = runtime
        .blessings()
        .iter()
        .filter(|definition| !held.contains(definition.id()) && !pool.contains(definition.id()))
        .map(|definition| definition.id().clone())
        .collect::<Vec<_>>();
    runtime
        .acquire_accepted_identities(activity, activity.state_hash(), &fill)
        .unwrap();
    pool
}

#[test]
fn sage_victory_uses_all_three_rarities_and_later_offer_excludes_immediate_grants() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for (target, count) in [("9192", 1), ("9193", 2), ("9194", 3)] {
            for category in [
                DivergentUniverseBlessingCategory::Common,
                DivergentUniverseBlessingCategory::Rare,
                DivergentUniverseBlessingCategory::Legendary,
            ] {
                let (flow, mut activity) = ready(&fixture, family);
                acquire(&fixture, &mut activity, target);
                let pool = leave_pool(&fixture, &mut activity, category, 4);
                let before = owned(&fixture, &activity);
                let battle = result(&fixture, &flow, &mut activity);
                settle_domain(
                    &flow,
                    &mut activity,
                    battle,
                    BattleRewardDomain::Elite,
                    false,
                )
                .unwrap();
                let after = owned(&fixture, &activity);
                let granted = after
                    .iter()
                    .filter(|id| !before.contains(id))
                    .collect::<Vec<_>>();
                assert_eq!(granted.len(), count);
                assert!(granted.iter().all(|id| pool.contains(id)));
                if category != DivergentUniverseBlessingCategory::Legendary {
                    let view = activity.player_view();
                    let offered = view.decision().unwrap();
                    assert_eq!(offered.options().len(), 4 - count);
                    let option = offered.options()[0].id();
                    flow.choose_battle_blessing(
                        &mut activity,
                        view.state_hash(),
                        offered.id(),
                        option,
                    )
                    .unwrap();
                    assert_eq!(owned(&fixture, &activity).len(), after.len() + 1);
                }
            }
        }
    }
}

#[test]
fn sage_victory_exhaustion_and_late_failure_preserve_exact_draws_and_retry() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for remaining in 0..=2 {
        let (flow, mut activity) = ready(&fixture, DivergentUniverseRunFamily::Ordinary);
        acquire(&fixture, &mut activity, "9194");
        leave_pool(
            &fixture,
            &mut activity,
            DivergentUniverseBlessingCategory::Legendary,
            remaining,
        );
        let before = owned(&fixture, &activity);
        let battle = result(&fixture, &flow, &mut activity);
        let bytes = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        for _ in 0..2 {
            assert!(
                settle_domain(
                    &flow,
                    &mut activity,
                    battle.clone(),
                    BattleRewardDomain::Aberration,
                    true
                )
                .is_err()
            );
            assert_eq!(bytes, activity.canonical_state_bytes());
            assert_eq!(debug, activity.debug_view());
        }
        let draws = reward_draws(&activity);
        settle_domain(
            &flow,
            &mut activity,
            battle,
            BattleRewardDomain::Aberration,
            false,
        )
        .unwrap();
        assert_eq!(owned(&fixture, &activity).len() - before.len(), remaining);
        assert_eq!(
            reward_draws(&activity) - draws,
            u64::try_from(remaining).unwrap()
        );
    }
}

#[test]
fn sage_victory_lifecycle_suppression_and_nonvictory_do_not_grant_or_draw() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for (destroyed, suppressed, outcome, expected) in [
        (true, false, BattleOutcome::Won, 0),
        (false, true, BattleOutcome::Won, 0),
        (false, false, BattleOutcome::Lost, 0),
        (false, false, BattleOutcome::Won, 3),
    ] {
        let (flow, mut activity) = ready(&fixture, DivergentUniverseRunFamily::Cyclical);
        acquire(&fixture, &mut activity, "9194");
        if destroyed {
            let hash = activity.state_hash();
            curios
                .destroy_accepted(&mut activity, hash, &state("9194"))
                .unwrap();
        }
        if suppressed {
            let hash = activity.state_hash();
            curios
                .acquire_accepted_state(&mut activity, hash, &state("9055"))
                .unwrap();
        }
        let before = owned(&fixture, &activity).len();
        let actual = result(&fixture, &flow, &mut activity);
        let battle = project(&actual, 4, outcome);
        let draws = reward_draws(&activity);
        settle_domain(
            &flow,
            &mut activity,
            battle,
            BattleRewardDomain::Elite,
            false,
        )
        .unwrap();
        if outcome == BattleOutcome::Won {
            assert_eq!(owned(&fixture, &activity).len() - before, expected);
            assert_eq!(
                reward_draws(&activity) - draws,
                u64::try_from(expected).unwrap() + if suppressed { 0 } else { 3 }
            );
        } else {
            assert_eq!(reward_draws(&activity), draws);
        }
    }
}

#[test]
fn sage_victory_production_normal_proxy_never_uses_source_elite_stage_markers() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for target in ["9192", "9193", "9194"] {
            let (flow, mut activity) = ready(&fixture, family);
            acquire(&fixture, &mut activity, target);
            // This released stage has elite source flags; the current flow binds
            // it to a Combat proxy. Its flags cannot authorize a Sage grant.
            assert_eq!(
                flow.offered_encounter(&activity).unwrap().unwrap().1,
                "83002081"
            );
            let before = owned(&fixture, &activity);
            let battle = result(&fixture, &flow, &mut activity);
            let draws = reward_draws(&activity);
            let hash = activity.state_hash();
            let resolution = fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, battle, None)
                .unwrap();
            assert_eq!(before, owned(&fixture, &activity));
            assert_eq!(reward_draws(&activity) - draws, 3);
            assert!(
                !resolution
                    .events()
                    .iter()
                    .any(|event| event.cause().program().get() == 24_050)
            );
        }
    }
}

#[test]
fn sage_victory_repair_and_replacement_change_only_the_current_active_effect() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for replacing in [false, true] {
        let (flow, mut activity) = ready(&fixture, DivergentUniverseRunFamily::Ordinary);
        acquire(&fixture, &mut activity, "9193");
        let hash = activity.state_hash();
        curios
            .destroy_accepted(&mut activity, hash, &state("9193"))
            .unwrap();
        let hash = activity.state_hash();
        curios
            .repair_accepted(&mut activity, hash, &state("9193"))
            .unwrap();
        if replacing {
            let hash = activity.state_hash();
            curios
                .replace_accepted(&mut activity, hash, &state("9193"), &state("9028"))
                .unwrap();
        }
        let before = owned(&fixture, &activity).len();
        let battle = result(&fixture, &flow, &mut activity);
        settle_domain(
            &flow,
            &mut activity,
            battle,
            BattleRewardDomain::Aberration,
            false,
        )
        .unwrap();
        assert_eq!(
            owned(&fixture, &activity).len() - before,
            if replacing { 0 } else { 2 }
        );
    }
}

#[test]
fn sage_victory_suppression_obeys_destroyed_and_repaired_suppressor_state() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for repaired in [false, true] {
        let (flow, mut activity) = ready(&fixture, DivergentUniverseRunFamily::Cyclical);
        acquire(&fixture, &mut activity, "9192");
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state("9055"))
            .unwrap();
        let hash = activity.state_hash();
        curios
            .destroy_accepted(&mut activity, hash, &state("9055"))
            .unwrap();
        if repaired {
            let hash = activity.state_hash();
            curios
                .repair_accepted(&mut activity, hash, &state("9055"))
                .unwrap();
        }
        let before = owned(&fixture, &activity).len();
        let battle = result(&fixture, &flow, &mut activity);
        let draws = reward_draws(&activity);
        settle_domain(
            &flow,
            &mut activity,
            battle,
            BattleRewardDomain::Elite,
            false,
        )
        .unwrap();
        assert_eq!(
            owned(&fixture, &activity).len() - before,
            usize::from(!repaired)
        );
        assert_eq!(
            reward_draws(&activity) - draws,
            if repaired { 0 } else { 4 }
        );
    }
}
