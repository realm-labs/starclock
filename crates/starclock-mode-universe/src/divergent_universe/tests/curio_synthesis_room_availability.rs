//! Current menu admission and sparse cached output pools, not original selectors.

use super::{
    CATEGORIES, FAMILIES, LEAVE_SYNTHESIS, OPEN_SYNTHESIS, bound, choose, enter, members, options,
    reward_draws, scenario_with_inventory, start,
};
use crate::divergent_universe::DivergentUniverseBaselineFixture;
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioCategory;

#[test]
fn synthesis_room_unavailable_pairs_offer_only_leave_without_rng_or_consumption() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let common = members(&curios, CATEGORIES[0])[0].clone();
    let rare = members(&curios, CATEGORIES[1])[0].clone();
    let negative = members(&curios, DivergentUniverseCurioCategory::Negative)[0].clone();
    for family in FAMILIES {
        for inventory in [
            vec![],
            vec![common.clone()],
            vec![common.clone(), rare.clone()],
            vec![common.clone(), negative.clone()],
        ] {
            let scenario = scenario_with_inventory(&fixture, family, 106, 1, false, &inventory);
            let rooms = bound(&scenario);
            let mut activity = start(&scenario);
            let _ = enter(&scenario, &rooms, &mut activity);
            assert_eq!(options(&activity), vec![LEAVE_SYNTHESIS]);
        }
    }
}

#[test]
fn synthesis_room_sparse_top_tier_pools_offer_all_remaining_owners_without_duplicates() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let legendary = members(&curios, CATEGORIES[2]);
    for family in FAMILIES {
        for remaining in 0..=2 {
            let scenario =
                scenario_with_inventory(&fixture, family, 111, 1, false, &legendary[remaining..]);
            let rooms = bound(&scenario);
            let mut activity = start(&scenario);
            let room = enter(&scenario, &rooms, &mut activity);
            if remaining == 0 {
                assert_eq!(options(&activity), vec![LEAVE_SYNTHESIS]);
                continue;
            }
            let before = reward_draws(&activity);
            choose(room, &mut activity, OPEN_SYNTHESIS);
            let first = curios.state(&legendary[remaining]).unwrap().state_key();
            let second = curios.state(&legendary[remaining + 1]).unwrap().state_key();
            choose(room, &mut activity, first);
            choose(room, &mut activity, second);
            assert_eq!(options(&activity).len(), remaining);
            assert_eq!(reward_draws(&activity) - before, remaining as u64);
            let owners = options(&activity)
                .into_iter()
                .map(|key| {
                    curios
                        .states()
                        .iter()
                        .find(|state| state.state_key() == key)
                        .unwrap()
                        .curio()
                        .unwrap()
                        .clone()
                })
                .collect::<Vec<_>>();
            assert!(
                owners
                    .iter()
                    .enumerate()
                    .all(|(index, owner)| !owners[..index].contains(owner))
            );
        }
    }
}
