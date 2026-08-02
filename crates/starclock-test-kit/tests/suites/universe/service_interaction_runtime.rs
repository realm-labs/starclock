use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExternalOutcomeId, ActivityGraphDefinition,
    ActivityInstanceId, ActivityInteractionBinding, ActivityInteractionRandomPolicy,
    ActivityInventoryId, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityPlayerView, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivityRngLabel, ActivityScope,
    ActivitySlotDefinition, ActivityStateDefinition, ActivityStateHash, ActivityStateSource,
    ActivityStateVisibility, ActivityTerminalOutcome, ActivityValue, BuildDigest, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram, LoadoutLockScope, NodeId,
    OpaqueParticipantBuild, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope, SectionId,
    SlotCarryPolicy, SlotResetPoint,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    entry::{CompiledActivity, StandardUniverseEntry, StandardUniverseProfile},
    id::{BlessingId, CurioId, ServiceId},
    service_interaction::{
        SERVICE_INTERACTION_HANDLER_ID, ServiceInteractionError, ServiceInteractionSelection,
        ServicePurchaseContent,
    },
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe")
    }))
}

fn compiled() -> CompiledActivity {
    compiled_with_ability(Vec::new())
}

fn compiled_with_ability(
    ability_tree: Vec<starclock_mode_universe::id::AbilityTreeNodeId>,
) -> CompiledActivity {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(),
            ability_tree,
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

fn inventory_total(view: &ActivityPlayerView, inventory: ActivityInventoryId) -> u32 {
    view.inventories()
        .iter()
        .find(|value| value.id() == inventory)
        .expect("inventory")
        .entries()
        .iter()
        .map(|(_, count)| *count)
        .sum()
}

#[test]
fn all_service_families_compile_to_concrete_checked_payloads() {
    let compiled = compiled();
    let runtime = compiled.service_interaction_runtime();
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
            ServiceInteractionSelection::ReviveCharacter(ParticipantId::new(1).unwrap()),
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
fn standard_trailblaze_bonuses_execute_exact_activity_effects() {
    let compiled = compiled();
    let cases = [
        ("universe.service.trailblaze-bonus.1", 50, 150, 0, 0),
        ("universe.service.trailblaze-bonus.2", 50, 50, 1, 0),
        ("universe.service.trailblaze-bonus.3", 50, 0, 0, 1),
        ("universe.service.trailblaze-bonus.4", 50, 200, 0, 0),
        ("universe.service.trailblaze-bonus.5", 50, 50, 1, 0),
        ("universe.service.trailblaze-bonus.6", 50, 50, 0, 1),
    ];
    for (index, (key, before_fragments, after_fragments, blessings, curios)) in
        cases.into_iter().enumerate()
    {
        let interaction = compiled
            .service_interaction_runtime()
            .compile_selection(service(key), &ServiceInteractionSelection::Activate)
            .unwrap();
        let outcome = ActivityExternalOutcomeId::new(92_000 + index as u64).unwrap();
        let mut activity = harness(
            &compiled,
            outcome,
            interaction.payload(),
            interaction.random_candidate_count(),
            before_fragments,
        );
        let before = activity.player_view();
        activity
            .submit_external_outcome(
                before.state_hash(),
                before.decision().unwrap().id(),
                outcome,
            )
            .unwrap_or_else(|error| panic!("{key}: {error:?}"));
        let after = activity.player_view();
        assert_eq!(
            after
                .slots()
                .iter()
                .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
                .map(|slot| slot.value()),
            Some(&ActivityValue::BoundedInteger(after_fragments)),
            "{key}"
        );
        assert_eq!(
            inventory_total(&after, compiled.blessing_inventory()),
            blessings,
            "{key}"
        );
        assert_eq!(
            inventory_total(&after, compiled.curio_inventory()),
            curios,
            "{key}"
        );
    }
}

#[test]
fn nonstandard_bonus_profiles_fail_closed_before_payload_or_rng() {
    let compiled = compiled();
    for suffix in (101_u32..=106)
        .chain(201..=205)
        .chain(401..=432)
        .chain(501..=530)
    {
        let key = format!("universe.service.trailblaze-bonus.{suffix}");
        assert_eq!(
            compiled
                .service_interaction_runtime()
                .compile_selection(service(&key), &ServiceInteractionSelection::Activate),
            Err(ServiceInteractionError::ProfileUnavailable),
            "{key}"
        );
    }
}

#[test]
fn production_entry_replaces_three_ordinary_bonuses_with_three_enhanced_bonuses() {
    let ordinary = compiled();
    let all_nodes = catalog()
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect();
    let enhanced = compiled_with_ability(all_nodes);
    for (index, (compiled, expected, selected, fragments)) in [
        (
            ordinary,
            [
                "universe.service.trailblaze-bonus.1",
                "universe.service.trailblaze-bonus.2",
                "universe.service.trailblaze-bonus.3",
            ],
            "universe.service.trailblaze-bonus.1",
            150,
        ),
        (
            enhanced,
            [
                "universe.service.trailblaze-bonus.4",
                "universe.service.trailblaze-bonus.5",
                "universe.service.trailblaze-bonus.6",
            ],
            "universe.service.trailblaze-bonus.4",
            250,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut activity = compiled
            .start(
                ActivityInstanceId::new(93_000 + index as u64).unwrap(),
                ActivityMasterSeed::from_u64(93_000 + index as u64),
            )
            .unwrap()
            .into_activity();
        let path = activity.player_view();
        activity
            .choose_option(
                path.state_hash(),
                path.decision().unwrap().id(),
                path.decision().unwrap().options()[0].id(),
            )
            .unwrap();
        let bonus = activity.player_view();
        assert_eq!(
            bonus.decision().unwrap().kind(),
            ActivityDecisionKind::ExternalOutcome
        );
        let offered = bonus
            .decision()
            .unwrap()
            .options()
            .iter()
            .map(|option| {
                compiled
                    .abstract_interactions()
                    .iter()
                    .find(|binding| {
                        binding.node() == bonus.current_node()
                            && binding.outcome().get() == option.id().get()
                    })
                    .expect("production Trailblaze Bonus binding")
                    .source_content_id()
            })
            .collect::<Vec<_>>();
        assert_eq!(offered, expected);
        let outcome = compiled
            .abstract_interactions()
            .iter()
            .find(|binding| {
                binding.node() == bonus.current_node() && binding.source_content_id() == selected
            })
            .unwrap()
            .outcome();
        activity
            .submit_external_outcome(bonus.state_hash(), bonus.decision().unwrap().id(), outcome)
            .unwrap();
        assert_eq!(
            activity
                .player_view()
                .slots()
                .iter()
                .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
                .map(|slot| slot.value()),
            Some(&ActivityValue::BoundedInteger(fragments))
        );
    }
}

#[test]
fn goal07_p4_m14_s01_executes_every_non_reviver_service_through_activity() {
    let compiled = compiled();
    let blessing = first_blessing();
    let keys = [
        "universe.currency.cosmic-fragments",
        "universe.service.downloader",
        "universe.service.enhance-blessing",
        "universe.service.reset-blessing-choice",
        "universe.service.respite-offers",
        "universe.service.shop.100011",
        "universe.service.shop.100021",
        "universe.service.shop.101010",
        "universe.service.shop.101011",
        "universe.service.shop.101012",
        "universe.service.shop.101020",
        "universe.service.shop.101021",
        "universe.service.shop.102011",
        "universe.service.shop.102021",
        "universe.service.trailblaze-bonus.1",
    ];
    for (index, key) in keys.into_iter().enumerate() {
        let selection = match key {
            "universe.service.enhance-blessing" => {
                ServiceInteractionSelection::EnhanceBlessing(blessing)
            }
            "universe.service.respite-offers" => ServiceInteractionSelection::RespiteBlessing,
            _ => ServiceInteractionSelection::Activate,
        };
        let interaction = compiled
            .service_interaction_runtime()
            .compile_selection(service(key), &selection)
            .unwrap();
        let outcome = ActivityExternalOutcomeId::new(91_000 + index as u64).unwrap();
        let mut activity = harness_with_inventory(
            &compiled,
            outcome,
            interaction.payload(),
            interaction.random_candidate_count(),
            1_000,
            None,
            (key == "universe.service.enhance-blessing").then_some(blessing),
        );
        let before = activity.player_view();
        activity
            .submit_external_outcome(
                before.state_hash(),
                before.decision().unwrap().id(),
                outcome,
            )
            .unwrap_or_else(|error| panic!("{key}: {error:?}"));
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed),
            "{key}"
        );
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

#[test]
fn curio_purchase_initializes_lifecycle_charge_and_boundary_event_atomically() {
    let compiled = compiled();
    let definition = compiled
        .curio_runtime()
        .definitions()
        .iter()
        .find(|definition| {
            definition
                .states()
                .iter()
                .find(|state| state.id() == definition.initial_state())
                .is_some_and(|state| state.maximum_charges().is_some())
        })
        .expect("charged Curio fixture");
    let shop = service("universe.service.shop.100021");
    let interaction = compiled
        .service_interaction_runtime()
        .compile_selection(
            shop,
            &ServiceInteractionSelection::ShopPurchase {
                content: ServicePurchaseContent::Curio(definition.curio()),
                cost: 120,
                offer_digest: [0x6a; 32],
            },
        )
        .expect("concrete Curio purchase");
    let outcome = ActivityExternalOutcomeId::new(90_033).unwrap();
    let mut activity = harness(
        &compiled,
        outcome,
        interaction.payload(),
        interaction.random_candidate_count(),
        200,
    );
    let before = activity.player_view();
    activity
        .submit_external_outcome(
            before.state_hash(),
            before.decision().unwrap().id(),
            outcome,
        )
        .expect("atomic Curio purchase");

    let player = activity.player_view();
    let inventory = player
        .inventories()
        .iter()
        .find(|value| value.id() == compiled.curio_inventory())
        .unwrap();
    let state = player
        .slots()
        .iter()
        .find(|value| value.id() == compiled.curio_state_slot())
        .unwrap();
    let charges = player
        .slots()
        .iter()
        .find(|value| value.id() == compiled.curio_charge_slot())
        .unwrap();
    let contributions = compiled
        .curio_runtime()
        .contributions(inventory, state, charges)
        .expect("lifecycle-complete Curio contribution");
    assert_eq!(contributions.entries().len(), 1);
    assert_eq!(contributions.entries()[0].curio(), definition.curio());
    assert!(
        activity
            .debug_view()
            .all_slots()
            .iter()
            .find(|value| value.id() == compiled.curio_event_slot())
            .is_some_and(|slot| matches!(
                slot.value(),
                ActivityValue::BoundedCounterMap(entries)
                    if entries.iter().any(|(_, count)| *count == 1)
            ))
    );
}

#[test]
fn faith_bond_discounts_only_authored_blessing_service_costs() {
    let compiled = compiled();
    let blessing = first_blessing();
    let interaction = compiled
        .service_interaction_runtime()
        .compile_selection(
            service("universe.service.enhance-blessing"),
            &ServiceInteractionSelection::EnhanceBlessing(blessing),
        )
        .expect("enhancement selection");
    let raw = interaction.required_fragments().expect("authored cost");
    let outcome = ActivityExternalOutcomeId::new(90_034).unwrap();
    let mut activity = harness_with_inventory(
        &compiled,
        outcome,
        interaction.payload(),
        interaction.random_candidate_count(),
        1_000,
        Some(CurioId::new(19).unwrap()),
        Some(blessing),
    );
    let before = activity.player_view();
    activity
        .submit_external_outcome(
            before.state_hash(),
            before.decision().unwrap().id(),
            outcome,
        )
        .expect("discounted enhancement");
    let expected = 1_000 - i64::from(raw * 70 / 100);
    assert_eq!(
        activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(expected))
    );
}

#[test]
fn ipc_cuckoo_clock_inflates_authored_blessing_enhancement_cost_by_twenty_five_percent() {
    let compiled = compiled();
    let blessing = first_blessing();
    let interaction = compiled
        .service_interaction_runtime()
        .compile_selection(
            service("universe.service.enhance-blessing"),
            &ServiceInteractionSelection::EnhanceBlessing(blessing),
        )
        .expect("enhancement selection");
    let raw = interaction.required_fragments().expect("authored cost");
    let outcome = ActivityExternalOutcomeId::new(90_035).unwrap();
    let mut activity = harness_with_inventory(
        &compiled,
        outcome,
        interaction.payload(),
        interaction.random_candidate_count(),
        1_000,
        Some(CurioId::new(70).unwrap()),
        Some(blessing),
    );
    let before = activity.player_view();
    activity
        .submit_external_outcome(
            before.state_hash(),
            before.decision().unwrap().id(),
            outcome,
        )
        .expect("inflated enhancement");
    let expected = 1_000 - i64::from(raw * 125 / 100);
    assert_eq!(
        activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(expected))
    );
}

#[test]
fn ipc_cuckoo_clock_inflates_the_first_blessing_reset_cost() {
    let compiled = compiled();
    let interaction = compiled
        .service_interaction_runtime()
        .compile_selection(
            service("universe.service.reset-blessing-choice"),
            &ServiceInteractionSelection::Activate,
        )
        .expect("reset selection");
    let outcome = ActivityExternalOutcomeId::new(90_036).unwrap();
    let mut activity = harness_with_inventory(
        &compiled,
        outcome,
        interaction.payload(),
        interaction.random_candidate_count(),
        1_000,
        Some(CurioId::new(70).unwrap()),
        None,
    );
    let before = activity.player_view();
    activity
        .submit_external_outcome(
            before.state_hash(),
            before.decision().unwrap().id(),
            outcome,
        )
        .expect("inflated reset");
    assert_eq!(
        activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(963))
    );
}

fn harness(
    compiled: &CompiledActivity,
    outcome: ActivityExternalOutcomeId,
    payload: &[u8],
    random_candidate_count: Option<u32>,
    fragments: i64,
) -> GraphActivity {
    harness_with_inventory(
        compiled,
        outcome,
        payload,
        random_candidate_count,
        fragments,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn harness_with_inventory(
    compiled: &CompiledActivity,
    outcome: ActivityExternalOutcomeId,
    payload: &[u8],
    random_candidate_count: Option<u32>,
    fragments: i64,
    curio: Option<CurioId>,
    blessing: Option<BlessingId>,
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
    let mut start_operations = Vec::new();
    if let Some(curio) = curio {
        start_operations.push(ActivityOperation::AddInventory {
            inventory: compiled.curio_inventory(),
            content: u64::from(curio.get()),
            count: starclock_activity::ActivityExpression::Literal(ActivityValue::BoundedInteger(
                1,
            )),
        });
    }
    if let Some(blessing) = blessing {
        start_operations.push(ActivityOperation::AddInventory {
            inventory: compiled.blessing_inventory(),
            content: u64::from(blessing.get()),
            count: starclock_activity::ActivityExpression::Literal(ActivityValue::BoundedInteger(
                1,
            )),
        });
    }
    start_operations.push(ActivityOperation::Offer {
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
    });
    let program = GraphActivityNodeProgram::new(
        node(1),
        ActivityProgramDefinition::new(ActivityProgramId::new(1).unwrap(), start_operations)
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
            compiled
                .state_definition()
                .slots()
                .iter()
                .find(|value| value.id() == compiled.curio_state_slot())
                .unwrap()
                .clone(),
            compiled
                .state_definition()
                .slots()
                .iter()
                .find(|value| value.id() == compiled.curio_charge_slot())
                .unwrap()
                .clone(),
            compiled
                .state_definition()
                .slots()
                .iter()
                .find(|value| value.id() == compiled.curio_event_slot())
                .unwrap()
                .clone(),
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
