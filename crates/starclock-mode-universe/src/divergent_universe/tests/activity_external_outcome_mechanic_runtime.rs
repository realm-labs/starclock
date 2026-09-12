#[test]
fn activity_external_outcome_mechanic_binds_exact_source_shape() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .activity_external_outcome_mechanic_runtime()
        .expect("external outcome mechanic runtime");
    let definition = runtime.definition();
    assert_eq!(
        definition.id().as_str(),
        "divergent-universe.mechanic-rule.config-configadventuremodifier-adventuremodifier-rogue-tourn1-json"
    );
    assert_eq!(
        definition.source_path(),
        "Config/ConfigAdventureModifier/AdventureModifier_Rogue_Tourn1.json"
    );
    assert_eq!(definition.source_sha256().len(), 64);
    assert_eq!(definition.operations().len(), 2);
    assert_eq!(
        definition.operations()[0].operation_type(),
        "RPG.GameCore.SetDynamicValueByCustomName"
    );
    assert_eq!(definition.operations()[0].source_occurrences(), 9);
    assert_eq!(definition.operations()[1].operation_type(), "None");
    assert_eq!(definition.operations()[1].source_occurrences(), 9);
    assert_eq!(
        runtime.digest(),
        factory
            .activity_external_outcome_mechanic_runtime()
            .expect("fresh external outcome mechanic runtime")
            .digest()
    );
}

#[test]
fn activity_external_outcome_mechanic_settles_typed_receipt_once_without_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .activity_external_outcome_mechanic_runtime()
        .expect("external outcome mechanic runtime");
    let service = factory
        .service_adventure_runtime()
        .expect("service/Adventure runtime");
    let adventure = service
        .adventures()
        .iter()
        .find(|value| {
            value.settlement_kind()
                == super::DivergentUniverseAdventureSettlementKind::ExternalResult
        })
        .expect("opaque Adventure result");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let mut activity = flow
        .start(instance(86), ActivityMasterSeed::from_u64(126))
        .expect("activity")
        .into_activity();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    let resolution = runtime
        .settle(
            &mut activity,
            hash,
            runtime.definition().id(),
            adventure.id(),
            super::DivergentUniverseAdventureExternalResult::AcceptedOpaque,
        )
        .expect("typed external settlement");
    assert_eq!(resolution.mechanic_digest(), runtime.definition().digest());
    assert_eq!(resolution.tier(), 1);
    assert!(!resolution.events().is_empty());
    assert_eq!(resolution.state_hash(), activity.state_hash());
    assert_eq!(reward_draws(&activity), draws);
    let before_duplicate = activity.canonical_state_bytes();
    assert!(matches!(
        runtime.settle(
            &mut activity,
            resolution.state_hash(),
            runtime.definition().id(),
            adventure.id(),
            super::DivergentUniverseAdventureExternalResult::AcceptedOpaque,
        ),
        Err(super::DivergentUniverseExternalOutcomeMechanicError::Service(
            super::DivergentUniverseServiceAdventureError::AdventureAlreadySettled
        ))
    ));
    assert_eq!(activity.canonical_state_bytes(), before_duplicate);
}

#[test]
fn activity_external_outcome_mechanic_rejections_are_atomic() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .activity_external_outcome_mechanic_runtime()
        .expect("external outcome mechanic runtime");
    let service = factory
        .service_adventure_runtime()
        .expect("service/Adventure runtime");
    let score = service
        .adventures()
        .iter()
        .find(|value| {
            value.settlement_kind()
                == super::DivergentUniverseAdventureSettlementKind::ScoreThreshold
        })
        .expect("score Adventure result");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let mut activity = flow
        .start(instance(87), ActivityMasterSeed::from_u64(127))
        .expect("activity")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    assert!(matches!(
        runtime.settle(
            &mut activity,
            hash,
            runtime.definition().id(),
            score.id(),
            super::DivergentUniverseAdventureExternalResult::AcceptedOpaque,
        ),
        Err(super::DivergentUniverseExternalOutcomeMechanicError::Service(
            super::DivergentUniverseServiceAdventureError::ExternalResultKindMismatch
        ))
    ));
    let unknown = starclock_data::divergent_universe_mechanic_catalog::DivergentUniverseMechanicRuleId::new(
        "divergent-universe.mechanic-rule.unknown".to_owned(),
    )
    .expect("unknown mechanic ID");
    let hash = activity.state_hash();
    assert!(matches!(
        runtime.settle(
            &mut activity,
            hash,
            &unknown,
            score.id(),
            accepted_adventure_result(score),
        ),
        Err(super::DivergentUniverseExternalOutcomeMechanicError::UnknownMechanic)
    ));
    let stale = ActivityStateHash::new([0x7c; 32]).expect("stale hash");
    assert!(matches!(
        runtime.settle(
            &mut activity,
            stale,
            runtime.definition().id(),
            score.id(),
            accepted_adventure_result(score),
        ),
        Err(super::DivergentUniverseExternalOutcomeMechanicError::Service(
            super::DivergentUniverseServiceAdventureError::StaleStateHash
        ))
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}
