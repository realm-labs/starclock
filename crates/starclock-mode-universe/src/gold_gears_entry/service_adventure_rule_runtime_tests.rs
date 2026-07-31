use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId, ActivityMasterSeed,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use super::{
    GOLD_AND_GEARS_SERVICE_ADVENTURE_EXECUTION_REVISION, GoldAndGearsAdventureExternalOutcome,
    GoldAndGearsEntryError, GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    GoldAndGearsServiceAdventureRuleAccuracy, GoldAndGearsServiceAdventureRuleKind,
    GoldAndGearsServiceKind, GoldAndGearsServiceOfferSelector,
    state_layout::{DEFERRED_EFFECTS_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY, RUN_RESOURCES_SLOT},
    tests::compiled_fixture,
};

const FIXTURE_FUNDING: i64 = 100_000;

#[test]
fn service_adventure_partition_binds_exactly_38_shared_rules() {
    let factory = factory();
    let bindings = factory.service_adventure_rule_bindings();
    assert_eq!(bindings.len(), 38);
    assert!(
        bindings
            .windows(2)
            .all(|pair| pair[0].rule_id() < pair[1].rule_id())
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.kind() == GoldAndGearsServiceAdventureRuleKind::AdventureOutcome
            })
            .count(),
        8
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.kind() == GoldAndGearsServiceAdventureRuleKind::ServiceBridge
            })
            .count(),
        15
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.kind() == GoldAndGearsServiceAdventureRuleKind::ReleasedService
            })
            .count(),
        15
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.accuracy() == GoldAndGearsServiceAdventureRuleAccuracy::ExactPublic
            })
            .count(),
        30
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.accuracy()
                    == GoldAndGearsServiceAdventureRuleAccuracy::VersionedProjectPolicy
            })
            .count(),
        8
    );
    assert!(
        bindings
            .iter()
            .all(|binding| binding.executor() == "ReleasedSharedExecutor")
    );
    assert_eq!(
        GOLD_AND_GEARS_SERVICE_ADVENTURE_EXECUTION_REVISION,
        "gold-and-gears-service-adventure-execution-v1"
    );
    assert_eq!(
        digest_hex(factory.service_adventure_execution_digest()),
        "5134055e448b30948f4e0521d4b030bc3e0cd089f5d381284d7befc31eb0e83d"
    );
}

#[test]
fn service_bridges_resolve_every_released_rule_without_duplicate_semantics() {
    let factory = factory();
    let bindings = factory.service_adventure_rule_bindings();
    for service in factory.service_definitions() {
        let owner = service.stable_key();
        assert!(bindings.iter().any(|binding| {
            binding.rule_id() == service.bridge_rule()
                && binding.owner_id() == owner
                && binding.kind() == GoldAndGearsServiceAdventureRuleKind::ServiceBridge
        }));
        assert!(bindings.iter().any(|binding| {
            binding.rule_id() == service.released_rule()
                && binding.owner_id() == owner
                && binding.kind() == GoldAndGearsServiceAdventureRuleKind::ReleasedService
        }));
    }
}

#[test]
fn all_38_service_adventure_rules_execute_through_the_production_fixture() {
    let instance = compiled_fixture(factory());
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = activity_rng(&instance, 14_507);
    commit(&instance, &mut state, funding_program());
    let mut purchased_services = 0;
    let mut structurally_resolved_services = 0;
    let mut shop_offers = 0;

    for service in instance.service_definitions() {
        let Some(stock) = service.stock().first().copied() else {
            assert_eq!(service.kind(), GoldAndGearsServiceKind::Currency);
            structurally_resolved_services += 1;
            continue;
        };
        match service.kind() {
            GoldAndGearsServiceKind::BlessingShop => {
                let GoldAndGearsServiceOfferSelector::BlessingRarity(rarity) = stock.selector()
                else {
                    panic!("blessing shop changed selector kind");
                };
                assert_eq!(
                    instance
                        .select_service_blessings(service.stable_key(), rarity, &[], 1, &mut rng,)
                        .unwrap()
                        .len(),
                    1
                );
                shop_offers += 1;
            }
            GoldAndGearsServiceKind::CurioShop => {
                assert_eq!(
                    instance
                        .select_service_curios(service.stable_key(), &[], 1, &mut rng)
                        .unwrap()
                        .len(),
                    1
                );
                shop_offers += 1;
            }
            _ => {}
        }
        commit(
            &instance,
            &mut state,
            instance
                .compile_service_purchase(service.stable_key(), stock.selector(), 0)
                .unwrap(),
        );
        purchased_services += 1;
    }

    let mut adventure_plans = 0;
    for adventure in instance.adventure_definitions() {
        let plan = instance
            .resolve_adventure_outcome(
                GoldAndGearsAdventureExternalOutcome::new(
                    adventure.id(),
                    adventure.maximum_value(),
                )
                .unwrap(),
                &[],
                &[],
                &mut rng,
            )
            .unwrap();
        assert_eq!(plan.completed_objectives(), 2);
        commit(
            &instance,
            &mut state,
            instance.compile_adventure_settlement(plan).unwrap(),
        );
        adventure_plans += 1;
    }

    assert_eq!(structurally_resolved_services, 1);
    assert_eq!(purchased_services, 14);
    assert_eq!(shop_offers, 9);
    assert_eq!(adventure_plans, 8);
    assert_eq!(state.command_sequence(), 23);
    assert_eq!(deferred_entry_count(&state), 22);
    assert_eq!(draws(&rng, ActivityRngLabel::Shop), 9);
    assert_eq!(draws(&rng, ActivityRngLabel::Reward), 24);
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "2a9189487f34eb445c9d3a3a2c99a26451d680f7fcc8d6d14f44183a43b020f4"
    );
}

#[test]
fn service_and_adventure_rejections_preserve_state_and_rng() {
    let instance = compiled_fixture(factory());
    let service = instance
        .service_definitions()
        .iter()
        .find(|service| !service.stock().is_empty())
        .unwrap();
    let stock = service.stock()[0];
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = activity_rng(&instance, 14_507);
    commit(&instance, &mut state, funding_program());
    let purchase = instance
        .compile_service_purchase(service.stable_key(), stock.selector(), 0)
        .unwrap();
    commit(&instance, &mut state, purchase.clone());
    let before_state = state_bytes(&instance, &state, &rng);
    let before_rng = rng.snapshots();
    assert!(matches!(
        apply(&instance, &mut state, purchase),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before_state);
    assert_eq!(rng.snapshots(), before_rng);

    let adventure = instance.adventure_definitions()[0].id();
    assert_eq!(
        instance.resolve_adventure_outcome(
            GoldAndGearsAdventureExternalOutcome::new(adventure, u32::MAX).unwrap(),
            &[],
            &[],
            &mut rng,
        ),
        Err(GoldAndGearsEntryError::InvalidAdventureOutcome)
    );
    assert_eq!(state_bytes(&instance, &state, &rng), before_state);
    assert_eq!(rng.snapshots(), before_rng);
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    super::tests::shared_factory()
}

fn funding_program() -> ActivityProgramDefinition {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(0x4E00_0001).unwrap(),
        vec![ActivityOperation::AddCounter {
            slot: ActivitySlotId::new(RUN_RESOURCES_SLOT).unwrap(),
            key: RESOURCE_COSMIC_FRAGMENTS_KEY,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(FIXTURE_FUNDING)),
        }],
    )
    .unwrap()
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) {
    assert!(matches!(
        apply(instance, state, program),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn apply(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) -> ActivityTransactionOutcome {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    state.apply_program(&program, cause, instance.graph_definition())
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

fn draws(rng: &ActivityRngStreams, label: ActivityRngLabel) -> u64 {
    rng.snapshots()
        .iter()
        .find(|snapshot| snapshot.label() == label)
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
