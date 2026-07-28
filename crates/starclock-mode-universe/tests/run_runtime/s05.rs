use super::*;

const ATTEMPT_KEY: u64 = 0x5000_0000_0000_0000 | 13_501;
const REPEAT_KEY: u64 = 0x5100_0000_0000_0000 | 13_501;

#[test]
fn goal07_p4_m13_s05_executes_exact_history_curio_fragment_and_progressive_outcomes() {
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

    for (occurrence, variant, first, last) in [
        (3, 10201, 1_u8, 8_u8),
        (26, 11901, 4, 6),
        (27, 12001, 1, 2),
        (28, 12101, 1, 2),
        (29, 13301, 1, 2),
        (29, 13302, 1, 2),
        (30, 13501, 1, 2),
    ] {
        for index in first..=last {
            let key =
                format!("universe.occurrence.{occurrence}.variant.{variant}.choice.{index:02}");
            assert_eq!(choice(&key).deferred_operations(), 0, "{key}");
        }
    }

    let one_star = choice("universe.occurrence.3.variant.10201.choice.01");
    let two_star = choice("universe.occurrence.3.variant.10201.choice.02");
    let three_star = choice("universe.occurrence.3.variant.10201.choice.03");
    assert_eq!(one_star.random_candidate_count(), Some(8));
    assert_eq!(two_star.random_candidate_count(), Some(7));
    assert_eq!(three_star.random_candidate_count(), Some(3));
    assert_eq!(
        choice("universe.occurrence.3.variant.10201.choice.05").random_candidate_count(),
        Some(8)
    );

    let preservation = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.preservation")
        .unwrap()
        .id();
    let remembrance = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.remembrance")
        .unwrap()
        .id();
    let preservation_blessings = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.path() == preservation && blessing.rarity() == 1)
        .take(3)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect::<Vec<_>>();
    let remembrance_blessings = catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.path() == remembrance && blessing.rarity() == 1)
        .take(2)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect::<Vec<_>>();
    let mut seeded = preservation_blessings.clone();
    seeded.extend_from_slice(&remembrance_blessings);
    let history = execute_with_seed(
        &compiled,
        one_star.payload(),
        99_501,
        Some((ActivityRngLabel::Occurrence, 501, 8)),
        seeded
            .iter()
            .map(|content| ActivityOperation::AddInventory {
                inventory: compiled.blessing_inventory(),
                content: *content,
                count: integer(1),
            })
            .collect(),
        99_501,
    );
    let entries = inventory_entries(&history, compiled.blessing_inventory());
    assert!(
        preservation_blessings
            .iter()
            .all(|content| inventory_count(&entries, *content) == 2)
    );
    assert!(
        remembrance_blessings
            .iter()
            .all(|content| inventory_count(&entries, *content) == 1)
    );

    assert_eq!(
        choice("universe.occurrence.26.variant.11901.choice.05").random_candidate_count(),
        Some(3_843)
    );
    let reset = execute_occurrence_payload(
        &compiled,
        choice("universe.occurrence.26.variant.11901.choice.04").payload(),
        99_502,
    );
    assert_eq!(fragments(&reset, &compiled), 150);

    let bounty = choice("universe.occurrence.27.variant.12001.choice.01");
    assert_eq!(bounty.external_results().len(), 61);
    let bounty_curio = bounty.external_results()[0].content();
    let curio = catalog
        .curios()
        .iter()
        .find(|curio| u64::from(curio.id().get()) == bounty_curio)
        .unwrap();
    let bounty_result = execute_with_seed(
        &compiled,
        bounty.external_results()[0].payload(),
        99_503,
        None,
        vec![
            ActivityOperation::AddInventory {
                inventory: compiled.curio_inventory(),
                content: bounty_curio,
                count: integer(1),
            },
            ActivityOperation::AddCounter {
                slot: compiled.curio_state_slot(),
                key: bounty_curio,
                delta: integer(i64::from(curio.initial_state().get())),
            },
        ],
        99_503,
    );
    assert_eq!(fragments(&bounty_result, &compiled), 250);
    assert!(inventory_entries(&bounty_result, compiled.curio_inventory()).is_empty());

    let error_code = choice("universe.occurrence.28.variant.12101.choice.01");
    assert_eq!(error_code.external_results().len(), 6);
    let code_result = execute_occurrence_payload(
        &compiled,
        error_code.external_results()[0].payload(),
        99_504,
    );
    let code = error_code.external_results()[0].content();
    assert_eq!(
        inventory_count(
            &inventory_entries(&code_result, compiled.curio_inventory()),
            code,
        ),
        1
    );
    assert!(
        counter(&code_result, compiled.curio_state_slot(), code) > 0,
        "Error Code acquisition must initialize its repairing lifecycle state"
    );

    for variant in [13301, 13302] {
        let cowboys = execute_occurrence_payload_with_fragments(
            &compiled,
            choice(&format!(
                "universe.occurrence.29.variant.{variant}.choice.01"
            ))
            .payload(),
            99_505 + (variant - 13301),
            101,
        );
        assert_eq!(fragments(&cowboys, &compiled), 51);
    }

    let nildis = choice("universe.occurrence.30.variant.13501.choice.01");
    assert_eq!(nildis.random_candidate_count(), Some(16_200));
    assert_eq!(nildis.repeat_key(), Some(REPEAT_KEY));
    assert!(nildis.external_results().is_empty());
    let mut saw_reward = false;
    let mut saw_battle = false;
    let mut saw_blank = false;
    for purpose in 1..=128 {
        let result = execute_with_seed(
            &compiled,
            nildis.payload(),
            100_000 + u32::from(purpose),
            Some((ActivityRngLabel::Occurrence, purpose, 16_200)),
            vec![],
            u64::from(purpose),
        );
        let blessings = inventory_entries(&result, compiled.blessing_inventory()).len();
        let repeat = counter(
            &result,
            compiled.occurrence_interaction_state_slot(),
            REPEAT_KEY,
        );
        match (blessings, repeat) {
            (1, 1) => saw_reward = true,
            (0, 0) => saw_battle = true,
            (0, 1) => saw_blank = true,
            state => panic!("unexpected Nildis first-draw state {state:?}"),
        }
        if saw_reward && saw_battle && saw_blank {
            break;
        }
    }
    assert!(
        saw_reward && saw_battle && saw_blank,
        "reward={saw_reward} battle={saw_battle} blank={saw_blank}"
    );

    let guaranteed_battle = execute_with_seed(
        &compiled,
        nildis.payload(),
        99_506,
        Some((ActivityRngLabel::Occurrence, 506, 16_200)),
        vec![ActivityOperation::AddCounter {
            slot: compiled.occurrence_interaction_state_slot(),
            key: ATTEMPT_KEY,
            delta: integer(3),
        }],
        99_506,
    );
    assert_eq!(
        counter(
            &guaranteed_battle,
            compiled.occurrence_interaction_state_slot(),
            ATTEMPT_KEY
        ),
        0
    );
    assert_eq!(
        counter(
            &guaranteed_battle,
            compiled.occurrence_interaction_state_slot(),
            REPEAT_KEY
        ),
        0
    );

    let give_up = choice("universe.occurrence.30.variant.13501.choice.02");
    let reset_attempt = execute_with_seed(
        &compiled,
        give_up.payload(),
        99_507,
        None,
        vec![ActivityOperation::AddCounter {
            slot: compiled.occurrence_interaction_state_slot(),
            key: ATTEMPT_KEY,
            delta: integer(2),
        }],
        99_507,
    );
    assert_eq!(
        counter(
            &reset_attempt,
            compiled.occurrence_interaction_state_slot(),
            ATTEMPT_KEY
        ),
        0
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
        let seed = ActivityProgramDefinition::new(program(95), seed).unwrap();
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
