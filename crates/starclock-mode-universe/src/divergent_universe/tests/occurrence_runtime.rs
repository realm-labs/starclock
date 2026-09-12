#[test]
fn occurrence_catalog_compiles_exact_identity_variant_and_empty_choice_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.occurrence_runtime().expect("Occurrence runtime");
    assert_eq!(runtime.occurrences().len(), 118);
    assert_eq!(runtime.variants().len(), 97);
    assert!(runtime.choice_programs().is_empty());
    assert!(runtime.occurrences().iter().all(|value| {
        value.handbook_used()
            && value.variants().len() == 1
            && !value.unlock_rules().is_empty()
            && value.selection_policy() == "OwningDomainOrServiceBindingRequired"
            && value.unresolved_offer_behavior() == "FailClosed"
    }));
    assert!(runtime.variants().iter().all(|value| {
        !value.graph_path().is_empty()
            && value.graph_resolution() == "MissingAtPinnedRevision"
            && value.fallback() == "RejectWithoutMutation"
            && !value.entry_conditions().is_empty()
    }));
}

#[test]
fn every_occurrence_offer_and_missing_variant_graph_fails_closed_without_rng_or_mutation() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.occurrence_runtime().expect("Occurrence runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let activity = flow
        .start(instance(45), ActivityMasterSeed::from_u64(85))
        .expect("activity")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    for occurrence in runtime.occurrences() {
        assert_eq!(
            runtime.reject_unresolved_occurrence_offer(
                &activity,
                activity.state_hash(),
                occurrence.id(),
            ),
            Err(super::DivergentUniverseOccurrenceRuntimeError::UnresolvedOffer)
        );
    }
    for variant in runtime.variants() {
        let result = super::DivergentUniverseOccurrenceExternalResult::new(
            super::DivergentUniverseOccurrenceExternalResultKind::DialogueInteraction,
            "explicit-unavailable-result",
        )
        .expect("typed result");
        assert_eq!(
            runtime.submit_external_result(
                &activity,
                activity.state_hash(),
                variant.occurrence(),
                variant.id(),
                &result,
            ),
            Err(super::DivergentUniverseOccurrenceRuntimeError::ExternalResultUnavailable(
                super::DivergentUniverseOccurrenceExternalResultKind::DialogueInteraction,
            ))
        );
    }
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn occurrence_external_result_boundary_is_typed_and_mismatches_reject_first() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.occurrence_runtime().expect("Occurrence runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let activity = flow
        .start(instance(46), ActivityMasterSeed::from_u64(86))
        .expect("activity")
        .into_activity();
    let first = &runtime.variants()[0];
    let mismatched = runtime
        .occurrences()
        .iter()
        .find(|value| !first.occurrences().contains(value.id()))
        .expect("mismatched Occurrence");
    let result = super::DivergentUniverseOccurrenceExternalResult::new(
        super::DivergentUniverseOccurrenceExternalResultKind::Minigame,
        "explicit-minigame-result",
    )
    .expect("typed result");
    assert_eq!(result.result_id(), "explicit-minigame-result");
    assert_eq!(
        runtime.submit_external_result(
            &activity,
            activity.state_hash(),
            mismatched.id(),
            first.id(),
            &result,
        ),
        Err(super::DivergentUniverseOccurrenceRuntimeError::VariantMismatch)
    );
    assert_eq!(
        runtime.reject_unresolved_occurrence_offer(
            &activity,
            ActivityStateHash::new([9; 32]).expect("stale hash"),
            mismatched.id(),
        ),
        Err(super::DivergentUniverseOccurrenceRuntimeError::StaleStateHash)
    );
}
