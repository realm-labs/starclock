use starclock_activity::{
    ActivityTerminalOutcome, ActivityValue, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

use crate::error::UniverseCatalogLoadErrorKind;

use super::*;

const BUNDLE: &[u8] = include_bytes!("../../../../config/swarm-disaster-generated/config.sora");

#[test]
fn compiles_entry_into_exact_sixteen_slot_activity_profile() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let entry = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![("swarm-disaster.communing-dimension.1".into(), 3)],
        vec!["swarm-disaster.communing-trail.101".into()],
        Some("swarm-disaster.trailblaze-bonus.101".into()),
    );
    let instance = factory.compile_entry(entry).unwrap();
    assert_eq!(instance.area(), "swarm-disaster.area.201");
    assert_eq!(instance.difficulty(), 1);
    assert_eq!(instance.path(), "universe.path.preservation");
    assert_eq!(instance.audience_die(), "swarm-disaster.audience-die.1");
    assert_eq!(instance.state_definition().slots().len(), 16);
    assert_eq!(
        instance.participants().digest(),
        participants(policy()).digest()
    );
    assert_eq!(
        slot_value(&instance, 0x5344_0008),
        &ActivityValue::BoundedInteger(20)
    );
    assert_eq!(
        slot_value(&instance, 0x5344_0005),
        &ActivityValue::BoundedCounterMap(vec![(1, 50)].into_boxed_slice())
    );
}

#[test]
fn compiles_all_five_difficulties_and_eight_path_die_pairs() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let pairs = [
        (
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
        ),
        ("universe.path.remembrance", "swarm-disaster.audience-die.2"),
        ("universe.path.nihility", "swarm-disaster.audience-die.3"),
        ("universe.path.abundance", "swarm-disaster.audience-die.4"),
        ("universe.path.hunt", "swarm-disaster.audience-die.5"),
        ("universe.path.destruction", "swarm-disaster.audience-die.6"),
        ("universe.path.elation", "swarm-disaster.audience-die.7"),
        ("universe.path.propagation", "swarm-disaster.audience-die.8"),
    ];
    let mut expected_graph = None;
    for difficulty in 1_u8..=5 {
        for (path, die) in pairs {
            let instance = factory
                .compile_entry(SwarmDisasterEntry::new(
                    format!("swarm-disaster.area.20{difficulty}"),
                    path,
                    die,
                    participants(policy()),
                ))
                .unwrap();
            assert_eq!(instance.difficulty(), difficulty);
            assert_eq!(instance.path(), path);
            assert_eq!(instance.audience_die(), die);
            let digest = instance.graph_definition().digest();
            assert_eq!(*expected_graph.get_or_insert(digest), digest);
        }
    }
}

#[test]
fn compiles_canonical_bounded_three_plane_topology() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = factory
        .compile_entry(SwarmDisasterEntry::new(
            "swarm-disaster.area.205",
            "universe.path.propagation",
            "swarm-disaster.audience-die.8",
            participants(policy()),
        ))
        .unwrap();
    let graph = instance.graph_definition();

    assert_eq!(
        SWARM_DISASTER_TOPOLOGY_REVISION,
        "swarm-disaster-topology-policy-v1"
    );
    assert_eq!(
        instance.planes().collect::<Vec<_>>(),
        [
            "swarm-disaster.plane.2011",
            "swarm-disaster.plane.2012",
            "swarm-disaster.plane.2013",
        ]
    );
    assert_eq!(
        instance.chessboards().collect::<Vec<_>>(),
        [
            "swarm-disaster.chessboard.20111",
            "swarm-disaster.chessboard.20121",
            "swarm-disaster.chessboard.20131",
        ]
    );
    assert_eq!(graph.nodes().len(), 48);
    assert_eq!(graph.edges().len(), 61);
    assert_eq!(graph.maximum_total_visits(), 48);
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
            .filter(|node| node.kind().terminal() == Some(ActivityTerminalOutcome::Completed))
            .count(),
        1
    );
    assert_eq!(
        graph.digest().bytes(),
        [
            0xe3, 0x71, 0xd5, 0xf7, 0xd6, 0x8f, 0x58, 0x9e, 0x50, 0xdd, 0x57, 0xe0, 0x33, 0xa8,
            0x57, 0x24, 0x16, 0x63, 0xa1, 0xa1, 0x02, 0x60, 0x21, 0x6b, 0x33, 0x42, 0xa0, 0xfa,
            0xac, 0x4f, 0x1c, 0x80,
        ]
    );
}

#[test]
fn topology_scopes_and_route_validation_are_bounded_and_fail_closed() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = factory
        .compile_entry(SwarmDisasterEntry::new(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            participants(policy()),
        ))
        .unwrap();
    let scopes = instance.state_definition().logical_scopes();
    assert_eq!(scopes.classes().len(), 3);
    assert_eq!(scopes.bindings().len(), 48);
    assert_eq!(
        scopes
            .classes()
            .iter()
            .map(|class| (class.id().get(), class.maximum_instances()))
            .collect::<Vec<_>>(),
        [
            (super::topology::PLANE_BOARD_SCOPE_CLASS, 3),
            (super::topology::BOARD_NODE_VISIT_SCOPE_CLASS, 1_991),
            (super::topology::NODE_INTERACTION_SCOPE_CLASS, 8_192),
        ]
    );
    assert_eq!(
        scopes
            .bindings()
            .iter()
            .filter(|binding| binding.path().len() == 3)
            .count(),
        47
    );

    let mut bad_order = factory.structural.topology_input(1).unwrap();
    bad_order.planes[0].plane_number = 2;
    assert_eq!(
        super::topology::compile(bad_order).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidGraph
    );
    let mut bad_route = factory.structural.topology_input(1).unwrap();
    bad_route.planes[0].edges[0].target = bad_route.planes[0].edges[0].source;
    assert_eq!(
        super::topology::compile(bad_route).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidGraph
    );
}

#[test]
fn rejects_mismatched_die_duplicate_progression_and_participant_policy() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let mismatch = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.2",
        participants(policy()),
    );
    assert_eq!(
        factory.compile_entry(mismatch).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
    let duplicate = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![
            ("swarm-disaster.communing-dimension.1".into(), 1),
            ("swarm-disaster.communing-dimension.1".into(), 2),
        ],
        vec![],
        None,
    );
    assert_eq!(
        factory.compile_entry(duplicate).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
    let wrong_policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Node,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let invalid = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(wrong_policy),
    );
    assert_eq!(
        factory.compile_entry(invalid).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
    let guide_area = SwarmDisasterEntry::new(
        "swarm-disaster.area.101",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    );
    assert_eq!(
        factory.compile_entry(guide_area).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
    let overflow = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![("swarm-disaster.communing-dimension.1".into(), u16::MAX)],
        vec![],
        None,
    );
    assert_eq!(
        factory.compile_entry(overflow).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
    let unknown_bonus = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![],
        vec![],
        Some("swarm-disaster.trailblaze-bonus.999".into()),
    );
    assert_eq!(
        factory.compile_entry(unknown_bonus).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
}

fn policy() -> ParticipantPolicy {
    ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap()
}

fn participants(policy: ParticipantPolicy) -> ParticipantLock {
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32]).unwrap(),
                BuildDigest::new([byte + 32; 32]).unwrap(),
                "swarm-entry-test-build-v1",
                ParticipantSourceKind::CompiledBuild,
            )
            .unwrap();
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1).unwrap(),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).unwrap(),
                build,
            )
            .unwrap()
        })
        .collect();
    ParticipantLock::seal(policy, entries).unwrap()
}

fn slot_value(instance: &SwarmDisasterRuntimeInstance, id: u32) -> &ActivityValue {
    instance
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id().get() == id)
        .unwrap()
        .initial()
}
