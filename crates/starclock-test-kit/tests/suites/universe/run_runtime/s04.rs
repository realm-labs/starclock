use super::*;

#[test]
fn goal07_p4_m13_s04_executes_exact_societal_saleo_and_cosmic_outcomes() {
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

    let assigned = [
        (22, 3_u8, 9_u8),
        (23, 1, 2),
        (24, 1, 6),
        (25, 1, 6),
        (26, 1, 3),
    ];
    for (occurrence, first, last) in assigned {
        for index in first..=last {
            let variant = if occurrence == 22 {
                11701
            } else if occurrence == 23 {
                11801
            } else {
                11901
            };
            let key =
                format!("universe.occurrence.{occurrence}.variant.{variant}.choice.{index:02}");
            assert_eq!(choice(&key).deferred_operations(), 0, "{key}");
        }
    }

    let cosmic_curio = choice("universe.occurrence.22.variant.11701.choice.04");
    assert_eq!(cosmic_curio.external_results().len(), 46);
    let activity = execute_occurrence_payload_with_fragments(
        &compiled,
        cosmic_curio.external_results()[0].payload(),
        99_401,
        500,
    );
    assert_eq!(
        activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(400))
    );
    assert_eq!(
        choice("universe.occurrence.22.variant.11701.choice.05")
            .external_results()
            .len(),
        162
    );
    assert_eq!(
        choice("universe.occurrence.22.variant.11701.choice.07").random_candidate_count(),
        Some(162)
    );
    assert_eq!(
        choice("universe.occurrence.22.variant.11701.choice.08")
            .external_results()
            .len(),
        27
    );

    let dreamscape = choice("universe.occurrence.23.variant.11801.choice.01");
    assert_eq!(dreamscape.external_results().len(), 15);
    let activity = execute_occurrence_payload(
        &compiled,
        dreamscape.external_results()[0].payload(),
        99_402,
    );
    let player = activity.player_view();
    assert_eq!(
        player
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(350))
    );
    assert_eq!(
        player
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == compiled.curio_inventory())
            .unwrap()
            .entries()
            .len(),
        1
    );
    let activity = execute_occurrence_payload(
        &compiled,
        choice("universe.occurrence.23.variant.11801.choice.02").payload(),
        99_403,
    );
    assert_eq!(
        activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(150))
    );

    let sal = choice("universe.occurrence.24.variant.11901.choice.01");
    assert_eq!(sal.external_results().len(), 46);
    let leo = choice("universe.occurrence.24.variant.11901.choice.02");
    assert_eq!(leo.random_candidate_count(), Some(3_843));
    let seeded = catalog
        .curios()
        .iter()
        .find(|curio| curio.stable_key() == "universe.curio.6")
        .unwrap();
    let activity = execute_occurrence_payload_with_seeded_curio(
        &compiled,
        leo.payload(),
        99_404,
        u64::from(seeded.id().get()),
        u64::from(seeded.initial_state().get()),
    );
    let player = activity.player_view();
    assert!(
        player
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == compiled.curio_inventory())
            .unwrap()
            .entries()
            .is_empty()
    );
    assert_eq!(
        player
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == compiled.blessing_inventory())
            .unwrap()
            .entries()
            .len(),
        1
    );
    let activity = execute_occurrence_payload(
        &compiled,
        choice("universe.occurrence.24.variant.11901.choice.04").payload(),
        99_405,
    );
    assert_eq!(
        activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(150))
    );
}

fn execute_occurrence_payload_with_seeded_curio(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    outcome: u32,
    curio: u64,
    initial_state: u64,
) -> GraphActivity {
    let outcome = ActivityExternalOutcomeId::new(u64::from(outcome)).unwrap();
    let binding = ActivityInteractionBinding::new(
        node(1),
        outcome,
        starclock_activity::ActivityHandlerId::new(OCCURRENCE_INTERACTION_HANDLER_ID).unwrap(),
        payload.to_vec(),
        "standard-universe.occurrence-choice.v2",
    )
    .unwrap();
    let registry = compiled
        .runtime_definition()
        .interactions()
        .unwrap()
        .registry();
    let mut activity = occurrence_harness(compiled, &binding, registry);
    let seed = ActivityProgramDefinition::new(
        program(90),
        vec![
            ActivityOperation::AddInventory {
                inventory: compiled.curio_inventory(),
                content: curio,
                count: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
            },
            ActivityOperation::AddCounter {
                slot: compiled.curio_state_slot(),
                key: curio,
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(
                    i64::try_from(initial_state).unwrap(),
                )),
            },
        ],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.player_view().state_hash(), &seed)
        .unwrap();
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
