#[test]
fn service_adventure_catalog_compiles_exact_shapes_and_policy_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .service_adventure_runtime()
        .expect("service/Adventure runtime");
    assert_eq!(runtime.services().len(), 23);
    assert_eq!(runtime.offers().len(), 161);
    assert_eq!(runtime.adventures().len(), 32);
    assert!(runtime.services().iter().all(|value| {
        value.service_kind() == "UnclassifiedMissingGraph"
            && value.graph_resolution() == "MissingAtPinnedRevision"
            && value.fallback() == "RejectWithoutMutation"
            && !value.graph_path().is_empty()
    }));
    assert_eq!(
        runtime
            .offers()
            .iter()
            .filter(|value| value.fallback() == "LeaveWithoutMutation")
            .count(),
        29
    );
    assert_eq!(
        runtime
            .adventures()
            .iter()
            .map(|value| value.parameter_rows())
            .sum::<usize>(),
        30
    );
}

#[test]
fn every_missing_service_and_empty_offer_fallback_preserves_state_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .service_adventure_runtime()
        .expect("service/Adventure runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let activity = flow
        .start(instance(47), ActivityMasterSeed::from_u64(87))
        .expect("activity")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    for service in runtime.services() {
        assert!(matches!(
            runtime.reject_missing_service_graph(&activity, activity.state_hash(), service.id()),
            Err(super::DivergentUniverseServiceAdventureError::MissingServiceGraph)
        ));
    }
    for offer in runtime.offers() {
        let resolution =
            runtime.resolve_empty_offer_fallback(&activity, activity.state_hash(), offer.id());
        if offer.fallback() == "LeaveWithoutMutation" {
            assert!(resolution.is_ok());
        } else {
            assert!(matches!(
                resolution,
                Err(super::DivergentUniverseServiceAdventureError::EmptyOfferRejected)
            ));
        }
    }
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn all_adventure_shapes_settle_typed_external_results_once_without_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .service_adventure_runtime()
        .expect("service/Adventure runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let mut activity = flow
        .start(instance(48), ActivityMasterSeed::from_u64(88))
        .expect("activity")
        .into_activity();
    let draws = reward_draws(&activity);
    for adventure in runtime.adventures() {
        let result = accepted_adventure_result(adventure);
        let hash = activity.state_hash();
        let resolution = runtime
            .settle_external_adventure_result(&mut activity, hash, adventure.id(), result)
            .expect("settlement");
        assert!((1..=3).contains(&resolution.tier()));
        assert!(!resolution.events().is_empty());
        assert_eq!(resolution.state_hash(), activity.state_hash());
        assert!(matches!(
            runtime.settle_external_adventure_result(
                &mut activity,
                resolution.state_hash(),
                adventure.id(),
                result,
            ),
            Err(super::DivergentUniverseServiceAdventureError::AdventureAlreadySettled)
        ));
    }
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn adventure_result_kind_and_stale_hash_rejections_are_atomic() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory
        .service_adventure_runtime()
        .expect("service/Adventure runtime");
    let flow = factory.compile(entry("401", "3011")).expect("flow");
    let mut activity = flow
        .start(instance(49), ActivityMasterSeed::from_u64(89))
        .expect("activity")
        .into_activity();
    let score = runtime
        .adventures()
        .iter()
        .find(|value| {
            value.settlement_kind()
                == super::DivergentUniverseAdventureSettlementKind::ScoreThreshold
        })
        .expect("score Adventure");
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert!(matches!(
        runtime.settle_external_adventure_result(
            &mut activity,
            hash,
            score.id(),
            super::DivergentUniverseAdventureExternalResult::AcceptedOpaque,
        ),
        Err(super::DivergentUniverseServiceAdventureError::ExternalResultKindMismatch)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
}

fn accepted_adventure_result(
    adventure: &super::DivergentUniverseAdventureRuntimeDefinition,
) -> super::DivergentUniverseAdventureExternalResult {
    match adventure.settlement_kind() {
        super::DivergentUniverseAdventureSettlementKind::RewardTier => {
            super::DivergentUniverseAdventureExternalResult::RewardTier(
                super::DivergentUniverseAdventureRewardTier::High,
            )
        }
        super::DivergentUniverseAdventureSettlementKind::ScoreThreshold
        | super::DivergentUniverseAdventureSettlementKind::RoundScore => {
            super::DivergentUniverseAdventureExternalResult::Score(
                *adventure.thresholds().last().expect("score threshold"),
            )
        }
        super::DivergentUniverseAdventureSettlementKind::RoundThreshold => {
            super::DivergentUniverseAdventureExternalResult::Rounds(
                u16::try_from(*adventure.thresholds().last().expect("round threshold"))
                    .expect("round threshold fits u16"),
            )
        }
        super::DivergentUniverseAdventureSettlementKind::ExternalResult => {
            super::DivergentUniverseAdventureExternalResult::AcceptedOpaque
        }
    }
}
