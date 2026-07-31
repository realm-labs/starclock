use super::*;

const BEAUTY_BUG_UNLOCK_KEY: u64 = 0x5d00_0000_0000_0001;
const DANCER_STAGE_KEY: u64 = 0x5e00_0000_0000_0001;
const DANCER_REPEAT_KEY: u64 = 0x5e00_0000_0000_0002;
const MIRROR_CANDLE_KEY: u64 = 0x5f00_0000_0000_0001;
const MIRROR_WISH_KEY: u64 = 0x5f00_0000_0000_0002;
const MIRROR_REPEAT_KEY: u64 = 0x5f00_0000_0000_0004;

#[test]
fn goal07_p4_m13_s11_executes_beauty_trash_shopping_dancer_and_mirror_outcomes() {
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

    let s11_keys = [
        "universe.occurrence.54.variant.14401.choice.02",
        "universe.occurrence.54.variant.14401.choice.03",
        "universe.occurrence.54.variant.14401.choice.04",
        "universe.occurrence.54.variant.14401.choice.05",
        "universe.occurrence.54.variant.14401.choice.06",
        "universe.occurrence.55.variant.14401.choice.01",
        "universe.occurrence.55.variant.14401.choice.02",
        "universe.occurrence.55.variant.14401.choice.03",
        "universe.occurrence.55.variant.14401.choice.04",
        "universe.occurrence.55.variant.14401.choice.05",
        "universe.occurrence.55.variant.14401.choice.06",
        "universe.occurrence.56.variant.14501.choice.01",
        "universe.occurrence.56.variant.14501.choice.02",
        "universe.occurrence.6.variant.10401.choice.01",
        "universe.occurrence.6.variant.10401.choice.02",
        "universe.occurrence.6.variant.10401.choice.03",
        "universe.occurrence.6.variant.10401.choice.04",
        "universe.occurrence.60.variant.19301.choice.01",
        "universe.occurrence.60.variant.19301.choice.02",
        "universe.occurrence.62.variant.19501.choice.01",
        "universe.occurrence.62.variant.19501.choice.02",
        "universe.occurrence.62.variant.19501.choice.03",
    ];
    let deferred = s11_keys
        .iter()
        .filter_map(|key| {
            let compiled = choice(key);
            (compiled.deferred_operations() != 0).then_some((*key, compiled.deferred_operations()))
        })
        .collect::<Vec<_>>();
    assert!(deferred.is_empty(), "deferred S11 choices: {deferred:?}");

    let feed_blessing = choice("universe.occurrence.54.variant.14401.choice.02");
    assert_eq!(
        feed_blessing.external_results().len(),
        catalog.blessings().len()
    );
    let selected_blessing = feed_blessing.external_results()[0].content();
    let activity = execute(
        &compiled,
        feed_blessing.external_results()[0].payload(),
        Some(100),
        120_001,
        0,
        vec![ActivityOperation::AddInventory {
            inventory: compiled.blessing_inventory(),
            content: selected_blessing,
            count: integer(1),
        }],
    );
    assert_eq!(
        inventory_count(
            &inventory_entries(&activity, compiled.blessing_inventory()),
            selected_blessing
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

    let feed_fragments = choice("universe.occurrence.54.variant.14401.choice.03");
    let activity = execute(
        &compiled,
        feed_fragments.payload(),
        feed_fragments.random_candidate_count(),
        120_002,
        150,
        vec![],
    );
    assert_eq!(fragments(&activity, &compiled), 50);

    let heartfelt = choice("universe.occurrence.54.variant.14401.choice.04");
    let positive_curios = catalog
        .curios()
        .iter()
        .filter(|curio| {
            curio
                .pool_tags()
                .iter()
                .any(|tag| tag.as_ref() == "polarity:positive")
        })
        .count();
    assert_eq!(
        heartfelt.random_candidate_count(),
        u32::try_from(positive_curios).ok()
    );
    let activity = execute(
        &compiled,
        heartfelt.payload(),
        heartfelt.random_candidate_count(),
        120_003,
        0,
        vec![],
    );
    assert_eq!(
        inventory_entries(&activity, compiled.curio_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        5
    );

    let life_favor = choice("universe.occurrence.54.variant.14401.choice.05");
    let three_star = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == 3)
        .count();
    assert_eq!(life_favor.external_results().len(), three_star);
    let selected_blessing = life_favor.external_results()[0].content();
    let activity = execute(
        &compiled,
        life_favor.external_results()[0].payload(),
        None,
        120_004,
        0,
        vec![],
    );
    assert_eq!(
        inventory_count(
            &inventory_entries(&activity, compiled.blessing_inventory()),
            selected_blessing
        ),
        1
    );

    let trash = choice("universe.occurrence.56.variant.14501.choice.01");
    assert_eq!(trash.external_results().len(), catalog.curios().len());
    let selected_curio = trash.external_results()[0].content();
    let selected_definition = catalog
        .curios()
        .iter()
        .find(|curio| u64::from(curio.id().get()) == selected_curio)
        .unwrap();
    let activity = execute(
        &compiled,
        trash.external_results()[0].payload(),
        trash.external_results()[0].random_candidate_count(),
        120_005,
        0,
        vec![
            ActivityOperation::AddInventory {
                inventory: compiled.curio_inventory(),
                content: selected_curio,
                count: integer(1),
            },
            add_counter(
                compiled.curio_state_slot(),
                selected_curio,
                i64::from(selected_definition.initial_state().get()),
            ),
        ],
    );
    assert_eq!(
        inventory_entries(&activity, compiled.curio_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        2
    );

    let doughnuts = choice("universe.occurrence.6.variant.10401.choice.01");
    assert_eq!(doughnuts.random_candidate_count(), Some(100));
    execute(
        &compiled,
        doughnuts.payload(),
        doughnuts.random_candidate_count(),
        120_006,
        0,
        vec![],
    );

    let lotus = choice("universe.occurrence.6.variant.10401.choice.02");
    assert_eq!(lotus.random_candidate_count(), Some(50_400));
    let one_star = catalog
        .blessings()
        .iter()
        .find(|blessing| blessing.rarity() == 1)
        .unwrap();
    let two_star = catalog
        .blessings()
        .iter()
        .find(|blessing| blessing.rarity() == 2)
        .unwrap();
    let activity = execute(
        &compiled,
        lotus.payload(),
        lotus.random_candidate_count(),
        120_007,
        0,
        vec![
            ActivityOperation::AddInventory {
                inventory: compiled.blessing_inventory(),
                content: u64::from(one_star.id().get()),
                count: integer(1),
            },
            ActivityOperation::AddInventory {
                inventory: compiled.blessing_inventory(),
                content: u64::from(two_star.id().get()),
                count: integer(1),
            },
        ],
    );
    assert_eq!(
        inventory_entries(&activity, compiled.blessing_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        2
    );

    let mechanical_box = choice("universe.occurrence.6.variant.10401.choice.03");
    assert_eq!(mechanical_box.random_candidate_count(), Some(65_000));
    let activity = execute(
        &compiled,
        mechanical_box.payload(),
        mechanical_box.random_candidate_count(),
        120_008,
        0,
        vec![],
    );
    assert_eq!(
        inventory_entries(&activity, compiled.curio_inventory())
            .iter()
            .map(|entry| entry.1)
            .sum::<u32>(),
        1
    );

    let dancer = choice("universe.occurrence.60.variant.19301.choice.01");
    assert_eq!(dancer.external_results().len(), three_star);
    assert_eq!(dancer.repeat_key(), Some(DANCER_REPEAT_KEY));
    let selected_blessing = dancer.external_results()[0].content();
    let activity = execute(
        &compiled,
        dancer.external_results()[0].payload(),
        Some(100),
        120_009,
        50,
        vec![add_counter(
            compiled.occurrence_interaction_state_slot(),
            DANCER_STAGE_KEY,
            2,
        )],
    );
    assert_eq!(fragments(&activity, &compiled), 0);
    assert_eq!(
        inventory_count(
            &inventory_entries(&activity, compiled.blessing_inventory()),
            selected_blessing
        ),
        1
    );
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            DANCER_STAGE_KEY
        ),
        0
    );

    let mirror_light = choice("universe.occurrence.62.variant.19501.choice.01");
    assert_eq!(mirror_light.repeat_key(), Some(MIRROR_REPEAT_KEY));
    let activity = execute(
        &compiled,
        mirror_light.payload(),
        mirror_light.random_candidate_count(),
        120_010,
        0,
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
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            MIRROR_REPEAT_KEY
        ),
        1
    );

    let mirror_wish = choice("universe.occurrence.62.variant.19501.choice.03");
    assert_eq!(mirror_wish.random_candidate_count(), Some(64_800));
    let blessing_seed =
        catalog
            .blessings()
            .iter()
            .take(3)
            .map(|blessing| ActivityOperation::AddInventory {
                inventory: compiled.blessing_inventory(),
                content: u64::from(blessing.id().get()),
                count: integer(1),
            });
    let activity = execute(
        &compiled,
        mirror_wish.payload(),
        mirror_wish.random_candidate_count(),
        120_011,
        0,
        std::iter::once(add_counter(
            compiled.occurrence_interaction_state_slot(),
            MIRROR_CANDLE_KEY,
            1,
        ))
        .chain(blessing_seed)
        .collect(),
    );
    assert_eq!(
        counter(
            &activity,
            compiled.occurrence_interaction_state_slot(),
            MIRROR_WISH_KEY
        ),
        1
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
                711,
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
        120_000,
    );
    if !seed.is_empty() {
        let program = ActivityProgramDefinition::new(program(101), seed).unwrap();
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
        .unwrap_or_else(|error| panic!("S11 outcome {} failed: {error:?}", outcome.get()));
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
