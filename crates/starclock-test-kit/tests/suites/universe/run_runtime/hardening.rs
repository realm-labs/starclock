use super::*;

#[test]
fn hundred_fragment_cycles_preserve_bounds_and_rejected_spends_roll_back() {
    const CYCLES: u64 = 100;

    let slot = ActivitySlotId::new(1).unwrap();
    let definition = ActivityStateDefinition::new(
        vec![
            ActivitySlotDefinition::new_with_policy(
                slot,
                ActivityScope::Activity,
                ActivityValue::BoundedInteger(0),
                Some((0, MAX_COSMIC_FRAGMENTS)),
                None,
                vec![SlotResetPoint::ActivityStart],
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(1).unwrap(),
            )
            .unwrap(),
        ],
        vec![],
        vec![],
    )
    .unwrap();
    let graph = graph();
    let mut state = ActivityTransactionState::new(definition.clone(), node(1));
    let credit = ActivityProgramDefinition::new(
        program(1),
        RunRuntimeCatalog::credit_fragments(slot, CosmicFragments::new(120).unwrap()).into_vec(),
    )
    .unwrap();
    let spend = ActivityProgramDefinition::new(
        program(2),
        RunRuntimeCatalog::spend_fragments(slot, CosmicFragments::new(120).unwrap()).into_vec(),
    )
    .unwrap();
    let overdraw = ActivityProgramDefinition::new(
        program(3),
        RunRuntimeCatalog::spend_fragments(slot, CosmicFragments::new(1).unwrap()).into_vec(),
    )
    .unwrap();
    for program in [&credit, &spend, &overdraw] {
        program.validate_against(&definition, &graph).unwrap();
    }

    for cycle in 0..CYCLES {
        let sequence = cycle * 2;
        commit(&mut state, &credit, sequence + 1, &graph);
        assert_eq!(state.slot(slot), Some(&ActivityValue::BoundedInteger(120)));
        commit(&mut state, &spend, sequence + 2, &graph);
        assert_eq!(state.slot(slot), Some(&ActivityValue::BoundedInteger(0)));
        assert_eq!(
            state.apply_program(&overdraw, cause(sequence + 3, 3), &graph),
            ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::ConditionNotSatisfied
            )
        );
        assert_eq!(
            state.slot(slot),
            Some(&ActivityValue::BoundedInteger(0)),
            "cycle {cycle} rejected spend mutated fragments"
        );
    }
}
