use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExternalOutcomeId, ActivityGraphDefinition,
    ActivityInstanceId, ActivityInteractionBinding, ActivityInteractionRandomPolicy,
    ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies,
    ActivityRngLabel, ActivityScope, ActivitySlotDefinition, ActivitySlotId,
    ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityTransactionOutcome, ActivityTransactionRejection, ActivityTransactionState,
    ActivityValue, BuildDigest, GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram,
    LoadoutLockScope, NodeId, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    SectionId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    encounter::RoomContentKind,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    occurrence::{OccurrenceOperation, OccurrenceTarget},
    occurrence_interaction::OCCURRENCE_INTERACTION_HANDLER_ID,
    run_runtime::{CosmicFragments, RUN_RUNTIME_REVISION, RunRuntimeCatalog},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

#[test]
fn all_occurrence_service_and_ability_inputs_compile_to_typed_runtime() {
    let catalog = catalog();
    let runtime = RunRuntimeCatalog::compile(&catalog).expect("run runtime");
    assert_eq!(RUN_RUNTIME_REVISION, "standard-universe-run-runtime-v1");
    assert_eq!(runtime.occurrence_choices().len(), 321);
    assert_eq!(runtime.services().len(), 94);
    assert_eq!(
        runtime
            .occurrence_choices()
            .iter()
            .flat_map(|choice| choice.outcomes())
            .filter(|outcome| outcome.random_policy().is_some())
            .count(),
        52
    );
    assert_eq!(
        runtime
            .services()
            .iter()
            .filter(|service| service.rule_key().is_empty())
            .count(),
        0
    );
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .take(3)
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let abilities = runtime
        .ability_contributions(&selected)
        .expect("Ability Tree contributions");
    assert_eq!(abilities.entries().len(), 3);
    assert!(
        abilities
            .entries()
            .iter()
            .all(|entry| { !entry.stable_key().is_empty() && !entry.rule_key().is_empty() })
    );
    assert_eq!(
        runtime.digest(),
        [
            89, 238, 195, 56, 168, 128, 97, 229, 35, 51, 141, 234, 207, 222, 143, 105, 9, 121, 206,
            115, 18, 218, 86, 217, 131, 255, 115, 83, 218, 202, 129, 107,
        ]
    );
    assert_eq!(
        abilities.digest(),
        [
            114, 247, 72, 232, 202, 29, 161, 49, 230, 67, 84, 199, 86, 79, 175, 117, 80, 118, 173,
            98, 68, 85, 156, 39, 122, 80, 97, 163, 4, 254, 206, 245,
        ]
    );
}

#[test]
fn cosmic_fragment_credit_and_spend_are_checked_atomic_activity_operations() {
    let slot = ActivitySlotId::new(1).unwrap();
    let definition = ActivityStateDefinition::new(
        vec![
            ActivitySlotDefinition::new_with_policy(
                slot,
                ActivityScope::Activity,
                ActivityValue::BoundedInteger(0),
                Some((
                    0,
                    starclock_mode_universe::run_runtime::MAX_COSMIC_FRAGMENTS,
                )),
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
    credit.validate_against(&definition, &graph).unwrap();
    commit(&mut state, &credit, 1, &graph);
    assert_eq!(state.slot(slot), Some(&ActivityValue::BoundedInteger(120)));

    let spend = ActivityProgramDefinition::new(
        program(2),
        RunRuntimeCatalog::spend_fragments(slot, CosmicFragments::new(45).unwrap()).into_vec(),
    )
    .unwrap();
    spend.validate_against(&definition, &graph).unwrap();
    commit(&mut state, &spend, 2, &graph);
    assert_eq!(state.slot(slot), Some(&ActivityValue::BoundedInteger(75)));

    let rejected = ActivityProgramDefinition::new(
        program(3),
        RunRuntimeCatalog::spend_fragments(slot, CosmicFragments::new(76).unwrap()).into_vec(),
    )
    .unwrap();
    assert_eq!(
        state.apply_program(&rejected, cause(3, 3), &graph),
        ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::ConditionNotSatisfied)
    );
    assert_eq!(state.slot(slot), Some(&ActivityValue::BoundedInteger(75)));
}

#[test]
fn noncombat_rooms_accept_only_offered_external_outcomes_without_granting_battle_rewards() {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(),
            vec![],
        ))
        .expect("compiled profile");
    assert!(!compiled.abstract_interactions().is_empty());
    assert!(
        compiled
            .abstract_interactions()
            .iter()
            .all(|binding| binding.kind() != RoomContentKind::EncounterGroup)
    );
    let bound = compiled
        .runtime_definition()
        .interactions()
        .expect("all external outcomes use the composed handler registry");
    assert!(bound.bindings().len() >= compiled.abstract_interactions().len());
    assert!(
        bound
            .bindings()
            .iter()
            .all(|binding| bound.registry().handler(binding.handler()).is_some())
    );
    assert!(
        compiled
            .abstract_interactions()
            .iter()
            .all(|binding| { bound.binding(binding.node(), binding.outcome()).is_some() })
    );

    let mut selected = None;
    for seed in 0..256 {
        let mut activity = compiled
            .start(
                ActivityInstanceId::new(seed + 1).unwrap(),
                ActivityMasterSeed::from_u64(seed),
            )
            .unwrap()
            .into_activity();
        let path = activity.player_view();
        let path_decision = path.decision().expect("Path decision");
        activity
            .choose_option(
                path.state_hash(),
                path_decision.id(),
                path_decision.options()[0].id(),
            )
            .unwrap();
        let content = activity.player_view();
        let decision = content.decision().expect("resolved room content");
        assert_eq!(decision.kind(), ActivityDecisionKind::ExternalOutcome);
        if let Some(binding) = compiled
            .abstract_interactions()
            .iter()
            .find(|binding| binding.outcome().get() == decision.options()[0].id().get())
        {
            selected = Some((activity, binding.outcome()));
            break;
        }
    }
    let (mut activity, outcome) = selected.expect("bounded seeds include noncombat room");
    let before = activity.player_view();
    let decision = before.decision().unwrap();
    let before_bytes = activity.canonical_state_bytes();
    assert!(
        activity
            .choose_option(
                before.state_hash(),
                decision.id(),
                decision.options()[0].id(),
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before_bytes);
    assert!(
        activity
            .submit_external_outcome(
                starclock_activity::ActivityStateHash::new([0; 32]).unwrap(),
                decision.id(),
                outcome,
            )
            .is_err()
    );
    activity
        .submit_external_outcome(before.state_hash(), decision.id(), outcome)
        .expect("offered external outcome");
    let after = activity.player_view();
    let external = after
        .slots()
        .iter()
        .find(|slot| slot.id() == compiled.external_outcome_slot())
        .expect("external-outcome slot");
    let ActivityValue::BoundedCounterMap(entries) = external.value() else {
        panic!("external-outcome counter map");
    };
    assert!(entries.iter().any(|(_, value)| *value == 1));
    let blessings = after
        .inventories()
        .iter()
        .find(|inventory| inventory.id() == compiled.blessing_inventory())
        .expect("Blessing inventory");
    assert!(blessings.entries().is_empty());
}

#[test]
fn occurrence_choices_compile_and_exact_room_sources_bind_executable_handlers() {
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
    let occurrence_bindings = compiled
        .abstract_interactions()
        .iter()
        .filter(|binding| {
            catalog
                .occurrence_choices()
                .iter()
                .any(|choice| choice.stable_key() == binding.source_content_id())
        })
        .collect::<Vec<_>>();
    assert!(occurrence_bindings.iter().all(|binding| {
        binding
            .source_content_id()
            .starts_with("universe.occurrence.1.variant.40398.choice.")
    }));
    assert_eq!(
        occurrence_bindings
            .iter()
            .map(|binding| binding.source_content_id())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    let runtime = compiled.runtime_definition().interactions().unwrap();
    assert!(occurrence_bindings.iter().all(|binding| {
        runtime
            .binding(binding.node(), binding.outcome())
            .is_some_and(|value| runtime.registry().handler(value.handler()).is_some())
    }));
    let interaction_catalog = compiled.occurrence_interaction_runtime();
    assert_eq!(interaction_catalog.choice_count(), 321);
    assert_eq!(interaction_catalog.immediate_operation_count(), 283);
    assert_eq!(interaction_catalog.deferred_operation_count(), 187);
    assert!(catalog.occurrence_choices().iter().any(|choice| {
        let outcome = &choice.outcomes()[0];
        outcome.operations().contains(&OccurrenceOperation::Obtain)
            && outcome.targets().iter().any(|target| {
                matches!(target, OccurrenceTarget::Blessing | OccurrenceTarget::Curio)
            })
    }));
}

#[test]
fn occurrence_choice_commits_inventory_rng_and_graph_transition_atomically() {
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
    let choice_key = "universe.occurrence.1.variant.40398.choice.02";
    let abstract_binding = compiled
        .abstract_interactions()
        .iter()
        .find(|binding| binding.source_content_id() == choice_key)
        .expect("exact Occurrence binding");
    let interactions = compiled.runtime_definition().interactions().unwrap();
    let binding = interactions
        .binding(abstract_binding.node(), abstract_binding.outcome())
        .expect("runtime interaction");
    let mut activity = occurrence_harness(&compiled, binding, interactions.registry());
    let outcome = abstract_binding.outcome();
    let before = activity.player_view();
    let decision = before.decision().unwrap();
    let before_bytes = activity.canonical_state_bytes();
    let before_rng = activity.debug_view().rng().to_vec();
    assert!(
        activity
            .submit_external_outcome(
                starclock_activity::ActivityStateHash::new([0x7f; 32]).unwrap(),
                decision.id(),
                outcome,
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before_bytes);
    assert_eq!(activity.debug_view().rng(), before_rng);

    activity
        .submit_external_outcome(before.state_hash(), decision.id(), outcome)
        .expect("Occurrence choice");
    assert_ne!(activity.canonical_state_bytes(), before_bytes);
    let after = activity.player_view();
    let blessings = after
        .inventories()
        .iter()
        .find(|inventory| inventory.id() == compiled.blessing_inventory())
        .expect("Blessing inventory");
    assert_eq!(
        blessings.entries().iter().map(|entry| entry.1).sum::<u32>(),
        1
    );
    let before_draws = before_rng
        .iter()
        .find(|stream| stream.label() == starclock_activity::ActivityRngLabel::Occurrence)
        .unwrap()
        .draw_count();
    let after_draws = activity
        .debug_view()
        .rng()
        .iter()
        .find(|stream| stream.label() == starclock_activity::ActivityRngLabel::Occurrence)
        .unwrap()
        .draw_count();
    assert_eq!(after_draws, before_draws + 1);
}

#[test]
fn occurrence_curio_acquisition_initializes_lifecycle_in_the_same_transaction() {
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
    let choice = catalog
        .occurrence_choices()
        .iter()
        .find(|choice| choice.stable_key() == "universe.occurrence.39.variant.12201.choice.06")
        .expect("Curio acquisition choice");
    let interaction = compiled
        .occurrence_interaction_runtime()
        .compile_choice(choice.id())
        .expect("compiled Curio choice");
    let outcome = ActivityExternalOutcomeId::new(99_002).unwrap();
    let mut binding = ActivityInteractionBinding::new(
        node(1),
        outcome,
        starclock_activity::ActivityHandlerId::new(OCCURRENCE_INTERACTION_HANDLER_ID).unwrap(),
        interaction.payload().to_vec(),
        "standard-universe.occurrence-choice.v2",
    )
    .unwrap();
    binding = binding.with_random_policy(
        ActivityInteractionRandomPolicy::new(
            ActivityRngLabel::Occurrence,
            91,
            interaction.random_candidate_count().unwrap(),
        )
        .unwrap(),
    );
    let registry = compiled
        .runtime_definition()
        .interactions()
        .unwrap()
        .registry();
    let mut activity = occurrence_harness(&compiled, &binding, registry);
    let before = activity.player_view();
    activity
        .submit_external_outcome(
            before.state_hash(),
            before.decision().unwrap().id(),
            outcome,
        )
        .expect("atomic Curio acquisition");

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
    assert_eq!(
        compiled
            .curio_runtime()
            .contributions(inventory, state, charges)
            .expect("valid lifecycle")
            .entries()
            .len(),
        1
    );
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

fn occurrence_harness(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    source: &ActivityInteractionBinding,
    registry: &Arc<starclock_activity::ActivityHandlerRegistry>,
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
            program(1),
            vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::ExternalOutcome,
                options: vec![ActivityOptionDefinition::new(
                    starclock_activity::ActivityOptionId::new(source.offered_outcome().get())
                        .unwrap(),
                    0,
                    starclock_activity::ActivityCondition::Boolean(
                        starclock_activity::ActivityExpression::Literal(ActivityValue::Boolean(
                            true,
                        )),
                    ),
                    vec![ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap())],
                )]
                .into_boxed_slice(),
            }],
        )
        .unwrap(),
    );
    let required_slots = [
        compiled.cosmic_fragments_slot(),
        compiled.occurrence_effect_slot(),
        compiled.curio_state_slot(),
        compiled.curio_charge_slot(),
        compiled.curio_event_slot(),
    ];
    let state = ActivityStateDefinition::new(
        compiled
            .state_definition()
            .slots()
            .iter()
            .filter(|slot| required_slots.contains(&slot.id()))
            .cloned()
            .collect(),
        compiled
            .state_definition()
            .inventories()
            .iter()
            .filter(|inventory| {
                matches!(
                    inventory.id(),
                    id if id == compiled.blessing_inventory() || id == compiled.curio_inventory()
                )
            })
            .copied()
            .collect(),
        vec![],
    )
    .unwrap();
    let mut binding = ActivityInteractionBinding::new(
        node(1),
        source.offered_outcome(),
        source.handler(),
        source.payload().to_vec(),
        source.component_id(),
    )
    .unwrap();
    if let Some(policy) = source.random_policy() {
        binding = binding.with_random_policy(policy);
    }
    let definition = GraphActivityDefinition::new(
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(9_001).unwrap(),
            ActivityDefinitionDigest::new([0x41; 32]).unwrap(),
            ActivityConfigDigest::new([0x42; 32]).unwrap(),
        ),
        graph,
        state,
        Arc::new(participants()),
        vec![program],
        None,
        ActivityRandomPolicies::default(),
    )
    .and_then(|definition| definition.with_interactions((**registry).clone(), vec![binding]))
    .unwrap();
    GraphActivity::start(
        Arc::new(definition),
        ActivityInstanceId::new(9_001).unwrap(),
        ActivityMasterSeed::from_u64(9_001),
    )
    .unwrap()
    .into_activity()
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
        "test-build-catalog-v1",
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

fn commit(
    state: &mut ActivityTransactionState,
    program: &ActivityProgramDefinition,
    sequence: u64,
    graph: &ActivityGraphDefinition,
) {
    assert!(matches!(
        state.apply_program(program, cause(sequence, program.id().get()), graph),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn graph() -> ActivityGraphDefinition {
    ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 8).unwrap(),
            ActivityNodeDefinition::new(
                node(2),
                section(1),
                ActivityNodeKind::Terminal(starclock_activity::ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        ],
        vec![
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(1).unwrap(),
                node(1),
                node(2),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        8,
    )
    .unwrap()
}

fn cause(sequence: u64, program: u32) -> ActivityCause {
    ActivityCause::new(sequence, self::program(program), node(1)).unwrap()
}
fn node(raw: u32) -> NodeId {
    NodeId::new(raw).unwrap()
}
fn section(raw: u32) -> SectionId {
    SectionId::new(raw).unwrap()
}
fn program(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).unwrap()
}
