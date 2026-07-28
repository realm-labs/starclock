use super::*;

#[test]
fn goal07_p4_m13_s07_executes_exchange_path_curio_and_fragment_outcomes() {
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
            .unwrap();
        runtime.compile_choice(definition.id()).unwrap()
    };

    let mut keys = (13_702..=13_705)
        .map(|variant| format!("universe.occurrence.35.variant.{variant}.choice.01"))
        .collect::<Vec<_>>();
    for (occurrence, variant, count) in [
        (36, 13_801, 3),
        (37, 13_901, 3),
        (38, 14_001, 3),
        (39, 12_201, 8),
    ] {
        for index in 1..=count {
            keys.push(format!(
                "universe.occurrence.{occurrence}.variant.{variant}.choice.{index:02}"
            ));
        }
    }
    assert_eq!(keys.len(), 21);
    assert!(
        keys.iter()
            .all(|key| choice(key).deferred_operations() == 0)
    );
    let periodic_members = (13_702..=13_705)
        .map(|variant| {
            choice(&format!(
                "universe.occurrence.35.variant.{variant}.choice.01"
            ))
            .battle_member()
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(periodic_members.len(), 4);
    assert!(periodic_members.windows(2).all(|pair| pair[0] != pair[1]));

    let preservation = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.preservation")
        .unwrap();
    let selected_path = u64::from(preservation.id().get());
    let selected_path_operation = ActivityOperation::SetSlot {
        slot: compiled.selected_path_slot(),
        value: ActivityExpression::Literal(ActivityValue::OptionalId(Some(selected_path))),
    };

    let three_star = catalog
        .blessings()
        .iter()
        .find(|blessing| blessing.rarity() == 3)
        .unwrap();
    let three_star_id = u64::from(three_star.id().get());
    let reforge = choice("universe.occurrence.36.variant.13801.choice.01");
    let result = execute(
        &compiled,
        reforge.payload(),
        107_001,
        reforge.random_candidate_count(),
        vec![
            selected_path_operation.clone(),
            add_inventory(compiled.blessing_inventory(), three_star_id),
        ],
    );
    let blessings = inventory_entries(&result, compiled.blessing_inventory());
    assert_eq!(blessings.len(), 1);
    assert_eq!(blessings[0].1, 1);
    assert_ne!(blessings[0].0, three_star_id);
    assert_eq!(blessing(&catalog, blessings[0].0).rarity(), 3);

    let one_star = catalog
        .blessings()
        .iter()
        .find(|blessing| blessing.rarity() == 1)
        .unwrap();
    let one_star_id = u64::from(one_star.id().get());
    let exchange = choice("universe.occurrence.36.variant.13801.choice.02");
    let result = execute(
        &compiled,
        exchange.payload(),
        107_002,
        exchange.random_candidate_count(),
        vec![
            selected_path_operation.clone(),
            add_inventory(compiled.blessing_inventory(), one_star_id),
        ],
    );
    let blessings = inventory_entries(&result, compiled.blessing_inventory());
    assert_eq!(blessings.len(), 1);
    assert_ne!(blessings[0].0, one_star_id);
    assert!((1..=3).contains(&blessing(&catalog, blessings[0].0).rarity()));

    for (key, loss, rarity) in [
        ("universe.occurrence.37.variant.13901.choice.01", 20, 2),
        ("universe.occurrence.37.variant.13901.choice.02", 80, 3),
    ] {
        let definition = catalog
            .occurrence_choices()
            .iter()
            .find(|choice| choice.stable_key() == key)
            .unwrap();
        let outcome = &definition.outcomes()[0];
        assert_eq!(
            outcome.operations(),
            &[OccurrenceOperation::Lose, OccurrenceOperation::Obtain]
        );
        assert_eq!(
            outcome.targets(),
            &[OccurrenceTarget::Hp, OccurrenceTarget::Blessing]
        );
        assert_eq!(
            outcome.numeric_literals()[0].unit(),
            AuthoredScalarUnit::Percent
        );
        assert_eq!(outcome.numeric_literals()[0].value().coefficient(), loss);
        assert!(
            outcome
                .parameter_refs()
                .iter()
                .any(|value| value.as_ref() == format!("universe.blessing-pool.rarity.{rarity}"))
        );
    }

    for (key, cost, rarities) in [
        (
            "universe.occurrence.38.variant.14001.choice.01",
            50,
            &[1_u8, 2][..],
        ),
        (
            "universe.occurrence.38.variant.14001.choice.02",
            100,
            &[1_u8, 2, 3][..],
        ),
    ] {
        let reward = choice(key);
        let result = execute(
            &compiled,
            reward.payload(),
            107_010 + u32::try_from(cost).unwrap(),
            reward.random_candidate_count(),
            vec![selected_path_operation.clone()],
        );
        assert_eq!(fragments(&result, &compiled), 150 - cost);
        let blessings = inventory_entries(&result, compiled.blessing_inventory());
        assert_eq!(blessings.len(), 1);
        assert!(rarities.contains(&blessing(&catalog, blessings[0].0).rarity()));
    }

    let owned_three_star = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 3)
        .take(2)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect::<Vec<_>>();
    let enhance = choice("universe.occurrence.39.variant.12201.choice.01");
    let result = execute(
        &compiled,
        enhance.payload(),
        107_101,
        enhance.random_candidate_count(),
        owned_three_star
            .iter()
            .map(|id| add_inventory(compiled.blessing_inventory(), *id))
            .collect(),
    );
    let entries = inventory_entries(&result, compiled.blessing_inventory());
    assert!(
        owned_three_star
            .iter()
            .all(|id| inventory_count(&entries, *id) == 2)
    );

    let formation = choice("universe.occurrence.39.variant.12201.choice.02");
    let result = execute(
        &compiled,
        formation.payload(),
        107_102,
        formation.random_candidate_count(),
        vec![selected_path_operation.clone()],
    );
    let formations = inventory_entries(&result, compiled.formation_inventory());
    assert_eq!(formations.len(), 1);
    assert!(
        preservation
            .formations()
            .iter()
            .any(|id| u64::from(id.get()) == formations[0].0)
    );

    for (key, quantity, rarities) in [
        (
            "universe.occurrence.39.variant.12201.choice.03",
            2,
            &[2_u8, 3][..],
        ),
        (
            "universe.occurrence.39.variant.12201.choice.04",
            3,
            &[1_u8, 2, 3][..],
        ),
    ] {
        let reward = choice(key);
        let result = execute(
            &compiled,
            reward.payload(),
            107_110 + quantity,
            reward.random_candidate_count(),
            vec![selected_path_operation.clone()],
        );
        let entries = inventory_entries(&result, compiled.blessing_inventory());
        assert_eq!(entries.len(), usize::try_from(quantity).unwrap());
        assert!(entries.iter().all(|(id, count)| {
            let blessing = blessing(&catalog, *id);
            *count == 1
                && blessing.path() == preservation.id()
                && rarities.contains(&blessing.rarity())
        }));
    }

    let exchange_seed = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 1 && blessing.path() != preservation.id())
        .take(4)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect::<Vec<_>>();
    let path_exchange = choice("universe.occurrence.39.variant.12201.choice.05");
    let mut seed = vec![selected_path_operation.clone()];
    seed.extend(
        exchange_seed
            .iter()
            .map(|id| add_inventory(compiled.blessing_inventory(), *id)),
    );
    let result = execute(
        &compiled,
        path_exchange.payload(),
        107_105,
        path_exchange.random_candidate_count(),
        seed,
    );
    let entries = inventory_entries(&result, compiled.blessing_inventory());
    assert_eq!(entries.len(), 4);
    assert!(
        exchange_seed
            .iter()
            .all(|id| inventory_count(&entries, *id) == 0)
    );
    assert!(
        entries
            .iter()
            .all(|(id, count)| *count == 1 && blessing(&catalog, *id).path() == preservation.id())
    );

    let curios_reward = choice("universe.occurrence.39.variant.12201.choice.06");
    let result = execute(
        &compiled,
        curios_reward.payload(),
        107_106,
        curios_reward.random_candidate_count(),
        vec![],
    );
    let curios = inventory_entries(&result, compiled.curio_inventory());
    assert_eq!(curios.len(), 3);
    assert!(curios.iter().all(|(id, count)| {
        *count == 1 && counter(&result, compiled.curio_state_slot(), *id) > 0
    }));

    let seeded_curios = catalog
        .curios()
        .iter()
        .filter(|curio| curio.id().get() != 8)
        .take(2)
        .collect::<Vec<_>>();
    let all_curios = choice("universe.occurrence.39.variant.12201.choice.07");
    let result = execute(
        &compiled,
        all_curios.payload(),
        107_107,
        all_curios.random_candidate_count(),
        seed_curios(&compiled, &seeded_curios),
    );
    assert!(inventory_entries(&result, compiled.curio_inventory()).is_empty());
    assert_eq!(fragments(&result, &compiled), 250);
    assert!(seeded_curios.iter().all(|curio| {
        counter(
            &result,
            compiled.curio_state_slot(),
            u64::from(curio.id().get()),
        ) == 0
    }));

    let cuckoo = catalog
        .curios()
        .iter()
        .find(|curio| curio.stable_key() == "universe.curio.65")
        .unwrap();
    let survivor = catalog
        .curios()
        .iter()
        .find(|curio| !curio.tags().iter().any(|tag| tag.as_ref() == "negative"))
        .unwrap();
    let discard_cuckoo = choice("universe.occurrence.39.variant.12201.choice.08");
    let result = execute(
        &compiled,
        discard_cuckoo.payload(),
        107_108,
        discard_cuckoo.random_candidate_count(),
        seed_curios(&compiled, &[cuckoo, survivor]),
    );
    let entries = inventory_entries(&result, compiled.curio_inventory());
    assert_eq!(inventory_count(&entries, u64::from(cuckoo.id().get())), 0);
    assert_eq!(inventory_count(&entries, u64::from(survivor.id().get())), 1);
}

fn execute(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    outcome: u32,
    random_candidates: Option<u32>,
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
                707,
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
        occurrence_harness_with_fragments_and_seed(compiled, &binding, registry, 150, 107_000);
    if !seed.is_empty() {
        let program = ActivityProgramDefinition::new(program(97), seed).unwrap();
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
        .unwrap_or_else(|error| panic!("S07 outcome {} failed: {error:?}", outcome.get()));
    activity
}

fn add_inventory(
    inventory: starclock_activity::ActivityInventoryId,
    content: u64,
) -> ActivityOperation {
    ActivityOperation::AddInventory {
        inventory,
        content,
        count: integer(1),
    }
}

fn seed_curios(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    curios: &[&starclock_mode_universe::curio::CurioDefinition],
) -> Vec<ActivityOperation> {
    curios
        .iter()
        .flat_map(|curio| {
            let id = u64::from(curio.id().get());
            [
                add_inventory(compiled.curio_inventory(), id),
                ActivityOperation::AddCounter {
                    slot: compiled.curio_state_slot(),
                    key: id,
                    delta: integer(i64::from(curio.initial_state().get())),
                },
            ]
        })
        .collect()
}

fn blessing(
    catalog: &UniverseCatalog,
    id: u64,
) -> &starclock_mode_universe::path::BlessingDefinition {
    catalog
        .blessings()
        .iter()
        .find(|blessing| u64::from(blessing.id().get()) == id)
        .unwrap()
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
