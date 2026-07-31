use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use super::{
    GOLD_AND_GEARS_OCCURRENCE_EXECUTION_REVISION, GoldAndGearsEntryError,
    GoldAndGearsOccurrenceEffectPhase, GoldAndGearsOccurrenceRuleAccuracy,
    GoldAndGearsOccurrenceRuleKind, GoldAndGearsOccurrenceRuleOwnership,
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance, state_layout::DEFERRED_EFFECTS_SLOT,
    tests::compiled_fixture,
};

#[test]
fn occurrence_partition_binds_exactly_384_terminal_rules() {
    let factory = factory();
    let bindings = factory.occurrence_rule_bindings();
    assert_eq!(bindings.len(), 384);
    assert!(
        bindings
            .windows(2)
            .all(|pair| pair[0].rule_id() < pair[1].rule_id())
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.kind() == GoldAndGearsOccurrenceRuleKind::Occurrence)
            .count(),
        62
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.kind() == GoldAndGearsOccurrenceRuleKind::Variant)
            .count(),
        65
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.kind() == GoldAndGearsOccurrenceRuleKind::Choice)
            .count(),
        257
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.ownership() == GoldAndGearsOccurrenceRuleOwnership::Shared
            })
            .count(),
        51
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.accuracy() == GoldAndGearsOccurrenceRuleAccuracy::ExactPublic
            })
            .count(),
        341
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.accuracy() == GoldAndGearsOccurrenceRuleAccuracy::VersionedProjectPolicy
            })
            .count(),
        43
    );
    assert!(bindings.iter().all(|binding| {
        binding.executor()
            == if binding.ownership() == GoldAndGearsOccurrenceRuleOwnership::Shared {
                "ReleasedSharedExecutor"
            } else {
                "ActivityProgram"
            }
    }));
    assert_eq!(
        GOLD_AND_GEARS_OCCURRENCE_EXECUTION_REVISION,
        "gold-and-gears-occurrence-execution-v1"
    );
    assert_eq!(
        digest_hex(factory.occurrence_execution_digest()),
        "eafc03c0952a6665ddee9523a2ef28c6fc9f0ce794fa4e5f16bdc71be7eac984"
    );
}

#[test]
fn all_384_occurrence_rules_execute_through_the_production_fixture() {
    let instance = compiled_fixture(factory());
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = activity_rng(&instance, 14_506);
    let mut effects = 0;
    let mut policy_choices = 0;

    for choice in instance.occurrence_choices() {
        let selection = if choice.outcome().uses_seeded_uniform_policy() {
            policy_choices += 1;
            let base = u64::from(choice.id().get()) * 10;
            Some(
                instance
                    .select_occurrence_candidates(
                        choice.id(),
                        &[base + 3, base + 1, base + 2],
                        1,
                        &mut rng,
                    )
                    .unwrap(),
            )
        } else {
            None
        };
        let plan = instance
            .compile_occurrence_choice_execution(choice.id(), selection.as_ref())
            .unwrap();
        assert_eq!(plan.choice(), choice.id());
        assert_eq!(plan.effects().len(), choice.costs().len() + 1);
        assert_eq!(
            plan.effects().last().unwrap().phase(),
            GoldAndGearsOccurrenceEffectPhase::Outcome
        );
        effects += plan.effects().len();
        commit(&instance, &mut state, plan.program().clone());
    }

    assert_eq!(policy_choices, 43);
    assert_eq!(effects, 312);
    assert_eq!(state.command_sequence(), 257);
    assert_eq!(deferred_entry_count(&state), 612);
    assert_eq!(
        occurrence_draws(&rng),
        43,
        "one stable uniform draw per policy choice"
    );
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "b973dc92aaf0dd568b028ba5923493f0b5352d8ae91ef864dce95d14cfbda615"
    );
}

#[test]
fn occurrence_choice_execution_preserves_authored_effect_order_and_payloads() {
    let instance = compiled_fixture(factory());
    for choice in instance.occurrence_choices() {
        let selection = choice.outcome().uses_seeded_uniform_policy().then(|| {
            let mut rng = activity_rng(&instance, u64::from(choice.id().get()));
            instance
                .select_occurrence_candidates(choice.id(), &[7], 1, &mut rng)
                .unwrap()
        });
        let plan = instance
            .compile_occurrence_choice_execution(choice.id(), selection.as_ref())
            .unwrap();
        for (effect, cost) in plan.effects().iter().zip(choice.costs()) {
            assert_eq!(effect.phase(), GoldAndGearsOccurrenceEffectPhase::Cost);
            assert_eq!(effect.operations(), [cost.operation()]);
            assert_eq!(effect.targets(), cost.targets());
            assert_eq!(effect.numeric_literals(), cost.numeric_literals());
            assert_eq!(effect.parameter_refs(), cost.parameter_refs());
            assert!(effect.chance_percentages().is_empty());
        }
        let outcome = plan.effects().last().unwrap();
        assert_eq!(outcome.operations(), choice.outcome().operations());
        assert_eq!(outcome.targets(), choice.outcome().targets());
        assert_eq!(
            outcome.numeric_literals(),
            choice.outcome().numeric_literals()
        );
        assert_eq!(outcome.parameter_refs(), choice.outcome().parameter_refs());
        assert_eq!(
            outcome.chance_percentages(),
            choice.outcome().chance_percentages()
        );
    }
}

#[test]
fn occurrence_selection_and_duplicate_execution_fail_without_state_or_rng_change() {
    let instance = compiled_fixture(factory());
    let random = instance
        .occurrence_choices()
        .iter()
        .find(|choice| choice.outcome().uses_seeded_uniform_policy())
        .unwrap();
    let deterministic = instance
        .occurrence_choices()
        .iter()
        .find(|choice| !choice.outcome().uses_seeded_uniform_policy())
        .unwrap();
    assert_eq!(
        instance.compile_occurrence_choice_execution(random.id(), None),
        Err(GoldAndGearsEntryError::InvalidOccurrenceSelection(
            random.id()
        ))
    );

    let mut rng = activity_rng(&instance, 14_506);
    let before_empty = rng.snapshots();
    let empty = instance
        .select_occurrence_candidates(random.id(), &[], 1, &mut rng)
        .unwrap();
    assert!(empty.selected().is_empty());
    assert_eq!(rng.snapshots(), before_empty);
    assert_eq!(
        instance.compile_occurrence_choice_execution(random.id(), Some(&empty)),
        Err(GoldAndGearsEntryError::InvalidOccurrenceSelection(
            random.id()
        ))
    );

    let selection = instance
        .select_occurrence_candidates(random.id(), &[3, 1, 2], 1, &mut rng)
        .unwrap();
    assert_eq!(
        instance.compile_occurrence_choice_execution(deterministic.id(), Some(&selection)),
        Err(GoldAndGearsEntryError::InvalidOccurrenceSelection(
            deterministic.id()
        ))
    );
    let plan = instance
        .compile_occurrence_choice_execution(random.id(), Some(&selection))
        .unwrap();
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    commit(&instance, &mut state, plan.program().clone());
    let before_state = state_bytes(&instance, &state, &rng);
    let before_rng = rng.snapshots();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        plan.program().id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(plan.program(), cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before_state);
    assert_eq!(rng.snapshots(), before_rng);
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    super::tests::shared_factory()
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn deferred_entry_count(state: &ActivityTransactionState) -> usize {
    match state
        .slot(ActivitySlotId::new(DEFERRED_EFFECTS_SLOT).unwrap())
        .unwrap()
    {
        ActivityValue::BoundedCounterMap(values) => values.len(),
        _ => panic!("deferred-effects slot changed kind"),
    }
}

fn occurrence_draws(rng: &ActivityRngStreams) -> u64 {
    rng.snapshots()
        .iter()
        .find(|snapshot| snapshot.label() == ActivityRngLabel::Occurrence)
        .unwrap()
        .draw_count()
}

fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = identity();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn state_bytes(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> Box<[u8]> {
    state.canonical_state_bytes(
        identity(),
        instance.graph_definition(),
        ActivityInstanceId::new(1).unwrap(),
        rng,
    )
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
}

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn state_hash(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> String {
    digest_hex(
        state
            .state_hash(
                identity(),
                instance.graph_definition(),
                ActivityInstanceId::new(1).unwrap(),
                rng,
            )
            .bytes(),
    )
}
