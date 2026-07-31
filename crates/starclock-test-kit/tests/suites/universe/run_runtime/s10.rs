use super::*;

const PERFECT_STAGE_KEY: u64 = 0x5a00_0000_0000_0001;
const BANK_KEY: u64 = 0x5c00_0000_0000_0001;
const BEAUTY_BUG_UNLOCK_KEY: u64 = 0x5d00_0000_0000_0001;

#[test]
fn goal07_p4_m13_s10_executes_popular_banking_blessing_and_beauty_bug_outcomes() {
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

    let s10_choices = catalog
        .occurrence_choices()
        .iter()
        .filter(|choice| {
            choice.stable_key() == "universe.occurrence.47.variant.13001.choice.12"
                || [5, 52, 53].iter().any(|occurrence| {
                    choice
                        .stable_key()
                        .starts_with(&format!("universe.occurrence.{occurrence}."))
                })
                || choice.stable_key() == "universe.occurrence.54.variant.14401.choice.01"
        })
        .collect::<Vec<_>>();
    assert_eq!(s10_choices.len(), 24);
    let deferred = s10_choices
        .iter()
        .filter_map(|definition| {
            let count = runtime
                .compile_choice(definition.id())
                .map_or(u16::MAX, |interaction| interaction.deferred_operations());
            (count != 0).then_some((definition.stable_key(), count))
        })
        .collect::<Vec<_>>();
    assert!(deferred.is_empty(), "deferred S10 choices: {deferred:?}");

    let popular = choice("universe.occurrence.47.variant.13001.choice.12");
    assert_eq!(popular.random_candidate_count(), Some(230));
    let activity = execute(
        &compiled,
        popular.payload(),
        popular.random_candidate_count(),
        110_001,
        0,
        vec![add_counter(
            compiled.occurrence_interaction_state_slot(),
            PERFECT_STAGE_KEY,
            1,
        )],
    );
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            PERFECT_STAGE_KEY
        ),
        2
    );
    assert!(
        inventory_entries(&activity, compiled.curio_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>()
            <= 1
    );

    let blessing = choice("universe.occurrence.5.variant.10301.choice.01");
    let blessing_candidates = catalog
        .blessings()
        .iter()
        .filter(|blessing| matches!(blessing.rarity(), 1 | 2))
        .count();
    assert_eq!(blessing.external_results().len(), blessing_candidates);
    let activity = execute(
        &compiled,
        blessing.external_results()[0].payload(),
        None,
        110_002,
        0,
        vec![],
    );
    assert_eq!(
        inventory_entries(&activity, compiled.blessing_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        1
    );

    let fragments_reward = choice("universe.occurrence.5.variant.10301.choice.02");
    let activity = execute(
        &compiled,
        fragments_reward.payload(),
        fragments_reward.random_candidate_count(),
        110_003,
        0,
        vec![],
    );
    assert_eq!(fragments(&activity, &compiled), 100);

    let deposit = choice("universe.occurrence.52.variant.14301.choice.01");
    let activity = execute(
        &compiled,
        deposit.payload(),
        deposit.random_candidate_count(),
        110_004,
        150,
        vec![],
    );
    assert_eq!(fragments(&activity, &compiled), 50);
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            BANK_KEY
        ),
        1
    );

    let withdraw = choice("universe.occurrence.53.variant.14301.choice.05");
    let activity = execute(
        &compiled,
        withdraw.payload(),
        withdraw.random_candidate_count(),
        110_005,
        0,
        vec![add_counter(
            compiled.occurrence_interaction_state_slot(),
            BANK_KEY,
            1,
        )],
    );
    assert_eq!(fragments(&activity, &compiled), 200);
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            BANK_KEY
        ),
        0
    );

    let preserve = choice("universe.occurrence.53.variant.14301.choice.06");
    let activity = execute(
        &compiled,
        preserve.payload(),
        preserve.random_candidate_count(),
        110_006,
        17,
        vec![add_counter(
            compiled.occurrence_interaction_state_slot(),
            BANK_KEY,
            1,
        )],
    );
    assert_eq!(fragments(&activity, &compiled), 17);
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            BANK_KEY
        ),
        1
    );

    let beauty_bug = choice("universe.occurrence.54.variant.14401.choice.01");
    assert_eq!(beauty_bug.external_results().len(), catalog.curios().len());
    assert!(beauty_bug.external_results().iter().all(|result| {
        result.random_candidate_count() == Some(100) && result.deferred_operations() == 0
    }));
    let selected = beauty_bug.external_results()[0].content();
    let curio = catalog
        .curios()
        .iter()
        .find(|curio| u64::from(curio.id().get()) == selected)
        .unwrap();
    let activity = execute(
        &compiled,
        beauty_bug.external_results()[0].payload(),
        Some(100),
        110_007,
        0,
        vec![
            ActivityOperation::AddInventory {
                inventory: compiled.curio_inventory(),
                content: selected,
                count: integer(1),
            },
            add_counter(
                compiled.curio_state_slot(),
                selected,
                i64::from(curio.initial_state().get()),
            ),
        ],
    );
    assert_eq!(
        inventory_count(
            &inventory_entries(&activity, compiled.curio_inventory()),
            selected
        ),
        0
    );
    assert!(matches!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            BEAUTY_BUG_UNLOCK_KEY
        ),
        0 | 1
    ));
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
                710,
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
        110_000,
    );
    if !seed.is_empty() {
        let program = ActivityProgramDefinition::new(program(100), seed).unwrap();
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
        .unwrap_or_else(|error| panic!("S10 outcome {} failed: {error:?}", outcome.get()));
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
