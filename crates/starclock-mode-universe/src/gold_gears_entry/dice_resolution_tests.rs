use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionEvent, ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};

use super::{
    GoldAndGearsDiceDomain, GoldAndGearsDicePassiveEvent, GoldAndGearsEntryError,
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    dice_resolution::compile_reroll,
    state_layout::{
        PROGRESSION_DICE_PATH_BOOST_STACKS_KEY, PROGRESSION_DICE_PATH_INTERVAL_KEY,
        PROGRESSION_DICE_PATH_SCALED_VALUE_KEY, PROGRESSION_DICE_PATH_VALUE_KEY, PROGRESSION_SLOT,
        RESOURCE_COSMIC_FRAGMENTS_KEY, RESOURCE_DICE_CHEATS_KEY, RESOURCE_DICE_REROLLS_KEY,
        RUN_RESOURCES_SLOT,
    },
    tests::entry,
};

const AREA: &str = "gold-gears.area.401";
const PATH: &str = "universe.path.preservation";

#[test]
fn all_authored_dice_parts_and_path_values_compile_exactly() {
    let factory = super::tests::shared_factory();
    assert_eq!(factory.dice_runtime.denominators(), (12, 108, 39));

    let mut compiled = 0;
    for dice in &factory.unique.dice {
        for path in &factory.unique.paths {
            let instance = factory
                .compile_entry(entry(factory, AREA, &path.identity.stable_key, dice))
                .unwrap();
            let authored = factory
                .unique
                .dice_path_values
                .iter()
                .find(|value| {
                    value.dice == dice.identity.id && value.path_key == path.identity.stable_key
                })
                .unwrap();
            assert_eq!(
                instance.dice_path_boost_stat(),
                authored.boost_stat.as_ref()
            );
            assert_eq!(
                instance.dice_path_trigger_interval(),
                authored.trigger_interval.parse::<i64>().unwrap()
            );
            assert_eq!(
                instance.dice_path_boost_value_scaled(),
                scaled(&authored.boost_value.0)
            );
            assert_eq!(
                instance.dice_path_boost_unit(),
                authored.boost_unit.as_ref()
            );

            let state = ActivityTransactionState::new(
                instance.state_definition().clone(),
                instance.graph_definition().entry(),
            );
            assert_eq!(
                counter(&state, PROGRESSION_SLOT, PROGRESSION_DICE_PATH_VALUE_KEY),
                i64::from(authored.identity.id.0)
            );
            assert_eq!(
                counter(&state, PROGRESSION_SLOT, PROGRESSION_DICE_PATH_INTERVAL_KEY),
                instance.dice_path_trigger_interval()
            );
            assert_eq!(
                counter(
                    &state,
                    PROGRESSION_SLOT,
                    PROGRESSION_DICE_PATH_SCALED_VALUE_KEY
                ),
                instance.dice_path_boost_value_scaled()
            );
            assert_eq!(instance.dice_path_boost_stacks(&state), Some(0));
            compiled += 1;
        }
    }
    assert_eq!(compiled, 108);
}

#[test]
fn roll_and_reroll_are_spawn_isolated_and_rejection_is_byte_identical() {
    let factory = super::tests::shared_factory();
    let dice = &factory.unique.dice[0];
    let neural = factory
        .unique
        .neural_nodes
        .iter()
        .map(|node| node.identity.stable_key.to_string())
        .collect();
    let instance = factory
        .compile_entry(entry(factory, AREA, PATH, dice).with_neural_network(neural))
        .unwrap();
    let mut state = new_state(&instance);
    let mut rng = activity_rng(&instance, 14_302);
    let before_roll = rng.snapshots();
    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    let after_roll = rng.snapshots();
    assert_one_spawn_draw(&before_roll, &after_roll);

    let mut duplicate_rng = activity_rng(&instance, 14_302);
    let duplicate_state = new_state(&instance);
    assert_eq!(
        roll,
        instance
            .compile_dice_roll(&duplicate_state, &mut duplicate_rng)
            .unwrap()
    );
    assert_eq!(after_roll, duplicate_rng.snapshots());

    commit(&instance, &mut state, roll);
    let first = instance.dice_resolution_face(&state).unwrap().to_owned();
    assert!(instance.dice_faces().any(|face| face == first));
    assert_eq!(instance.dice_resolution_kind(&state), Some(1));

    let before_reroll = rng.snapshots();
    let reroll = instance.compile_dice_reroll(&state, &mut rng).unwrap();
    let after_reroll = rng.snapshots();
    assert_one_spawn_draw(&before_reroll, &after_reroll);
    commit(&instance, &mut state, reroll);
    assert_ne!(instance.dice_resolution_face(&state), Some(first.as_str()));
    assert_eq!(instance.dice_resolution_kind(&state), Some(2));
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        0
    );

    let bytes = state_bytes(&instance, &state, &rng);
    let snapshots = rng.snapshots();
    assert_eq!(
        instance.compile_dice_reroll(&state, &mut rng).unwrap_err(),
        GoldAndGearsEntryError::NoDiceRerolls
    );
    assert_eq!(snapshots, rng.snapshots());
    assert_eq!(bytes, state_bytes(&instance, &state, &rng));
}

#[test]
fn empty_reroll_candidates_keep_previous_consume_attempt_and_draw_nothing() {
    let factory = super::tests::shared_factory();
    let instance = factory
        .compile_entry(entry(factory, AREA, PATH, &factory.unique.dice[0]))
        .unwrap();
    let mut state = new_state(&instance);
    let mut rng = activity_rng(&instance, 14_303);
    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    commit(&instance, &mut state, roll);
    let selected = instance.dice_resolution_face(&state).unwrap().to_owned();
    let face_id = factory
        .unique
        .dice_faces
        .iter()
        .find(|face| face.identity.stable_key.as_ref() == selected)
        .unwrap()
        .identity
        .id
        .0;
    let only_previous = [(selected.clone().into_boxed_str(), face_id)];
    let before = rng.snapshots();
    let reroll = compile_reroll(&state, &only_previous, true, &mut rng).unwrap();
    assert_eq!(before, rng.snapshots());
    commit(&instance, &mut state, reroll);
    assert_eq!(
        instance.dice_resolution_face(&state),
        Some(selected.as_str())
    );
    assert_eq!(instance.dice_resolution_kind(&state), Some(4));
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        0
    );
}

#[test]
fn plane_initials_and_cheats_execute_with_exact_masks_and_no_rng() {
    let factory = super::tests::shared_factory();
    for dice in &factory.unique.dice {
        let instance = factory
            .compile_entry(entry(factory, AREA, PATH, dice))
            .unwrap();
        let active = (1..=3)
            .filter(|layer| instance.compile_dice_plane_start(*layer).unwrap().is_some())
            .collect::<Vec<_>>();
        let expected = match dice.identity.source_id.as_ref() {
            "203" => vec![1, 3],
            "403" => vec![1, 2, 3],
            _ => vec![1, 2],
        };
        assert_eq!(active, expected);
    }

    let transaction = instance_by_source(factory, "401");
    let mut transaction_state = new_state(&transaction);
    commit(
        &transaction,
        &mut transaction_state,
        transaction.compile_dice_plane_start(1).unwrap().unwrap(),
    );
    assert_eq!(
        counter(
            &transaction_state,
            RUN_RESOURCES_SLOT,
            RESOURCE_COSMIC_FRAGMENTS_KEY
        ),
        200
    );

    let general = instance_by_source(factory, "403");
    let mut state = new_state(&general);
    commit(
        &general,
        &mut state,
        general.compile_dice_plane_start(1).unwrap().unwrap(),
    );
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_DICE_CHEATS_KEY),
        1
    );
    let rng = activity_rng(&general, 14_304);
    let snapshots = rng.snapshots();
    let bytes = state_bytes(&general, &state, &rng);
    assert_eq!(
        general
            .compile_dice_cheat(&state, "not-a-face")
            .unwrap_err(),
        GoldAndGearsEntryError::DiceFaceNotInLoadout("not-a-face".into())
    );
    assert_eq!(snapshots, rng.snapshots());
    assert_eq!(bytes, state_bytes(&general, &state, &rng));

    let selected = general.dice_faces().last().unwrap().to_owned();
    let cheat = general.compile_dice_cheat(&state, &selected).unwrap();
    assert_eq!(snapshots, rng.snapshots());
    commit(&general, &mut state, cheat);
    assert_eq!(
        general.dice_resolution_face(&state),
        Some(selected.as_str())
    );
    assert_eq!(general.dice_resolution_kind(&state), Some(3));
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_DICE_CHEATS_KEY),
        0
    );
}

#[test]
fn all_twelve_passives_emit_typed_operations_and_exact_immediate_values() {
    let factory = super::tests::shared_factory();
    let cases = [
        (
            "101",
            GoldAndGearsDicePassiveEvent::TrottersDefeated { count: 2 },
        ),
        (
            "102",
            GoldAndGearsDicePassiveEvent::KnowledgeApplied { count: 2 },
        ),
        (
            "103",
            GoldAndGearsDicePassiveEvent::DomainEntered {
                plane_layer: 1,
                domain: GoldAndGearsDiceDomain::Other,
                beacon_id: Some(41),
                has_knowledge: false,
                non_adjacent: false,
                knowledge_domain_count: 0,
            },
        ),
        (
            "201",
            GoldAndGearsDicePassiveEvent::DomainEntered {
                plane_layer: 1,
                domain: GoldAndGearsDiceDomain::Occurrence,
                beacon_id: None,
                has_knowledge: false,
                non_adjacent: false,
                knowledge_domain_count: 0,
            },
        ),
        (
            "202",
            GoldAndGearsDicePassiveEvent::BattleVictory { elite: true },
        ),
        (
            "203",
            GoldAndGearsDicePassiveEvent::DomainEntered {
                plane_layer: 1,
                domain: GoldAndGearsDiceDomain::Other,
                beacon_id: None,
                has_knowledge: false,
                non_adjacent: true,
                knowledge_domain_count: 0,
            },
        ),
        (
            "301",
            GoldAndGearsDicePassiveEvent::CountdownSnapshot { remaining: 9 },
        ),
        (
            "302",
            GoldAndGearsDicePassiveEvent::DomainEntered {
                plane_layer: 1,
                domain: GoldAndGearsDiceDomain::Boss,
                beacon_id: None,
                has_knowledge: false,
                non_adjacent: false,
                knowledge_domain_count: 2,
            },
        ),
        (
            "303",
            GoldAndGearsDicePassiveEvent::KnowledgeDomainsCollapsed {
                count: 2,
                premium_domain: true,
                had_beacon: true,
            },
        ),
        (
            "401",
            GoldAndGearsDicePassiveEvent::StorePurchase {
                cosmic_fragments_spent: 250,
            },
        ),
        (
            "402",
            GoldAndGearsDicePassiveEvent::CuriosAcquired {
                count: 2,
                total_owned: 5,
            },
        ),
        (
            "403",
            GoldAndGearsDicePassiveEvent::MovementCompleted { count: 2 },
        ),
    ];

    for (source, event) in cases {
        let instance = instance_by_source(factory, source);
        let mut state = new_state(&instance);
        let program = instance
            .compile_dice_passive(&state, event)
            .unwrap()
            .expect("selected dice reacts to its released trigger");
        commit(&instance, &mut state, program);
    }

    let trotter = apply_passive(
        factory,
        "101",
        GoldAndGearsDicePassiveEvent::TrottersDefeated { count: 2 },
    );
    assert_eq!(
        counter(
            &trotter.1,
            RUN_RESOURCES_SLOT,
            RESOURCE_COSMIC_FRAGMENTS_KEY
        ),
        260
    );
    assert_eq!(trotter.0.dice_path_boost_stacks(&trotter.1), Some(2));

    let collapse = apply_passive(
        factory,
        "303",
        GoldAndGearsDicePassiveEvent::KnowledgeDomainsCollapsed {
            count: 2,
            premium_domain: true,
            had_beacon: true,
        },
    );
    assert_eq!(
        counter(
            &collapse.1,
            RUN_RESOURCES_SLOT,
            RESOURCE_COSMIC_FRAGMENTS_KEY
        ),
        500
    );

    let transaction = apply_passive(
        factory,
        "401",
        GoldAndGearsDicePassiveEvent::StorePurchase {
            cosmic_fragments_spent: 250,
        },
    );
    assert_eq!(
        counter(
            &transaction.1,
            RUN_RESOURCES_SLOT,
            RESOURCE_COSMIC_FRAGMENTS_KEY
        ),
        175
    );
    assert_eq!(
        counter(
            &transaction.1,
            PROGRESSION_SLOT,
            PROGRESSION_DICE_PATH_BOOST_STACKS_KEY
        ),
        2
    );

    let general = apply_passive(
        factory,
        "403",
        GoldAndGearsDicePassiveEvent::MovementCompleted { count: 2 },
    );
    assert_eq!(
        counter(&general.1, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        3
    );

    assert!(instance_by_source(factory, "203").dice_allows_same_domain_movement());
    assert!(instance_by_source(factory, "302").dice_preserves_knowledge_domains());
    assert!(instance_by_source(factory, "403").dice_persists_general_buff_faces());
    assert_eq!(
        instance_by_source(factory, "101")
            .compile_dice_passive(
                &new_state(&instance_by_source(factory, "101")),
                GoldAndGearsDicePassiveEvent::TrottersDefeated { count: 0 }
            )
            .unwrap_err(),
        GoldAndGearsEntryError::InvalidDicePassiveEvent
    );
}

fn apply_passive(
    factory: &GoldAndGearsRuntimeFactory,
    source: &str,
    event: GoldAndGearsDicePassiveEvent,
) -> (GoldAndGearsRuntimeInstance, ActivityTransactionState) {
    let instance = instance_by_source(factory, source);
    let mut state = new_state(&instance);
    let program = instance
        .compile_dice_passive(&state, event)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, program);
    (instance, state)
}

fn instance_by_source(
    factory: &GoldAndGearsRuntimeFactory,
    source: &str,
) -> GoldAndGearsRuntimeInstance {
    let dice = factory
        .unique
        .dice
        .iter()
        .find(|dice| dice.identity.source_id.as_ref() == source)
        .unwrap();
    factory
        .compile_entry(entry(factory, AREA, PATH, dice))
        .unwrap()
}

fn new_state(instance: &GoldAndGearsRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) -> Box<[ActivityTransactionEvent]> {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    match state.apply_program(&program, cause, instance.graph_definition()) {
        ActivityTransactionOutcome::Committed(events) => events,
        outcome => panic!("program did not commit: {outcome:?}"),
    }
}

fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = identity();
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

fn state_bytes(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> Box<[u8]> {
    state.canonical_state_bytes(
        identity(),
        instance.graph_definition(),
        ActivityInstanceId::new(1).unwrap(),
        rng,
    )
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
}

fn assert_one_spawn_draw(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
) {
    assert_eq!(before.len(), after.len());
    for (old, new) in before.iter().zip(after) {
        let expected = u64::from(old.label() == ActivityRngLabel::Spawn);
        assert_eq!(new.draw_count(), old.draw_count() + expected);
        assert_eq!(new.seed(), old.seed());
    }
}

fn counter(state: &ActivityTransactionState, slot_id: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map(|index| values[index].1)
            .unwrap_or(0),
        value => panic!("unexpected counter slot: {value:?}"),
    }
}

fn scaled(value: &str) -> i64 {
    let (whole, fractional) = value.split_once('.').unwrap_or((value, ""));
    let mut digits = fractional.to_owned();
    digits.extend(core::iter::repeat_n('0', 6 - digits.len()));
    whole.parse::<i64>().unwrap() * 1_000_000 + digits.parse::<i64>().unwrap_or(0)
}
