use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityExternalOutcomeId,
    ActivityGraphDefinition, ActivityInstanceId, ActivityInteractionBinding, ActivityMasterSeed,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivityRngLabel,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
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
    occurrence::{AuthoredScalarUnit, OccurrenceOperation, OccurrenceTarget},
    occurrence_interaction::OCCURRENCE_INTERACTION_HANDLER_ID,
    run_runtime::{CosmicFragments, MAX_COSMIC_FRAGMENTS, RUN_RUNTIME_REVISION, RunRuntimeCatalog},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

#[path = "run_runtime/s04.rs"]
mod s04;
#[path = "run_runtime/s05.rs"]
mod s05;
#[path = "run_runtime/s06.rs"]
mod s06;
#[path = "run_runtime/s07.rs"]
mod s07;
#[path = "run_runtime/s08.rs"]
mod s08;
#[path = "run_runtime/s09.rs"]
mod s09;
#[path = "run_runtime/s10.rs"]
mod s10;
#[path = "run_runtime/s11.rs"]
mod s11;
#[path = "run_runtime/s12.rs"]
mod s12;
#[path = "run_runtime/s13.rs"]
mod s13;
#[path = "run_runtime/s14.rs"]
mod s14;

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
    assert_eq!(RUN_RUNTIME_REVISION, "standard-universe-run-runtime-v2");
    assert_eq!(runtime.occurrence_choices().len(), 321);
    assert_eq!(runtime.services().len(), 94);
    assert_eq!(
        runtime
            .occurrence_choices()
            .iter()
            .flat_map(|choice| choice.outcomes())
            .filter(|outcome| outcome.random_policy().is_some())
            .count(),
        127
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
            33, 105, 179, 221, 104, 218, 184, 61, 119, 52, 223, 225, 103, 153, 19, 117, 220, 231,
            206, 181, 5, 71, 203, 233, 0, 223, 170, 183, 47, 99, 222, 175,
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
fn noncombat_rooms_accept_only_offered_external_outcomes_through_bound_handlers() {
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
            .all(|binding| binding.kind() != Some(RoomContentKind::EncounterGroup))
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
        let trailblaze = activity.player_view();
        let trailblaze_decision = trailblaze.decision().expect("Trailblaze Bonus decision");
        assert_eq!(
            trailblaze_decision.kind(),
            ActivityDecisionKind::ExternalOutcome
        );
        activity
            .submit_external_outcome(
                trailblaze.state_hash(),
                trailblaze_decision.id(),
                ActivityExternalOutcomeId::new(trailblaze_decision.options()[0].id().get())
                    .expect("Trailblaze Bonus outcome"),
            )
            .expect("Trailblaze Bonus");
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
    assert_eq!(occurrence_bindings.len(), 492);
    let runtime = compiled.runtime_definition().interactions().unwrap();
    assert!(occurrence_bindings.iter().all(|binding| {
        runtime
            .binding(binding.node(), binding.outcome())
            .is_some_and(|value| runtime.registry().handler(value.handler()).is_some())
    }));
    let interaction_catalog = compiled.occurrence_interaction_runtime();
    assert_eq!(interaction_catalog.choice_count(), 321);
    assert_eq!(interaction_catalog.immediate_operation_count(), 403);
    assert_eq!(interaction_catalog.deferred_operation_count(), 0);
    assert_eq!(interaction_catalog.external_result_count(), 2_968);
    assert!(catalog.occurrence_choices().iter().any(|choice| {
        let outcome = &choice.outcomes()[0];
        outcome.operations().contains(&OccurrenceOperation::Obtain)
            && outcome.targets().iter().any(|target| {
                matches!(target, OccurrenceTarget::Blessing | OccurrenceTarget::Curio)
            })
    }));
}

#[test]
fn occurrence_external_result_commits_inventory_without_hidden_rng_and_transitions_atomically() {
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
    assert_eq!(after_draws, before_draws);
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
    assert!(interaction.external_results().is_empty());
    let candidates = interaction
        .random_candidate_count()
        .expect("random Curio candidates");
    let outcome = ActivityExternalOutcomeId::new(99_002).unwrap();
    let binding = ActivityInteractionBinding::new(
        node(1),
        outcome,
        starclock_activity::ActivityHandlerId::new(OCCURRENCE_INTERACTION_HANDLER_ID).unwrap(),
        interaction.payload().to_vec(),
        "standard-universe.occurrence-choice.v2",
    )
    .unwrap()
    .with_random_policy(
        starclock_activity::ActivityInteractionRandomPolicy::new(
            ActivityRngLabel::Occurrence,
            206,
            candidates,
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
        3
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

#[test]
fn goal07_p4_m13_s01_executes_exact_fragments_named_curio_transitions_and_external_blessing_results()
 {
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

    let s01_choices = catalog
        .occurrence_choices()
        .iter()
        .filter(|choice| {
            choice
                .stable_key()
                .starts_with("universe.occurrence.1.variant.40398.choice.")
                || choice
                    .stable_key()
                    .starts_with("universe.occurrence.10.variant.10801.choice.")
                || choice
                    .stable_key()
                    .starts_with("universe.occurrence.11.variant.10901.choice.")
                || matches!(
                    choice.stable_key(),
                    "universe.occurrence.12.variant.10901.choice.01"
                        | "universe.occurrence.12.variant.10901.choice.02"
                        | "universe.occurrence.12.variant.10901.choice.03"
                        | "universe.occurrence.12.variant.10901.choice.04"
                        | "universe.occurrence.12.variant.10901.choice.05"
                        | "universe.occurrence.12.variant.10901.choice.06"
                )
        })
        .collect::<Vec<_>>();
    assert_eq!(s01_choices.len(), 24);
    assert!(s01_choices.iter().all(|choice| {
        runtime
            .compile_choice(choice.id())
            .is_some_and(|interaction| interaction.deferred_operations() == 0)
    }));

    let fragments = |key: &str, expected: i64, outcome: u32| {
        let choice = catalog
            .occurrence_choices()
            .iter()
            .find(|choice| choice.stable_key() == key)
            .unwrap();
        let interaction = runtime.compile_choice(choice.id()).unwrap();
        let activity = execute_occurrence_payload(&compiled, interaction.payload(), outcome);
        assert_eq!(
            activity
                .player_view()
                .slots()
                .iter()
                .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
                .map(|slot| slot.value()),
            Some(&ActivityValue::BoundedInteger(50 + expected))
        );
    };
    fragments("universe.occurrence.1.variant.40398.choice.01", 150, 99_101);
    fragments(
        "universe.occurrence.11.variant.10901.choice.04",
        100,
        99_104,
    );

    let named_curio = catalog
        .occurrence_choices()
        .iter()
        .find(|choice| choice.stable_key() == "universe.occurrence.11.variant.10901.choice.01")
        .unwrap();
    let named_curio = runtime.compile_choice(named_curio.id()).unwrap();
    assert!(named_curio.external_results().is_empty());
    let activity = execute_occurrence_payload(&compiled, named_curio.payload(), 99_105);
    let player = activity.player_view();
    let inventory = player
        .inventories()
        .iter()
        .find(|inventory| inventory.id() == compiled.curio_inventory())
        .unwrap();
    let angel_dispenser = catalog
        .curios()
        .iter()
        .find(|curio| curio.stable_key() == "universe.curio.60")
        .unwrap();
    assert_eq!(
        inventory.entries(),
        &[(u64::from(angel_dispenser.id().get()), 1)]
    );

    let transition = catalog
        .occurrence_choices()
        .iter()
        .find(|choice| choice.stable_key() == "universe.occurrence.1.variant.40398.choice.03")
        .unwrap();
    let transition = runtime.compile_choice(transition.id()).unwrap();
    assert_eq!(transition.immediate_operations(), 1);
    assert_eq!(transition.deferred_operations(), 0);
    let activity = execute_occurrence_payload(&compiled, transition.payload(), 99_106);
    assert!(activity.player_view().decision().is_none());

    let blessing = catalog
        .occurrence_choices()
        .iter()
        .find(|choice| choice.stable_key() == "universe.occurrence.1.variant.40398.choice.02")
        .unwrap();
    let blessing = runtime.compile_choice(blessing.id()).unwrap();
    assert_eq!(blessing.external_results().len(), 162);
    assert!(
        blessing
            .external_results()
            .iter()
            .all(|result| result.immediate_operations() == 1 && result.deferred_operations() == 0)
    );
    let activity =
        execute_occurrence_payload(&compiled, blessing.external_results()[0].payload(), 99_107);
    let player = activity.player_view();
    let inventory = player
        .inventories()
        .iter()
        .find(|inventory| inventory.id() == compiled.blessing_inventory())
        .unwrap();
    assert_eq!(
        inventory
            .entries()
            .iter()
            .map(|(_, count)| count)
            .sum::<u32>(),
        1
    );
    assert_eq!(
        activity
            .debug_view()
            .rng()
            .iter()
            .find(|stream| stream.label() == ActivityRngLabel::Occurrence)
            .unwrap()
            .draw_count(),
        0
    );
}

#[test]
fn goal07_p4_m13_s02_executes_exact_hp_path_curio_fragment_and_occurrence_battle_outcomes() {
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
    let s02_choices = catalog
        .occurrence_choices()
        .iter()
        .filter(|choice| {
            matches!(
                choice.stable_key(),
                "universe.occurrence.12.variant.10901.choice.07"
                    | "universe.occurrence.12.variant.10901.choice.08"
                    | "universe.occurrence.12.variant.10901.choice.09"
            ) || (13..=18).any(|occurrence| {
                choice
                    .stable_key()
                    .starts_with(&format!("universe.occurrence.{occurrence}."))
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(s02_choices.len(), 18);
    assert!(s02_choices.iter().all(|choice| {
        runtime
            .compile_choice(choice.id())
            .is_some_and(|interaction| interaction.deferred_operations() == 0)
    }));

    let choice = |key: &str| {
        let choice = catalog
            .occurrence_choices()
            .iter()
            .find(|choice| choice.stable_key() == key)
            .unwrap();
        runtime.compile_choice(choice.id()).unwrap()
    };
    let fragments = choice("universe.occurrence.12.variant.10901.choice.07");
    let activity = execute_occurrence_payload(&compiled, fragments.payload(), 99_201);
    assert_eq!(
        activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(150))
    );

    let path = choice("universe.occurrence.15.variant.11201.choice.01");
    assert_eq!(path.random_candidate_count(), Some(9));
    assert!(path.external_results().is_empty());
    let activity = execute_occurrence_payload(&compiled, path.payload(), 99_202);
    let blessings = activity
        .player_view()
        .inventories()
        .iter()
        .find(|inventory| inventory.id() == compiled.blessing_inventory())
        .unwrap()
        .entries()
        .to_vec();
    assert_eq!(blessings.len(), 18);
    assert!(blessings.iter().all(|(_, count)| *count == 1));

    let insect = choice("universe.occurrence.14.variant.11101.choice.03");
    assert_eq!(insect.random_candidate_count(), Some(27));
    assert!(insect.external_results().is_empty());
    let activity = execute_occurrence_payload(&compiled, insect.payload(), 99_203);
    let player = activity.player_view();
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
    let insect_web = catalog
        .curios()
        .iter()
        .find(|curio| curio.stable_key() == "universe.curio.59")
        .unwrap();
    assert_eq!(
        player
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == compiled.curio_inventory())
            .unwrap()
            .entries(),
        &[(u64::from(insect_web.id().get()), 1)]
    );

    assert_eq!(
        choice("universe.occurrence.13.variant.11001.choice.01")
            .external_results()
            .len(),
        63
    );
    assert_eq!(
        choice("universe.occurrence.13.variant.11001.choice.02")
            .external_results()
            .len(),
        27
    );
    assert_eq!(
        choice("universe.occurrence.17.variant.11401.choice.01")
            .external_results()
            .len(),
        15
    );
    assert_eq!(
        choice("universe.occurrence.18.variant.11501.choice.01")
            .external_results()
            .len(),
        46
    );
    let pigs = choice("universe.occurrence.16.variant.11301.choice.01");
    assert!(pigs.battle_member().is_some());
    assert_eq!(pigs.immediate_operations(), 1);
}

#[test]
fn goal07_p4_m13_s03_executes_exact_blessing_curio_enhancement_and_fragment_costs() {
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

    let nomadic = choice("universe.occurrence.2.variant.10101.choice.01");
    assert_eq!(nomadic.random_candidate_count(), Some(162));
    assert_eq!(nomadic.deferred_operations(), 0);
    assert_eq!(
        choice("universe.occurrence.2.variant.10101.choice.02")
            .external_results()
            .len(),
        7
    );

    let kindling = choice("universe.occurrence.19.variant.11601.choice.01");
    assert_eq!(kindling.random_candidate_count(), Some(135));
    assert_eq!(kindling.deferred_operations(), 0);
    let activity =
        execute_occurrence_payload_with_fragments(&compiled, kindling.payload(), 99_301, 500);
    let player = activity.player_view();
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

    let merchant = choice("universe.occurrence.20.variant.11701.choice.01");
    assert_eq!(merchant.external_results().len(), 72);
    let activity = execute_occurrence_payload_with_fragments(
        &compiled,
        merchant.external_results()[0].payload(),
        99_302,
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
        choice("universe.occurrence.20.variant.11701.choice.02")
            .external_results()
            .len(),
        15
    );
    assert_eq!(
        choice("universe.occurrence.20.variant.11701.choice.04")
            .external_results()
            .len(),
        46
    );
    assert_eq!(
        choice("universe.occurrence.20.variant.11701.choice.05")
            .external_results()
            .len(),
        162
    );
    assert_eq!(
        choice("universe.occurrence.20.variant.11701.choice.08")
            .external_results()
            .len(),
        27
    );
    assert_eq!(
        choice("universe.occurrence.20.variant.11701.choice.07").random_candidate_count(),
        Some(162)
    );
    assert_eq!(
        choice("universe.occurrence.19.variant.11601.choice.02").immediate_operations(),
        1
    );
}

fn execute_occurrence_payload(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    outcome: u32,
) -> GraphActivity {
    execute_occurrence_payload_with_fragments(compiled, payload, outcome, 50)
}

fn execute_occurrence_payload_with_fragments(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    payload: &[u8],
    outcome: u32,
    initial_fragments: i64,
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
    let mut activity =
        occurrence_harness_with_fragments(compiled, &binding, registry, initial_fragments);
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

fn occurrence_harness(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    source: &ActivityInteractionBinding,
    registry: &Arc<starclock_activity::ActivityHandlerRegistry>,
) -> GraphActivity {
    occurrence_harness_with_fragments(compiled, source, registry, 50)
}

fn occurrence_harness_with_fragments(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    source: &ActivityInteractionBinding,
    registry: &Arc<starclock_activity::ActivityHandlerRegistry>,
    initial_fragments: i64,
) -> GraphActivity {
    occurrence_harness_with_fragments_and_seed(compiled, source, registry, initial_fragments, 9_001)
}

fn occurrence_harness_with_fragments_and_seed(
    compiled: &starclock_mode_universe::entry::CompiledActivity,
    source: &ActivityInteractionBinding,
    registry: &Arc<starclock_activity::ActivityHandlerRegistry>,
    initial_fragments: i64,
    master_seed: u64,
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
        compiled.occurrence_interaction_state_slot(),
        compiled.curio_state_slot(),
        compiled.curio_charge_slot(),
        compiled.curio_event_slot(),
        compiled.selected_path_slot(),
    ];
    let state = ActivityStateDefinition::new(
        compiled
            .state_definition()
            .slots()
            .iter()
            .filter(|slot| required_slots.contains(&slot.id()))
            .map(|slot| {
                if slot.id() == compiled.cosmic_fragments_slot() {
                    ActivitySlotDefinition::new_with_policy(
                        slot.id(),
                        slot.owner(),
                        ActivityValue::BoundedInteger(initial_fragments),
                        Some((0, MAX_COSMIC_FRAGMENTS)),
                        slot.maximum_entries(),
                        slot.resets().to_vec(),
                        slot.carry(),
                        slot.visibility(),
                        slot.source().unwrap(),
                    )
                    .unwrap()
                } else {
                    slot.clone()
                }
            })
            .collect(),
        compiled
            .state_definition()
            .inventories()
            .iter()
            .filter(|inventory| {
                matches!(
                    inventory.id(),
                    id if id == compiled.blessing_inventory()
                        || id == compiled.formation_inventory()
                        || id == compiled.curio_inventory()
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
        ActivityMasterSeed::from_u64(master_seed),
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
