use super::*;

#[test]
fn goal07_p4_m13_s14_executes_mirror_interactive_arts_and_pixel_world_outcomes() {
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

    let mut keys = (17..=18)
        .map(|ordinal| format!("universe.occurrence.77.variant.19501.choice.{ordinal:02}"))
        .collect::<Vec<_>>();
    keys.extend(
        (1..=18).map(|ordinal| format!("universe.occurrence.78.variant.19501.choice.{ordinal:02}")),
    );
    keys.extend(
        (1..=3).map(|ordinal| format!("universe.occurrence.8.variant.10601.choice.{ordinal:02}")),
    );
    keys.extend(
        (1..=2).map(|ordinal| format!("universe.occurrence.9.variant.10701.choice.{ordinal:02}")),
    );
    assert_eq!(keys.len(), 25);
    let deferred = keys
        .iter()
        .filter_map(|key| {
            let compiled = choice(key);
            (compiled.deferred_operations() != 0)
                .then_some((key.as_str(), compiled.deferred_operations()))
        })
        .collect::<Vec<_>>();
    assert!(deferred.is_empty(), "deferred S14 choices: {deferred:?}");

    for ordinal in 17..=18 {
        let mirror = choice(&format!(
            "universe.occurrence.77.variant.19501.choice.{ordinal:02}"
        ));
        assert_eq!(mirror.immediate_operations(), 1);
        assert_eq!(mirror.random_candidate_count(), None);
    }
    for ordinal in 1..=18 {
        let mirror = choice(&format!(
            "universe.occurrence.78.variant.19501.choice.{ordinal:02}"
        ));
        assert_eq!(mirror.immediate_operations(), 1);
        assert_eq!(mirror.random_candidate_count(), None);
    }
    let mirror = choice("universe.occurrence.78.variant.19501.choice.01");
    let activity = execute(
        &compiled,
        mirror.payload(),
        mirror.random_candidate_count(),
        150_001,
    );
    assert_eq!(slot_integer(&activity, compiled.cosmic_fragments_slot()), 0);
    assert_eq!(inventory_total(&activity, compiled.blessing_inventory()), 0);
    assert_eq!(inventory_total(&activity, compiled.curio_inventory()), 0);

    for (ordinal, path_key) in [(1, "universe.path.elation"), (2, "universe.path.hunt")] {
        let blessing = choice(&format!(
            "universe.occurrence.8.variant.10601.choice.{ordinal:02}"
        ));
        let expected_path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == path_key)
            .unwrap()
            .id();
        let expected = catalog
            .blessings()
            .iter()
            .filter(|value| value.path() == expected_path && value.rarity() == 2)
            .count();
        assert_eq!(blessing.external_results().len(), expected);
        let selected = blessing.external_results()[0].content();
        let selected_definition = catalog
            .blessings()
            .iter()
            .find(|value| u64::from(value.id().get()) == selected)
            .unwrap();
        assert_eq!(selected_definition.path(), expected_path);
        assert_eq!(selected_definition.rarity(), 2);
        let activity = execute(
            &compiled,
            blessing.external_results()[0].payload(),
            None,
            150_010 + ordinal,
        );
        assert_eq!(
            inventory_count(&activity, compiled.blessing_inventory(), selected),
            1
        );
    }

    let restore = choice("universe.occurrence.8.variant.10601.choice.03");
    assert_eq!(restore.immediate_operations(), 1);
    assert_eq!(restore.random_candidate_count(), None);

    let fragments = choice("universe.occurrence.9.variant.10701.choice.01");
    let activity = execute(
        &compiled,
        fragments.payload(),
        fragments.random_candidate_count(),
        150_020,
    );
    assert_eq!(
        slot_integer(&activity, compiled.cosmic_fragments_slot()),
        200
    );

    let bricks = choice("universe.occurrence.9.variant.10701.choice.02");
    let one_star = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 1)
        .count();
    assert_eq!(
        bricks.random_candidate_count(),
        u32::try_from(one_star).ok()
    );
    let activity = execute(
        &compiled,
        bricks.payload(),
        bricks.random_candidate_count(),
        150_021,
    );
    let entries = inventory_entries(&activity, compiled.blessing_inventory());
    assert_eq!(entries.iter().map(|entry| entry.1).sum::<u32>(), 2);
    assert!(entries.iter().all(|entry| {
        catalog
            .blessings()
            .iter()
            .find(|blessing| u64::from(blessing.id().get()) == entry.0)
            .is_some_and(|blessing| blessing.rarity() == 1)
    }));
}

fn execute(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    random_candidates: Option<u32>,
    outcome: u32,
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
                714,
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
    let before =
        occurrence_harness_with_fragments_and_seed(compiled, &binding, registry, 0, 150_000);
    let mut activity = before;
    let view = activity.player_view();
    activity
        .submit_external_outcome(view.state_hash(), view.decision().unwrap().id(), outcome)
        .unwrap_or_else(|error| panic!("S14 outcome {} failed: {error:?}", outcome.get()));
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

fn inventory_total(
    activity: &GraphActivity,
    inventory: starclock_activity::ActivityInventoryId,
) -> u32 {
    inventory_entries(activity, inventory)
        .iter()
        .map(|entry| entry.1)
        .sum()
}

fn inventory_count(
    activity: &GraphActivity,
    inventory: starclock_activity::ActivityInventoryId,
    content: u64,
) -> u32 {
    inventory_entries(activity, inventory)
        .iter()
        .find(|entry| entry.0 == content)
        .map_or(0, |entry| entry.1)
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
