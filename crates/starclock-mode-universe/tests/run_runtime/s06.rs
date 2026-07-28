use super::*;

const WILDBOAR_ATTEMPT_KEY: u64 = 0x5000_0000_0000_0000 | 13_502;
const WILDBOAR_REPEAT_KEY: u64 = 0x5100_0000_0000_0000 | 13_502;
const ROBOT_ATTEMPT_KEY: u64 = 0x5000_0000_0000_0000 | 13_503;
const ROBOT_REPEAT_KEY: u64 = 0x5100_0000_0000_0000 | 13_503;

#[test]
fn goal07_p4_m13_s06_executes_progressive_battle_cost_and_path_reward_outcomes() {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(),
            vec![],
        ))
        .unwrap();
    let runtime = compiled.occurrence_interaction_runtime();
    let choice = |key: &str| {
        let choice = catalog
            .occurrence_choices()
            .iter()
            .find(|choice| choice.stable_key() == key)
            .unwrap();
        runtime.compile_choice(choice.id()).unwrap()
    };

    let mut keys = vec![
        "universe.occurrence.31.variant.13502.choice.01".to_owned(),
        "universe.occurrence.31.variant.13502.choice.02".to_owned(),
        "universe.occurrence.32.variant.13503.choice.01".to_owned(),
        "universe.occurrence.32.variant.13503.choice.02".to_owned(),
    ];
    for variant in [13_401, 13_402] {
        for index in 1..=2 {
            keys.push(format!(
                "universe.occurrence.33.variant.{variant}.choice.{index:02}"
            ));
        }
    }
    for variant in [13_601, 13_602, 13_603] {
        for index in 1..=3 {
            keys.push(format!(
                "universe.occurrence.34.variant.{variant}.choice.{index:02}"
            ));
        }
    }
    keys.push("universe.occurrence.35.variant.13701.choice.01".to_owned());
    assert_eq!(keys.len(), 18);
    assert!(
        keys.iter()
            .all(|key| choice(key).deferred_operations() == 0)
    );

    let wildboar = choice("universe.occurrence.31.variant.13502.choice.01");
    assert_eq!(wildboar.random_candidate_count(), Some(6_100));
    assert_eq!(wildboar.repeat_key(), Some(WILDBOAR_REPEAT_KEY));
    assert!(wildboar.battle_member().is_some());
    let mut saw_curio = false;
    let mut saw_battle = false;
    let mut saw_blank = false;
    for purpose in 1..=128 {
        let result = execute_with_seed(
            &compiled,
            wildboar.payload(),
            106_000 + u32::from(purpose),
            Some((ActivityRngLabel::Occurrence, purpose, 6_100)),
            vec![],
            u64::from(purpose),
        );
        let curios = inventory_entries(&result, compiled.curio_inventory()).len();
        let repeat = counter(
            &result,
            compiled.occurrence_interaction_state_slot(),
            WILDBOAR_REPEAT_KEY,
        );
        match (curios, repeat) {
            (1, 1) => saw_curio = true,
            (0, 0) => saw_battle = true,
            (0, 1) => saw_blank = true,
            state => panic!("unexpected Wildboar draw state {state:?}"),
        }
        if saw_curio && saw_battle && saw_blank {
            break;
        }
    }
    assert!(saw_curio && saw_battle && saw_blank);

    let robot = choice("universe.occurrence.32.variant.13503.choice.01");
    assert_eq!(robot.random_candidate_count(), Some(100));
    assert_eq!(robot.repeat_key(), Some(ROBOT_REPEAT_KEY));
    assert!(robot.battle_member().is_some());
    let mut saw_fragments = false;
    let mut saw_robot_battle = false;
    let mut saw_robot_blank = false;
    for purpose in 129..=256 {
        let result = execute_with_seed(
            &compiled,
            robot.payload(),
            106_000 + u32::from(purpose),
            Some((ActivityRngLabel::Occurrence, purpose, 100)),
            vec![],
            u64::from(purpose),
        );
        let fragments = fragments(&result, &compiled);
        let repeat = counter(
            &result,
            compiled.occurrence_interaction_state_slot(),
            ROBOT_REPEAT_KEY,
        );
        match (fragments, repeat) {
            (150, 1) => saw_fragments = true,
            (50, 0) => saw_robot_battle = true,
            (50, 1) => saw_robot_blank = true,
            state => panic!("unexpected Robot draw state {state:?}"),
        }
        if saw_fragments && saw_robot_battle && saw_robot_blank {
            break;
        }
    }
    assert!(saw_fragments && saw_robot_battle && saw_robot_blank);

    for (payload, attempt_key, candidates, outcome) in [
        (wildboar.payload(), WILDBOAR_ATTEMPT_KEY, 6_100, 106_301),
        (robot.payload(), ROBOT_ATTEMPT_KEY, 100, 106_302),
    ] {
        let result = execute_with_seed(
            &compiled,
            payload,
            outcome,
            Some((ActivityRngLabel::Occurrence, 601, candidates)),
            vec![ActivityOperation::AddCounter {
                slot: compiled.occurrence_interaction_state_slot(),
                key: attempt_key,
                delta: integer(3),
            }],
            u64::from(outcome),
        );
        assert_eq!(
            counter(
                &result,
                compiled.occurrence_interaction_state_slot(),
                attempt_key
            ),
            0
        );
    }

    for variant in [13_401, 13_402] {
        let battle = choice(&format!(
            "universe.occurrence.33.variant.{variant}.choice.01"
        ));
        assert!(battle.battle_member().is_some());
        let safe = execute_occurrence_payload_with_fragments(
            &compiled,
            choice(&format!(
                "universe.occurrence.33.variant.{variant}.choice.02"
            ))
            .payload(),
            106_400 + (variant - 13_401),
            150,
        );
        assert_eq!(fragments(&safe, &compiled), 50);
    }

    let preservation = path_id(&catalog, "universe.path.preservation");
    let nihility = path_id(&catalog, "universe.path.nihility");
    let both = choice("universe.occurrence.34.variant.13601.choice.03");
    let prepared = execute_occurrence_payload(&compiled, both.payload(), 106_501);
    assert_eq!(
        counter(
            &prepared,
            compiled.occurrence_interaction_state_slot(),
            preservation
        ),
        1
    );
    assert_eq!(
        counter(
            &prepared,
            compiled.occurrence_interaction_state_slot(),
            nihility
        ),
        1
    );
    for path in catalog.paths() {
        let path = u64::from(path.id().get());
        if path != preservation && path != nihility {
            assert_eq!(
                counter(
                    &prepared,
                    compiled.occurrence_interaction_state_slot(),
                    path
                ),
                0
            );
        }
    }
    assert!(
        choice("universe.occurrence.35.variant.13701.choice.01")
            .battle_member()
            .is_some()
    );
}

fn execute_with_seed(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    outcome: u32,
    random: Option<(ActivityRngLabel, u16, u32)>,
    seed: Vec<ActivityOperation>,
    master_seed: u64,
) -> GraphActivity {
    let outcome = ActivityExternalOutcomeId::new(u64::from(outcome)).unwrap();
    let mut binding = ActivityInteractionBinding::new(
        node(1),
        outcome,
        starclock_activity::ActivityHandlerId::new(OCCURRENCE_INTERACTION_HANDLER_ID).unwrap(),
        payload.to_vec(),
        "standard-universe.occurrence-choice.v2",
    )
    .unwrap();
    if let Some((label, purpose, candidates)) = random {
        binding = binding.with_random_policy(
            starclock_activity::ActivityInteractionRandomPolicy::new(label, purpose, candidates)
                .unwrap(),
        );
    }
    let registry = compiled
        .runtime_definition()
        .interactions()
        .unwrap()
        .registry();
    let mut activity =
        occurrence_harness_with_fragments_and_seed(compiled, &binding, registry, 50, master_seed);
    if !seed.is_empty() {
        let seed = ActivityProgramDefinition::new(program(96), seed).unwrap();
        activity
            .apply_boundary_program(activity.player_view().state_hash(), &seed)
            .unwrap();
    }
    let before = activity.player_view();
    activity
        .submit_external_outcome(
            before.state_hash(),
            before.decision().unwrap().id(),
            outcome,
        )
        .unwrap();
    activity
}

fn path_id(catalog: &UniverseCatalog, key: &str) -> u64 {
    u64::from(
        catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == key)
            .unwrap()
            .id()
            .get(),
    )
}

fn inventory_entries(
    activity: &GraphActivity,
    inventory: starclock_activity::ActivityInventoryId,
) -> Vec<(u64, u32)> {
    activity
        .player_view()
        .inventories()
        .iter()
        .find(|value| value.id() == inventory)
        .unwrap()
        .entries()
        .to_vec()
}

fn fragments(
    activity: &GraphActivity,
    compiled: &starclock_mode_universe::entry::CompiledActivity,
) -> i64 {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
        .and_then(|slot| match slot.value() {
            ActivityValue::BoundedInteger(value) => Some(*value),
            _ => None,
        })
        .unwrap()
}

fn counter(activity: &GraphActivity, slot: ActivitySlotId, key: u64) -> i64 {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(entries) => Some(
                entries
                    .iter()
                    .find(|entry| entry.0 == key)
                    .map_or(0, |entry| entry.1),
            ),
            _ => None,
        })
        .unwrap()
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
