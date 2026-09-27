//! Explicit required sequences at one reviewed position, not original room counts.
//! Other payloads remain probes; no complete-run or encoded profile claim.

use std::sync::Arc;

use super::{Scenario, advance, compile_sequences, payload, prepare, start};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioRuntime, DivergentUniverseCurrencyKind,
    DivergentUniverseLogicalScopeKind,
    battle_room::{BattleRoomError, BattleRoomSelection, BattleRoomSequenceLength},
    domain_route::DomainRoomComposition,
    state::{BATTLE_BLESSING_ACCEPTED_SLOT, CURRENCIES_SLOT},
    tests::{curio_battle_grants::project, currency_balance, reward_draws},
};
use starclock_activity::{
    ActivityDecisionKind, ActivityExpression, ActivityNodeKind, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivityTerminalOutcome,
    ActivityValue, BattleOutcome, BattleResult, GraphActivity, GraphActivityDefinition,
    GraphActivityNodeProgram, LogicalScopeInstance, ProjectedValue,
};
use starclock_combat::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_layout::FixedDomainKind,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn build(
    source: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    length: u16,
) -> Scenario {
    compile_sequences(source, family, &[BattleRewardDomain::Combat], |context| {
        // Deliberately independent of source Boss composition/count: this position
        // supplies a stable namespace for a controlled generic sequence fixture.
        (context.plane_ordinal == 1
            && context.composition == DomainRoomComposition::Fixed(FixedDomainKind::Boss))
        .then_some((0, BattleRoomSequenceLength::new(length).unwrap()))
    })
}
fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
fn acquire(runtime: &DivergentUniverseCurioRuntime, activity: &mut GraphActivity, raw: &str) {
    runtime
        .acquire_accepted_state(activity, activity.state_hash(), &state(raw))
        .unwrap();
}
fn charges(runtime: &DivergentUniverseCurioRuntime, activity: &GraphActivity, raw: &str) -> u16 {
    runtime
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == &state(raw))
        .unwrap()
        .charges()
}
fn room_scope(activity: &GraphActivity) -> LogicalScopeInstance {
    *activity
        .player_view()
        .logical_scopes()
        .iter()
        .find(|scope| scope.address().class() == DivergentUniverseLogicalScopeKind::Node.class_id())
        .unwrap()
}
fn preceding(source: &DivergentUniverseBaselineFixture, scenario: &Scenario) -> GraphActivity {
    let context = scenario.rooms[0].context();
    let mut activity = start(scenario);
    for _ in 0..64 {
        if scenario.route.rooms.iter().any(|candidate| {
            candidate.plane_ordinal == context.plane_ordinal
                && candidate.position_ordinal.checked_add(1) == Some(context.position_ordinal)
                && candidate.entry_node() == activity.current_node()
        }) {
            return activity;
        }
        advance(source, scenario, &mut activity);
    }
    panic!("controlled source route must reach the predecessor");
}

#[test]
fn battle_sequence_lengths_bind_configuration_and_single_compilation_is_equivalent() {
    for invalid in [0, 5, u16::MAX] {
        assert!(matches!(
            BattleRoomSequenceLength::new(invalid),
            Err(BattleRoomError::InvalidSequenceLength)
        ));
    }
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let scenario = build(&source, FAMILIES[0], 4);
    let room = &scenario.rooms[0];
    let compiler = source
        .factory()
        .battle_room_compiler(room.selection().clone())
        .unwrap();
    let single = compiler.compile(room.context()).unwrap();
    let equivalent = compiler
        .compile_sequence(room.context(), BattleRoomSequenceLength::SINGLE)
        .unwrap();
    assert_eq!(single.fragment().nodes, equivalent.fragment().nodes);
    assert_eq!(single.fragment().edges, equivalent.fragment().edges);
    assert_eq!(single.fragment().programs, equivalent.fragment().programs);
    assert_eq!(
        single.fragment().random_offers,
        equivalent.fragment().random_offers
    );
    assert_eq!(single.fragment().exit_node, equivalent.fragment().exit_node);
    assert_eq!(
        single.configuration_digest(),
        equivalent.configuration_digest()
    );
    let mut digests = Vec::new();
    for count in 1..=4 {
        let compiled = compiler
            .compile_sequence(
                room.context(),
                BattleRoomSequenceLength::new(count).unwrap(),
            )
            .unwrap();
        assert_eq!(compiled.sequence_length().get(), count);
        assert_eq!(compiled.battle_nodes().len(), usize::from(count));
        assert_eq!(
            compiled
                .fragment()
                .nodes
                .iter()
                .filter(|node| node.kind() == ActivityNodeKind::Battle)
                .count(),
            usize::from(count)
        );
        digests.push(compiled.configuration_digest());
    }
    digests.sort_unstable();
    digests.dedup();
    assert_eq!(digests.len(), 4);
}

#[test]
fn battle_sequence_real_handoffs_preserve_room_lifetimes_carry_and_reward_gates_in_both_families() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = source.factory().curio_runtime().unwrap();
    let fresh_runtime = fresh_source.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for count in 1..=4 {
            for suppressed in [false, true] {
                // These states share one handbook owner and cannot coexist.
                for raw in ["9071", "9068"] {
                    let domain_lifetime = raw == "9071";
                    let scenario = build(&source, family, count);
                    let fresh = build(&fresh_source, family, count);
                    let mut activity = preceding(&source, &scenario);
                    let mut rebuilt = preceding(&fresh_source, &fresh);
                    acquire(&runtime, &mut activity, raw);
                    acquire(&fresh_runtime, &mut rebuilt, raw);
                    if suppressed {
                        acquire(&runtime, &mut activity, "9055");
                        acquire(&fresh_runtime, &mut rebuilt, "9055");
                    }
                    assert_eq!(
                        charges(&runtime, &activity, raw),
                        if domain_lifetime { 3 } else { 5 }
                    );
                    let wallet = scenario
                        .flow
                        .economy()
                        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                        .key();
                    let before = currency_balance(&activity, wallet);
                    let grant = if domain_lifetime {
                        i64::from(
                            source.factory().decision_catalog().curio_domain_grants()[0].amount,
                        )
                    } else {
                        0
                    };
                    assert_eq!(
                        advance(&source, &scenario, &mut activity),
                        advance(&fresh_source, &fresh, &mut rebuilt)
                    );
                    assert_eq!(
                        activity.player_view().decision().unwrap().kind(),
                        ActivityDecisionKind::Encounter
                    );
                    assert_eq!(
                        charges(&runtime, &activity, raw),
                        if domain_lifetime { 2 } else { 5 }
                    );
                    assert_eq!(
                        currency_balance(&activity, wallet),
                        before + if suppressed { grant * 3 / 2 } else { grant }
                    );
                    let scope = room_scope(&activity);
                    for index in 0..count {
                        assert_eq!(room_scope(&activity), scope);
                        assert_eq!(
                            charges(&runtime, &activity, raw),
                            if domain_lifetime { 2 } else { 5 - index }
                        );
                        let prior = activity.player_view().participant_carry().to_vec();
                        let balance = currency_balance(&activity, wallet);
                        let draws = reward_draws(&activity);
                        let pending = prepare(&source, &scenario, &mut activity);
                        let pending_fresh = prepare(&fresh_source, &fresh, &mut rebuilt);
                        assert_eq!(
                            activity.canonical_state_bytes(),
                            rebuilt.canonical_state_bytes()
                        );
                        let battle_scope = activity
                            .player_view()
                            .logical_scopes()
                            .iter()
                            .find(|scope| {
                                scope.address().class()
                                    == DivergentUniverseLogicalScopeKind::Battle.class_id()
                            })
                            .unwrap()
                            .address();
                        assert_eq!(
                            battle_scope.key(),
                            u64::from(
                                scenario.rooms[0]
                                    .battle_nodes()
                                    .nth(usize::from(index))
                                    .unwrap()
                                    .get()
                            )
                        );
                        for current in pending.handoff().participant_carry() {
                            if let Some(previous) = prior
                                .iter()
                                .find(|prior| prior.participant() == current.participant())
                            {
                                assert_eq!(
                                    current.current_hp(),
                                    previous.current_hp().min(current.maximum_hp())
                                );
                                assert_eq!(
                                    current.current_energy(),
                                    previous.current_energy().min(current.maximum_energy())
                                );
                                assert_eq!(current.life(), previous.life());
                                assert_eq!(current.presence(), previous.presence());
                            }
                        }
                        let (result, report) = pending.execute().unwrap();
                        let (fresh_result, fresh_report) = pending_fresh.execute().unwrap();
                        assert_eq!(result, fresh_result);
                        assert!(
                            result
                                .values()
                                .contains(&ProjectedValue::Outcome(BattleOutcome::Won))
                        );
                        let hash = activity.state_hash();
                        source
                            .factory()
                            .battle_settlement_runtime()
                            .settle_started_result(
                                &scenario.flow,
                                &mut activity,
                                hash,
                                result,
                                Some(report),
                            )
                            .unwrap();
                        let hash = rebuilt.state_hash();
                        fresh_source
                            .factory()
                            .battle_settlement_runtime()
                            .settle_started_result(
                                &fresh.flow,
                                &mut rebuilt,
                                hash,
                                fresh_result,
                                Some(fresh_report),
                            )
                            .unwrap();
                        assert_eq!(
                            activity.canonical_state_bytes(),
                            rebuilt.canonical_state_bytes()
                        );
                        assert_eq!(
                            currency_balance(&activity, wallet),
                            balance + if suppressed { 60 } else { 40 }
                        );
                        assert_eq!(
                            reward_draws(&activity) - draws,
                            if suppressed { 0 } else { 3 }
                        );
                        if !domain_lifetime {
                            assert_eq!(charges(&runtime, &activity, raw), 4 - index);
                        }
                        if !suppressed {
                            assert_eq!(room_scope(&activity), scope);
                            assert_eq!(
                                charges(&runtime, &activity, raw),
                                if domain_lifetime { 2 } else { 4 - index }
                            );
                            let offer = activity.player_view().decision().unwrap().clone();
                            assert_eq!(offer.kind(), ActivityDecisionKind::Reward);
                            let before = activity.canonical_state_bytes();
                            let draws = reward_draws(&activity);
                            assert!(
                                activity
                                    .choose_option(
                                        activity.state_hash(),
                                        offer.id(),
                                        offer.options()[0].id()
                                    )
                                    .is_err(),
                                "raw reward {index} must not inherit acceptance"
                            );
                            assert_eq!(activity.canonical_state_bytes(), before);
                            assert_eq!(reward_draws(&activity), draws);
                            assert_eq!(
                                advance(&source, &scenario, &mut activity),
                                advance(&fresh_source, &fresh, &mut rebuilt)
                            );
                        }
                        if index + 1 < count {
                            assert_eq!(
                                activity.player_view().decision().unwrap().kind(),
                                ActivityDecisionKind::Encounter
                            );
                            assert_eq!(room_scope(&activity), scope);
                            assert_eq!(
                                charges(&runtime, &activity, raw),
                                if domain_lifetime { 2 } else { 4 - index }
                            );
                        }
                    }
                    assert_eq!(
                        activity.player_view().completed_battle_count(),
                        u32::from(count)
                    );
                    assert_ne!(room_scope(&activity), scope);
                    // Crossing a plane may first enter selection staging, not
                    // a domain. Only the next actual room prefix consumes it.
                    for _ in 0..8 {
                        if scenario.route.rooms.iter().any(|context| {
                            context.plane_ordinal == 2
                                && context.entry_node() == activity.current_node()
                        }) {
                            break;
                        }
                        assert_eq!(
                            charges(&runtime, &activity, raw),
                            if domain_lifetime { 2 } else { 5 - count }
                        );
                        assert_eq!(
                            advance(&source, &scenario, &mut activity),
                            advance(&fresh_source, &fresh, &mut rebuilt)
                        );
                    }
                    assert!(scenario.route.rooms.iter().any(|context| {
                        context.plane_ordinal == 2
                            && context.entry_node() == activity.current_node()
                    }));
                    assert_eq!(
                        charges(&runtime, &activity, raw),
                        if domain_lifetime { 1 } else { 5 - count }
                    );
                    for _ in 0..64 {
                        assert_eq!(
                            activity.canonical_state_bytes(),
                            rebuilt.canonical_state_bytes()
                        );
                        if activity.player_view().terminal().is_some() {
                            break;
                        }
                        assert_eq!(
                            advance(&source, &scenario, &mut activity),
                            advance(&fresh_source, &fresh, &mut rebuilt)
                        );
                    }
                    assert_eq!(
                        activity.player_view().terminal(),
                        Some(ActivityTerminalOutcome::Completed)
                    );
                    assert_eq!(
                        activity.player_view().completed_battle_count(),
                        u32::from(count)
                    );
                }
            }
        }
    }
}

#[test]
fn battle_sequence_prior_result_and_late_credit_failure_preserve_pending_second_handoff() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let scenario = build(&source, family, 2);
        let mut activity = preceding(&source, &scenario);
        advance(&source, &scenario, &mut activity);
        let first = prepare(&source, &scenario, &mut activity);
        let (result, report) = first.execute().unwrap();
        let stale = result.clone();
        let hash = activity.state_hash();
        source
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(&scenario.flow, &mut activity, hash, result, Some(report))
            .unwrap();
        advance(&source, &scenario, &mut activity);
        let wallet = scenario
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let inject = ActivityProgramDefinition::new(
            ActivityProgramId::new(24800).unwrap(),
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
            }],
        )
        .unwrap();
        activity
            .apply_boundary_program(activity.state_hash(), &inject)
            .unwrap();
        let second = prepare(&source, &scenario, &mut activity);
        let bytes = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        let hash = activity.state_hash();
        assert!(
            source
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&scenario.flow, &mut activity, hash, stale, None)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        assert_eq!(reward_draws(&activity), draws);
        let (result, report) = second.execute().unwrap();
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let hash = activity.state_hash();
        assert!(
            source
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&scenario.flow, &mut activity, hash, result, Some(report))
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        assert!(activity.pending_battle().is_some());
        assert_eq!(activity.player_view().completed_battle_count(), 1);
    }
}

#[test]
fn battle_sequence_second_handoff_counterfactual_failure_terminates_without_rewards() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for (outcome, terminal) in [
            (BattleOutcome::Lost, ActivityTerminalOutcome::Failed),
            (BattleOutcome::Faulted, ActivityTerminalOutcome::Faulted),
        ] {
            let scenario = build(&source, family, 4);
            let mut activity = preceding(&source, &scenario);
            advance(&source, &scenario, &mut activity);
            let (result, report) = prepare(&source, &scenario, &mut activity)
                .execute()
                .unwrap();
            let hash = activity.state_hash();
            source
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&scenario.flow, &mut activity, hash, result, Some(report))
                .unwrap();
            advance(&source, &scenario, &mut activity);
            let wallet = scenario
                .flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            let fragments = currency_balance(&activity, wallet);
            let draws = reward_draws(&activity);
            let (actual, _) = prepare(&source, &scenario, &mut activity)
                .execute()
                .unwrap();
            // Deliberately counterfactual projection, not an observed battle outcome.
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
            source
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&scenario.flow, &mut activity, hash, projected, None)
                .unwrap();
            assert_eq!(activity.player_view().terminal(), Some(terminal));
            assert_eq!(currency_balance(&activity, wallet), fragments);
            assert_eq!(reward_draws(&activity), draws);
            assert_eq!(activity.player_view().completed_battle_count(), 2);
            assert!(activity.pending_battle().is_none());
            assert!(activity.player_view().decision().is_none());
        }
    }
}

#[test]
fn battle_sequence_binding_rejects_second_reward_bypass_and_foreign_shorter_sequence() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let scenario = build(&source, FAMILIES[0], 4);
    let room = &scenario.rooms[0];
    let factory = source.factory();
    let compiler = factory
        .battle_room_compiler(BattleRoomSelection {
            group: room.selection().group.clone(),
            stage: room.selection().stage.clone(),
            domain: room.selection().domain,
        })
        .unwrap();
    let shorter = compiler
        .compile_sequence(room.context(), BattleRoomSequenceLength::new(2).unwrap())
        .unwrap();
    assert!(
        factory
            .bind_battle_rooms(
                scenario.base.clone(),
                Arc::clone(scenario.flow.definition()),
                &[shorter],
                payload()
            )
            .is_err()
    );
    let mut programs = scenario.flow.definition().programs().to_vec();
    let second = room.context().node(6).unwrap();
    let program = programs
        .iter_mut()
        .find(|program| program.node() == second)
        .unwrap();
    let mut operations = program.program().operations().to_vec();
    operations.retain(|operation| !matches!(operation, ActivityOperation::SetSlot { slot, .. } if *slot == BATTLE_BLESSING_ACCEPTED_SLOT));
    *program = GraphActivityNodeProgram::new(
        second,
        ActivityProgramDefinition::new(program.program().id(), operations).unwrap(),
    );
    let definition = scenario.flow.definition();
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
    assert!(
        factory
            .bind_battle_rooms(scenario.base.clone(), changed, &scenario.rooms, payload())
            .is_err()
    );
}
