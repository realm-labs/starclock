use super::*;

const DESTROYED_CURIO_KEY_BASE: u64 = 0x7ffe_0000_0000_0000;
const PERFECT_STAGE_KEY: u64 = 0x5a00_0000_0000_0001;
const PERFECT_REPEAT_KEY: u64 = 0x5b00_0000_0000_0001;

#[test]
fn goal07_p4_m13_s09_executes_repairs_exchanges_ruan_mei_and_perfect_challenge() {
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
        let definition = catalog
            .occurrence_choices()
            .iter()
            .find(|choice| choice.stable_key() == key)
            .unwrap_or_else(|| panic!("missing {key}"));
        runtime.compile_choice(definition.id()).unwrap()
    };

    let s09_choices = catalog
        .occurrence_choices()
        .iter()
        .filter(|choice| {
            (43..=46).any(|occurrence| {
                choice
                    .stable_key()
                    .starts_with(&format!("universe.occurrence.{occurrence}."))
            }) || (1..=11).any(|choice_index| {
                choice.stable_key()
                    == format!("universe.occurrence.47.variant.13001.choice.{choice_index:02}")
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(s09_choices.len(), 22);
    let deferred = s09_choices
        .iter()
        .filter_map(|definition| {
            let count = runtime
                .compile_choice(definition.id())
                .map_or(u16::MAX, |interaction| interaction.deferred_operations());
            (count != 0).then_some((definition.stable_key(), count))
        })
        .collect::<Vec<_>>();
    assert!(deferred.is_empty(), "deferred S09 choices: {deferred:?}");

    let repair_one = choice("universe.occurrence.43.variant.12601.choice.01");
    assert_eq!(repair_one.external_results().len(), catalog.curios().len());
    let repaired_id = repair_one.external_results()[0].content();
    let activity = execute(
        &compiled,
        repair_one.external_results()[0].payload(),
        None,
        109_001,
        50,
        vec![add_counter(
            compiled.curio_event_slot(),
            DESTROYED_CURIO_KEY_BASE | repaired_id,
            1,
        )],
    );
    assert_eq!(fragments(&activity, &compiled), 0);
    assert_eq!(
        inventory_count(
            &inventory_entries(&activity, compiled.curio_inventory()),
            repaired_id
        ),
        1
    );
    assert_eq!(
        counter(
            &activity,
            compiled.curio_event_slot(),
            DESTROYED_CURIO_KEY_BASE | repaired_id
        ),
        0
    );

    let repair_all = choice("universe.occurrence.43.variant.12601.choice.02");
    let repair_ids = repair_one
        .external_results()
        .iter()
        .take(2)
        .map(|result| result.content())
        .collect::<Vec<_>>();
    let activity = execute(
        &compiled,
        repair_all.payload(),
        repair_all.random_candidate_count(),
        109_002,
        100,
        repair_ids
            .iter()
            .map(|id| {
                add_counter(
                    compiled.curio_event_slot(),
                    DESTROYED_CURIO_KEY_BASE | id,
                    1,
                )
            })
            .collect(),
    );
    let entries = inventory_entries(&activity, compiled.curio_inventory());
    assert_eq!(fragments(&activity, &compiled), 0);
    assert!(
        repair_ids
            .iter()
            .all(|id| inventory_count(&entries, *id) == 1)
    );

    let owned_blessings = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 2)
        .take(2)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect::<Vec<_>>();
    let showman = choice("universe.occurrence.44.variant.12701.choice.01");
    let activity = execute(
        &compiled,
        showman.payload(),
        showman.random_candidate_count(),
        109_003,
        0,
        owned_blessings
            .iter()
            .map(|content| ActivityOperation::AddInventory {
                inventory: compiled.blessing_inventory(),
                content: *content,
                count: integer(1),
            })
            .collect(),
    );
    assert_eq!(
        inventory_entries(&activity, compiled.blessing_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        4
    );

    let lottery_definition = catalog
        .occurrence_choices()
        .iter()
        .find(|choice| choice.stable_key() == "universe.occurrence.45.variant.12801.choice.01")
        .unwrap();
    let lottery_ids = lottery_definition.outcomes()[0]
        .parameter_refs()
        .iter()
        .filter_map(|reference| {
            catalog
                .curios()
                .iter()
                .find(|curio| curio.stable_key() == reference.as_ref())
        })
        .map(|curio| u64::from(curio.id().get()))
        .collect::<Vec<_>>();
    assert_eq!(lottery_ids.len(), 2);
    let lottery = choice(lottery_definition.stable_key());
    let activity = execute(
        &compiled,
        lottery.payload(),
        lottery.random_candidate_count(),
        109_004,
        100,
        vec![add_counter(
            compiled.curio_event_slot(),
            DESTROYED_CURIO_KEY_BASE | lottery_ids[0],
            1,
        )],
    );
    let entries = inventory_entries(&activity, compiled.curio_inventory());
    assert_eq!(fragments(&activity, &compiled), 0);
    assert_eq!(
        lottery_ids
            .iter()
            .filter(|id| inventory_count(&entries, **id) == 1)
            .count(),
        2
    );

    let enhanced = catalog
        .blessings()
        .iter()
        .take(2)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect::<Vec<_>>();
    let ruan_enhance = choice("universe.occurrence.46.variant.12901.choice.01");
    let activity = execute(
        &compiled,
        ruan_enhance.payload(),
        ruan_enhance.random_candidate_count(),
        109_005,
        0,
        enhanced
            .iter()
            .map(|content| ActivityOperation::AddInventory {
                inventory: compiled.blessing_inventory(),
                content: *content,
                count: integer(1),
            })
            .collect(),
    );
    let entries = inventory_entries(&activity, compiled.blessing_inventory());
    assert!(
        enhanced
            .iter()
            .all(|content| inventory_count(&entries, *content) == 2)
    );

    let ruan_curios = choice("universe.occurrence.46.variant.12901.choice.02");
    let activity = execute(
        &compiled,
        ruan_curios.payload(),
        ruan_curios.random_candidate_count(),
        109_006,
        0,
        vec![],
    );
    assert_eq!(
        inventory_entries(&activity, compiled.curio_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        10
    );

    let pay_first = choice("universe.occurrence.47.variant.13001.choice.05");
    let activity = execute(
        &compiled,
        pay_first.payload(),
        pay_first.random_candidate_count(),
        109_007,
        100,
        vec![],
    );
    assert_eq!(fragments(&activity, &compiled), 60);
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            PERFECT_STAGE_KEY
        ),
        1
    );
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            PERFECT_REPEAT_KEY
        ),
        1
    );

    let clay_first = choice("universe.occurrence.47.variant.13001.choice.11");
    let activity = execute(
        &compiled,
        clay_first.payload(),
        clay_first.random_candidate_count(),
        109_008,
        0,
        vec![add_counter(
            compiled.occurrence_interaction_state_slot(),
            PERFECT_STAGE_KEY,
            1,
        )],
    );
    assert_eq!(
        inventory_entries(&activity, compiled.curio_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        1
    );
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            PERFECT_STAGE_KEY
        ),
        2
    );
}

fn execute(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    random_candidates: Option<u32>,
    outcome: u32,
    initial_fragments: i64,
    seed: Vec<ActivityOperation>,
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
    if let Some(candidates) = random_candidates {
        binding = binding.with_random_policy(
            starclock_activity::ActivityInteractionRandomPolicy::new(
                ActivityRngLabel::Occurrence,
                709,
                candidates,
            )
            .unwrap(),
        );
    }
    let registry = compiled
        .runtime_definition()
        .interactions()
        .unwrap()
        .registry();
    let mut activity = occurrence_harness_with_fragments_and_seed(
        compiled,
        &binding,
        registry,
        initial_fragments,
        109_000,
    );
    if !seed.is_empty() {
        let program = ActivityProgramDefinition::new(program(99), seed).unwrap();
        activity
            .apply_boundary_program(activity.player_view().state_hash(), &program)
            .unwrap();
    }
    let before = activity.player_view();
    activity
        .submit_external_outcome(
            before.state_hash(),
            before.decision().unwrap().id(),
            outcome,
        )
        .unwrap_or_else(|error| panic!("S09 outcome {} failed: {error:?}", outcome.get()));
    activity
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

fn inventory_count(entries: &[(u64, u32)], content: u64) -> u32 {
    entries
        .iter()
        .find(|entry| entry.0 == content)
        .map_or(0, |entry| entry.1)
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
            ActivityValue::BoundedCounterMap(entries) => entries
                .iter()
                .find(|entry| entry.0 == key)
                .map(|entry| entry.1),
            _ => None,
        })
        .unwrap_or(0)
}

fn add_counter(slot: ActivitySlotId, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot,
        key,
        delta: integer(delta),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
