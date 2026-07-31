use std::sync::OnceLock;

use starclock_activity::{
    ActivityScope, ActivityTerminalOutcome, ActivityValue, BuildDigest, LoadoutLockScope,
    OpaqueParticipantBuild, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope, SlotValueKind,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

use crate::{
    gold_gears_entry::state_layout::{
        RESOURCE_COSMIC_FRAGMENTS_KEY, RESOURCE_DICE_CHEATS_KEY, RESOURCE_DICE_REROLLS_KEY,
        RUN_RESOURCES_SLOT,
    },
    gold_gears_structural::AreaGroup,
    gold_gears_unique::DiceDefinition,
};

use super::*;

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

pub(super) fn shared_factory() -> &'static GoldAndGearsRuntimeFactory {
    static FACTORY: OnceLock<GoldAndGearsRuntimeFactory> = OnceLock::new();
    FACTORY.get_or_init(|| GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap())
}

fn participant_policy() -> ParticipantPolicy {
    ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("test policy")
}

fn participants(policy: ParticipantPolicy) -> ParticipantLock {
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32]).expect("resolved digest"),
                BuildDigest::new([byte + 32; 32]).expect("build digest"),
                "gold-entry-test-build-v1",
                ParticipantSourceKind::CompiledBuild,
            )
            .expect("opaque build");
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1).expect("participant"),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).expect("unit"),
                build,
            )
            .expect("participant entry")
        })
        .collect();
    ParticipantLock::seal(policy, entries).expect("participant lock")
}

fn default_face_keys(factory: &GoldAndGearsRuntimeFactory, dice: &DiceDefinition) -> Vec<String> {
    dice.default_face_sources
        .iter()
        .map(|source| {
            factory
                .unique
                .dice_faces
                .iter()
                .find(|face| face.identity.source_id == *source)
                .expect("default face source")
                .identity
                .stable_key
                .to_string()
        })
        .collect()
}

fn all_dice(factory: &GoldAndGearsRuntimeFactory) -> Vec<String> {
    factory
        .unique
        .dice
        .iter()
        .map(|dice| dice.identity.stable_key.to_string())
        .collect()
}

pub(super) fn entry(
    factory: &GoldAndGearsRuntimeFactory,
    area: &str,
    path: &str,
    dice: &DiceDefinition,
) -> GoldAndGearsEntry {
    GoldAndGearsEntry::new(
        area,
        path,
        dice.identity.stable_key.clone(),
        default_face_keys(factory, dice),
        participants(participant_policy()),
    )
    .with_unlocked_dice(all_dice(factory))
}

#[test]
fn every_formal_difficulty_path_and_custom_dice_compiles() {
    let factory = shared_factory();
    let areas = factory
        .structural
        .areas
        .iter()
        .filter(|area| area.group == AreaGroup::Formal)
        .map(|area| area.stable_key.to_string())
        .collect::<Vec<_>>();
    let paths = factory
        .unique
        .paths
        .iter()
        .map(|path| path.identity.stable_key.to_string())
        .collect::<Vec<_>>();
    let mut compiled = 0;

    for area in &areas {
        for path in &paths {
            for dice in &factory.unique.dice {
                let instance = factory
                    .compile_entry(entry(factory, area, path, dice))
                    .expect("valid explicit entry");
                assert_eq!(instance.area(), area);
                assert_eq!(instance.path(), path);
                assert_eq!(instance.custom_dice(), dice.identity.stable_key.as_ref());
                assert_eq!(instance.dice_faces().len(), 6);
                compiled += 1;
            }
        }
    }

    assert_eq!(areas.len(), 5);
    assert_eq!(paths.len(), 9);
    assert_eq!(factory.unique.dice.len(), 12);
    assert_eq!(compiled, 540);
}

#[test]
fn full_progression_and_combined_conundrum_compile_canonically() {
    let factory = shared_factory();
    let area = factory
        .structural
        .areas
        .iter()
        .find(|area| area.stable_key.as_ref() == CONUNDRUM_AREA_KEY)
        .unwrap();
    let path = &factory.unique.paths[0];
    let dice = &factory.unique.dice[0];
    let mut neural = factory
        .unique
        .neural_nodes
        .iter()
        .map(|node| node.identity.stable_key.to_string())
        .collect::<Vec<_>>();
    neural.reverse();
    let bonus = &factory.unique.trailblaze_bonuses[0];
    let instance = factory
        .compile_entry(
            entry(factory, &area.stable_key, &path.identity.stable_key, dice)
                .with_neural_network(neural)
                .with_conundrum(6, 6, vec![CONUNDRUM_AREA_KEY.to_owned()])
                .with_trailblaze_bonus(bonus.identity.stable_key.clone()),
        )
        .unwrap();

    assert_eq!(instance.difficulty(), 5);
    assert_eq!(instance.neural_network().len(), 40);
    assert_eq!(instance.stats_conundrum(), 6);
    assert_eq!(instance.auxiliary_conundrum(), 6);
    assert_eq!(
        instance.trailblaze_bonus(),
        Some(bonus.identity.stable_key.as_ref())
    );
    let ids = instance
        .neural_network()
        .map(|key| {
            factory
                .unique
                .neural_nodes
                .iter()
                .find(|node| node.identity.stable_key.as_ref() == key)
                .unwrap()
                .identity
                .id
                .0
        })
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn compiled_state_matches_all_seventeen_frozen_slot_families() {
    let factory = shared_factory();
    let dice = &factory.unique.dice[0];
    let instance = factory
        .compile_entry(entry(
            factory,
            "gold-gears.area.401",
            "universe.path.preservation",
            dice,
        ))
        .unwrap();
    let slots = instance.state_definition().slots();

    assert_eq!(slots.len(), 17);
    assert_eq!(
        slots.iter().map(|slot| slot.kind()).collect::<Vec<_>>(),
        vec![
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::OrderedIdSet,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedInteger,
            SlotValueKind::OrderedIdSet,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
            SlotValueKind::BoundedCounterMap,
        ]
    );
    assert_eq!(
        slots.iter().map(|slot| slot.owner()).collect::<Vec<_>>(),
        [
            vec![ActivityScope::Activity; 10],
            vec![ActivityScope::Section; 5],
            vec![ActivityScope::Node],
            vec![ActivityScope::Attempt],
        ]
        .concat()
    );
    let resources = slots
        .iter()
        .find(|slot| slot.id().get() == RUN_RESOURCES_SLOT)
        .unwrap();
    assert_eq!(
        resources.initial(),
        &ActivityValue::BoundedCounterMap(
            vec![
                (RESOURCE_COSMIC_FRAGMENTS_KEY, 100),
                (RESOURCE_DICE_REROLLS_KEY, 1),
                (RESOURCE_DICE_CHEATS_KEY, 0),
            ]
            .into_boxed_slice()
        )
    );
}

#[test]
fn invalid_roster_neural_and_conundrum_inputs_fail_closed() {
    let factory = shared_factory();
    let dice = &factory.unique.dice[0];
    let team_policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Team,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let wrong_roster = GoldAndGearsEntry::new(
        "gold-gears.area.401",
        "universe.path.preservation",
        dice.identity.stable_key.clone(),
        default_face_keys(factory, dice),
        participants(team_policy),
    );
    assert_eq!(
        factory.compile_entry(wrong_roster).unwrap_err(),
        GoldAndGearsEntryError::ParticipantPolicyMismatch
    );

    let child = factory
        .unique
        .neural_nodes
        .iter()
        .find(|node| !node.prerequisites.is_empty())
        .unwrap();
    let missing = factory
        .compile_entry(
            entry(
                factory,
                "gold-gears.area.401",
                "universe.path.preservation",
                dice,
            )
            .with_neural_network(vec![child.identity.stable_key.to_string()]),
        )
        .unwrap_err();
    assert!(matches!(
        missing,
        GoldAndGearsEntryError::MissingNeuralPrerequisite { .. }
    ));

    assert_eq!(
        factory
            .compile_entry(
                entry(
                    factory,
                    "gold-gears.area.401",
                    "universe.path.preservation",
                    dice,
                )
                .with_conundrum(1, 0, vec![CONUNDRUM_AREA_KEY.to_owned()])
            )
            .unwrap_err(),
        GoldAndGearsEntryError::ConundrumDifficultyMismatch
    );
    assert_eq!(
        factory
            .compile_entry(
                entry(
                    factory,
                    CONUNDRUM_AREA_KEY,
                    "universe.path.preservation",
                    dice,
                )
                .with_conundrum(1, 0, vec![])
            )
            .unwrap_err(),
        GoldAndGearsEntryError::MissingConundrumPrerequisite
    );
}

#[test]
fn locked_unknown_and_incompatible_loadouts_fail_closed() {
    let factory = shared_factory();
    let locked = factory
        .unique
        .dice
        .iter()
        .find(|dice| !dice.available_by_default)
        .unwrap();
    let locked_entry = GoldAndGearsEntry::new(
        "gold-gears.area.401",
        "universe.path.preservation",
        locked.identity.stable_key.clone(),
        default_face_keys(factory, locked),
        participants(participant_policy()),
    );
    assert_eq!(
        factory.compile_entry(locked_entry).unwrap_err(),
        GoldAndGearsEntryError::LockedDice(locked.identity.stable_key.clone())
    );

    let dice = &factory.unique.dice[0];
    let mut duplicate = default_face_keys(factory, dice);
    duplicate[1] = duplicate[0].clone();
    let duplicate_entry = GoldAndGearsEntry::new(
        "gold-gears.area.401",
        "universe.path.preservation",
        dice.identity.stable_key.clone(),
        duplicate,
        participants(participant_policy()),
    );
    assert!(matches!(
        factory.compile_entry(duplicate_entry),
        Err(GoldAndGearsEntryError::DuplicateDiceFace(_))
            | Err(GoldAndGearsEntryError::DiceFaceSlotMismatch(_))
    ));

    let unknown = GoldAndGearsEntry::new(
        "gold-gears.area.999",
        "universe.path.preservation",
        dice.identity.stable_key.clone(),
        default_face_keys(factory, dice),
        participants(participant_policy()),
    );
    assert_eq!(
        factory.compile_entry(unknown).unwrap_err(),
        GoldAndGearsEntryError::UnknownArea("gold-gears.area.999".into())
    );
}

#[test]
fn entry_revision_is_frozen() {
    assert_eq!(
        GOLD_AND_GEARS_ENTRY_REVISION,
        "gold-and-gears-entry-policy-v1"
    );
}

#[test]
fn formal_entry_compiles_canonical_three_plane_activity_graph() {
    let factory = shared_factory();
    let instance = factory
        .compile_entry(entry(
            factory,
            "gold-gears.area.401",
            &factory.unique.paths[0].identity.stable_key,
            &factory.unique.dice[0],
        ))
        .expect("formal topology");
    let graph = instance.graph_definition();

    assert_eq!(
        instance.planes().collect::<Vec<_>>(),
        [
            "gold-gears.plane.2021",
            "gold-gears.plane.2022",
            "gold-gears.plane.2023",
        ]
    );
    assert_eq!(
        instance.chessboards().collect::<Vec<_>>(),
        [
            "gold-gears.chessboard.2112021",
            "gold-gears.chessboard.2112022",
            "gold-gears.chessboard.2112023",
        ]
    );
    assert_eq!(graph.nodes().len(), 82);
    assert_eq!(graph.edges().len(), 123);
    assert_eq!(graph.maximum_total_visits(), 82);
    assert_eq!(
        graph.digest().bytes(),
        [
            79, 7, 24, 58, 74, 83, 24, 146, 8, 164, 2, 166, 174, 105, 163, 219, 228, 145, 246, 120,
            37, 45, 138, 27, 160, 76, 155, 165, 0, 11, 202, 72,
        ]
    );
    assert!(
        graph
            .nodes()
            .windows(2)
            .all(|pair| pair[0].id() < pair[1].id())
    );
    assert!(
        graph
            .edges()
            .windows(2)
            .all(|pair| pair[0].id() < pair[1].id())
    );
    assert!(graph.nodes().iter().all(|node| node.maximum_visits() == 1));
    assert!(
        graph
            .edges()
            .iter()
            .all(|edge| edge.maximum_traversals() == 1)
    );
    assert_eq!(
        graph
            .nodes()
            .iter()
            .filter_map(|node| node.kind().terminal())
            .collect::<Vec<_>>(),
        [ActivityTerminalOutcome::Completed]
    );
    for section in 1..=3 {
        assert_eq!(
            graph
                .nodes()
                .iter()
                .filter(|node| node.section().get() == section)
                .count(),
            if section == 3 { 28 } else { 27 }
        );
    }

    let authored_edge_count = u32::try_from(factory.structural.edges.len()).unwrap();
    let columns = &factory.structural.columns;
    let nodes = &factory.structural.nodes;
    for edge in graph
        .edges()
        .iter()
        .filter(|edge| edge.id().get() <= authored_edge_count)
    {
        let source = nodes
            .iter()
            .find(|node| node.id.0 == edge.from().get())
            .unwrap();
        let target = nodes
            .iter()
            .find(|node| node.id.0 == edge.to().get())
            .unwrap();
        let source_column = columns
            .iter()
            .find(|column| column.id == source.column)
            .unwrap();
        let target_column = columns
            .iter()
            .find(|column| column.id == target.column)
            .unwrap();
        assert_eq!(target_column.index, source_column.index + 1);
    }
    assert_eq!(
        graph
            .edges()
            .iter()
            .filter(|edge| edge.id().get() > authored_edge_count)
            .count(),
        3
    );
}

#[test]
fn topology_binds_three_bounded_logical_lifetimes_to_every_board_node() {
    let factory = shared_factory();
    let instance = factory
        .compile_entry(entry(
            factory,
            "gold-gears.area.405",
            &factory.unique.paths[0].identity.stable_key,
            &factory.unique.dice[0],
        ))
        .expect("formal topology");
    let scopes = instance.state_definition().logical_scopes();

    assert_eq!(
        GOLD_AND_GEARS_TOPOLOGY_REVISION,
        "gold-and-gears-topology-policy-v1"
    );
    assert_eq!(scopes.classes().len(), 3);
    assert_eq!(scopes.bindings().len(), 82);
    assert_eq!(
        scopes
            .classes()
            .iter()
            .map(|class| (class.id().get(), class.maximum_instances()))
            .collect::<Vec<_>>(),
        [
            (super::topology::PLANE_BOARD_SCOPE_CLASS, 3),
            (super::topology::BOARD_NODE_VISIT_SCOPE_CLASS, 2_502),
            (super::topology::NODE_INTERACTION_SCOPE_CLASS, 8_192),
        ]
    );
    assert_eq!(
        scopes
            .bindings()
            .iter()
            .filter(|binding| binding.path().len() == 3)
            .count(),
        81
    );
    assert_eq!(
        scopes
            .bindings()
            .iter()
            .filter(|binding| binding.path().len() == 1)
            .count(),
        1
    );
}

pub(super) fn compiled_fixture(
    factory: &GoldAndGearsRuntimeFactory,
) -> GoldAndGearsRuntimeInstance {
    factory
        .compile_entry(entry(
            factory,
            "gold-gears.area.401",
            &factory.unique.paths[0].identity.stable_key,
            &factory.unique.dice[0],
        ))
        .expect("compiled fixture")
}
