//! Fixed Conversion execution with explicit battle substitutes; other rooms
//! remain probes. Natural combat and counterfactual contract probes are separate.

use super::{Scenario, advance, prepare, probe, start};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBattleRetryDisposition,
    DivergentUniverseCurrencyKind, DivergentUniverseCyclicalRefresh, DivergentUniverseEntry,
    battle_room::{
        BattleRoomError, BattleRoomSelection, BattleRoomSequenceLength, BattleRoomSequencePolicy,
    },
    domain_route::DomainRoomComposition,
    state::BATTLE_BLESSING_CANDIDATES_SLOT,
    tests::{curio_battle_grants::project, currency_balance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityExpression,
    ActivityNodeKind, ActivityOperation, ActivityProgramDefinition, ActivityRandomPolicies,
    ActivityStateDefinition, ActivityTerminalOutcome, ActivityValue, BattleOutcome, BattleResult,
    GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram, ProjectedValue,
};
use starclock_combat::{
    BattleFault, FaultBoundary, FaultKind, FaultPolicy, LifeState, PresenceState,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseAreaId,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_layout::FixedDomainKind,
};
use std::sync::Arc;

const AREAS: [&str; 2] = ["406", "20406"];

fn payload(fail_tail: bool) -> ActivityConfigDigest {
    // Owning test profile: first authored deck/width three, parent deck slots,
    // probe payloads, optional four-battle prefix and successor failure switch.
    // The graph and bound rooms separately bind prefix/context/count/candidates.
    let mut bytes = [0x63; 32];
    bytes[31] = u8::from(fail_tail);
    ActivityConfigDigest::new(bytes).unwrap()
}

fn build(
    source: &DivergentUniverseBaselineFixture,
    area: &str,
    count: u16,
    prefix: bool,
    fail_tail: bool,
) -> Scenario {
    let factory = source.factory();
    let catalog = factory.bundle.catalog();
    let area = DivergentUniverseAreaId::new(format!("divergent-universe.area.{area}")).unwrap();
    let area_definition = catalog.area(&area).unwrap();
    let original = source.flow(area_definition.run_family()).unwrap();
    let mut entry = DivergentUniverseEntry::new(
        area.clone(),
        area_definition.difficulties[0].clone(),
        Arc::clone(source.participants()),
        original.input_snapshot().clone(),
        Vec::new(),
    )
    .unwrap()
    .with_mapping_snapshot(Arc::clone(source.mapping()))
    .with_runtime_battle_route();
    if let Some(challenge) = catalog
        .cyclical_challenges()
        .iter()
        .find(|value| value.area == area)
    {
        entry = entry.with_cyclical_refresh(
            DivergentUniverseCyclicalRefresh::new(1, challenge.id.clone()).unwrap(),
        );
    }
    let base = factory.compile(entry).unwrap();
    let pool = factory.decision_catalog().encounter_pool();
    let compiler = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Combat,
        })
        .unwrap();
    let mut rooms = Vec::new();
    let mut route = factory
        .compile_curio_domain_route(
            &area,
            &factory.decision_catalog().domain_decks()[0].key,
            3,
            super::SLOTS,
            |context| {
                let room = if context.composition
                    == DomainRoomComposition::Fixed(FixedDomainKind::Conversion)
                {
                    Some(
                        compiler
                            .compile_conversion_sequence(
                                context,
                                BattleRoomSequenceLength::new(count).unwrap(),
                            )
                            .unwrap(),
                    )
                } else if prefix && context.plane_ordinal == 1 && context.position_ordinal == 1 {
                    Some(
                        compiler
                            .compile_sequence(context, BattleRoomSequenceLength::new(4).unwrap())
                            .unwrap(),
                    )
                } else {
                    None
                };
                if let Some(room) = room {
                    let fragment = room.fragment().clone();
                    rooms.push(room);
                    Ok(fragment)
                } else {
                    probe(context)
                }
            },
        )
        .unwrap();
    if fail_tail {
        // Fail automatic staging before its next card-selection offer, not a
        // room alternative whose entry requires another accepted command.
        let successor = rooms.last().unwrap().context().successor();
        let program = route
            .programs
            .iter_mut()
            .find(|program| program.node() == successor)
            .unwrap();
        let mut operations = program.program().operations().to_vec();
        operations.insert(
            0,
            ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
                ActivityValue::Boolean(false),
            ))),
        );
        *program = GraphActivityNodeProgram::new(
            successor,
            ActivityProgramDefinition::new(program.program().id(), operations).unwrap(),
        );
    }
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    let identity = factory
        .battle_room_identity(&base, &route.graph, &rooms, payload(fail_tail))
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
        .bind_battle_rooms(base.clone(), definition, &rooms, payload(fail_tail))
        .unwrap();
    Scenario {
        base,
        flow,
        route,
        rooms,
    }
}

fn ready(
    source: &DivergentUniverseBaselineFixture,
    scenario: &Scenario,
    suppressed: bool,
) -> GraphActivity {
    let mut activity = start(scenario);
    if suppressed {
        let hash = activity.state_hash();
        source
            .factory()
            .curio_runtime()
            .unwrap()
            .acquire_accepted_state(
                &mut activity,
                hash,
                &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9055").unwrap(),
            )
            .unwrap();
    }
    let room = scenario.rooms.last().unwrap();
    for _ in 0..128 {
        if activity.current_node() == room.context().node(1).unwrap() {
            return activity;
        }
        assert!(activity.player_view().terminal().is_none());
        advance(source, scenario, &mut activity);
    }
    panic!("controlled profile must reach fixed Conversion");
}

#[test]
fn conversion_room_admits_only_exact_current_fixed_context_and_binds_policy_identity() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for area in ["406", "20406", "409", "20409"] {
        let scenario = build(&source, area, 4, false, false);
        let room = &scenario.rooms[0];
        let compiler = source
            .factory()
            .battle_room_compiler(room.selection().clone())
            .unwrap();
        assert_eq!(room.context().preset_source.as_ref(), "1004");
        assert_eq!(room.context().position_ordinal, 4);
        assert_eq!(room.context().plane_ordinal, 2);
        assert_eq!(room.sequence_policy(), BattleRoomSequencePolicy::VersionedProjectPolicyConversionSameCandidateSequenceLossEndsRoomWithoutHealing);
        let mut digests = Vec::new();
        for count in 1..=4 {
            let length = BattleRoomSequenceLength::new(count).unwrap();
            let conversion = compiler
                .compile_conversion_sequence(room.context(), length)
                .unwrap();
            let ordinary = compiler.compile_sequence(room.context(), length).unwrap();
            assert_ne!(
                conversion.configuration_digest(),
                ordinary.configuration_digest()
            );
            assert_eq!(conversion.battle_nodes().len(), usize::from(count));
            assert!(
                !conversion.fragment().nodes.iter().any(|node| node.kind()
                    == ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed))
            );
            digests.push(conversion.configuration_digest());
        }
        digests.sort_unstable();
        digests.dedup();
        assert_eq!(digests.len(), 4);
        for changed in 0..5 {
            let mut context = room.context().clone();
            match changed {
                0 => context.level = 2,
                1 => context.preset_source = "1002".into(),
                2 => context.composition = DomainRoomComposition::Fixed(FixedDomainKind::Boss),
                3 => context.position_ordinal = 5,
                _ => context.plane_ordinal = 1,
            }
            assert!(matches!(
                compiler.compile_conversion_sequence(&context, BattleRoomSequenceLength::SINGLE),
                Err(BattleRoomError::InvalidContext)
            ));
        }
        let ordinary = compiler
            .compile_sequence(room.context(), BattleRoomSequenceLength::new(4).unwrap())
            .unwrap();
        assert!(
            source
                .factory()
                .bind_battle_rooms(
                    scenario.base.clone(),
                    Arc::clone(scenario.flow.definition()),
                    &[ordinary],
                    payload(false)
                )
                .is_err()
        );
    }
}

#[test]
fn conversion_room_real_victories_and_natural_defeat_continue_with_verified_carry_in_both_families()
{
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for area in AREAS {
        let scenario = build(&source, area, 4, true, false);
        let fresh = build(&fresh_source, area, 4, true, false);
        let mut activity = ready(&source, &scenario, true);
        let mut rebuilt = ready(&fresh_source, &fresh, true);
        assert_eq!(activity.player_view().completed_battle_count(), 4);
        assert_eq!(
            activity.canonical_state_bytes(),
            rebuilt.canonical_state_bytes()
        );
        let room = scenario.rooms.last().unwrap();
        let wallet = scenario
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let mut defeated = false;
        for _ in 0..4 {
            let previous = activity.player_view().participant_carry().to_vec();
            let before = currency_balance(&activity, wallet);
            let draws = reward_draws(&activity);
            let pending = prepare(&source, &scenario, &mut activity);
            let rebuilt_pending = prepare(&fresh_source, &fresh, &mut rebuilt);
            for current in pending.handoff().participant_carry() {
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
            let (result, report) = pending.execute().unwrap();
            let (fresh_result, fresh_report) = rebuilt_pending.execute().unwrap();
            assert_eq!(result, fresh_result);
            let hash = activity.state_hash();
            let resolution = source
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(
                    &scenario.flow,
                    &mut activity,
                    hash,
                    result.clone(),
                    Some(report),
                )
                .unwrap();
            let hash = rebuilt.state_hash();
            let fresh_resolution = fresh_source
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
            assert_eq!(resolution.events(), fresh_resolution.events());
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(activity.debug_view(), rebuilt.debug_view());
            assert_eq!(resolution.retry_disposition(), DivergentUniverseBattleRetryDisposition::DefeatContinuesExplorationWithoutRetryOrHealing);
            for value in result.values() {
                if let ProjectedValue::ParticipantState(value) = value {
                    let carry = activity
                        .player_view()
                        .participant_carry()
                        .iter()
                        .find(|carry| carry.participant() == value.participant())
                        .copied()
                        .unwrap();
                    assert_eq!(carry.current_hp(), value.current_hp());
                    assert_eq!(carry.current_energy(), value.current_energy());
                    assert_eq!(carry.life(), value.life());
                    assert_eq!(
                        carry.presence(),
                        if value.life() == LifeState::Defeated {
                            PresenceState::Departed
                        } else {
                            value.presence()
                        }
                    );
                }
            }
            assert_eq!(reward_draws(&activity), draws);
            assert!(activity.player_view().terminal().is_none());
            if resolution.settlement().outcome() == BattleOutcome::Lost {
                defeated = true;
                assert_eq!(currency_balance(&activity, wallet), before);
                assert!(
                    !room
                        .fragment()
                        .nodes
                        .iter()
                        .any(|node| node.id() == activity.current_node())
                );
                assert!(activity.pending_battle().is_none());
                assert!(activity.player_view().decision().is_some());
                let bytes = activity.canonical_state_bytes();
                let debug = activity.debug_view();
                let hash = activity.state_hash();
                for _ in 0..3 {
                    assert!(
                        source
                            .factory()
                            .battle_settlement_runtime()
                            .settle_started_result(
                                &scenario.flow,
                                &mut activity,
                                hash,
                                result.clone(),
                                None
                            )
                            .is_err()
                    );
                    assert_eq!(activity.canonical_state_bytes(), bytes);
                    assert_eq!(activity.debug_view(), debug);
                }
                let carry = activity.player_view().participant_carry().to_vec();
                assert_eq!(
                    advance(&source, &scenario, &mut activity),
                    advance(&fresh_source, &fresh, &mut rebuilt)
                );
                assert_eq!(activity.player_view().participant_carry(), carry.as_slice());
                assert!(activity.player_view().terminal().is_none());
                assert_eq!(
                    activity.player_view().decision().unwrap().kind(),
                    ActivityDecisionKind::Service
                );
                assert_eq!(
                    activity.canonical_state_bytes(),
                    rebuilt.canonical_state_bytes()
                );
                assert_eq!(activity.debug_view(), rebuilt.debug_view());
                break;
            }
            assert_eq!(resolution.settlement().outcome(), BattleOutcome::Won);
            assert!(currency_balance(&activity, wallet) > before);
        }
        assert!(
            defeated,
            "actual carried party must lose before the four substitute battles finish"
        );
    }
}

#[test]
fn conversion_room_real_complete_sequences_settle_every_victory_and_leave_after_final_reward() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for area in AREAS {
        for count in 1..=4 {
            let scenario = build(&source, area, count, false, false);
            let fresh = build(&fresh_source, area, count, false, false);
            let mut activity = ready(&source, &scenario, false);
            let mut rebuilt = ready(&fresh_source, &fresh, false);
            let wallet = scenario
                .flow
                .economy()
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key();
            for index in 0..count {
                let before = currency_balance(&activity, wallet);
                let draws = reward_draws(&activity);
                let (result, report) = prepare(&source, &scenario, &mut activity)
                    .execute()
                    .unwrap();
                let (fresh_result, fresh_report) = prepare(&fresh_source, &fresh, &mut rebuilt)
                    .execute()
                    .unwrap();
                assert_eq!(result, fresh_result);
                assert!(
                    result
                        .values()
                        .contains(&ProjectedValue::Outcome(BattleOutcome::Won))
                );
                let hash = activity.state_hash();
                let resolution = source
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
                let fresh_resolution = fresh_source
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
                assert_eq!(resolution.events(), fresh_resolution.events());
                assert_eq!(currency_balance(&activity, wallet), before + 40);
                assert_eq!(reward_draws(&activity), draws + 3);
                assert_eq!(
                    activity.player_view().decision().unwrap().kind(),
                    ActivityDecisionKind::Reward
                );
                assert_eq!(
                    advance(&source, &scenario, &mut activity),
                    advance(&fresh_source, &fresh, &mut rebuilt)
                );
                assert_eq!(
                    activity.canonical_state_bytes(),
                    rebuilt.canonical_state_bytes()
                );
                assert_eq!(activity.debug_view(), rebuilt.debug_view());
                assert_eq!(
                    activity.player_view().completed_battle_count(),
                    u32::from(index + 1)
                );
            }
            assert!(activity.player_view().terminal().is_none());
            assert!(
                !scenario.rooms[0]
                    .fragment()
                    .nodes
                    .iter()
                    .any(|node| node.id() == activity.current_node())
            );
            assert!(activity.pending_battle().is_none());
        }
    }
}

#[test]
fn conversion_room_counterfactual_loss_skips_all_remaining_rewards_but_fault_terminates() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for area in AREAS {
        for count in 1..=4 {
            for index in 0..count {
                for outcome in [BattleOutcome::Lost, BattleOutcome::Faulted] {
                    let scenario = build(&source, area, count, false, false);
                    let mut activity = ready(&source, &scenario, false);
                    for _ in 0..index {
                        let (result, report) = prepare(&source, &scenario, &mut activity)
                            .execute()
                            .unwrap();
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
                        advance(&source, &scenario, &mut activity);
                    }
                    let (actual, _) = prepare(&source, &scenario, &mut activity)
                        .execute()
                        .unwrap();
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
                    let wallet = scenario
                        .flow
                        .economy()
                        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                        .key();
                    let balance = currency_balance(&activity, wallet);
                    let draws = reward_draws(&activity);
                    let hash = activity.state_hash();
                    let resolution = source
                        .factory()
                        .battle_settlement_runtime()
                        .settle_started_result(&scenario.flow, &mut activity, hash, projected, None)
                        .unwrap();
                    assert_eq!(currency_balance(&activity, wallet), balance);
                    assert_eq!(reward_draws(&activity), draws);
                    assert_eq!(
                        activity.player_view().completed_battle_count(),
                        u32::from(index + 1)
                    );
                    assert!(activity.pending_battle().is_none());
                    if outcome == BattleOutcome::Lost {
                        assert!(activity.player_view().terminal().is_none());
                        assert_eq!(resolution.retry_disposition(), DivergentUniverseBattleRetryDisposition::DefeatContinuesExplorationWithoutRetryOrHealing);
                        let view = activity.player_view();
                        assert_eq!(
                            view.slots()
                                .iter()
                                .find(|slot| slot.id() == BATTLE_BLESSING_CANDIDATES_SLOT)
                                .unwrap()
                                .value(),
                            &ActivityValue::OrderedIdSet(Vec::new().into_boxed_slice())
                        );
                        assert_ne!(
                            view.decision().unwrap().kind(),
                            ActivityDecisionKind::Reward
                        );
                    } else {
                        assert_eq!(
                            activity.player_view().terminal(),
                            Some(ActivityTerminalOutcome::Faulted)
                        );
                        assert_eq!(resolution.retry_disposition(), DivergentUniverseBattleRetryDisposition::DefeatTerminatesCurrentActivityFreshActivityRequired);
                    }
                }
            }
        }
    }
}

#[test]
fn conversion_room_late_exit_error_rolls_back_result_carry_lifetimes_and_rng() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for area in AREAS {
        let scenario = build(&source, area, 4, false, true);
        let mut activity = ready(&source, &scenario, true);
        let curio =
            DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9068").unwrap();
        let hash = activity.state_hash();
        source
            .factory()
            .curio_runtime()
            .unwrap()
            .acquire_accepted_state(&mut activity, hash, &curio)
            .unwrap();
        let (actual, _) = prepare(&source, &scenario, &mut activity)
            .execute()
            .unwrap();
        let result = project(&actual, 0, BattleOutcome::Lost);
        let bytes = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let hash = activity.state_hash();
        for _ in 0..3 {
            assert!(
                source
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(
                        &scenario.flow,
                        &mut activity,
                        hash,
                        result.clone(),
                        None
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(activity.debug_view(), debug);
            assert_eq!(activity.state_hash(), hash);
            assert!(activity.pending_battle().is_some());
        }
    }
}
