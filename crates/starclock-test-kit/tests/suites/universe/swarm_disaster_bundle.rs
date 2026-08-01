use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngStreams, ActivityTransactionOutcome, ActivityTransactionState, BuildDigest,
    LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
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
fn public_factory_compiles_entry_topology_and_labeled_dice_controls() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = factory
        .compile_entry(
            SwarmDisasterEntry::new(
                "swarm-disaster.area.205",
                "universe.path.propagation",
                "swarm-disaster.audience-die.8",
                participants(),
            )
            .with_audience_unlocks(audience_unlocks())
            .with_dice_control_unlocks(vec!["1000022".into()]),
        )
        .unwrap();
    assert_eq!(instance.difficulty(), 5);
    assert_eq!(instance.state_definition().slots().len(), 16);
    assert_eq!(instance.graph_definition().nodes().len(), 48);
    assert_eq!(instance.graph_definition().edges().len(), 61);
    assert_eq!(instance.audience_path_sort(), 8);
    assert_eq!(instance.audience_path_unlock_id(), Some("1000008"));
    assert!(instance.audience_path_requires_unlock());
    assert_eq!(instance.audience_initial_rule(), "AddMazeBuff");
    assert_eq!(
        instance.audience_initial_parameters().collect::<Vec<_>>(),
        ["641270"]
    );
    assert_eq!(instance.audience_passive_rule(), "RandomGenSwarm");
    assert_eq!(
        instance.audience_die_faces().collect::<Vec<_>>(),
        [
            "swarm-disaster.dice-face.805",
            "swarm-disaster.dice-face.801",
            "swarm-disaster.dice-face.802",
            "swarm-disaster.dice-face.803",
            "swarm-disaster.dice-face.804",
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
    let replacement = instance
        .compile_node_replacement(
            instance.graph_definition().entry(),
            "swarm-disaster.domain.reward",
            Some("swarm-disaster.beacon.1"),
        )
        .unwrap();
    replacement
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let movement = instance.compile_countdown_move(&state, &[]).unwrap();
    movement
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let decay = instance
        .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.1"])
        .unwrap();
    decay
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let mut audience_state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let audience = instance
        .compile_audience_initialization(&audience_state)
        .unwrap();
    apply(&instance, &mut audience_state, &audience);
    assert!(
        instance
            .audience_initialization_applied(&audience_state)
            .unwrap()
    );
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    );
    let mut rng = ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(0x2003_3206),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ));
    let roll = instance
        .compile_dice_roll(&audience_state, &mut rng)
        .unwrap();
    apply(&instance, &mut audience_state, &roll);
    assert!(instance.dice_resolution_face(&audience_state).is_some());
    assert_eq!(
        instance.dice_resolution_kind(&audience_state).unwrap(),
        Some(1)
    );
    let abandon = instance.compile_dice_abandon(&audience_state).unwrap();
    apply(&instance, &mut audience_state, &abandon);
    assert_eq!(instance.dice_resolution_face(&audience_state), None);
    assert_eq!(
        instance.dice_resolution_kind(&audience_state).unwrap(),
        Some(4)
    );
    assert!(!instance.dice_roll_available(&audience_state).unwrap());

    assert_eq!(
        instance.boss_choices().collect::<Vec<_>>(),
        [
            "swarm-disaster.boss-choice.8003051",
            "swarm-disaster.boss-choice.8024010",
        ]
    );
    let mut transition_state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_ends().next().unwrap(),
    );
    let decay = instance
        .compile_boss_decay_selection(&transition_state, &["swarm-disaster.boss-decay.1"])
        .unwrap();
    apply(&instance, &mut transition_state, &decay);
    let selection = instance
        .compile_boss_selection(1, "swarm-disaster.boss-choice.8003051")
        .unwrap();
    apply(&instance, &mut transition_state, &selection);
    assert_eq!(
        instance.selected_boss(&transition_state, 1),
        Some("swarm-disaster.boss-choice.8003051")
    );
    let completion = instance
        .compile_plane_completion(&transition_state, 1)
        .unwrap();
    apply(&instance, &mut transition_state, &completion);
    assert_eq!(
        transition_state.current_node(),
        instance.plane_starts().nth(1).unwrap()
    );
    assert_eq!(instance.countdown(&transition_state).unwrap(), 20);
}

fn apply(
    instance: &starclock_mode_universe::swarm_disaster_entry::SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: &starclock_activity::ActivityProgramDefinition,
) {
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
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

fn audience_unlocks() -> Vec<String> {
    [
        "1000008", "1000013", "1000014", "1000015", "1000016", "1000017", "1000018",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
