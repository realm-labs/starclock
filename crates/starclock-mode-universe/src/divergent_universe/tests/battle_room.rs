//! Real source-position battle fragments; other payloads are explicit probes.
//! Placement and stage/domain inputs are controlled caller policies, not original
//! room admission, source-specific bosses or complete-run release evidence.

use std::sync::Arc;

use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseBattleAssemblyError,
    DivergentUniverseBattleAssemblyPolicy, DivergentUniverseBattleStart,
    DivergentUniverseCyclicalRefresh, DivergentUniverseEntry, DivergentUniverseFlowInstance,
    DivergentUniverseRoomPolicy,
    battle_room::{BattleRoomError, BattleRoomSelection, CompiledBattleRoom},
    domain_deck::DomainDeckSlots,
    domain_route::{
        CompiledDomainRoute, DomainRoomComposition, DomainRoomContext, DomainRoomProgram,
        DomainRouteError,
    },
    economy::DivergentUniverseCurrencyKind,
    tests::{curio_battle_grants::project, currency_balance, instance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityExpression,
    ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomPolicies, ActivitySlotId, ActivityStateDefinition, ActivityTerminalOutcome,
    ActivityValue, AttemptId, BattleOutcome, BattleResult, BattleSequence, GraphActivity,
    GraphActivityDefinition, GraphActivityNodeProgram, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, ProjectedValue,
};
use starclock_combat::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_layout::FixedDomainKind,
};

#[path = "battle_room_rewards.rs"]
mod rewards;

const SLOTS: DomainDeckSlots = DomainDeckSlots {
    draw: slot(66),
    discard: slot(67),
    selected: slot(68),
    accepted: slot(69),
};
const fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn payload() -> ActivityConfigDigest {
    // Versioned test-only owner identity for the explicit non-battle probes,
    // selected first authored deck, width three and the slot namespace above.
    ActivityConfigDigest::new([0x31; 32]).unwrap()
}
fn base(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseFlowInstance {
    let original = fixture.flow(family).unwrap();
    let mut entry = DivergentUniverseEntry::new(
        original.area().clone(),
        original.difficulty().clone(),
        Arc::clone(fixture.participants()),
        original.input_snapshot().clone(),
        Vec::new(),
    )
    .unwrap()
    .with_mapping_snapshot(Arc::clone(fixture.mapping()))
    .with_runtime_battle_route();
    if let Some(challenge) = original.cyclical_challenge() {
        entry = entry.with_cyclical_refresh(
            DivergentUniverseCyclicalRefresh::new(1, challenge.clone()).unwrap(),
        );
    }
    fixture.factory().compile(entry).unwrap()
}
fn probe(context: &DomainRoomContext) -> Result<DomainRoomProgram, DomainRouteError> {
    let node = context.entry_node();
    Ok(DomainRoomProgram {
        exit_node: node,
        nodes: vec![
            ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                .unwrap(),
        ],
        edges: Vec::new(),
        random_offers: Vec::new(),
        programs: vec![GraphActivityNodeProgram::new(
            node,
            ActivityProgramDefinition::new(
                ActivityProgramId::new(node.get()).unwrap(),
                vec![ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Service,
                    options: vec![ActivityOptionDefinition::new(
                        ActivityOptionId::new(1).unwrap(),
                        0,
                        ActivityCondition::Boolean(ActivityExpression::Literal(
                            ActivityValue::Boolean(true),
                        )),
                        vec![ActivityOperation::Traverse(context.exit_edge())],
                    )]
                    .into_boxed_slice(),
                }],
            )
            .unwrap(),
        )],
    })
}
struct Scenario {
    base: DivergentUniverseFlowInstance,
    flow: DivergentUniverseFlowInstance,
    route: CompiledDomainRoute,
    rooms: Vec<CompiledBattleRoom>,
}
fn compile_scenario(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> Scenario {
    compile_rooms(
        fixture,
        family,
        &[
            BattleRewardDomain::Combat,
            BattleRewardDomain::Elite,
            BattleRewardDomain::Aberration,
        ],
        |context| {
            if context.composition == DomainRoomComposition::Fixed(FixedDomainKind::Boss) {
                // This controlled scenario deliberately retains an independent
                // Combat proxy even though a Boss reward policy is available.
                Some(0)
            } else if context.position_ordinal <= 3
                && context.composition != DomainRoomComposition::Fixed(FixedDomainKind::Respite)
            {
                Some(usize::from(context.position_ordinal - 1))
            } else {
                None
            }
        },
    )
}

fn compile_rooms(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    domains: &[BattleRewardDomain],
    mut placement: impl FnMut(&DomainRoomContext) -> Option<usize>,
) -> Scenario {
    let base = base(fixture, family);
    let factory = fixture.factory();
    let policy = factory.decision_catalog().encounter_pool();
    let compilers = domains
        .iter()
        .copied()
        .enumerate()
        .map(|(index, domain)| {
            factory
                .battle_room_compiler(BattleRoomSelection {
                    group: policy.encounter_group.clone(),
                    stage: policy.candidate_stages[index].clone(),
                    domain,
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    let deck = &factory.decision_catalog().domain_decks()[0];
    let mut rooms = Vec::new();
    let route = factory
        .compile_domain_route(base.area(), &deck.key, 3, SLOTS, |context| {
            // Independent explicit placement; do not interpret composition level as
            // a battle count or these candidates as original Boss membership.
            let index = placement(context);
            if let Some(index) = index {
                let room = compilers[index].compile(context).unwrap();
                let fragment = room.fragment().clone();
                rooms.push(room);
                Ok(fragment)
            } else {
                probe(context)
            }
        })
        .unwrap();
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    let identity = factory
        .battle_room_identity(&base, &route.graph, &rooms, payload())
        .unwrap();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            route.graph.clone(),
            ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                .unwrap()
                .with_logical_scopes(route.logical_scopes.clone()),
            Arc::clone(base.definition().participants()),
            route.programs.clone(),
            None,
            ActivityRandomPolicies::new(Vec::new(), route.random_offers.clone()),
        )
        .unwrap(),
    );
    let flow = factory
        .bind_battle_rooms(base.clone(), definition, &rooms, payload())
        .unwrap();
    Scenario {
        base,
        flow,
        route,
        rooms,
    }
}
fn start(scenario: &Scenario) -> GraphActivity {
    scenario
        .flow
        .start(instance(24240), ActivityMasterSeed::from_u64(24240))
        .unwrap()
        .into_activity()
}
fn advance(
    fixture: &DivergentUniverseBaselineFixture,
    scenario: &Scenario,
    activity: &mut GraphActivity,
) -> Option<DivergentUniverseBaselineStep> {
    let offer = activity.player_view().decision().unwrap().clone();
    if offer.kind() == ActivityDecisionKind::Route {
        let hash = activity.state_hash();
        scenario
            .route
            .deck
            .choose(activity, hash, offer.id(), offer.options()[0].id())
            .unwrap();
        None
    } else {
        Some(
            DivergentUniverseBaselineRunner::default()
                .advance(
                    fixture.factory(),
                    &scenario.flow,
                    activity,
                    fixture.core(),
                    &fixture.policy().unwrap(),
                )
                .unwrap(),
        )
    }
}

#[test]
fn battle_room_source_positions_execute_multiple_real_battles_with_exact_drops_natural_defeat_and_fresh_carry()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let scenario = compile_scenario(&fixture, family);
        let fresh = compile_scenario(&fixture, family);
        let mut activity = start(&scenario);
        let mut reconstructed = start(&fresh);
        assert_ne!(
            scenario.base.definition().identity(),
            scenario.flow.definition().identity()
        );
        assert_eq!(
            scenario.flow.room_policy(),
            DivergentUniverseRoomPolicy::ExplicitSourcePositionProgramsNoAutomaticAdmission
        );
        let currency = scenario
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let mut battles = Vec::new();
        let mut domains = Vec::new();
        while activity.player_view().terminal().is_none() {
            let view = activity.player_view();
            let is_battle = view.decision().unwrap().kind() == ActivityDecisionKind::Encounter;
            let before = currency_balance(&activity, currency);
            let domain = if is_battle {
                let domain = scenario
                    .flow
                    .current_battle_domain(&activity)
                    .unwrap()
                    .unwrap();
                domains.push(domain);
                let room = scenario
                    .rooms
                    .iter()
                    .find(|room| room.context().node(1).unwrap() == activity.current_node())
                    .unwrap();
                assert_eq!(domain, room.selection().domain);
                assert_eq!(
                    scenario.flow.offered_encounter(&activity).unwrap().unwrap(),
                    (&room.selection().group, room.selection().stage.as_ref())
                );
                let (battle, _) = scenario
                    .flow
                    .encounter_destination(activity.current_node())
                    .unwrap();
                assert!(!battles.contains(&battle));
                battles.push(battle);
                Some(domain)
            } else {
                None
            };
            let step = advance(&fixture, &scenario, &mut activity);
            let replayed = advance(&fixture, &fresh, &mut reconstructed);
            assert_eq!(step, replayed);
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
            assert_eq!(
                activity.player_view().participant_carry(),
                reconstructed.player_view().participant_carry()
            );
            if let Some(domain) = domain {
                let Some(DivergentUniverseBaselineStep::Battle { outcome, .. }) = step else {
                    panic!("an authenticated encounter must execute a real battle");
                };
                if outcome == BattleOutcome::Lost {
                    assert_eq!(currency_balance(&activity, currency), before);
                    assert_eq!(
                        activity.player_view().terminal(),
                        Some(ActivityTerminalOutcome::Failed)
                    );
                    continue;
                }
                assert_eq!(outcome, BattleOutcome::Won);
                let drop = fixture
                    .factory()
                    .decision_catalog()
                    .battle_fragments()
                    .iter()
                    .find(|definition| definition.domain == domain)
                    .unwrap()
                    .amount;
                assert_eq!(
                    currency_balance(&activity, currency),
                    before + i64::try_from(drop).unwrap()
                );
                assert_eq!(
                    scenario.flow.current_battle_domain(&activity).unwrap(),
                    Some(domain)
                );
                assert_eq!(
                    activity.player_view().decision().unwrap().kind(),
                    ActivityDecisionKind::Reward
                );
            }
        }
        // The unchanged baseline party naturally loses battle six without room
        // heals. Never fabricate victory or heal carry to traverse the profile.
        assert_eq!(battles.len(), 6);
        assert_eq!(activity.player_view().completed_battle_count(), 6);
        assert!(domains.contains(&BattleRewardDomain::Combat));
        assert!(domains.contains(&BattleRewardDomain::Elite));
        assert!(domains.contains(&BattleRewardDomain::Aberration));
        assert!(!domains.contains(&BattleRewardDomain::Boss));
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Failed)
        );
    }
}

#[test]
fn battle_room_suppressed_reward_advances_to_authenticated_deck_offer_and_rejects_wrong_stage_before_cache()
 {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let scenario = compile_scenario(&fixture, DivergentUniverseRunFamily::Ordinary);
    let mut activity = start(&scenario);
    let factory = fixture.factory();
    let curios = factory.curio_runtime().unwrap();
    let state = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9055").unwrap();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state)
        .unwrap();
    let (group, stage) = scenario.flow.offered_encounter(&activity).unwrap().unwrap();
    let wrong = factory
        .decision_catalog()
        .encounter_pool()
        .candidate_stages
        .iter()
        .find(|candidate| candidate.as_ref() != stage)
        .unwrap();
    let selected = factory
        .encounter_reachability_runtime()
        .unwrap()
        .select_stage_candidate(&activity, activity.state_hash(), group, wrong)
        .unwrap();
    let contribution = factory
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&scenario.flow, &activity)
        .unwrap();
    let assembly = factory.battle_assembly_runtime();
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    let metrics = assembly.cache_metrics().unwrap();
    assert!(matches!(assembly.resolve_current_battle(
        &scenario.flow, &activity, fixture.core(), &contribution, &selected,
        DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
    ), Err(DivergentUniverseBattleAssemblyError::InvalidEncounter)));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view(), debug);
    assert_eq!(assembly.cache_metrics().unwrap(), metrics);
    let draws = reward_draws(&activity);
    advance(&fixture, &scenario, &mut activity);
    assert_eq!(activity.player_view().completed_battle_count(), 1);
    assert_eq!(
        activity.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Route
    );
    assert_eq!(reward_draws(&activity), draws);
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    assert!(
        activity
            .choose_option(activity.state_hash(), offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    advance(&fixture, &scenario, &mut activity);
    assert_eq!(
        activity.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Encounter
    );
}

#[test]
fn battle_room_consecutive_rooms_preserve_handoff_carry_and_reject_prior_results() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let scenario = compile_scenario(&fixture, family);
        let mut activity = start(&scenario);
        let first = prepare(&fixture, &scenario, &mut activity);
        let (result, report) = first.execute().unwrap();
        let stale = result.clone();
        let hash = activity.state_hash();
        fixture
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(&scenario.flow, &mut activity, hash, result, Some(report))
            .unwrap();
        advance(&fixture, &scenario, &mut activity); // authenticated Blessing
        advance(&fixture, &scenario, &mut activity); // authenticated next card
        let previous = activity.player_view().participant_carry().to_vec();
        assert_eq!(previous.len(), 4);
        let second = prepare(&fixture, &scenario, &mut activity);
        for current in second.handoff().participant_carry() {
            let prior = previous
                .iter()
                .find(|prior| prior.participant() == current.participant())
                .unwrap();
            assert_eq!(
                current.current_hp(),
                prior.current_hp().min(current.maximum_hp())
            );
            assert_eq!(
                current.current_energy(),
                prior.current_energy().min(current.maximum_energy())
            );
            assert_eq!(current.life(), prior.life());
            assert_eq!(current.presence(), prior.presence());
        }
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let hash = activity.state_hash();
        assert!(
            fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&scenario.flow, &mut activity, hash, stale, None,)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        let (result, report) = second.execute().unwrap();
        let hash = activity.state_hash();
        fixture
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(&scenario.flow, &mut activity, hash, result, Some(report))
            .unwrap();
        assert_eq!(activity.player_view().completed_battle_count(), 2);
        assert_eq!(
            scenario.flow.current_battle_domain(&activity).unwrap(),
            Some(BattleRewardDomain::Elite)
        );
    }
}

fn prepare(
    fixture: &DivergentUniverseBaselineFixture,
    scenario: &Scenario,
    activity: &mut GraphActivity,
) -> DivergentUniverseBattleStart {
    let factory = fixture.factory();
    let hash = activity.state_hash();
    let (group, stage) = scenario.flow.offered_encounter(activity).unwrap().unwrap();
    let encounter = factory
        .encounter_reachability_runtime()
        .unwrap()
        .select_stage_candidate(activity, hash, group, stage)
        .unwrap();
    let contribution = factory
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&scenario.flow, activity)
        .unwrap();
    let assembled = factory.battle_assembly_runtime().materialize_current_battle(
        &scenario.flow, activity, fixture.core(), &contribution, &encounter,
        DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
    ).unwrap();
    let sequence = activity.player_view().completed_battle_count() + 1;
    factory
        .battle_settlement_runtime()
        .start_current_battle(
            &scenario.flow,
            activity,
            hash,
            AttemptId::new(sequence).unwrap(),
            BattleSequence::new(sequence).unwrap(),
            &assembled,
        )
        .unwrap()
}

#[test]
fn battle_room_profile_binding_rejects_missing_changed_foreign_and_legacy_inputs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let scenario = compile_scenario(&fixture, DivergentUniverseRunFamily::Ordinary);
    let factory = fixture.factory();
    let definition = scenario.flow.definition();
    assert!(
        factory
            .bind_battle_rooms(
                scenario.base.clone(),
                Arc::clone(definition),
                &scenario.rooms[..1],
                payload()
            )
            .is_err()
    );
    assert!(
        factory
            .bind_battle_rooms(
                fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap(),
                Arc::clone(definition),
                &scenario.rooms,
                payload()
            )
            .is_err()
    );
    assert!(
        factory
            .bind_battle_rooms(
                scenario.base.clone(),
                Arc::clone(definition),
                &scenario.rooms,
                ActivityConfigDigest::new([0x32; 32]).unwrap()
            )
            .is_err()
    );
    let mut reordered = scenario.rooms.clone();
    reordered.reverse();
    assert_eq!(
        factory
            .battle_room_identity(&scenario.base, &scenario.route.graph, &reordered, payload())
            .unwrap(),
        definition.identity()
    );
    assert!(
        factory
            .bind_battle_rooms(
                scenario.base.clone(),
                Arc::clone(definition),
                &reordered,
                payload()
            )
            .is_ok()
    );
    for corruption in 0..3 {
        let mut programs = definition.programs().to_vec();
        let mut slots = definition.state_definition().slots().to_vec();
        let mut bindings = definition
            .state_definition()
            .logical_scopes()
            .bindings()
            .to_vec();
        let room = &scenario.rooms[0];
        match corruption {
            0 => {
                let index = programs
                    .iter()
                    .position(|program| program.node() == room.context().entry_node())
                    .unwrap();
                programs[index] = GraphActivityNodeProgram::new(
                    programs[index].node(),
                    ActivityProgramDefinition::new(
                        programs[index].program().id(),
                        vec![ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Literal(ActivityValue::Boolean(false)),
                        ))],
                    )
                    .unwrap(),
                );
            }
            1 => slots.retain(|slot| slot.id().get() != 62),
            2 => {
                let binding = bindings
                    .iter_mut()
                    .find(|binding| binding.node() == room.context().node(2).unwrap())
                    .unwrap();
                *binding =
                    LogicalScopeNodeBinding::new(binding.node(), binding.path()[..3].to_vec())
                        .unwrap();
            }
            _ => unreachable!(),
        }
        let scopes = LogicalScopeDefinitions::new(
            definition
                .state_definition()
                .logical_scopes()
                .classes()
                .to_vec(),
            bindings,
        )
        .unwrap();
        let changed = Arc::new(
            GraphActivityDefinition::new(
                definition.identity(),
                definition.graph().clone(),
                ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                    .unwrap()
                    .with_logical_scopes(scopes),
                Arc::clone(definition.participants()),
                programs,
                None,
                ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
            )
            .unwrap(),
        );
        assert!(matches!(
            factory.bind_battle_rooms(scenario.base.clone(), changed, &scenario.rooms, payload()),
            Err(BattleRoomError::InvalidDefinition)
        ));
    }
    let room = &scenario.rooms[0];
    let compiler = factory
        .battle_room_compiler(room.selection().clone())
        .unwrap();
    for corruption in 0..3 {
        let mut context = room.context().clone();
        match corruption {
            0 => context.level += 1,
            1 => context.position_ordinal = 0,
            2 => context.plane_ordinal = 99,
            _ => unreachable!(),
        }
        assert!(matches!(
            compiler.compile(&context),
            Err(BattleRoomError::InvalidContext)
        ));
    }
    let mut selection = room.selection().clone();
    selection.stage = "missing-stage".into();
    assert!(matches!(
        factory.battle_room_compiler(selection),
        Err(BattleRoomError::InvalidEncounter)
    ));
}

#[test]
fn battle_room_counterfactual_loss_and_fault_take_typed_terminals_without_rewards() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for (outcome, terminal) in [
            (BattleOutcome::Lost, ActivityTerminalOutcome::Failed),
            (BattleOutcome::Faulted, ActivityTerminalOutcome::Faulted),
        ] {
            let scenario = compile_scenario(&fixture, family);
            let mut activity = start(&scenario);
            let currency = scenario
                .flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            let before = currency_balance(&activity, currency);
            let draws = reward_draws(&activity);
            let (actual, _) = prepare(&fixture, &scenario, &mut activity)
                .execute()
                .unwrap();
            // Controlled terminal projection verifies routing, not natural loss
            // or original Boss fault behavior.
            let projected = project(&actual, 0, outcome);
            let projected = BattleResult::seal(
                projected.identity(),
                projected
                    .values()
                    .iter()
                    .map(|value| {
                        if matches!(value, ProjectedValue::TerminalFault(_))
                            && outcome == BattleOutcome::Faulted
                        {
                            ProjectedValue::TerminalFault(Some(BattleFault::from_parts(
                                FaultKind::Numeric,
                                FaultBoundary::Command,
                                FaultPolicy::Rollback,
                                1,
                                None,
                            )))
                        } else {
                            value.clone()
                        }
                    })
                    .collect(),
            );
            let hash = activity.state_hash();
            fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&scenario.flow, &mut activity, hash, projected.clone(), None)
                .unwrap();
            assert_eq!(activity.player_view().terminal(), Some(terminal));
            assert_eq!(currency_balance(&activity, currency), before);
            assert_eq!(reward_draws(&activity), draws);
            let bytes = activity.canonical_state_bytes();
            let debug = activity.debug_view();
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&scenario.flow, &mut activity, hash, projected, None)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(activity.debug_view(), debug);
        }
    }
}

#[test]
fn battle_room_observation_rejects_foreign_programs_even_with_matching_identity_and_graph() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let scenario = compile_scenario(&fixture, DivergentUniverseRunFamily::Ordinary);
    let definition = scenario.flow.definition();
    let mut programs = definition.programs().to_vec();
    let probe = programs
        .iter_mut()
        .find(|program| {
            !scenario.rooms.iter().any(|room| {
                room.fragment()
                    .nodes
                    .iter()
                    .any(|node| node.id() == program.node())
            }) && program.node() != definition.graph().entry()
        })
        .unwrap();
    *probe = GraphActivityNodeProgram::new(
        probe.node(),
        ActivityProgramDefinition::new(
            probe.program().id(),
            vec![ActivityOperation::Require(ActivityCondition::Boolean(
                ActivityExpression::Literal(ActivityValue::Boolean(false)),
            ))],
        )
        .unwrap(),
    );
    let changed = Arc::new(
        GraphActivityDefinition::new(
            definition.identity(),
            definition.graph().clone(),
            definition.state_definition().clone(),
            Arc::clone(definition.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
        )
        .unwrap(),
    );
    let mut foreign = scenario.flow.clone();
    foreign.definition = changed;
    let activity = foreign
        .start(instance(24240), ActivityMasterSeed::from_u64(24240))
        .unwrap()
        .into_activity();
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    assert!(scenario.flow.offered_encounter(&activity).is_err());
    assert!(scenario.flow.current_battle_domain(&activity).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view(), debug);
}
