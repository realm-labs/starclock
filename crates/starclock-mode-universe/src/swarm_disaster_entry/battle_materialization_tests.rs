use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngStreams, ActivityTransactionOutcome, ActivityTransactionState, BuildDigest,
    OpaqueParticipantBuild, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantSourceKind,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, TeamSide, UnitDefinitionId, UnitLevel, catalog::action::AbilityKind,
};

use crate::battle_materialization::UniverseBattleRoster;

use super::{
    SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    battle_enemy_catalog::SWARM_DISASTER_ENEMY_DEFINITION_REVISION,
    battle_materialization::SWARM_DISASTER_BATTLE_MATERIALIZATION_REVISION,
    battle_snapshot::SWARM_DISASTER_BATTLE_SNAPSHOT_REVISION,
};

#[test]
fn current_activity_materializes_a_real_construction_validated_battle() {
    let instance = instance();
    let state = combat_state(&instance);
    let roster = roster(&instance);
    let mut first_rng = activity_rng(&instance, 0x2006_0501);
    let mut second_rng = activity_rng(&instance, 0x2006_0501);
    let first = instance
        .materialize_current_battle(&state, &mut first_rng, &roster)
        .unwrap();
    let second = instance
        .materialize_current_battle(&state, &mut second_rng, &roster)
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.assembly_digest(), second.assembly_digest());
    assert_eq!(first.combat_input_digest(), second.combat_input_digest());
    assert_eq!(instance.battle_catalog.summary().0, 71);
    assert_eq!(instance.battle_catalog.summary().1, 12);
    let stat_summary = instance.battle_catalog.runtime_stat_summary(
        &instance.content_runtime.standard,
        UnitLevel::new(54).unwrap(),
    );
    assert_eq!(stat_summary, (24, 47));
    let mut snapshot_rng = activity_rng(&instance, 0x2006_0501);
    let selection = instance
        .select_current_encounter(&state, &mut snapshot_rng)
        .unwrap();
    let snapshot = instance.compile_battle_snapshot(&state, &selection).unwrap();
    assert_eq!(
        digest_hex(first.assembly_digest().bytes()),
        "8ca070f188dedc8c84eab54b72d8f0dd4827518742ca9daee14725b52a99ccb5"
    );
    assert_eq!(
        digest_hex(first.combat_input_digest().bytes()),
        "fa2e2dea4ca41cce48a975250874a2c483b4b35daf993625e61ed2798bba7090"
    );
    assert_eq!(
        digest_hex(snapshot.digest),
        "3bb3ed6e3fc140a2d29128e030bcfa98d7975acbdbf78094749b1f9a2a09f791"
    );
    assert_eq!(
        digest_hex(instance.battle_catalog.summary().2),
        "df5dc26217f6cd07c1d7c1cde45ee03bc98791d0e35c7c0ce53ab0ebcd0b7db6"
    );
    assert_eq!(first.rules_revision(), SWARM_DISASTER_BATTLE_MATERIALIZATION_REVISION);
    assert_eq!(
        first
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Player)
            .count(),
        4
    );
    assert!(
        first
            .participants()
            .iter()
            .any(|participant| participant.side() == TeamSide::Enemy)
    );
    assert_eq!(
        SWARM_DISASTER_ENEMY_DEFINITION_REVISION,
        "swarm-disaster-enemy-definition-composition-v1"
    );
    assert_eq!(
        SWARM_DISASTER_BATTLE_SNAPSHOT_REVISION,
        "swarm-disaster-battle-snapshot-v1"
    );
}

#[test]
fn inventories_and_disarray_change_the_immutable_assembly_identity() {
    let instance = instance();
    let roster = roster(&instance);
    let mut state = combat_state(&instance);
    let mut empty_rng = activity_rng(&instance, 0x2006_0502);
    let empty = instance
        .materialize_current_battle(&state, &mut empty_rng, &roster)
        .unwrap();

    let blessing = instance.blessing_candidates(1, 3, &[]).unwrap()[0];
    let curio = instance.curio_candidates("Normal", &[]).unwrap()[0];
    commit(
        &instance,
        &mut state,
        instance.compile_blessing_acquisition(blessing).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(curio).unwrap(),
    );
    let mut content_rng = activity_rng(&instance, 0x2006_0502);
    let populated = instance
        .materialize_current_battle(&state, &mut content_rng, &roster)
        .unwrap();
    assert_ne!(empty.assembly_digest(), populated.assembly_digest());
    let mut snapshot_rng = activity_rng(&instance, 0x2006_0502);
    let selection = instance
        .select_current_encounter(&state, &mut snapshot_rng)
        .unwrap();
    let snapshot = instance.compile_battle_snapshot(&state, &selection).unwrap();
    assert_eq!(snapshot.shared.rules().len(), 4);

    let adjustment = instance
        .compile_countdown_adjustments(&state, &[(1, -20)])
        .unwrap();
    commit(&instance, &mut state, adjustment);
    let movement = instance.compile_countdown_move(&state, &[]).unwrap();
    commit(&instance, &mut state, movement);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (5, 4, 0));
    let mut disarray_rng = activity_rng(&instance, 0x2006_0502);
    let disarray = instance
        .materialize_current_battle(&state, &mut disarray_rng, &roster)
        .unwrap();
    assert_ne!(populated.assembly_digest(), disarray.assembly_digest());
    assert!(disarray
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Enemy)
        .all(|participant| !participant.combatant().modifiers().is_empty()));
}

#[test]
fn unresolved_domain_rejects_without_consuming_encounter_rng() {
    let instance = instance();
    let roster = roster(&instance);
    let mut state = combat_state(&instance);
    let node = state.current_node();
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(node, "swarm-disaster.domain.reward", None)
            .unwrap(),
    );
    let mut rng = activity_rng(&instance, 0x2006_0503);
    let before = rng.snapshots();
    let sequence = state.command_sequence();
    assert!(
        instance
            .materialize_current_battle(&state, &mut rng, &roster)
            .is_err()
    );
    assert_eq!(rng.snapshots(), before);
    assert_eq!(state.command_sequence(), sequence);
}

#[test]
fn trail_path_and_next_battle_die_face_are_bound_into_the_spec() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap();
    let progression = factory
        .unique
        .trail_runtime_input()
        .nodes
        .iter()
        .map(|node| node.key.to_string())
        .collect::<Vec<_>>();
    let points = (1..=7)
        .map(|id| (format!("swarm-disaster.communing-dimension.{id}"), 20))
        .collect::<Vec<_>>();
    let instance = factory
        .compile_entry(
            super::tests::released_entry(
                "swarm-disaster.area.201",
                "universe.path.preservation",
                "swarm-disaster.audience-die.1",
                battle_participants(),
            )
            .with_progression(points, progression, None),
        )
        .unwrap();
    assert_eq!(instance.communing_trail_battle_effects().count(), 58);
    let roster = roster(&instance);
    let mut state = combat_state(&instance);
    let run_start = instance.compile_trail_run_start(&state).unwrap();
    commit(&instance, &mut state, run_start);
    let mut before_rng = activity_rng(&instance, 0x2006_0504);
    let before = instance
        .materialize_current_battle(&state, &mut before_rng, &roster)
        .unwrap();
    let face = instance
        .audience_die_faces()
        .find(|face| instance.dice_face_activation_stage(face) == Some(3))
        .unwrap()
        .to_owned();
    let mut dice_rng = activity_rng(&instance, 0x2006_05f4);
    let roll = instance.compile_dice_roll(&state, &mut dice_rng).unwrap();
    commit(&instance, &mut state, roll);
    let cheat = instance.compile_dice_cheat(&state, &face).unwrap();
    commit(&instance, &mut state, cheat);
    let mut after_rng = activity_rng(&instance, 0x2006_0504);
    let after = instance
        .materialize_current_battle(&state, &mut after_rng, &roster)
        .unwrap();
    assert_ne!(before.assembly_digest(), after.assembly_digest());
}

fn instance() -> SwarmDisasterRuntimeInstance {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE)
        .unwrap()
        .compile_entry(super::tests::released_entry(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            battle_participants(),
        ))
        .unwrap()
}

fn battle_participants() -> ParticipantLock {
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32]).unwrap(),
                BuildDigest::new([byte + 32; 32]).unwrap(),
                "swarm-battle-test-build-v1",
                ParticipantSourceKind::CompiledBuild,
            )
            .unwrap();
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1).unwrap(),
                0,
                index,
                UnitDefinitionId::new(u32::from(index) + 1).unwrap(),
                build,
            )
            .unwrap()
        })
        .collect();
    ParticipantLock::seal(super::tests::policy(), entries).unwrap()
}

fn combat_state(instance: &SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    let node = instance.graph_definition().entry();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    commit(
        instance,
        &mut state,
        instance
            .compile_node_replacement(node, "swarm-disaster.domain.monsternormal", None)
            .unwrap(),
    );
    state
}

fn roster(instance: &SwarmDisasterRuntimeInstance) -> UniverseBattleRoster {
    let combat = instance
        .content_runtime
        .standard
        .simulation_catalog()
        .combat_catalog();
    let combatants = instance
        .participants()
        .entries()
        .iter()
        .map(|locked| {
            let unit = combat.unit(locked.character()).unwrap();
            let basic = unit
                .abilities()
                .iter()
                .copied()
                .find(|ability| {
                    combat
                        .ability(*ability)
                        .and_then(|definition| definition.action())
                        .is_some_and(|action| action.kind() == AbilityKind::Basic)
                })
                .unwrap();
            let spec = ResolvedCombatantSpec::new(
                locked.character(),
                UnitLevel::new(80).unwrap(),
                Hp::new(100_000).unwrap(),
                Speed::from_scaled(100_000_000).unwrap(),
                ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new()).unwrap(),
                CombatantSpecDigest::new(locked.build().resolved_spec_digest().bytes()).unwrap(),
            )
            .unwrap()
            .with_base_attack_defense(
                StatValue::from_scaled(100_000_000).unwrap(),
                StatValue::from_scaled(100_000_000).unwrap(),
            )
            .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
            .unwrap();
            (locked.participant(), spec)
        })
        .collect();
    UniverseBattleRoster::new(instance.participants(), combatants).unwrap()
}

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) {
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn activity_rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x6d; 32]).unwrap(),
    );
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
