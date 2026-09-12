//! Actual public domain choices produce Sage grants and fresh selection replay.

use super::{instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    DivergentUniverseOfferedSelection, encode_divergent_universe_replay,
    record_divergent_universe_transcript, verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityOptionId, ActivityTerminalOutcome,
    GraphActivity,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_data::divergent_universe_decisions::BattleRewardDomain;

pub(super) fn advance(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    option: Option<u64>,
) -> DivergentUniverseBaselineStep {
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    if let Some(option) = option {
        let decision = activity.player_view().decision().unwrap().id();
        runner
            .advance_selected(
                fixture.factory(),
                flow,
                activity,
                fixture.core(),
                &policy,
                DivergentUniverseOfferedSelection::new(
                    decision,
                    ActivityOptionId::new(option).unwrap(),
                ),
            )
            .unwrap()
    } else {
        runner
            .advance(fixture.factory(), flow, activity, fixture.core(), &policy)
            .unwrap()
    }
}

#[test]
fn public_domains_produce_sage_victory_rewards_and_replay_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let curios = fixture.factory().curio_runtime().unwrap();
        let blessings = fixture.factory().blessing_runtime().unwrap();
        let (seed, mut activity, mut steps) = (0..2048).find_map(|seed| {
            let mut activity = flow.start(instance(1), ActivityMasterSeed::from_u64(seed)).unwrap().into_activity();
            let first = advance(&fixture, &flow, &mut activity, None);
            let color = advance(&fixture, &flow, &mut activity, Some(2));
            let owned = curios.owned(&activity).unwrap();
            (owned.iter().any(|held| held.state().as_str() == "divergent-universe.curio-state.9192")
                && !owned.iter().any(|held| ["divergent-universe.curio-state.9195", "divergent-universe.curio-state.9055"].contains(&held.state().as_str())))
                .then_some((seed, activity, vec![first, color]))
        }).expect("bounded public initial-event seed produces Sage without a competing treasure or suppressor");
        let mut domain_count = 0;
        let mut battle_count = 0;
        let mut upgrades = 0;
        while activity.player_view().terminal().is_none() {
            assert!(steps.len() < 12);
            let view = activity.player_view();
            let decision = view.decision().unwrap();
            let before = blessings.owned(&activity).unwrap().len();
            let draws = reward_draws(&activity);
            let selection =
                if let Some(choices) = flow.offered_battle_domain_choices(&activity).unwrap() {
                    assert_eq!(flow.current_battle_domain(&activity).unwrap(), None);
                    assert_eq!(choices.len(), 3);
                    let domain =
                        [BattleRewardDomain::Elite, BattleRewardDomain::Aberration][domain_count];
                    let ordinal = choices
                        .iter()
                        .find(|choice| choice.domain == domain)
                        .unwrap()
                        .ordinal;
                    let bytes = activity.canonical_state_bytes();
                    assert!(
                        activity
                            .choose_option(
                                view.state_hash(),
                                decision.id(),
                                ActivityOptionId::new(4).unwrap()
                            )
                            .is_err()
                    );
                    assert_eq!(bytes, activity.canonical_state_bytes());
                    assert_eq!(draws, reward_draws(&activity));
                    domain_count += 1;
                    Some(u64::from(ordinal))
                } else if let Some(event) = flow.offered_evolution_event(&activity) {
                    assert!(event.key.starts_with("du.evolution-event.sage."));
                    upgrades += 1;
                    Some(1)
                } else {
                    None
                };
            if decision.kind() == ActivityDecisionKind::Encounter {
                assert_eq!(
                    flow.current_battle_domain(&activity).unwrap(),
                    Some(
                        [
                            BattleRewardDomain::Combat,
                            BattleRewardDomain::Elite,
                            BattleRewardDomain::Aberration,
                        ][battle_count]
                    )
                );
            }
            let step = advance(&fixture, &flow, &mut activity, selection);
            if decision.kind() == ActivityDecisionKind::Route {
                assert_eq!(reward_draws(&activity), draws);
                let after = activity.canonical_state_bytes();
                let hash = activity.state_hash();
                assert!(
                    activity
                        .choose_option(hash, decision.id(), decision.options()[0].id())
                        .is_err()
                );
                assert_eq!(after, activity.canonical_state_bytes());
            }
            if matches!(step, DivergentUniverseBaselineStep::Battle { .. }) {
                let expected = [0, 2, 3][battle_count];
                assert_eq!(blessings.owned(&activity).unwrap().len() - before, expected);
                assert_eq!(
                    reward_draws(&activity) - draws,
                    u64::try_from(expected).unwrap() + 3
                );
                battle_count += 1;
            }
            steps.push(step);
        }
        assert_eq!(
            (domain_count, battle_count, upgrades, steps.len()),
            (2, 3, 2, 12)
        );
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        let recorded =
            record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps).unwrap();
        let replay = verify_divergent_universe_replay(
            &encode_divergent_universe_replay(&recorded).unwrap(),
            &fresh,
        )
        .unwrap();
        assert_eq!(replay.action_count(), 12);
        assert_eq!(replay.battle_count(), 3);
        assert_eq!(
            replay.final_state_hash().bytes(),
            activity.state_hash().bytes()
        );
    }
}
