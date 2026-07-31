use std::collections::BTreeSet;

use starclock_activity::ActivityValue;

use crate::gold_gears_unique::{DiceDefinition, DiceFace};

use super::{
    GOLD_AND_GEARS_DICE_LOADOUT_REVISION, GoldAndGearsEntryError, GoldAndGearsRuntimeFactory,
    state_layout::{DICE_LOADOUT_MAX_RARITY_KEY_BASE, DICE_LOADOUT_SLOT},
    tests::entry,
};

const AREA: &str = "gold-gears.area.401";
const PATH: &str = "universe.path.preservation";

#[test]
fn all_default_loadouts_and_recommendation_pools_are_legal() {
    let factory = super::tests::shared_factory();

    for dice in &factory.unique.dice {
        let instance = factory
            .compile_entry(entry(factory, AREA, PATH, dice))
            .expect("released default loadout");
        assert_eq!(
            instance.dice_slot_max_rarities().collect::<Vec<_>>(),
            [3, 3, 2, 2, 1, 1]
        );
        let selected = instance.dice_faces().collect::<Vec<_>>();
        for (index, face) in selected.iter().enumerate() {
            assert!(
                instance
                    .eligible_dice_faces(u8::try_from(index + 1).unwrap())
                    .expect("six stable slots")
                    .any(|candidate| candidate == *face),
                "selected face {face} must remain eligible"
            );
        }
        assert_recommendations_are_eligible(&instance, instance.suggestive_dice_faces().collect());
        assert_recommendations_are_eligible(&instance, instance.recommended_dice_faces().collect());
        assert!(instance.eligible_dice_faces(0).is_none());
        assert!(instance.eligible_dice_faces(7).is_none());
    }

    assert_eq!(factory.unique.dice.len(), 12);
    assert_eq!(
        GOLD_AND_GEARS_DICE_LOADOUT_REVISION,
        "gold-and-gears-dice-loadout-policy-v1"
    );
}

#[test]
fn face_unlock_groups_fail_closed_until_their_dice_are_unlocked() {
    let factory = super::tests::shared_factory();
    let default_dice = factory
        .unique
        .dice
        .iter()
        .find(|dice| dice.available_by_default)
        .unwrap();
    factory
        .compile_entry(entry(factory, AREA, PATH, default_dice).with_unlocked_dice(vec![]))
        .expect("baseline dice and its face group are implicitly unlocked");

    let locked_dice = factory
        .unique
        .dice
        .iter()
        .find(|dice| dice.identity.source_id.as_ref() == "102")
        .unwrap();
    let unlocked_sources = BTreeSet::from([
        "100",
        default_dice.identity.source_id.as_ref(),
        locked_dice.identity.source_id.as_ref(),
    ]);
    let locked_face = default_faces(factory, locked_dice)
        .into_iter()
        .find(|face| !unlocked_sources.contains(face.unlock_display_source.as_ref()))
        .expect("dice 102 proves cross-dice face unlock closure");
    assert_eq!(
        factory
            .compile_entry(
                entry(factory, AREA, PATH, locked_dice)
                    .with_unlocked_dice(vec![locked_dice.identity.stable_key.to_string()])
            )
            .unwrap_err(),
        GoldAndGearsEntryError::LockedDiceFace(locked_face.identity.stable_key.clone())
    );

    factory
        .compile_entry(entry(factory, AREA, PATH, locked_dice))
        .expect("all-released unlock profile closes every default face");
}

#[test]
fn neural_nodes_upgrade_slots_five_three_then_six_in_policy_order() {
    let factory = super::tests::shared_factory();
    let dice = &factory.unique.dice[0];
    let cases = [
        (0, vec![3, 3, 2, 2, 1, 1]),
        (5, vec![3, 3, 2, 2, 2, 1]),
        (26, vec![3, 3, 3, 2, 2, 1]),
        (40, vec![3, 3, 3, 2, 2, 2]),
    ];

    for (maximum_topological_index, expected) in cases {
        let neural = factory
            .unique
            .neural_nodes
            .iter()
            .filter(|node| node.topological_index <= maximum_topological_index)
            .map(|node| node.identity.stable_key.to_string())
            .collect();
        let instance = factory
            .compile_entry(entry(factory, AREA, PATH, dice).with_neural_network(neural))
            .expect("topological Neural prefix");
        assert_eq!(
            instance.dice_slot_max_rarities().collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn upgraded_rarity_and_color_constraints_change_entry_legality_and_state() {
    let factory = super::tests::shared_factory();
    let dice = &factory.unique.dice[0];
    let all_neural = factory
        .unique
        .neural_nodes
        .iter()
        .map(|node| node.identity.stable_key.to_string())
        .collect::<Vec<_>>();
    let upgraded = factory
        .compile_entry(entry(factory, AREA, PATH, dice).with_neural_network(all_neural.clone()))
        .unwrap();
    let selected = upgraded.dice_faces().map(str::to_owned).collect::<Vec<_>>();
    let rarity_three = upgraded
        .eligible_dice_faces(3)
        .unwrap()
        .find(|candidate| {
            !selected.iter().any(|selected| selected == candidate)
                && face(factory, candidate).rarity == 3
        })
        .expect("upgraded third slot has an additional rarity-three face");
    let mut upgraded_faces = selected.clone();
    upgraded_faces[2] = rarity_three.to_owned();

    assert_eq!(
        factory
            .compile_entry(entry_with_faces(factory, dice, upgraded_faces.clone()))
            .unwrap_err(),
        GoldAndGearsEntryError::DiceFaceRarityMismatch(rarity_three.into())
    );
    let accepted = factory
        .compile_entry(
            entry_with_faces(factory, dice, upgraded_faces).with_neural_network(all_neural.clone()),
        )
        .expect("slot-three Neural upgrade raises the effective cap");
    let loadout = accepted
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id().get() == DICE_LOADOUT_SLOT)
        .unwrap();
    let ActivityValue::BoundedCounterMap(values) = loadout.initial() else {
        panic!("dice loadout is a bounded map");
    };
    assert_eq!(
        values
            .iter()
            .filter(|(key, _)| *key > DICE_LOADOUT_MAX_RARITY_KEY_BASE)
            .map(|(_, value)| *value)
            .collect::<Vec<_>>(),
        [3, 3, 3, 2, 2, 2]
    );
    let color_limited = face(factory, &selected[0]);
    assert_eq!(color_limited.rarity, 3);
    let mut wrong_color = selected;
    wrong_color.swap(0, 3);
    assert_eq!(
        factory
            .compile_entry(
                entry_with_faces(factory, dice, wrong_color).with_neural_network(all_neural)
            )
            .unwrap_err(),
        GoldAndGearsEntryError::DiceFaceRarityMismatch(color_limited.identity.stable_key.clone())
    );
}

fn entry_with_faces(
    factory: &GoldAndGearsRuntimeFactory,
    dice: &DiceDefinition,
    faces: Vec<String>,
) -> super::GoldAndGearsEntry {
    let template = entry(factory, AREA, PATH, dice);
    super::GoldAndGearsEntry::new(
        template.area(),
        template.path(),
        template.custom_dice(),
        faces,
        template.participants().clone(),
    )
    .with_unlocked_dice(
        factory
            .unique
            .dice
            .iter()
            .map(|candidate| candidate.identity.stable_key.to_string())
            .collect(),
    )
}

fn default_faces<'a>(
    factory: &'a GoldAndGearsRuntimeFactory,
    dice: &DiceDefinition,
) -> Vec<&'a DiceFace> {
    dice.default_face_sources
        .iter()
        .map(|source| {
            factory
                .unique
                .dice_faces
                .iter()
                .find(|face| face.identity.source_id == *source)
                .unwrap()
        })
        .collect()
}

fn face<'a>(factory: &'a GoldAndGearsRuntimeFactory, key: &str) -> &'a DiceFace {
    factory
        .unique
        .dice_faces
        .iter()
        .find(|face| face.identity.stable_key.as_ref() == key)
        .unwrap()
}

fn assert_recommendations_are_eligible(
    instance: &super::GoldAndGearsRuntimeInstance,
    recommendations: Vec<&str>,
) {
    assert!(!recommendations.is_empty());
    let unique = recommendations.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), recommendations.len());
    for recommendation in recommendations {
        assert!((1..=6).any(|slot| {
            instance
                .eligible_dice_faces(slot)
                .is_some_and(|mut candidates| {
                    candidates.any(|candidate| candidate == recommendation)
                })
        }));
    }
}
