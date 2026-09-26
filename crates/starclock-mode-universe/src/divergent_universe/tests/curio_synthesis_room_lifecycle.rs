//! Logical-room reset and downstream graph failure use the same transaction.

use super::{
    CATEGORIES, FAMILIES, LEAVE_SYNTHESIS, OPEN_SYNTHESIS, RECEIPT, SERVICE, bound, choose, enter,
    members, options, scenario_with_inventory, start, value,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, curio_synthesis::room::CurioSynthesisRoomPhase,
    state::SERVICE_RECEIPTS_SLOT,
};
use starclock_activity::{ActivityOptionId, ActivityValue};

#[test]
fn synthesis_room_limits_reset_only_at_next_logical_room_and_run_receipts_persist() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let inputs = members(&curios, CATEGORIES[0])[..4].to_vec();
    for family in FAMILIES {
        let scenario = scenario_with_inventory(&fixture, family, 107, 1, false, &inputs);
        let rooms = bound(&scenario);
        let mut activity = start(&scenario);
        let room = enter(&scenario, &rooms, &mut activity);
        choose(room, &mut activity, OPEN_SYNTHESIS);
        choose(
            room,
            &mut activity,
            curios.state(&inputs[0]).unwrap().state_key(),
        );
        choose(
            room,
            &mut activity,
            curios.state(&inputs[1]).unwrap().state_key(),
        );
        let selected = options(&activity)[0];
        choose(room, &mut activity, selected);
        assert_eq!(
            value(&activity, SERVICE.completed),
            ActivityValue::BoundedInteger(1)
        );
        choose(room, &mut activity, LEAVE_SYNTHESIS);
        let next = enter(&scenario, &rooms, &mut activity);
        assert_eq!(next.offered(&activity), Some(CurioSynthesisRoomPhase::Menu));
        assert_eq!(
            value(&activity, SERVICE.completed),
            ActivityValue::BoundedInteger(0)
        );
        assert_eq!(
            value(&activity, SERVICE.opens),
            ActivityValue::BoundedInteger(0)
        );
        assert_eq!(
            value(&activity, SERVICE_RECEIPTS_SLOT),
            ActivityValue::BoundedCounterMap(vec![(RECEIPT, 1)].into())
        );
        assert!(options(&activity).contains(&OPEN_SYNTHESIS));
    }
}

#[test]
fn synthesis_room_leave_failure_rolls_back_scope_clock_cache_and_inventory() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let inputs = members(&curios, CATEGORIES[0])[..2].to_vec();
    for family in FAMILIES {
        let scenario = scenario_with_inventory(&fixture, family, 111, 1, true, &inputs);
        let rooms = bound(&scenario);
        let mut activity = start(&scenario);
        let second = loop {
            let room = enter(&scenario, &rooms, &mut activity);
            let context = scenario
                .rooms
                .iter()
                .find(|compiled| compiled.menu_node() == activity.current_node())
                .unwrap()
                .context();
            if context.position_ordinal == 3 {
                break room;
            }
            choose(room, &mut activity, LEAVE_SYNTHESIS);
        };
        let before = activity.canonical_state_bytes();
        let offer = activity.player_view().decision().unwrap().clone();
        for _ in 0..2 {
            let hash = activity.state_hash();
            assert!(
                second
                    .choose(
                        &mut activity,
                        hash,
                        offer.id(),
                        ActivityOptionId::new(LEAVE_SYNTHESIS).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
}
