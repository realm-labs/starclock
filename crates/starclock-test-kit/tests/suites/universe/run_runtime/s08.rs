use super::*;

#[test]
fn goal07_p4_m13_s08_executes_external_history_cosmic_and_branching_outcomes() {
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

    let s08_choices = catalog
        .occurrence_choices()
        .iter()
        .filter(|choice| {
            matches!(
                choice.stable_key(),
                "universe.occurrence.39.variant.12201.choice.09"
                    | "universe.occurrence.39.variant.12201.choice.10"
                    | "universe.occurrence.39.variant.12201.choice.11"
            ) || choice
                .stable_key()
                .starts_with("universe.occurrence.4.variant.10201.")
                || (40..=42).any(|occurrence| {
                    choice
                        .stable_key()
                        .starts_with(&format!("universe.occurrence.{occurrence}."))
                })
        })
        .collect::<Vec<_>>();
    assert_eq!(s08_choices.len(), 24);
    assert!(s08_choices.iter().all(|definition| {
        runtime
            .compile_choice(definition.id())
            .is_some_and(|interaction| interaction.deferred_operations() == 0)
    }));

    for (key, candidates) in [
        ("universe.occurrence.4.variant.10201.choice.01", 8),
        ("universe.occurrence.4.variant.10201.choice.02", 7),
        ("universe.occurrence.4.variant.10201.choice.03", 3),
        ("universe.occurrence.4.variant.10201.choice.05", 8),
        ("universe.occurrence.4.variant.10201.choice.06", 7),
        ("universe.occurrence.4.variant.10201.choice.07", 3),
    ] {
        let history = choice(key);
        assert_eq!(history.external_results().len(), 9);
        assert!(history.external_results().iter().all(|result| {
            result.random_candidate_count() == Some(candidates)
                && result.immediate_operations() == 1
        }));
    }

    let preservation = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.preservation")
        .unwrap();
    let owned = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.path() == preservation.id() && blessing.rarity() == 1)
        .take(3)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect::<Vec<_>>();
    let history = choice("universe.occurrence.4.variant.10201.choice.01");
    let result = history
        .external_results()
        .iter()
        .find(|result| result.content() == u64::from(preservation.id().get()))
        .unwrap();
    let activity = execute(
        &compiled,
        result.payload(),
        result.random_candidate_count(),
        108_001,
        50,
        owned
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
        owned
            .iter()
            .all(|content| inventory_count(&entries, *content) == 2)
    );

    let cosmic = choice("universe.occurrence.40.variant.12301.choice.01");
    assert_eq!(cosmic.external_results().len(), 16);
    assert!(cosmic.repeat_key().is_some());
    assert!(
        cosmic
            .external_results()
            .iter()
            .all(|result| result.random_candidate_count().is_some())
    );
    let fragments_effect = &cosmic.external_results()[1];
    let activity = execute(
        &compiled,
        fragments_effect.payload(),
        fragments_effect.random_candidate_count(),
        108_002,
        50,
        vec![],
    );
    assert_eq!(fragments(&activity, &compiled), 150);
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            cosmic.repeat_key().unwrap()
        ),
        9
    );

    for (key, chances) in [
        (
            "universe.occurrence.41.variant.12401.choice.03",
            &[70_i64, 20, 10][..],
        ),
        (
            "universe.occurrence.41.variant.12401.choice.04",
            &[50_i64, 50][..],
        ),
        (
            "universe.occurrence.41.variant.12401.choice.05",
            &[80_i64, 20][..],
        ),
        (
            "universe.occurrence.41.variant.12401.choice.06",
            &[50_i64, 30, 20][..],
        ),
    ] {
        let definition = catalog
            .occurrence_choices()
            .iter()
            .find(|choice| choice.stable_key() == key)
            .unwrap();
        assert_eq!(
            definition.outcomes()[0]
                .chance_percentages()
                .iter()
                .map(|value| value.coefficient())
                .collect::<Vec<_>>(),
            chances
        );
        assert_eq!(choice(key).random_candidate_count(), Some(100));
    }
    assert!(
        choice("universe.occurrence.41.variant.12401.choice.07")
            .battle_member()
            .is_some()
    );
    assert!(
        choice("universe.occurrence.41.variant.12401.choice.09")
            .battle_member()
            .is_some()
    );

    assert_eq!(
        choice("universe.occurrence.39.variant.12201.choice.09").random_candidate_count(),
        Some(3)
    );
    let fragments_reward = choice("universe.occurrence.39.variant.12201.choice.10");
    let activity = execute(
        &compiled,
        fragments_reward.payload(),
        fragments_reward.random_candidate_count(),
        108_010,
        50,
        vec![],
    );
    assert_eq!(fragments(&activity, &compiled), 450);
    assert!(
        choice("universe.occurrence.42.variant.12501.choice.01")
            .battle_member()
            .is_some()
    );
    assert!(
        choice("universe.occurrence.42.variant.12501.choice.02")
            .battle_member()
            .is_some()
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
                708,
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
        108_000,
    );
    if !seed.is_empty() {
        let program = ActivityProgramDefinition::new(program(98), seed).unwrap();
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
        .unwrap_or_else(|error| panic!("S08 outcome {} failed: {error:?}", outcome.get()));
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

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
