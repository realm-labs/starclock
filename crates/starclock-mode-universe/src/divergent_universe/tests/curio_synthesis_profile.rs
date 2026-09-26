//! Controlled synthesis placement, trusted inventory and three real Boss proxies.
//! Other source-position payloads are probes, not default/full-run release parity.

use super::{
    CurioSynthesisSlots, DECK, FAMILIES, RECEIPT, ROOT, SERVICE, inventory_operations, members,
    probe, program, slot, value,
};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseFlowInstance,
    DivergentUniverseLogicalScopeKind,
    battle_room::BattleRoomSelection,
    curio_synthesis::room::{
        CompiledCurioSynthesisRoom, CurioSynthesisRoomPhase, CurioSynthesisRoomPolicy,
        LEAVE_SYNTHESIS,
    },
    domain_route::DomainRoomComposition,
    state::SERVICE_RECEIPTS_SLOT,
    tests::{battle_room::base, instance, reward_draws},
};
use starclock_activity::{
    ActivityConfigDigest, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityRandomPolicies, ActivityStateDefinition, ActivityTerminalOutcome,
    ActivityValue, GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
    LogicalScopeAddress, LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioCategory,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_layout::FixedDomainKind,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};
use std::sync::Arc;

struct Profile {
    flow: DivergentUniverseFlowInstance,
    unbound: DivergentUniverseFlowInstance,
    rooms: Vec<CompiledCurioSynthesisRoom>,
}
fn compile(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> Profile {
    let factory = fixture.factory();
    let base = base(fixture, family);
    let pool = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let compiler = factory
        .curio_synthesis_room_compiler(
            &DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap(),
            CurioSynthesisRoomPolicy::new(1).unwrap(),
            SERVICE,
        )
        .unwrap();
    let mut battles = Vec::new();
    let mut rooms = Vec::new();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    let mut route = factory
        .compile_curio_domain_route(base.area(), deck, 3, DECK, |context| {
            if context.composition == DomainRoomComposition::Fixed(FixedDomainKind::Boss) {
                let room = battle.compile(context).unwrap();
                let fragment = room.fragment().clone();
                battles.push(room);
                Ok(fragment)
            } else if context.position_ordinal == 1 && context.plane_ordinal == 1 {
                let room = compiler.compile(context).unwrap();
                let fragment = room.fragment().clone();
                rooms.push(room);
                Ok(fragment)
            } else {
                probe(context, false)
            }
        })
        .unwrap();
    assert_eq!(rooms.len(), 1);
    assert_eq!(battles.len(), 3);
    let root = NodeId::new(ROOT).unwrap();
    let edge = ActivityEdgeId::new(ROOT).unwrap();
    let mut nodes = route.graph.nodes().to_vec();
    nodes.push(
        ActivityNodeDefinition::new(
            root,
            SectionId::new(1).unwrap(),
            ActivityNodeKind::Choice,
            1,
        )
        .unwrap(),
    );
    let mut edges = route.graph.edges().to_vec();
    edges.push(
        ActivityEdgeDefinition::new(
            edge,
            root,
            route.graph.entry(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    );
    route.graph =
        ActivityGraphDefinition::new(root, nodes, edges, route.graph.maximum_total_visits() + 1)
            .unwrap();
    let mut bindings = route.logical_scopes.bindings().to_vec();
    bindings.push(
        LogicalScopeNodeBinding::new(
            root,
            vec![
                LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Run.class_id(), 1)
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    route.logical_scopes =
        LogicalScopeDefinitions::new(route.logical_scopes.classes().to_vec(), bindings).unwrap();
    let curios = factory.curio_runtime().unwrap();
    let inputs = members(&curios, DivergentUniverseCurioCategory::Common)[..2].to_vec();
    let mut operations = inventory_operations(&curios, &inputs);
    operations.push(ActivityOperation::Traverse(edge));
    route.programs.push(program(root, operations));
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    slots.extend_from_slice(rooms[0].slot_definitions());
    let mut owner = CanonicalDigestBuilder::new();
    owner.update(b"du.test.synthesis.explicit-first-position.three-boss-proxies.other-payloads-probes.trusted-inventory");
    owner.update(u64::try_from(deck.len()).unwrap().to_le_bytes());
    owner.update(deck.as_bytes());
    owner.update(3_u16.to_le_bytes());
    for slot in [DECK.draw, DECK.discard, DECK.selected, DECK.accepted] {
        owner.update(slot.get().to_le_bytes());
    }
    let mut keys = inputs
        .iter()
        .map(|id| curios.state(id).unwrap().state_key())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    owner.update(u32::try_from(keys.len()).unwrap().to_le_bytes());
    for key in keys {
        owner.update(key.to_le_bytes());
    }
    owner.update(u32::try_from(rooms.len()).unwrap().to_le_bytes());
    for room in &rooms {
        owner.update(room.configuration_digest());
    }
    let payload = ActivityConfigDigest::new(owner.finalize()).unwrap();
    let identity = factory
        .battle_room_identity(&base, &route.graph, &battles, payload)
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
        .bind_battle_rooms(base, definition, &battles, payload)
        .unwrap();
    let flow = factory.bind_position_domain_route(flow, &route).unwrap();
    let unbound = flow.clone();
    let flow = factory
        .bind_position_curio_synthesis_rooms(flow, &rooms)
        .unwrap();
    Profile {
        flow,
        unbound,
        rooms,
    }
}
fn start(flow: &DivergentUniverseFlowInstance) -> GraphActivity {
    flow.start(instance(2566), ActivityMasterSeed::from_u64(2566))
        .unwrap()
        .into_activity()
}
fn policy(fixture: &DivergentUniverseBaselineFixture) -> DivergentUniverseBaselinePolicy {
    let original = fixture.policy().unwrap();
    DivergentUniverseBaselinePolicy::new(
        original.hints().clone(),
        original.encounter_group().clone(),
        original.encounter_stage(),
        128,
    )
    .unwrap()
}
fn advance(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
) {
    DivergentUniverseBaselineRunner::default()
        .advance(
            fixture.factory(),
            flow,
            activity,
            fixture.core(),
            &policy(fixture),
        )
        .unwrap();
}
fn ready(fixture: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(&profile.flow);
    for _ in 0..128 {
        if profile.flow.offered_curio_synthesis(&activity).is_some() {
            return activity;
        }
        advance(fixture, &profile.flow, &mut activity);
    }
    panic!("controlled profile must reach synthesis");
}

#[test]
fn synthesis_controller_reconstructs_both_families_with_three_real_boss_battles() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&fixture, family);
        let fresh = compile(&fresh_fixture, family);
        let mut activity = start(&profile.flow);
        let mut reconstructed = start(&fresh.flow);
        let mut phases = Vec::new();
        for _ in 0..128 {
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
            if activity.player_view().terminal().is_some() {
                break;
            }
            if let Some(phase) = profile.flow.offered_curio_synthesis(&activity) {
                phases.push(phase);
                let before_draws = reward_draws(&activity);
                let before_owned = fixture
                    .factory()
                    .curio_runtime()
                    .unwrap()
                    .owned(&activity)
                    .unwrap()
                    .len();
                advance(&fixture, &profile.flow, &mut activity);
                advance(&fresh_fixture, &fresh.flow, &mut reconstructed);
                match phase {
                    CurioSynthesisRoomPhase::Menu | CurioSynthesisRoomPhase::FirstInput => {
                        assert_eq!(reward_draws(&activity), before_draws);
                        assert_eq!(
                            fixture
                                .factory()
                                .curio_runtime()
                                .unwrap()
                                .owned(&activity)
                                .unwrap()
                                .len(),
                            before_owned
                        );
                    }
                    CurioSynthesisRoomPhase::SecondInput => {
                        assert_eq!(reward_draws(&activity), before_draws + 3);
                        assert_eq!(
                            fixture
                                .factory()
                                .curio_runtime()
                                .unwrap()
                                .owned(&activity)
                                .unwrap()
                                .len(),
                            before_owned
                        );
                    }
                    CurioSynthesisRoomPhase::Confirmation => {
                        assert_eq!(
                            fixture
                                .factory()
                                .curio_runtime()
                                .unwrap()
                                .owned(&activity)
                                .unwrap()
                                .len(),
                            before_owned - 1
                        );
                        assert_eq!(
                            value(&activity, SERVICE_RECEIPTS_SLOT),
                            ActivityValue::BoundedCounterMap(vec![(RECEIPT, 1)].into())
                        );
                        assert_eq!(
                            activity.player_view().decision().unwrap().options().len(),
                            1
                        );
                        assert_eq!(
                            activity.player_view().decision().unwrap().options()[0]
                                .id()
                                .get(),
                            LEAVE_SYNTHESIS
                        );
                    }
                }
            } else {
                advance(&fixture, &profile.flow, &mut activity);
                advance(&fresh_fixture, &fresh.flow, &mut reconstructed);
            }
        }
        assert_eq!(
            phases,
            vec![
                CurioSynthesisRoomPhase::Menu,
                CurioSynthesisRoomPhase::FirstInput,
                CurioSynthesisRoomPhase::SecondInput,
                CurioSynthesisRoomPhase::Confirmation,
                CurioSynthesisRoomPhase::Menu
            ]
        );
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(activity.player_view().completed_battle_count(), 3);
        assert_eq!(
            activity.canonical_state_bytes(),
            reconstructed.canonical_state_bytes()
        );
    }
}

#[test]
fn synthesis_controller_capability_attachment_and_all_phase_rejections_are_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&fixture, DivergentUniverseRunFamily::Ordinary);
    let factory = fixture.factory();
    assert!(
        factory
            .bind_position_curio_synthesis_rooms(profile.unbound.clone(), &[])
            .is_err()
    );
    assert!(
        factory
            .bind_position_curio_synthesis_rooms(
                profile.unbound.clone(),
                &[profile.rooms[0].clone(), profile.rooms[0].clone()]
            )
            .is_err()
    );
    assert!(
        factory
            .bind_position_curio_synthesis_rooms(profile.flow.clone(), &profile.rooms)
            .is_err()
    );
    let foreign = compile(&fixture, DivergentUniverseRunFamily::Cyclical);
    assert!(
        factory
            .bind_position_curio_synthesis_rooms(profile.unbound.clone(), &foreign.rooms)
            .is_err()
    );
    for (limit, slots) in [
        (2, SERVICE),
        (
            1,
            CurioSynthesisSlots {
                first: slot(76),
                ..SERVICE
            },
        ),
    ] {
        let changed = factory
            .curio_synthesis_room_compiler(
                &DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap(),
                CurioSynthesisRoomPolicy::new(limit).unwrap(),
                slots,
            )
            .unwrap()
            .compile(profile.rooms[0].context())
            .unwrap();
        assert!(
            factory
                .bind_position_curio_synthesis_rooms(profile.unbound.clone(), &[changed],)
                .is_err()
        );
    }
    let mut activity = ready(&fixture, &profile);
    for phase in [
        CurioSynthesisRoomPhase::Menu,
        CurioSynthesisRoomPhase::FirstInput,
        CurioSynthesisRoomPhase::SecondInput,
        CurioSynthesisRoomPhase::Confirmation,
    ] {
        assert_eq!(profile.flow.offered_curio_synthesis(&activity), Some(phase));
        let before = activity.canonical_state_bytes();
        let stale = start(&foreign.flow).state_hash();
        let offer = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        assert_ne!(hash, stale);
        let option = offer.options()[0].id();
        assert_eq!(
            profile
                .flow
                .choose_curio_synthesis_option(&mut activity, stale, offer.id(), option),
            Err(GraphActivityCommandError::StaleStateHash)
        );
        assert!(profile.unbound.offered_curio_synthesis(&activity).is_none());
        assert_eq!(
            profile
                .unbound
                .choose_curio_synthesis_option(&mut activity, hash, offer.id(), option),
            Err(GraphActivityCommandError::DecisionNotOffered)
        );
        assert!(foreign.flow.offered_curio_synthesis(&activity).is_none());
        assert_eq!(
            foreign
                .flow
                .choose_curio_synthesis_option(&mut activity, hash, offer.id(), option),
            Err(GraphActivityCommandError::DecisionNotOffered)
        );
        assert!(activity.choose_option(hash, offer.id(), option).is_err());
        assert!(
            DivergentUniverseBaselineRunner::default()
                .advance(
                    factory,
                    &profile.unbound,
                    &mut activity,
                    fixture.core(),
                    &policy(&fixture),
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        advance(&fixture, &profile.flow, &mut activity);
    }
}
