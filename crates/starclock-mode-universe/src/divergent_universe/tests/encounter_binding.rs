//! Address-independent production handoffs, not original room admission or
//! complete Persona-run evidence. Readdressing is confined to these tests.

use std::{collections::BTreeMap, sync::Arc};

use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBattleAssemblyError, DivergentUniverseBattleAssemblyPolicy,
    DivergentUniverseCyclicalRefresh, DivergentUniverseEntry, DivergentUniverseFlowInstance,
    DivergentUniverseLogicalScopeKind, tests::instance,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityRandomOffer, ActivityRandomPolicies, ActivityTerminalOutcome, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram, LogicalScopeAddress,
    LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

fn compile_flow(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    tawot: bool,
) -> DivergentUniverseFlowInstance {
    let base = fixture.flow(family).unwrap();
    let mut entry = DivergentUniverseEntry::new(
        base.area().clone(),
        base.difficulty().clone(),
        Arc::clone(fixture.participants()),
        base.input_snapshot().clone(),
        Vec::new(),
    )
    .unwrap()
    .with_mapping_snapshot(Arc::clone(fixture.mapping()))
    .with_layer_battle_route();
    if let Some(challenge) = base.cyclical_challenge() {
        entry = entry.with_cyclical_refresh(
            DivergentUniverseCyclicalRefresh::new(1, challenge.clone()).unwrap(),
        );
    }
    if tawot {
        entry = entry.with_initial_tawot_service(2);
    }
    fixture.factory().compile(entry).unwrap()
}

fn readdress(
    mut flow: DivergentUniverseFlowInstance,
    nested: bool,
) -> DivergentUniverseFlowInstance {
    let old = Arc::clone(flow.definition());
    let mut addresses = BTreeMap::new();
    for node in old.graph().nodes() {
        if let Some((battle, plane)) = flow.encounter_destination(node.id()) {
            let base = 100_000 + 10_000 * plane.get();
            addresses.insert(node.id(), NodeId::new(base + 100).unwrap());
            addresses.insert(battle, NodeId::new(base + 101).unwrap());
        }
    }
    let remap = |node| addresses.get(&node).copied().unwrap_or(node);
    let nodes = old
        .graph()
        .nodes()
        .iter()
        .map(|node| {
            ActivityNodeDefinition::new(
                remap(node.id()),
                node.section(),
                node.kind(),
                node.maximum_visits(),
            )
            .unwrap()
        })
        .collect();
    let edges = old
        .graph()
        .edges()
        .iter()
        .map(|edge| {
            ActivityEdgeDefinition::new(
                edge.id(),
                remap(edge.from()),
                remap(edge.to()),
                edge.condition(),
                edge.priority(),
                edge.maximum_traversals(),
            )
            .unwrap()
        })
        .collect();
    let graph = ActivityGraphDefinition::new(
        remap(old.graph().entry()),
        nodes,
        edges,
        old.graph().maximum_total_visits(),
    )
    .unwrap();
    let scopes = old.state_definition().logical_scopes();
    let bindings = scopes
        .bindings()
        .iter()
        .map(|binding| {
            let node = remap(binding.node());
            let mut path = binding.path().to_vec();
            if nested && graph.node(node).unwrap().kind() == ActivityNodeKind::Battle {
                path.push(
                    LogicalScopeAddress::new(
                        DivergentUniverseLogicalScopeKind::Battle.class_id(),
                        u64::from(node.get()),
                    )
                    .unwrap(),
                );
            }
            LogicalScopeNodeBinding::new(node, path).unwrap()
        })
        .collect();
    let scopes = LogicalScopeDefinitions::new(scopes.classes().to_vec(), bindings).unwrap();
    let programs = old
        .programs()
        .iter()
        .map(|program| {
            GraphActivityNodeProgram::new(remap(program.node()), program.program().clone())
        })
        .collect();
    let offers = old
        .random_offers()
        .iter()
        .map(|offer| {
            if !addresses.contains_key(&offer.node()) {
                return offer.clone();
            }
            // These are the exact simple production EncounterPolicies. Fail if the
            // owning author later adds semantics this test remapping cannot retain.
            assert_eq!(offer.reroll_counter(), None);
            assert!(offer.maximum_options_reductions().is_empty());
            assert!(offer.inactive_condition().is_none());
            assert!(offer.conditional_weight_multipliers().is_empty());
            assert!(offer.conditional_candidate_filters().is_empty());
            assert!(offer.selected_option_marker().is_none());
            assert!(offer.selection_prefix().is_empty());
            ActivityRandomOffer::new(
                remap(offer.node()),
                offer.label(),
                offer.purpose(),
                offer.maximum_options(),
                offer.weights().to_vec(),
                None,
            )
            .unwrap()
        })
        .collect();
    assert!(old.random_checkpoints().is_empty());
    assert!(old.interactions().is_none());
    flow.definition = Arc::new(
        GraphActivityDefinition::new(
            old.identity(),
            graph,
            old.state_definition().clone().with_logical_scopes(scopes),
            Arc::clone(old.participants()),
            programs,
            old.bootstrap().cloned(),
            ActivityRandomPolicies::new(Vec::new(), offers),
        )
        .unwrap(),
    );
    flow
}

fn start(flow: &DivergentUniverseFlowInstance) -> GraphActivity {
    flow.start(instance(24230), ActivityMasterSeed::from_u64(24230))
        .unwrap()
        .into_activity()
}

#[test]
fn encounter_binding_readdressed_first_later_and_tawot_nodes_execute_real_battles() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let policy = fixture.policy().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for tawot in [false, true] {
            for nested in [false, true] {
                let flow = readdress(compile_flow(&fixture, family, tawot), nested);
                let fresh = readdress(compile_flow(&fixture, family, tawot), nested);
                let mut activity = start(&flow);
                let mut reconstructed = start(&fresh);
                let mut stages = Vec::new();
                while activity.player_view().terminal().is_none() {
                    let view = activity.player_view();
                    if view.decision().unwrap().kind() == ActivityDecisionKind::Encounter {
                        assert!(activity.current_node().get() > 100_000);
                        let (battle, section) =
                            flow.encounter_destination(activity.current_node()).unwrap();
                        assert!(battle.get() > 100_000);
                        let observed = flow.offered_encounter(&activity).unwrap().unwrap();
                        let before = activity.canonical_state_bytes();
                        let debug = activity.debug_view();
                        assert_eq!(
                            flow.offered_encounter(&activity).unwrap().unwrap(),
                            observed
                        );
                        assert_eq!(activity.canonical_state_bytes(), before);
                        assert_eq!(activity.debug_view(), debug);
                        let stage = observed.1.to_owned();
                        let pool = flow.encounter_pool_policy().unwrap();
                        if section.get() == 1 {
                            assert_eq!(stage, pool.first_stage.as_ref());
                        } else {
                            assert!(
                                pool.candidate_stages
                                    .iter()
                                    .any(|candidate| candidate.as_ref() == stage)
                            );
                        }
                        stages.push(stage);
                    }
                    let actual = runner
                        .advance(
                            fixture.factory(),
                            &flow,
                            &mut activity,
                            fixture.core(),
                            &policy,
                        )
                        .unwrap();
                    let replayed = runner
                        .advance(
                            fixture.factory(),
                            &fresh,
                            &mut reconstructed,
                            fixture.core(),
                            &policy,
                        )
                        .unwrap();
                    assert_eq!(actual, replayed);
                    assert_eq!(
                        activity.canonical_state_bytes(),
                        reconstructed.canonical_state_bytes()
                    );
                }
                assert_eq!(stages.len(), 3);
                assert_eq!(activity.player_view().completed_battle_count(), 3);
                assert_eq!(
                    activity.player_view().terminal(),
                    Some(ActivityTerminalOutcome::Completed)
                );
            }
        }
    }
}

fn replace_definition(
    flow: &mut DivergentUniverseFlowInstance,
    graph: ActivityGraphDefinition,
    scopes: LogicalScopeDefinitions,
) {
    let old = flow.definition();
    flow.definition = Arc::new(
        GraphActivityDefinition::new(
            old.identity(),
            graph,
            old.state_definition().clone().with_logical_scopes(scopes),
            Arc::clone(old.participants()),
            old.programs().to_vec(),
            old.bootstrap().cloned(),
            ActivityRandomPolicies::new(
                old.random_checkpoints().to_vec(),
                old.random_offers().to_vec(),
            ),
        )
        .unwrap(),
    );
}

#[test]
fn encounter_binding_malformed_target_paths_reject_before_rng_state_or_cache_changes() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let base = readdress(
        compile_flow(&fixture, DivergentUniverseRunFamily::Ordinary, false),
        true,
    );
    let current = base.definition().graph().entry();
    let (battle, _) = base.encounter_destination(current).unwrap();
    let assembly = fixture.factory().battle_assembly_runtime();
    for corruption in 0..8 {
        let mut flow = base.clone();
        let scopes = flow.definition().state_definition().logical_scopes();
        let mut bindings = scopes.bindings().to_vec();
        let mut nodes = flow.definition().graph().nodes().to_vec();
        let mut edges = flow.definition().graph().edges().to_vec();
        match corruption {
            0 => bindings.retain(|binding| binding.node() != current),
            1 => bindings.retain(|binding| binding.node() != battle),
            2 | 3 | 6 | 7 => {
                for binding in &mut bindings {
                    if binding.node() == battle || (corruption != 2 && binding.node() == current) {
                        let mut path = binding.path().to_vec();
                        match corruption {
                            2 => {
                                path[2] = LogicalScopeAddress::new(
                                    DivergentUniverseLogicalScopeKind::Node.class_id(),
                                    2,
                                )
                                .unwrap()
                            }
                            3 => {
                                path[1] = LogicalScopeAddress::new(
                                    DivergentUniverseLogicalScopeKind::Plane.class_id(),
                                    2,
                                )
                                .unwrap()
                            }
                            6 => {
                                path[0] = LogicalScopeAddress::new(
                                    DivergentUniverseLogicalScopeKind::Run.class_id(),
                                    2,
                                )
                                .unwrap()
                            }
                            7 => path.truncate(2),
                            _ => unreachable!(),
                        }
                        *binding = LogicalScopeNodeBinding::new(binding.node(), path).unwrap();
                    }
                }
            }
            4 => {
                let node = nodes.iter_mut().find(|node| node.id() == battle).unwrap();
                *node = ActivityNodeDefinition::new(
                    node.id(),
                    SectionId::new(2).unwrap(),
                    node.kind(),
                    node.maximum_visits(),
                )
                .unwrap();
            }
            5 => edges.push(
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(999_001).unwrap(),
                    current,
                    battle,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap(),
            ),
            _ => unreachable!(),
        }
        let graph = ActivityGraphDefinition::new(
            current,
            nodes,
            edges,
            flow.definition().graph().maximum_total_visits(),
        )
        .unwrap();
        let scopes = LogicalScopeDefinitions::new(scopes.classes().to_vec(), bindings).unwrap();
        replace_definition(&mut flow, graph, scopes);
        let activity = start(&flow);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let metrics = assembly.cache_metrics().unwrap();
        assert!(
            flow.encounter_destination(current).is_none(),
            "{corruption}"
        );
        assert!(flow.offered_encounter(&activity).is_err(), "{corruption}");
        let pool = flow.encounter_pool_policy().unwrap();
        let encounter = fixture
            .factory()
            .encounter_reachability_runtime()
            .unwrap()
            .select_stage_candidate(
                &activity,
                activity.state_hash(),
                &pool.encounter_group,
                &pool.first_stage,
            )
            .unwrap();
        let contribution = fixture
            .factory()
            .contribution_snapshot_runtime()
            .unwrap()
            .snapshot(&flow, &activity)
            .unwrap();
        assert!(matches!(assembly.resolve_current_battle(
            &flow, &activity, fixture.core(), &contribution, &encounter,
            DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        ), Err(DivergentUniverseBattleAssemblyError::InvalidEncounter)), "{corruption}");
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        assert_eq!(assembly.cache_metrics().unwrap(), metrics);
    }
}

#[test]
fn encounter_binding_foreign_logical_definition_cannot_reuse_identity_and_graph() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = readdress(
        compile_flow(&fixture, DivergentUniverseRunFamily::Ordinary, false),
        true,
    );
    let current = flow.definition().graph().entry();
    let (battle, _) = flow.encounter_destination(current).unwrap();
    let mut foreign = flow.clone();
    let scopes = foreign.definition().state_definition().logical_scopes();
    let mut bindings = scopes.bindings().to_vec();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.node() == battle)
        .unwrap();
    let mut path = binding.path().to_vec();
    path[2] =
        LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Node.class_id(), 2).unwrap();
    *binding = LogicalScopeNodeBinding::new(battle, path).unwrap();
    let scopes = LogicalScopeDefinitions::new(scopes.classes().to_vec(), bindings).unwrap();
    let graph = foreign.definition().graph().clone();
    replace_definition(&mut foreign, graph, scopes);
    assert_eq!(
        flow.definition().identity(),
        foreign.definition().identity()
    );
    assert_eq!(
        flow.definition().graph().digest(),
        foreign.definition().graph().digest()
    );
    let activity = start(&foreign);
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    assert!(flow.offered_encounter(&activity).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view(), debug);
}
