use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivityTransactionOutcome, ActivityTransactionState,
};

use crate::error::UniverseCatalogLoadErrorKind;

use super::encounter_runtime::{
    EncounterRole, SWARM_DISASTER_ENCOUNTER_DIFFICULTY_REVISION,
    SWARM_DISASTER_ENCOUNTER_POLICY_ACCURACY,
    SWARM_DISASTER_ENCOUNTER_POLICY_REPLACEMENT_CONDITION,
    SWARM_DISASTER_ENCOUNTER_SELECTION_REVISION,
};
use super::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

#[test]
fn catalog_closes_groups_members_waves_slots_and_boss_pools() {
    let instance = instance(&factory(), 1);
    assert_eq!(
        instance.encounter_runtime.denominators(),
        (179, 347, 347, 1_070, 15)
    );
    assert_eq!(
        SWARM_DISASTER_ENCOUNTER_SELECTION_REVISION,
        "swarm-disaster-encounter-selection-policy-v1"
    );
    assert_eq!(
        SWARM_DISASTER_ENCOUNTER_DIFFICULTY_REVISION,
        "swarm-disaster-encounter-difficulty-policy-v1"
    );
    assert_eq!(
        SWARM_DISASTER_ENCOUNTER_POLICY_ACCURACY,
        "DeterministicProjectPolicyNotObservedParity"
    );
    assert!(
        SWARM_DISASTER_ENCOUNTER_POLICY_REPLACEMENT_CONDITION
            .contains("room/domain/group join")
    );
}

#[test]
fn all_five_formal_areas_bind_three_exact_released_level_schedules() {
    let factory = factory();
    let levels = [
        [(54, 57), (58, 61), (62, 63)],
        [(62, 65), (66, 69), (70, 71)],
        [(70, 73), (74, 77), (78, 79)],
        [(78, 81), (82, 85), (86, 87)],
        [(86, 89), (90, 93), (94, 95)],
    ];
    for difficulty in 1_u8..=5 {
        let instance = instance(&factory, difficulty);
        for (plane_index, (first, last)) in levels[usize::from(difficulty - 1)]
            .into_iter()
            .enumerate()
        {
            let plane = u8::try_from(plane_index + 1).unwrap();
            for (position, expected) in [(0, first), (12, last)] {
                let (segment, level) = instance
                    .encounter_runtime
                    .effective_level_at(plane, position)
                    .unwrap();
                assert_eq!(level, expected);
                assert!(segment.ends_with(&format!("20{difficulty}{plane}")));
            }
        }
    }
}

#[test]
fn cut_positions_use_the_released_left_closed_level_buckets() {
    let instance = instance(&factory(), 1);
    let vectors = [
        (1, 0, 54),
        (1, 2, 55),
        (1, 5, 56),
        (1, 8, 57),
        (2, 0, 58),
        (2, 4, 59),
        (2, 8, 60),
        (2, 12, 61),
        (3, 0, 62),
        (3, 6, 63),
        (3, 12, 63),
    ];
    for (plane, position, expected) in vectors {
        assert_eq!(
            instance
            .encounter_runtime
                .effective_level_at(plane, position)
                .map(|(_, level)| level),
            Some(expected)
        );
    }
}

#[test]
fn combat_elite_and_all_three_boss_pools_select_complete_81_series_rows() {
    let instance = instance(&factory(), 3);
    let mut rng = encounter_rng(&instance, 0x2006_0201);
    let first = instance.plane_ends().next().unwrap();
    let second = instance.plane_ends().nth(1).unwrap();
    let third = instance.plane_ends().nth(2).unwrap();
    let vectors = [
        (first, "swarm-disaster.domain.monsternormal", EncounterRole::Combat),
        (first, "swarm-disaster.domain.monsterelite", EncounterRole::Elite),
        (first, "swarm-disaster.domain.monsterboss", EncounterRole::FirstPlaneBoss),
        (second, "swarm-disaster.domain.monsterboss", EncounterRole::SecondPlaneBoss),
        (third, "swarm-disaster.domain.monsterswarmboss", EncounterRole::FinalBoss),
    ];
    for (node, domain, role) in vectors {
        let selection = instance.encounter_runtime.select(node, domain, &mut rng).unwrap();
        assert_eq!(selection.role, role);
        assert!(selection.source_group_id >= 100_000);
        assert!(!selection.source_rogue_monster_id.is_empty());
        assert!(!selection.source_primary_monster_id.is_empty());
        assert!(selection.source_stage_id.starts_with("81"));
        assert!(!selection.waves.is_empty());
        assert!(selection.waves.iter().all(|wave| {
            !wave.key.is_empty()
                && wave.ordinal != 0
                && wave.stage_type.as_ref() == "VerseSimulation"
                && wave.authored_level != 0
                && wave.hard_level_group != 0
                && !wave.slots.is_empty()
                && wave.slots.iter().all(|slot| {
                    !slot.key.is_empty()
                        && slot.formation_index != 0
                        && !slot.enemy_variant.is_empty()
                })
        }));
    }
    let final_boss = instance
        .encounter_runtime
        .select(
            third,
            "swarm-disaster.domain.monsterswarmboss",
            &mut rng,
        )
        .unwrap();
    assert_eq!(final_boss.group.as_ref(), "swarm-disaster.encounter-group.123001");
    assert_eq!(final_boss.waves.len(), 1);
    assert_eq!(final_boss.waves[0].slots.len(), 1);
    assert_eq!(
        final_boss.waves[0].slots[0].boss_choices.as_ref(),
        [Box::<str>::from("swarm-disaster.boss-choice.8024010")]
    );
    assert_eq!(active_labels(&rng), [ActivityRngLabel::Encounter]);
}

#[test]
fn current_activity_domain_is_the_only_runtime_room_join() {
    let instance = instance(&factory(), 1);
    let node = instance.graph_definition().entry();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(node, "swarm-disaster.domain.monsternormal", None)
            .unwrap(),
        1,
    );
    let mut rng = encounter_rng(&instance, 0x2006_0301);
    let selection = instance.select_current_encounter(&state, &mut rng).unwrap();
    assert_eq!(selection.role, EncounterRole::Combat);
    let mut digest_rng = encounter_rng(&instance, 0x2006_0301);
    let digest = instance
        .select_current_encounter_digest(&state, &mut digest_rng)
        .unwrap();
    assert_eq!(
        hex(&digest),
        "bf1f44de90b5a2a7a2aa58273e5e517634b28a79b56363434b4ec2559ca3ab27"
    );

    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(node, "swarm-disaster.domain.reward", None)
            .unwrap(),
        2,
    );
    let before = snapshots(&rng);
    assert_eq!(
        instance
            .select_current_encounter(&state, &mut rng)
            .unwrap_err()
            .kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
    assert_eq!(snapshots(&rng), before);
}

#[test]
fn unresolved_nodes_and_mismatched_boss_domains_fail_before_rng() {
    let instance = instance(&factory(), 5);
    let mut rng = encounter_rng(&instance, 0x2006_0401);
    let before = snapshots(&rng);
    assert!(
        instance
            .encounter_runtime
            .select(
                starclock_activity::NodeId::new(u32::MAX).unwrap(),
                "swarm-disaster.domain.monsternormal",
                &mut rng,
            )
            .is_err()
    );
    let third = instance.plane_ends().nth(2).unwrap();
    assert!(
        instance
            .encounter_runtime
            .select(third, "swarm-disaster.domain.monsterboss", &mut rng)
            .is_err()
    );
    assert_eq!(snapshots(&rng), before);
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory, difficulty: u8) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::tests::released_entry(
            format!("swarm-disaster.area.20{difficulty}"),
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            super::tests::participants(super::tests::policy()),
        ))
        .unwrap()
}

fn encounter_rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
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

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
    sequence: u64,
) {
    assert!(matches!(
        state.apply_program(
            &program,
            ActivityCause::new(sequence, program.id(), state.current_node()).unwrap(),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn snapshots(rng: &ActivityRngStreams) -> Vec<(ActivityRngLabel, u64)> {
    rng.snapshots()
        .iter()
        .map(|snapshot| (snapshot.label(), snapshot.draw_count()))
        .collect()
}

fn active_labels(rng: &ActivityRngStreams) -> Vec<ActivityRngLabel> {
    rng.snapshots()
        .iter()
        .filter(|snapshot| snapshot.draw_count() != 0)
        .map(|snapshot| snapshot.label())
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
