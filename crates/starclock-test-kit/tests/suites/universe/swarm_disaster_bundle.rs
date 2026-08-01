use starclock_activity::{
    BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    error::UniverseCatalogLoadErrorKind,
    gold_gears_components::gold_and_gears_component_set,
    gold_gears_identity::GoldAndGearsCatalogIdentity,
    swarm_disaster_catalog::validate_swarm_disaster_bundle,
    swarm_disaster_components::swarm_disaster_component_set,
    swarm_disaster_entry::{SwarmDisasterEntry, SwarmDisasterRuntimeFactory},
};
use starclock_replay::component::ConfigurationComponentKind;

const BUNDLE: &[u8] = include_bytes!("../../../../../config/swarm-disaster-generated/config.sora");

#[test]
fn exact_goal09_bundle_loads_through_the_generated_type_free_boundary() {
    validate_swarm_disaster_bundle(BUNDLE).unwrap();
}

#[test]
fn a_different_or_tampered_bundle_is_rejected_before_lowering() {
    let mut tampered = BUNDLE.to_vec();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert_eq!(
        validate_swarm_disaster_bundle(&tampered)
            .unwrap_err()
            .kind(),
        UniverseCatalogLoadErrorKind::InvalidEmbeddedData
    );
    assert_eq!(
        validate_swarm_disaster_bundle(include_bytes!(
            "../../../../../config/universe-generated/config.sora"
        ))
        .unwrap_err()
        .kind(),
        UniverseCatalogLoadErrorKind::InvalidEmbeddedData
    );
}

#[test]
fn component_set_has_exact_ten_component_closure_and_stable_order() {
    let components = swarm_components(0x66);
    assert_eq!(components.components().len(), 10);
    assert_eq!(
        components
            .components()
            .iter()
            .map(|component| (component.kind(), component.id()))
            .collect::<Vec<_>>(),
        [
            (ConfigurationComponentKind::CombatCatalog, "combat-catalog"),
            (ConfigurationComponentKind::BuildCatalog, "build-catalog"),
            (
                ConfigurationComponentKind::ActivityCore,
                "swarm-disaster-activity"
            ),
            (
                ConfigurationComponentKind::ModeProfile,
                "swarm-disaster-profile"
            ),
            (
                ConfigurationComponentKind::ModeContent,
                "swarm-disaster-content"
            ),
            (
                ConfigurationComponentKind::ModeContent,
                "universe-shared-content"
            ),
            (
                ConfigurationComponentKind::ActivityHandlerRegistry,
                "swarm-disaster-activity-handlers"
            ),
            (
                ConfigurationComponentKind::CombatRuleRegistry,
                "swarm-disaster-combat-rules"
            ),
            (
                ConfigurationComponentKind::EncounterOverlay,
                "swarm-disaster-encounter-overlay"
            ),
            (
                ConfigurationComponentKind::Controller,
                "baseline-controller"
            ),
        ]
    );
    assert_eq!(
        components.root().bytes(),
        [
            0x26, 0xb6, 0x94, 0x3c, 0x7f, 0xd7, 0xf0, 0x78, 0xaf, 0xc0, 0x87, 0x0d, 0x01, 0x5b,
            0x7b, 0x44, 0x00, 0xb8, 0xb5, 0xf5, 0xd8, 0xb4, 0xa0, 0xb8, 0xae, 0xc1, 0xe2, 0x2d,
            0xf5, 0xf0, 0x7a, 0x0a,
        ]
    );
    assert_eq!(
        components.components()[3].digest().bytes(),
        [
            0x7d, 0xa5, 0xca, 0x6c, 0xf0, 0x30, 0x42, 0x93, 0x7a, 0x20, 0x62, 0x45, 0xbd, 0x82,
            0xec, 0x4a, 0x78, 0xe4, 0x95, 0xd6, 0x38, 0xb6, 0xbb, 0x0c, 0x07, 0xa6, 0xea, 0xbc,
            0xcd, 0x02, 0x04, 0x68,
        ]
    );
    assert_eq!(
        components.components()[6].digest().bytes(),
        [
            0x67, 0x0c, 0xc5, 0x3e, 0x9b, 0x31, 0x4b, 0x20, 0xe1, 0xfe, 0x07, 0x52, 0xdb, 0xb2,
            0x0a, 0x70, 0xc5, 0x40, 0x4d, 0x28, 0xe8, 0x2f, 0xfc, 0xc4, 0xc0, 0x0a, 0x2e, 0x8c,
            0x4c, 0xce, 0x05, 0x28,
        ]
    );
}

#[test]
fn component_roots_are_mode_scoped_and_controller_sensitive() {
    let swarm = swarm_components(0x66);
    let changed_controller = swarm_components(0x67);
    assert_ne!(swarm.root(), changed_controller.root());

    let gold_identity = GoldAndGearsCatalogIdentity::load(include_bytes!(
        "../../../../../config/gold-and-gears-generated/config.sora"
    ))
    .unwrap();
    let gold = gold_and_gears_component_set(
        &gold_identity,
        ("combat-v1", [0x11; 32]),
        ("build-v1", [0x22; 32]),
        [0x33; 32],
        [0x44; 32],
        [0x55; 32],
        ("baseline-controller", "baseline-v1", [0x66; 32]),
    )
    .unwrap();
    assert_eq!(
        gold.root().bytes(),
        [
            0x93, 0xc5, 0x0f, 0x43, 0x0c, 0xf8, 0x95, 0x0b, 0xb4, 0x0f, 0xc1, 0x80, 0xd3, 0x55,
            0xad, 0xbc, 0x67, 0x19, 0xd8, 0xf5, 0x6a, 0xe4, 0x34, 0xda, 0x4f, 0x8a, 0xac, 0x30,
            0x68, 0x50, 0x9b, 0x18,
        ]
    );
    assert_ne!(swarm.root(), gold.root());
}

#[test]
fn public_factory_compiles_a_locked_entry_and_bounded_topology_without_rng() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = factory
        .compile_entry(SwarmDisasterEntry::new(
            "swarm-disaster.area.205",
            "universe.path.propagation",
            "swarm-disaster.audience-die.8",
            participants(),
        ))
        .unwrap();
    assert_eq!(instance.difficulty(), 5);
    assert_eq!(instance.state_definition().slots().len(), 16);
    assert_eq!(instance.graph_definition().nodes().len(), 48);
    assert_eq!(instance.graph_definition().edges().len(), 61);
    assert_eq!(
        instance.chessboards().collect::<Vec<_>>(),
        [
            "swarm-disaster.chessboard.20111",
            "swarm-disaster.chessboard.20121",
            "swarm-disaster.chessboard.20131",
        ]
    );
}

fn swarm_components(controller: u8) -> starclock_replay::component::ConfigurationComponentSet {
    swarm_disaster_component_set(
        BUNDLE,
        ("combat-v1", [0x11; 32]),
        ("build-v1", [0x22; 32]),
        [0x33; 32],
        [0x44; 32],
        [0x55; 32],
        ("baseline-controller", "baseline-v1", [controller; 32]),
    )
    .unwrap()
}

fn participants() -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32]).unwrap(),
                BuildDigest::new([byte + 32; 32]).unwrap(),
                "swarm-entry-integration-v1",
                ParticipantSourceKind::CompiledBuild,
            )
            .unwrap();
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1).unwrap(),
                0,
                index,
                UnitDefinitionId::new(30_001 + u32::from(index)).unwrap(),
                build,
            )
            .unwrap()
        })
        .collect();
    ParticipantLock::seal(policy, entries).unwrap()
}
