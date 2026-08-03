use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivityTransactionOutcome, ActivityTransactionState,
    NodeId,
};

use super::{
    GOLD_AND_GEARS_ENCOUNTER_POLICY_ACCURACY, GoldAndGearsEncounterRole, GoldAndGearsEntryError,
    GoldAndGearsRuntimeInstance,
};
use super::{tests};

#[test]
fn catalog_closes_all_groups_waves_slots_and_enemy_identities() {
    let instance = tests::compiled_fixture(tests::shared_factory());
    assert_eq!(instance.encounter_runtime.denominators(), (181, 478, 1_513));
    assert_eq!(
        GOLD_AND_GEARS_ENCOUNTER_POLICY_ACCURACY,
        "DeterministicProjectPolicyNotObservedParity"
    );
}

#[test]
fn area_plane_and_cut_positions_resolve_exact_released_levels() {
    let instance = tests::compiled_fixture(tests::shared_factory());
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
    let mut rng = encounter_rng(&instance, 0x1406_0001);
    for (plane, position, expected) in vectors {
        let node = instance.encounter_runtime.node_at(plane, position).unwrap();
        let selection = instance
            .encounter_runtime
            .select(node, "gold-gears.domain.monsternormal", None, &mut rng)
            .unwrap();
        assert_eq!(selection.effective_level(), expected);
        assert_eq!(selection.role(), GoldAndGearsEncounterRole::Combat);
        assert!(
            selection
                .difficulty_segment()
                .ends_with(&format!("401{plane}"))
        );
    }
}

#[test]
fn every_formal_area_binds_all_three_released_difficulty_segments() {
    let factory = tests::shared_factory();
    let vectors = [
        ("gold-gears.area.401", [(54, 57), (58, 61), (62, 63)]),
        ("gold-gears.area.402", [(62, 65), (66, 69), (70, 71)]),
        ("gold-gears.area.403", [(70, 73), (74, 77), (78, 79)]),
        ("gold-gears.area.404", [(78, 81), (82, 85), (86, 87)]),
        ("gold-gears.area.405", [(86, 89), (90, 93), (94, 95)]),
    ];
    for (area, levels) in vectors {
        let instance = factory
            .compile_entry(tests::entry(
                factory,
                area,
                &factory.unique.paths[0].identity.stable_key,
                &factory.unique.dice[0],
            ))
            .unwrap();
        let mut rng = encounter_rng(&instance, 0x1406_0010);
        for (index, (first, last)) in levels.into_iter().enumerate() {
            let plane = u8::try_from(index + 1).unwrap();
            for (position, expected) in [(0, first), (12, last)] {
                let node = instance.encounter_runtime.node_at(plane, position).unwrap();
                let selection = instance
                    .encounter_runtime
                    .select(node, "gold-gears.domain.monsternormal", None, &mut rng)
                    .unwrap();
                assert_eq!(selection.effective_level(), expected);
            }
        }
    }
}

#[test]
fn combat_elite_and_plane_boss_families_use_bounded_encounter_rng() {
    let instance = tests::compiled_fixture(tests::shared_factory());
    let mut rng = encounter_rng(&instance, 0x1406_0002);
    let first = instance.encounter_runtime.node_at(1, 12).unwrap();
    let second = instance.encounter_runtime.node_at(2, 12).unwrap();
    let third = instance.encounter_runtime.node_at(3, 12).unwrap();

    let combat = instance
        .encounter_runtime
        .select(first, "gold-gears.domain.monsternormal", None, &mut rng)
        .unwrap();
    let elite = instance
        .encounter_runtime
        .select(first, "gold-gears.domain.monsterelite", None, &mut rng)
        .unwrap();
    let first_boss = instance
        .encounter_runtime
        .select(first, "gold-gears.domain.monsterboss", None, &mut rng)
        .unwrap();
    let second_boss = instance
        .encounter_runtime
        .select(second, "gold-gears.domain.monsternousboss", None, &mut rng)
        .unwrap();
    let final_boss = instance
        .encounter_runtime
        .select(
            third,
            "gold-gears.domain.monsterboss",
            Some("gold-gears.boss-choice.1013014"),
            &mut rng,
        )
        .unwrap();

    assert_eq!(combat.role(), GoldAndGearsEncounterRole::Combat);
    assert_eq!(elite.role(), GoldAndGearsEncounterRole::Elite);
    assert_eq!(first_boss.role(), GoldAndGearsEncounterRole::FirstPlaneBoss);
    assert_eq!(
        second_boss.role(),
        GoldAndGearsEncounterRole::SecondPlaneBoss
    );
    assert_eq!(final_boss.role(), GoldAndGearsEncounterRole::FinalBoss);
    assert_eq!(final_boss.group(), "gold-gears.encounter-group.223003");
    assert_eq!(final_boss.waves().len(), 1);
    assert_eq!(final_boss.waves()[0].slots().len(), 2);
    assert!(final_boss.waves()[0].slots().iter().any(|slot| {
        slot.boss_choices()
            .any(|choice| choice == "gold-gears.boss-choice.1013014")
    }));
    assert!(encounter_draws(&rng) >= 4);

    let before = snapshots(&rng);
    assert_eq!(
        instance.encounter_runtime.select(
            third,
            "gold-gears.domain.monsterboss",
            Some("gold-gears.boss-choice.8003051"),
            &mut rng,
        ),
        Err(GoldAndGearsEntryError::NoEncounterCandidates)
    );
    assert_eq!(snapshots(&rng), before);
}

#[test]
fn current_activity_domain_join_is_fail_closed_and_transactional() {
    let instance = tests::compiled_fixture(tests::shared_factory());
    let node = instance.encounter_runtime.node_at(1, 2).unwrap();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    commit(
        &instance,
        &mut state,
        node,
        instance
            .compile_node_replacement(node, "gold-gears.domain.monsternormal", None)
            .unwrap(),
        1,
    );
    let mut rng = encounter_rng(&instance, 0x1406_0003);
    let selection = instance.select_current_encounter(&state, &mut rng).unwrap();
    assert_eq!(selection.effective_level(), 55);
    assert_eq!(selection.role(), GoldAndGearsEncounterRole::Combat);

    commit(
        &instance,
        &mut state,
        node,
        instance
            .compile_node_replacement(node, "gold-gears.domain.reward", None)
            .unwrap(),
        2,
    );
    let before = snapshots(&rng);
    assert_eq!(
        instance.select_current_encounter(&state, &mut rng),
        Err(GoldAndGearsEntryError::NonCombatEncounterDomain(
            "gold-gears.domain.reward".into()
        ))
    );
    assert_eq!(snapshots(&rng), before);
}

#[test]
fn final_boss_join_requires_the_committed_plane_three_choice() {
    let instance = tests::compiled_fixture(tests::shared_factory());
    let node = instance.encounter_runtime.node_at(3, 12).unwrap();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    commit(
        &instance,
        &mut state,
        node,
        instance
            .compile_node_replacement(node, "gold-gears.domain.monsterboss", None)
            .unwrap(),
        1,
    );
    let mut rng = encounter_rng(&instance, 0x1406_0004);
    let before = snapshots(&rng);
    assert_eq!(
        instance.select_current_encounter(&state, &mut rng),
        Err(GoldAndGearsEntryError::MissingEncounterBossChoice)
    );
    assert_eq!(snapshots(&rng), before);

    commit(
        &instance,
        &mut state,
        node,
        instance
            .compile_boss_selection(3, "gold-gears.boss-choice.8024011")
            .unwrap(),
        2,
    );
    let selection = instance.select_current_encounter(&state, &mut rng).unwrap();
    assert_eq!(selection.group(), "gold-gears.encounter-group.223001");
    assert_eq!(selection.effective_level(), 63);
    assert_eq!(encounter_draws(&rng), 0);
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    node: NodeId,
    program: starclock_activity::ActivityProgramDefinition,
    sequence: u64,
) {
    assert!(matches!(
        state.apply_program(
            &program,
            ActivityCause::new(sequence, program.id(), node).unwrap(),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn encounter_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x61; 32]).unwrap(),
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

fn encounter_draws(rng: &ActivityRngStreams) -> u64 {
    rng.snapshots()
        .iter()
        .find(|snapshot| snapshot.label() == ActivityRngLabel::Encounter)
        .unwrap()
        .draw_count()
}

fn snapshots(rng: &ActivityRngStreams) -> Vec<(ActivityRngLabel, u64)> {
    rng.snapshots()
        .iter()
        .map(|snapshot| (snapshot.label(), snapshot.draw_count()))
        .collect()
}
