use super::*;

const MIRROR_CANDLE_KEY: u64 = 0x5f00_0000_0000_0001;
const MIRROR_PART_TWO_KEY: u64 = 0x5f00_0000_0000_0005;

#[test]
fn goal07_p4_m13_s12_executes_mirror_battle_and_cuckoo_clock_outcomes() {
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

    let mut keys = (4..=18)
        .map(|ordinal| format!("universe.occurrence.62.variant.19501.choice.{ordinal:02}"))
        .collect::<Vec<_>>();
    keys.extend(
        (1..=7).map(|ordinal| format!("universe.occurrence.63.variant.19601.choice.{ordinal:02}")),
    );
    keys.extend(
        (1..=6).map(|ordinal| format!("universe.occurrence.64.variant.19601.choice.{ordinal:02}")),
    );
    let deferred = keys
        .iter()
        .filter_map(|key| {
            let compiled = choice(key);
            (compiled.deferred_operations() != 0)
                .then_some((key.as_str(), compiled.deferred_operations()))
        })
        .collect::<Vec<_>>();
    assert!(deferred.is_empty(), "deferred S12 choices: {deferred:?}");

    let rescue = choice("universe.occurrence.62.variant.19501.choice.04");
    assert!(rescue.battle_member().is_some());
    let activity = execute(
        &compiled,
        rescue.payload(),
        rescue.random_candidate_count(),
        130_001,
        vec![],
    );
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            MIRROR_PART_TWO_KEY
        ),
        1
    );
    for key in [
        "universe.occurrence.62.variant.19501.choice.08",
        "universe.occurrence.62.variant.19501.choice.10",
    ] {
        assert!(choice(key).battle_member().is_some());
    }

    let light = choice("universe.occurrence.62.variant.19501.choice.05");
    let activity = execute(
        &compiled,
        light.payload(),
        light.random_candidate_count(),
        130_002,
        vec![],
    );
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            MIRROR_CANDLE_KEY
        ),
        1
    );

    let wish = choice("universe.occurrence.62.variant.19501.choice.07");
    assert_eq!(wish.random_candidate_count(), Some(64_800));
    execute(
        &compiled,
        wish.payload(),
        wish.random_candidate_count(),
        130_003,
        vec![add_counter(
            compiled.occurrence_interaction_state_slot(),
            MIRROR_CANDLE_KEY,
            1,
        )],
    );

    let clock_keys = [
        "universe.curio.65",
        "universe.curio.66",
        "universe.curio.67",
        "universe.curio.70",
        "universe.curio.71",
        "universe.curio.108",
    ];
    let clocks = clock_keys
        .iter()
        .map(|key| {
            catalog
                .curios()
                .iter()
                .find(|curio| curio.stable_key() == *key)
                .unwrap_or_else(|| panic!("missing {key}"))
        })
        .collect::<Vec<_>>();
    assert_eq!(clocks.len(), 6);
    let clock_ids = clocks
        .iter()
        .map(|curio| u64::from(curio.id().get()))
        .collect::<Vec<_>>();

    let acquire_one = choice("universe.occurrence.63.variant.19601.choice.01");
    assert_eq!(acquire_one.random_candidate_count(), Some(6));
    let activity = execute(
        &compiled,
        acquire_one.payload(),
        acquire_one.random_candidate_count(),
        130_004,
        vec![],
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids,),
        1
    );

    let acquire_two = choice("universe.occurrence.63.variant.19601.choice.02");
    let activity = execute(
        &compiled,
        acquire_two.payload(),
        acquire_two.random_candidate_count(),
        130_005,
        vec![],
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids,),
        2
    );

    let accept_again = choice("universe.occurrence.63.variant.19601.choice.03");
    let two_star = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 2)
        .count();
    assert_eq!(
        accept_again.random_candidate_count(),
        u32::try_from(6 * two_star).ok()
    );
    let activity = execute(
        &compiled,
        accept_again.payload(),
        accept_again.random_candidate_count(),
        130_006,
        vec![],
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids,),
        1
    );
    assert_eq!(
        inventory_entries(&activity, compiled.blessing_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        1
    );

    let discard = choice("universe.occurrence.63.variant.19601.choice.04");
    let activity = execute(
        &compiled,
        discard.payload(),
        discard.random_candidate_count(),
        130_007,
        seed_curios(&compiled, &clocks[..2]),
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids,),
        0
    );

    let accept_third = choice("universe.occurrence.63.variant.19601.choice.05");
    let three_star = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 3)
        .count();
    assert_eq!(
        accept_third.random_candidate_count(),
        u32::try_from(6 * three_star).ok()
    );
    execute(
        &compiled,
        accept_third.payload(),
        accept_third.random_candidate_count(),
        130_008,
        vec![],
    );

    let exchange_curios = choice("universe.occurrence.63.variant.19601.choice.06");
    let activity = execute(
        &compiled,
        exchange_curios.payload(),
        exchange_curios.random_candidate_count(),
        130_009,
        seed_curios(&compiled, &clocks[..2]),
    );
    assert_eq!(
        inventory_entries(&activity, compiled.curio_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        2
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids,),
        0
    );

    let exchange_blessings = choice("universe.occurrence.63.variant.19601.choice.07");
    assert_eq!(
        exchange_blessings.random_candidate_count(),
        u32::try_from(catalog.blessings().len()).ok()
    );
    let activity = execute(
        &compiled,
        exchange_blessings.payload(),
        exchange_blessings.random_candidate_count(),
        130_010,
        seed_curios(&compiled, &clocks[..2]),
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids,),
        0
    );
    assert_eq!(
        inventory_entries(&activity, compiled.blessing_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        2
    );
}

fn execute(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    random_candidates: Option<u32>,
    outcome: u32,
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
                712,
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
    let mut activity =
        occurrence_harness_with_fragments_and_seed(compiled, &binding, registry, 0, 130_000);
    if !seed.is_empty() {
        let program = ActivityProgramDefinition::new(program(102), seed).unwrap();
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
        .unwrap_or_else(|error| panic!("S12 outcome {} failed: {error:?}", outcome.get()));
    activity
}

fn seed_curios(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    curios: &[&starclock_mode_universe::curio::CurioDefinition],
) -> Vec<ActivityOperation> {
    curios
        .iter()
        .flat_map(|curio| {
            [
                ActivityOperation::AddInventory {
                    inventory: compiled.curio_inventory(),
                    content: u64::from(curio.id().get()),
                    count: integer(1),
                },
                add_counter(
                    compiled.curio_state_slot(),
                    u64::from(curio.id().get()),
                    i64::from(curio.initial_state().get()),
                ),
            ]
        })
        .collect()
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

fn owned_count(
    activity: &GraphActivity,
    inventory: starclock_activity::ActivityInventoryId,
    candidates: &[u64],
) -> u32 {
    inventory_entries(activity, inventory)
        .iter()
        .filter(|entry| candidates.contains(&entry.0))
        .map(|entry| entry.1)
        .sum()
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
