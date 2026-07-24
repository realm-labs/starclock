use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExternalOutcomeId, ActivityGraphDefinition,
    ActivityInstanceId, ActivityInteractionBinding, ActivityInteractionRandomPolicy,
    ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies,
    ActivityRngLabel, ActivityScope, ActivitySlotDefinition, ActivityStateDefinition,
    ActivityStateHash, ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityValue, BuildDigest, GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram,
    LoadoutLockScope, NodeId, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    SectionId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    entry::{CompiledActivity, StandardUniverseEntry, StandardUniverseProfile},
    id::{BlessingId, CurioId, ServiceId},
    service_interaction::{
        SERVICE_INTERACTION_HANDLER_ID, SERVICE_INTERACTION_RUNTIME_REVISION,
        ServiceInteractionSelection, ServicePurchaseContent,
    },
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe")
    }))
}

fn compiled() -> CompiledActivity {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(),
            vec![],
        ))
        .expect("compiled Standard Universe")
}

fn service(key: &str) -> ServiceId {
    catalog()
        .services()
        .iter()
        .find(|value| value.stable_key() == key)
        .expect("service fixture")
        .id()
}

fn first_blessing() -> BlessingId {
    catalog().blessings()[0].id()
}

fn first_curio() -> CurioId {
    catalog().curios()[0].id()
}

#[test]
fn all_service_families_compile_to_concrete_checked_payloads() {
    let compiled = compiled();
    let runtime = compiled.service_interaction_runtime();
    assert_eq!(
        SERVICE_INTERACTION_RUNTIME_REVISION,
        "standard-universe-service-interaction-runtime-v1"
    );
    assert_eq!(runtime.service_count(), 94);
    assert_ne!(runtime.digest(), [0; 32]);

    let fixtures = [
        (
            service("universe.currency.cosmic-fragments"),
            ServiceInteractionSelection::Activate,
        ),
        (
            service("universe.service.reset-blessing-choice"),
            ServiceInteractionSelection::Activate,
        ),
        (
            service("universe.service.reviver"),
            ServiceInteractionSelection::Activate,
        ),
        (
            service("universe.service.downloader"),
            ServiceInteractionSelection::Activate,
        ),
        (
            service("universe.service.respite-offers"),
            ServiceInteractionSelection::RespiteBlessing,
        ),
        (
            service("universe.service.enhance-blessing"),
            ServiceInteractionSelection::EnhanceBlessing(first_blessing()),
        ),
        (
            service("universe.service.shop.100011"),
            ServiceInteractionSelection::ShopPurchase {
                content: ServicePurchaseContent::Blessing(first_blessing()),
                cost: 99,
                offer_digest: [0x51; 32],
            },
        ),
        (
            service("universe.service.shop.100021"),
            ServiceInteractionSelection::ShopPurchase {
                content: ServicePurchaseContent::Curio(first_curio()),
                cost: 120,
                offer_digest: [0x52; 32],
            },
        ),
        (
            service("universe.service.trailblaze-bonus.1"),
            ServiceInteractionSelection::Activate,
        ),
    ];
    for (service, selection) in fixtures {
        let interaction = runtime
            .compile_selection(service, &selection)
            .expect("concrete service selection");
        assert!(!interaction.payload().is_empty());
        assert!(interaction.immediate_operations() + interaction.deferred_operations() > 0);
    }
}

#[test]
fn production_respite_and_transaction_rooms_offer_bound_service_handlers() {
    let compiled = compiled();
    let services = compiled
        .abstract_interactions()
        .iter()
        .filter(|binding| binding.source_content_id().starts_with("universe.service."))
        .collect::<Vec<_>>();
    assert!(!services.is_empty());
    assert!(services.iter().any(|binding| {
        binding.source_content_id() == "universe.service.respite-offers.one-star-blessing"
    }));
    assert!(
        services
            .iter()
            .any(|binding| { binding.source_content_id() == "universe.service.downloader" })
    );
    assert!(
        services
            .iter()
            .any(|binding| { binding.source_content_id() == "universe.service.shop.100011" })
    );
    let interactions = compiled.runtime_definition().interactions().unwrap();
    for abstract_binding in services {
        let binding = interactions
            .binding(abstract_binding.node(), abstract_binding.outcome())
            .expect("production service binding");
        assert_eq!(binding.handler().get(), SERVICE_INTERACTION_HANDLER_ID);
        if abstract_binding
            .source_content_id()
            .starts_with("universe.service.respite-offers")
        {
            assert_eq!(
                binding.random_policy().map(|policy| policy.label()),
                Some(ActivityRngLabel::Shop)
            );
        }
    }
}

#[test]
fn service_purchase_charges_and_grants_in_one_activity_transaction() {
    let compiled = compiled();
    let interaction = compiled
        .service_interaction_runtime()
        .compile_selection(
            service("universe.service.respite-offers"),
            &ServiceInteractionSelection::RespiteBlessing,
        )
        .expect("respite Blessing purchase");
    let outcome = ActivityExternalOutcomeId::new(90_031).unwrap();
    let mut activity = harness(
        &compiled,
        outcome,
        interaction.payload(),
        interaction.random_candidate_count(),
        200,
    );
    let before = activity.player_view();
    let decision = before.decision().unwrap();
    let before_bytes = activity.canonical_state_bytes();
    assert!(
        activity
            .submit_external_outcome(
                ActivityStateHash::new([0x7f; 32]).unwrap(),
                decision.id(),
                outcome,
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before_bytes);

    activity
        .submit_external_outcome(before.state_hash(), decision.id(), outcome)
        .expect("atomic service purchase");
    let after = activity.player_view();
    assert_eq!(
        after
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(120))
    );
    assert_eq!(
        after
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == compiled.blessing_inventory())
            .unwrap()
            .entries()
            .iter()
            .map(|(_, count)| count)
            .sum::<u32>(),
        1
    );
}

#[test]
fn unaffordable_service_preserves_state_and_does_not_consume_the_offer() {
    let compiled = compiled();
    let interaction = compiled
        .service_interaction_runtime()
        .compile_selection(
            service("universe.service.respite-offers"),
            &ServiceInteractionSelection::RespiteBlessing,
        )
        .expect("respite Blessing");
    let outcome = ActivityExternalOutcomeId::new(90_032).unwrap();
    let mut activity = harness(
        &compiled,
        outcome,
        interaction.payload(),
        interaction.random_candidate_count(),
        50,
    );
    let before = activity.player_view();
    let decision = before.decision().unwrap();
    let bytes = activity.canonical_state_bytes();
    let rng = activity.debug_view().rng().to_vec();
    assert!(
        activity
            .submit_external_outcome(before.state_hash(), decision.id(), outcome)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), bytes);
    assert_eq!(activity.debug_view().rng(), rng);
    assert_eq!(
        activity.player_view().decision().map(|value| value.id()),
        Some(decision.id())
    );
}

fn harness(
    compiled: &CompiledActivity,
    outcome: ActivityExternalOutcomeId,
    payload: &[u8],
    random_candidate_count: Option<u32>,
    fragments: i64,
) -> GraphActivity {
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::ExternalOutcome, 1)
                .unwrap(),
            ActivityNodeDefinition::new(
                node(2),
                section(1),
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        ],
        vec![
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(1).unwrap(),
                node(1),
                node(2),
                ActivityEdgeCondition::OptionSelected,
                0,
                1,
            )
            .unwrap(),
        ],
        2,
    )
    .unwrap();
    let program = GraphActivityNodeProgram::new(
        node(1),
        ActivityProgramDefinition::new(
            ActivityProgramId::new(1).unwrap(),
            vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::ExternalOutcome,
                options: vec![ActivityOptionDefinition::new(
                    starclock_activity::ActivityOptionId::new(outcome.get()).unwrap(),
                    0,
                    ActivityCondition::Boolean(starclock_activity::ActivityExpression::Literal(
                        ActivityValue::Boolean(true),
                    )),
                    vec![ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap())],
                )]
                .into_boxed_slice(),
            }],
        )
        .unwrap(),
    );
    let state = ActivityStateDefinition::new(
        vec![
            integer_slot(compiled.cosmic_fragments_slot(), fragments, 0x7001),
            counter_slot(
                compiled.service_use_slot(),
                ActivityStateVisibility::Player,
                0x7002,
            ),
            counter_slot(
                compiled.service_effect_slot(),
                ActivityStateVisibility::Private,
                0x7003,
            ),
        ],
        vec![
            *compiled
                .state_definition()
                .inventories()
                .iter()
                .find(|value| value.id() == compiled.blessing_inventory())
                .unwrap(),
            *compiled
                .state_definition()
                .inventories()
                .iter()
                .find(|value| value.id() == compiled.curio_inventory())
                .unwrap(),
        ],
        vec![],
    )
    .unwrap();
    let mut binding = ActivityInteractionBinding::new(
        node(1),
        outcome,
        starclock_activity::ActivityHandlerId::new(SERVICE_INTERACTION_HANDLER_ID).unwrap(),
        payload.to_vec(),
        "standard-universe.service-selection.v1",
    )
    .unwrap();
    if let Some(candidate_count) = random_candidate_count {
        binding = binding.with_random_policy(
            ActivityInteractionRandomPolicy::new(ActivityRngLabel::Shop, 1, candidate_count)
                .unwrap(),
        );
    }
    let registry = compiled
        .runtime_definition()
        .interactions()
        .unwrap()
        .registry()
        .as_ref()
        .clone();
    let definition = GraphActivityDefinition::new(
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(9_003).unwrap(),
            ActivityDefinitionDigest::new([0x61; 32]).unwrap(),
            ActivityConfigDigest::new([0x62; 32]).unwrap(),
        ),
        graph,
        state,
        Arc::new(participants()),
        vec![program],
        None,
        ActivityRandomPolicies::default(),
    )
    .and_then(|definition| definition.with_interactions(registry, vec![binding]))
    .unwrap();
    GraphActivity::start(
        Arc::new(definition),
        ActivityInstanceId::new(9_003).unwrap(),
        ActivityMasterSeed::from_u64(9_003),
    )
    .unwrap()
    .into_activity()
}

fn node(value: u32) -> NodeId {
    NodeId::new(value).unwrap()
}

fn section(value: u32) -> SectionId {
    SectionId::new(value).unwrap()
}

fn integer_slot(
    id: starclock_activity::ActivitySlotId,
    initial: i64,
    source: u64,
) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((
            0,
            starclock_mode_universe::run_runtime::MAX_COSMIC_FRAGMENTS,
        )),
        None,
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        ActivityStateSource::new(source).unwrap(),
    )
    .unwrap()
}

fn counter_slot(
    id: starclock_activity::ActivitySlotId,
    visibility: ActivityStateVisibility,
    source: u64,
) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(Box::new([])),
        Some((0, i64::from(u32::MAX))),
        Some(94),
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(source).unwrap(),
    )
    .unwrap()
}

fn participants() -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let build = OpaqueParticipantBuild::new(
        CombatantSpecDigest::new([1; 32]).unwrap(),
        BuildDigest::new([2; 32]).unwrap(),
        "service-interaction-test-v1",
        ParticipantSourceKind::CompiledBuild,
    )
    .unwrap();
    ParticipantLock::seal(
        policy,
        vec![
            ParticipantLockEntry::new(
                ParticipantId::new(1).unwrap(),
                0,
                0,
                UnitDefinitionId::new(20_001).unwrap(),
                build,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
