//! Raw/stale/hidden/foreign choices and hostile immutable graph changes.

use super::{
    BoundCurioSynthesisRoom, CATEGORIES, FAMILIES, OPEN_SYNTHESIS, SERVICE, bound, choose, enter,
    instance, members, scenario_with_inventory, start, value,
};
use crate::divergent_universe::DivergentUniverseBaselineFixture;
use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityRandomPolicies, ActivityStateHash, ActivityValue,
    GraphActivity, GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram,
};
use std::sync::Arc;

#[test]
fn synthesis_room_raw_hidden_stale_and_foreign_commands_are_byte_and_rng_inert_in_all_phases() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let inputs = members(&curios, CATEGORIES[0])[..2].to_vec();
    for family in FAMILIES {
        let scenario = scenario_with_inventory(&fixture, family, 106, 1, false, &inputs);
        let rooms = bound(&scenario);
        let mut activity = start(&scenario);
        let room = enter(&scenario, &rooms, &mut activity);
        let stale = GraphActivity::start(
            Arc::clone(&scenario.definition),
            instance(2565),
            ActivityMasterSeed::from_u64(999),
        )
        .unwrap()
        .into_activity()
        .state_hash();
        let foreign = scenario_with_inventory(&fixture, family, 107, 1, false, &inputs);
        let foreign_rooms = bound(&foreign);
        for selected in [
            OPEN_SYNTHESIS,
            curios.state(&inputs[0]).unwrap().state_key(),
            curios.state(&inputs[1]).unwrap().state_key(),
        ] {
            reject_all(&mut activity, room, &foreign_rooms, stale);
            choose(room, &mut activity, selected);
        }
        reject_all(&mut activity, room, &foreign_rooms, stale);
        assert_eq!(
            value(&activity, SERVICE.accepted),
            ActivityValue::Boolean(false)
        );
    }
}

fn reject_all(
    activity: &mut GraphActivity,
    room: &BoundCurioSynthesisRoom,
    foreign: &[BoundCurioSynthesisRoom],
    stale: ActivityStateHash,
) {
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    for _ in 0..2 {
        assert!(
            activity
                .choose_option(hash, offer.id(), offer.options()[0].id())
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert!(matches!(
            room.choose(activity, stale, offer.id(), offer.options()[0].id()),
            Err(GraphActivityCommandError::StaleStateHash)
        ));
        assert_eq!(activity.canonical_state_bytes(), before);
        assert!(
            room.choose(
                activity,
                hash,
                offer.id(),
                ActivityOptionId::new(u64::MAX - 999).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        for foreign in foreign {
            assert!(foreign.offered(activity).is_none());
            assert!(
                foreign
                    .choose(activity, hash, offer.id(), offer.options()[0].id())
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
}

#[test]
fn synthesis_room_binding_rejects_changed_program_before_issuing_a_capability() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let inputs = members(&curios, CATEGORIES[0])[..2].to_vec();
    let scenario = scenario_with_inventory(&fixture, FAMILIES[0], 106, 1, false, &inputs);
    let room = &scenario.rooms[0];
    let definition = &scenario.definition;
    let mut programs = definition.programs().to_vec();
    let record = programs
        .iter_mut()
        .find(|record| record.node() == room.menu_node())
        .unwrap();
    let mut operations = record.program().operations().to_vec();
    operations[1] = ActivityOperation::SetSlot {
        slot: SERVICE.accepted,
        value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
    };
    *record = GraphActivityNodeProgram::new(
        record.node(),
        ActivityProgramDefinition::new(record.program().id(), operations).unwrap(),
    );
    let hostile = Arc::new(
        GraphActivityDefinition::new(
            definition.identity(),
            definition.graph().clone(),
            definition.state_definition().clone(),
            Arc::clone(definition.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(
                definition.random_checkpoints().to_vec(),
                definition.random_offers().to_vec(),
            ),
        )
        .unwrap(),
    );
    assert!(room.bind(Arc::clone(&hostile)).is_err());
    let mut activity =
        GraphActivity::start(hostile, instance(2565), ActivityMasterSeed::from_u64(2565))
            .unwrap()
            .into_activity();
    let original = room.bind(Arc::clone(definition)).unwrap();
    assert!(original.offered(&activity).is_none());
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    assert!(
        original
            .choose(&mut activity, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
