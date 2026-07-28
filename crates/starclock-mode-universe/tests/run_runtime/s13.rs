use super::*;

#[test]
fn goal07_p4_m13_s13_executes_cuckoo_mirror_and_cremator_outcomes() {
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

    let mut keys = vec!["universe.occurrence.64.variant.19601.choice.07".to_owned()];
    keys.extend(
        (1..=7).map(|ordinal| format!("universe.occurrence.65.variant.19601.choice.{ordinal:02}")),
    );
    keys.extend(
        (1..=16).map(|ordinal| format!("universe.occurrence.77.variant.19501.choice.{ordinal:02}")),
    );
    keys.extend([
        "universe.occurrence.7.variant.10501.choice.01".to_owned(),
        "universe.occurrence.7.variant.10501.choice.02".to_owned(),
    ]);
    assert_eq!(keys.len(), 26);
    let deferred = keys
        .iter()
        .filter_map(|key| {
            let compiled = choice(key);
            (compiled.deferred_operations() != 0)
                .then_some((key.as_str(), compiled.deferred_operations()))
        })
        .collect::<Vec<_>>();
    assert!(deferred.is_empty(), "deferred S13 choices: {deferred:?}");

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
    let clock_ids = clocks
        .iter()
        .map(|curio| u64::from(curio.id().get()))
        .collect::<Vec<_>>();

    let bridge = choice("universe.occurrence.64.variant.19601.choice.07");
    let activity = execute(
        &compiled,
        bridge.payload(),
        bridge.random_candidate_count(),
        140_001,
        seed_curios(&compiled, &clocks[..2]),
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids),
        0
    );
    assert_eq!(inventory_total(&activity, compiled.blessing_inventory()), 2);

    let accept_third = choice("universe.occurrence.65.variant.19601.choice.05");
    let three_star = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 3)
        .count();
    assert_eq!(
        accept_third.random_candidate_count(),
        u32::try_from(6 * three_star).ok()
    );
    let activity = execute(
        &compiled,
        accept_third.payload(),
        accept_third.random_candidate_count(),
        140_002,
        vec![],
    );
    assert_eq!(
        owned_count(&activity, compiled.curio_inventory(), &clock_ids),
        1
    );
    assert_eq!(inventory_total(&activity, compiled.blessing_inventory()), 1);

    for ordinal in 1..=16 {
        let mirror = choice(&format!(
            "universe.occurrence.77.variant.19501.choice.{ordinal:02}"
        ));
        assert_eq!(mirror.immediate_operations(), 1);
        assert_eq!(mirror.random_candidate_count(), None);
    }
    let mirror = choice("universe.occurrence.77.variant.19501.choice.01");
    let activity = execute(
        &compiled,
        mirror.payload(),
        mirror.random_candidate_count(),
        140_003,
        vec![],
    );
    assert_eq!(slot_integer(&activity, compiled.cosmic_fragments_slot()), 0);
    assert_eq!(inventory_total(&activity, compiled.blessing_inventory()), 0);
    assert_eq!(inventory_total(&activity, compiled.curio_inventory()), 0);

    let lost = catalog
        .blessings()
        .iter()
        .find(|blessing| blessing.rarity() == 1)
        .unwrap();
    let exchange = choice("universe.occurrence.7.variant.10501.choice.01");
    let activity = execute(
        &compiled,
        exchange.payload(),
        exchange.random_candidate_count(),
        140_004,
        vec![ActivityOperation::AddInventory {
            inventory: compiled.blessing_inventory(),
            content: u64::from(lost.id().get()),
            count: integer(1),
        }],
    );
    let entries = inventory_entries(&activity, compiled.blessing_inventory());
    assert_eq!(entries.iter().map(|entry| entry.1).sum::<u32>(), 1);
    assert!(
        entries
            .iter()
            .all(|entry| entry.0 != u64::from(lost.id().get()))
    );

    let fragments = choice("universe.occurrence.7.variant.10501.choice.02");
    let activity = execute(
        &compiled,
        fragments.payload(),
        fragments.random_candidate_count(),
        140_005,
        vec![],
    );
    assert_eq!(
        slot_integer(&activity, compiled.cosmic_fragments_slot()),
        80
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
                713,
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
        occurrence_harness_with_fragments_and_seed(compiled, &binding, registry, 0, 140_000);
    if !seed.is_empty() {
        let program = ActivityProgramDefinition::new(program(103), seed).unwrap();
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
        .unwrap_or_else(|error| panic!("S13 outcome {} failed: {error:?}", outcome.get()));
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
                ActivityOperation::AddCounter {
                    slot: compiled.curio_state_slot(),
                    key: u64::from(curio.id().get()),
                    delta: integer(i64::from(curio.initial_state().get())),
                },
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

fn inventory_total(
    activity: &GraphActivity,
    inventory: starclock_activity::ActivityInventoryId,
) -> u32 {
    inventory_entries(activity, inventory)
        .iter()
        .map(|entry| entry.1)
        .sum()
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

fn slot_integer(activity: &GraphActivity, slot: ActivitySlotId) -> i64 {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedInteger(value) => Some(*value),
            _ => None,
        })
        .unwrap()
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
