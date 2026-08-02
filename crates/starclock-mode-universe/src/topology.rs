//! Spatial-free Standard Universe topology, room and encounter compilation.
mod blessing_offer;
mod error;
mod graph_layout;
mod occurrence_binding;
mod reward_program;
mod route_definition;
mod route_program;
pub type UniverseTopologyCompileError = error::UniverseTopologyCompileError;
use self::graph_layout::*;
use self::occurrence_binding::*;
use self::route_program::compile_route_program;
use self::{blessing_offer::compile_blessing_offer_policy, reward_program::node_program_id};
use crate::{
    ability_runtime::AbilityTarget,
    blessing_runtime::{BlessingOfferEligibility, BlessingRuntimeCatalog},
    catalog::UniverseCatalog,
    definition::DomainKind,
    encounter::RoomContentKind,
    handler_bundle::{
        STANDARD_UNIVERSE_EXTERNAL_INTERACTION_HANDLER_ID, activity_handler_registry,
    },
    id::{EncounterGroupId, EncounterMemberId, RoomId, TopologyId, TopologyNodeId},
    occurrence_interaction::{
        OCCURRENCE_INTERACTION_HANDLER_ID, OccurrenceInteractionRuntimeCatalog,
    },
    path_runtime::{FormationSelectionBindings, PathRuntimeCatalog},
    service_interaction::{
        SERVICE_INTERACTION_HANDLER_ID, ServiceInteractionRuntimeCatalog,
        ServiceInteractionSelection,
    },
    topology_identity::{
        blessing_option, content_option, engage_option, exit_option, formation_option,
        formation_skip_option, interaction_option, member_option, occurrence_choice_option,
        occurrence_external_result_option, path_option, room_option, route_option,
        service_interaction_option, topology_option, trailblaze_bonus_option,
    },
    topology_reward::{BlessingRewardCompletion, compile_blessing_reward},
    topology_service::{
        compile_room_services, option_condition as service_option_condition,
        trailblaze_bonus_condition,
    },
    topology_support::{
        domain_logical_scopes, exact_weight, occurrence_for_source, optional_equals, resolve_rooms,
        set_optional,
    },
};
use starclock_activity::{
    ActivityBootstrapSelection, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityExternalOutcomeId,
    ActivityGraphDefinition, ActivityInteractionBinding, ActivityInteractionRandomPolicy,
    ActivityInventoryId, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomCheckpoint, ActivityRandomOffer, ActivityRandomPolicies, ActivityRngLabel,
    ActivitySlotId, ActivityStateDefinition, ActivityTerminalOutcome, ActivityValue,
    GraphActivityDefinition, GraphActivityNodeProgram, NodeId, ParticipantLock, SectionId,
    TerminalOutcome,
};
use std::{collections::BTreeSet, sync::Arc};
pub const STANDARD_UNIVERSE_DOMAIN_VISIT_CLASS: u32 = 1;
const PATH_NODE: u32 = 1;
const TOPOLOGY_SELECTOR_NODE: u32 = 2;
const COMPLETED_NODE: u32 = 3;
const FAILED_NODE: u32 = 4;
const FAULTED_NODE: u32 = 6;
const TRAILBLAZE_BONUS_NODE: u32 = 7;
const RESOLUTION_NODE_OFFSET: u32 = 10_000;
const CONTENT_NODE_OFFSET: u32 = 20_000;
const MEMBER_NODE_OFFSET: u32 = 30_000;
const BATTLE_NODE_OFFSET: u32 = 40_000;
const REWARD_NODE_OFFSET: u32 = 50_000;
const REWARD_RESOLUTION_NODE_OFFSET: u32 = 52_500;
const FORMATION_NODE_OFFSET: u32 = 55_000;
const ROUTE_NODE_OFFSET: u32 = 60_000;
const PATH_PROGRAM: u32 = 1;
const TOPOLOGY_PROGRAM: u32 = 2;
const TRAILBLAZE_BONUS_PROGRAM: u32 = 3;
const RESOLUTION_PROGRAM_OFFSET: u32 = 10_000;
const CONTENT_PROGRAM_OFFSET: u32 = 20_000;
const MEMBER_PROGRAM_OFFSET: u32 = 30_000;
const BATTLE_PROGRAM_OFFSET: u32 = 40_000;
const REWARD_PROGRAM_OFFSET: u32 = 50_000;
const REWARD_RESOLUTION_PROGRAM_OFFSET: u32 = 52_500;
const FORMATION_PROGRAM_OFFSET: u32 = 55_000;
const ROUTE_PROGRAM_OFFSET: u32 = 60_000;
const TOPOLOGY_DRAW_PURPOSE: u16 = 1;
const ROOM_DRAW_PURPOSE: u16 = 2;
const MEMBER_DRAW_PURPOSE: u16 = 3;
const BLESSING_DRAW_PURPOSE: u16 = 4;
const BLESSING_ENHANCEMENT_DRAW_PURPOSE: u16 = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainRouteDefinition {
    pub(super) option: ActivityOptionId,
    pub(super) target: Option<TopologyNodeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRoomContent {
    room: RoomId,
    domain_kind: DomainKind,
    kind: RoomContentKind,
    encounter_group: Option<EncounterGroupId>,
    source_content_id: Box<str>,
}

impl ResolvedRoomContent {
    pub(crate) fn new(
        room: RoomId,
        domain_kind: DomainKind,
        kind: RoomContentKind,
        encounter_group: Option<EncounterGroupId>,
        source_content_id: &str,
    ) -> Self {
        Self {
            room,
            domain_kind,
            kind,
            encounter_group,
            source_content_id: source_content_id.into(),
        }
    }

    #[must_use]
    pub const fn room(&self) -> RoomId {
        self.room
    }
    #[must_use]
    pub const fn domain_kind(&self) -> DomainKind {
        self.domain_kind
    }
    #[must_use]
    pub const fn kind(&self) -> RoomContentKind {
        self.kind
    }
    #[must_use]
    pub const fn encounter_group(&self) -> Option<EncounterGroupId> {
        self.encounter_group
    }
    #[must_use]
    pub fn source_content_id(&self) -> &str {
        &self.source_content_id
    }
}

/// One abstract domain micrograph. No coordinate or collision state is retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainHubDefinition {
    topology: TopologyId,
    source_node: TopologyNodeId,
    section_index: u32,
    resolution_node: NodeId,
    content_node: NodeId,
    member_node: NodeId,
    battle_node: NodeId,
    reward_node: NodeId,
    reward_resolution_node: Option<NodeId>,
    formation_node: NodeId,
    route_node: NodeId,
    eligible_rooms: Box<[RoomId]>,
    rooms: Box<[ResolvedRoomContent]>,
    routes: Box<[DomainRouteDefinition]>,
}

impl DomainHubDefinition {
    #[must_use]
    pub const fn topology(&self) -> TopologyId {
        self.topology
    }
    #[must_use]
    pub const fn source_node(&self) -> TopologyNodeId {
        self.source_node
    }
    #[must_use]
    pub const fn section_index(&self) -> u32 {
        self.section_index
    }
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.resolution_node
    }
    #[must_use]
    pub const fn content_node(&self) -> NodeId {
        self.content_node
    }
    #[must_use]
    pub const fn member_node(&self) -> NodeId {
        self.member_node
    }
    #[must_use]
    pub const fn battle_node(&self) -> NodeId {
        self.battle_node
    }
    #[must_use]
    pub const fn reward_node(&self) -> NodeId {
        self.reward_node
    }
    #[must_use]
    pub const fn reward_resolution_node(&self) -> Option<NodeId> {
        self.reward_resolution_node
    }
    #[must_use]
    pub const fn formation_node(&self) -> NodeId {
        self.formation_node
    }
    #[must_use]
    pub const fn route_node(&self) -> NodeId {
        self.route_node
    }
    #[must_use]
    pub fn eligible_rooms(&self) -> &[RoomId] {
        &self.eligible_rooms
    }
    #[must_use]
    pub fn rooms(&self) -> &[ResolvedRoomContent] {
        &self.rooms
    }
    #[must_use]
    pub fn routes(&self) -> &[DomainRouteDefinition] {
        &self.routes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncounterOptionBinding {
    option: ActivityOptionId,
    member: EncounterMemberId,
    room: RoomId,
    domain_kind: DomainKind,
}

impl EncounterOptionBinding {
    #[must_use]
    pub const fn option(self) -> ActivityOptionId {
        self.option
    }
    #[must_use]
    pub const fn member(self) -> EncounterMemberId {
        self.member
    }
    #[must_use]
    pub const fn room(self) -> RoomId {
        self.room
    }
    #[must_use]
    pub const fn domain_kind(self) -> DomainKind {
        self.domain_kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbstractInteractionBinding {
    node: NodeId,
    outcome: ActivityExternalOutcomeId,
    room: Option<RoomId>,
    kind: Option<RoomContentKind>,
    source_content_id: Box<str>,
    handler: u32,
    payload: Box<[u8]>,
    random_candidate_count: Option<u32>,
    random_label: Option<ActivityRngLabel>,
}

impl AbstractInteractionBinding {
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
    #[must_use]
    pub const fn outcome(&self) -> ActivityExternalOutcomeId {
        self.outcome
    }
    #[must_use]
    pub const fn room(&self) -> Option<RoomId> {
        self.room
    }
    #[must_use]
    pub const fn kind(&self) -> Option<RoomContentKind> {
        self.kind
    }
    #[must_use]
    pub fn source_content_id(&self) -> &str {
        &self.source_content_id
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompiledUniverseTopology {
    pub(crate) runtime: Arc<GraphActivityDefinition>,
    pub(crate) hubs: Arc<[DomainHubDefinition]>,
    pub(crate) candidates: Arc<[TopologyId]>,
    pub(crate) encounter_options: Arc<[EncounterOptionBinding]>,
    pub(crate) interactions: Arc<[AbstractInteractionBinding]>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile(
    catalog: &UniverseCatalog,
    blessing_runtime: &BlessingRuntimeCatalog,
    path_runtime: &PathRuntimeCatalog,
    identity: starclock_activity::ActivityDefinitionIdentity,
    state: ActivityStateDefinition,
    participants: Arc<ParticipantLock>,
    path_slot: ActivitySlotId,
    topology_slot: ActivitySlotId,
    hub_clear_slot: ActivitySlotId,
    room_slot: ActivitySlotId,
    member_slot: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    blessing_reroll_slot: ActivitySlotId,
    blessing_offer_marker_slot: ActivitySlotId,
    path_blessing_count_slot: ActivitySlotId,
    ability_projection_slot: ActivitySlotId,
    curio_bindings: crate::curio_activity::CurioActivityBindings,
    formation_capability_slot: ActivitySlotId,
    formation_inventory: ActivityInventoryId,
    occurrence_interactions: &OccurrenceInteractionRuntimeCatalog,
    service_interactions: &ServiceInteractionRuntimeCatalog,
    external_outcome_slot: ActivitySlotId,
    occurrence_effect_slot: ActivitySlotId,
    occurrence_battle_active_slot: ActivitySlotId,
    occurrence_battle_reward_count_slot: ActivitySlotId,
) -> Result<CompiledUniverseTopology, UniverseTopologyCompileError> {
    let mut occurrence_battle_hubs = BTreeSet::new();
    for topology in catalog.topologies() {
        for source in topology.nodes() {
            let rooms = resolve_rooms(catalog, source.source_node_id())?;
            if rooms
                .iter()
                .any(|room| room_has_occurrence_battle(catalog, room, occurrence_interactions))
            {
                occurrence_battle_hubs.insert(source.id());
            }
        }
    }
    let mut nodes = terminal_nodes()?;
    nodes.push(activity_node(PATH_NODE, 1, ActivityNodeKind::Choice)?);
    nodes.push(activity_node(
        TRAILBLAZE_BONUS_NODE,
        1,
        ActivityNodeKind::ExternalOutcome,
    )?);
    nodes.push(activity_node(
        TOPOLOGY_SELECTOR_NODE,
        1,
        ActivityNodeKind::Checkpoint,
    )?);
    let mut edges = Vec::new();
    let path_edge = push_edge(&mut edges, node(PATH_NODE), node(TRAILBLAZE_BONUS_NODE))?;
    let trailblaze_bonus_edge = push_edge(
        &mut edges,
        node(TRAILBLAZE_BONUS_NODE),
        node(TOPOLOGY_SELECTOR_NODE),
    )?;
    let mut topology_entry_edges = Vec::new();
    let mut topology_edges = Vec::new();
    let mut exit_edges = Vec::new();
    let mut hub_edges = Vec::new();
    let mut hubs = Vec::new();
    for topology in catalog.topologies() {
        let section_id = topology.source_map_id();
        for source in topology.nodes() {
            for (node_id, kind) in [
                (resolution_node(source.id()), ActivityNodeKind::Checkpoint),
                (content_node(source.id()), ActivityNodeKind::ExternalOutcome),
                (member_node(source.id()), ActivityNodeKind::Checkpoint),
                (battle_node(source.id()), ActivityNodeKind::Battle),
                (reward_node(source.id()), ActivityNodeKind::Reward),
                (formation_node(source.id()), ActivityNodeKind::Choice),
                (route_node(source.id()), ActivityNodeKind::Choice),
            ] {
                let maximum_visits = if node_id == content_node(source.id()) {
                    4
                } else if node_id == reward_node(source.id())
                    && occurrence_battle_hubs.contains(&source.id())
                {
                    8
                } else {
                    1
                };
                nodes.push(activity_node_with_visits(
                    node_id.get(),
                    section_id,
                    kind,
                    maximum_visits,
                )?);
            }
            if occurrence_battle_hubs.contains(&source.id()) {
                nodes.push(activity_node_with_visits(
                    reward_resolution_node(source.id()).get(),
                    section_id,
                    ActivityNodeKind::Choice,
                    8,
                )?);
            }
            hub_edges.push(build_hub_edges(
                &mut edges,
                source.id(),
                occurrence_battle_hubs.contains(&source.id()),
            )?);
        }
        topology_entry_edges.push((
            topology.id(),
            push_edge(
                &mut edges,
                node(TOPOLOGY_SELECTOR_NODE),
                resolution_node(topology.start()),
            )?,
        ));
        for source in topology.nodes() {
            let mut routes = Vec::new();
            if source.is_terminal() {
                let edge = push_edge(&mut edges, route_node(source.id()), node(COMPLETED_NODE))?;
                exit_edges.push((source.id(), edge));
                routes.push(DomainRouteDefinition {
                    option: exit_option(source.id().get()),
                    target: None,
                });
            } else {
                for target in source.outgoing() {
                    let edge = push_edge(
                        &mut edges,
                        route_node(source.id()),
                        resolution_node(*target),
                    )?;
                    topology_edges.push((source.id(), *target, edge));
                    routes.push(DomainRouteDefinition {
                        option: route_option(edge.get()),
                        target: Some(*target),
                    });
                }
            }
            let rooms = resolve_rooms(catalog, source.source_node_id())?;
            let eligible_rooms = rooms
                .iter()
                .map(ResolvedRoomContent::room)
                .collect::<Vec<_>>()
                .into_boxed_slice();
            hubs.push(DomainHubDefinition {
                topology: topology.id(),
                source_node: source.id(),
                section_index: source.source_node_id(),
                resolution_node: resolution_node(source.id()),
                content_node: content_node(source.id()),
                member_node: member_node(source.id()),
                battle_node: battle_node(source.id()),
                reward_node: reward_node(source.id()),
                reward_resolution_node: occurrence_battle_hubs
                    .contains(&source.id())
                    .then(|| reward_resolution_node(source.id())),
                formation_node: formation_node(source.id()),
                route_node: route_node(source.id()),
                eligible_rooms,
                rooms,
                routes: routes.into_boxed_slice(),
            });
        }
    }
    let graph = ActivityGraphDefinition::new(
        node(PATH_NODE),
        nodes,
        edges,
        u32::try_from(
            hubs.len()
                .saturating_mul(7)
                .saturating_add(occurrence_battle_hubs.len())
                .saturating_add(6),
        )
        .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    )
    .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?;
    let state = state.with_logical_scopes(domain_logical_scopes(&graph, &hubs)?);
    let CompiledPrograms {
        programs,
        random_checkpoints,
        random_offers,
        encounter_options,
        interactions,
    } = compile_programs(
        catalog,
        participants.as_ref(),
        path_slot,
        topology_slot,
        hub_clear_slot,
        room_slot,
        member_slot,
        blessing_runtime,
        path_runtime,
        blessing_inventory,
        blessing_reroll_slot,
        blessing_offer_marker_slot,
        path_blessing_count_slot,
        ability_projection_slot,
        curio_bindings,
        formation_capability_slot,
        formation_inventory,
        occurrence_interactions,
        service_interactions,
        external_outcome_slot,
        occurrence_effect_slot,
        occurrence_battle_active_slot,
        occurrence_battle_reward_count_slot,
        path_edge,
        trailblaze_bonus_edge,
        &topology_entry_edges,
        &topology_edges,
        &exit_edges,
        &hub_edges,
        &hubs,
    )?;
    let candidates = catalog
        .topologies()
        .iter()
        .map(|topology| topology.id())
        .collect::<Vec<_>>();
    let bootstrap = ActivityBootstrapSelection::new(
        topology_slot,
        ActivityRngLabel::Graph,
        TOPOLOGY_DRAW_PURPOSE,
        candidates
            .iter()
            .map(|topology| u64::from(topology.get()))
            .collect(),
    )
    .map_err(UniverseTopologyCompileError::RuntimeDefinition)?;
    let activity_interactions = programs
        .iter()
        .flat_map(|program| {
            program
                .program()
                .operations()
                .iter()
                .filter_map(|operation| match operation {
                    ActivityOperation::Offer { kind, options }
                        if *kind == ActivityDecisionKind::ExternalOutcome =>
                    {
                        Some(
                            options
                                .iter()
                                .map(|option| (program.node(), option.id()))
                                .collect::<Vec<_>>(),
                        )
                    }
                    _ => None,
                })
                .flatten()
        })
        .map(|(node, option)| {
            let outcome = ActivityExternalOutcomeId::new(option.get())
                .expect("offered option ID is non-zero");
            let authored = interactions
                .iter()
                .find(|binding| binding.node == node && binding.outcome == outcome);
            let payload = authored.map_or_else(
                || b"room-selection".to_vec(),
                |value| value.payload.to_vec(),
            );
            let handler = authored
                .map_or(STANDARD_UNIVERSE_EXTERNAL_INTERACTION_HANDLER_ID, |value| {
                    value.handler
                });
            let binding = ActivityInteractionBinding::new(
                node,
                outcome,
                starclock_activity::ActivityHandlerId::new(handler)
                    .expect("static handler ID is non-zero"),
                payload,
                "standard-universe.content.v4.4",
            )
            .expect("compiled source interaction identity is valid");
            authored
                .and_then(|value| value.random_candidate_count.zip(value.random_label))
                .map_or(binding.clone(), |(candidate_count, label)| {
                    binding.with_random_policy(
                        ActivityInteractionRandomPolicy::new(
                            label,
                            occurrence_random_purpose(node, outcome),
                            candidate_count,
                        )
                        .unwrap_or_else(|error| {
                            panic!(
                                "compiled Occurrence RNG policy is bounded: node={} outcome={} candidates={candidate_count} error={error:?}",
                                node.get(),
                                outcome.get(),
                            )
                        }),
                    )
                })
        })
        .collect();
    let runtime = GraphActivityDefinition::new(
        identity,
        graph,
        state,
        participants,
        programs,
        Some(bootstrap),
        ActivityRandomPolicies::new(random_checkpoints, random_offers),
    )
    .and_then(|definition| {
        definition.with_interactions(activity_handler_registry(), activity_interactions)
    })
    .map_err(UniverseTopologyCompileError::RuntimeDefinition)?;
    Ok(CompiledUniverseTopology {
        runtime: Arc::new(runtime),
        hubs: hubs.into(),
        candidates: candidates.into(),
        encounter_options: encounter_options.into(),
        interactions: interactions.into(),
    })
}

pub(crate) fn rebind(
    template: &CompiledUniverseTopology,
    identity: starclock_activity::ActivityDefinitionIdentity,
    state: ActivityStateDefinition,
    participants: Arc<ParticipantLock>,
) -> Result<CompiledUniverseTopology, UniverseTopologyCompileError> {
    let state =
        state.with_logical_scopes(template.runtime.state_definition().logical_scopes().clone());
    let runtime = template
        .runtime
        .rebind(identity, state, participants)
        .map_err(UniverseTopologyCompileError::RuntimeDefinition)?;
    Ok(CompiledUniverseTopology {
        runtime: Arc::new(runtime),
        hubs: Arc::clone(&template.hubs),
        candidates: Arc::clone(&template.candidates),
        encounter_options: Arc::clone(&template.encounter_options),
        interactions: Arc::clone(&template.interactions),
    })
}

struct CompiledPrograms {
    programs: Vec<GraphActivityNodeProgram>,
    random_checkpoints: Vec<ActivityRandomCheckpoint>,
    random_offers: Vec<ActivityRandomOffer>,
    encounter_options: Vec<EncounterOptionBinding>,
    interactions: Vec<AbstractInteractionBinding>,
}

#[allow(clippy::too_many_arguments)]
fn compile_programs(
    catalog: &UniverseCatalog,
    participants: &ParticipantLock,
    path_slot: ActivitySlotId,
    topology_slot: ActivitySlotId,
    hub_clear_slot: ActivitySlotId,
    room_slot: ActivitySlotId,
    member_slot: ActivitySlotId,
    blessing_runtime: &BlessingRuntimeCatalog,
    path_runtime: &PathRuntimeCatalog,
    blessing_inventory: ActivityInventoryId,
    blessing_reroll_slot: ActivitySlotId,
    blessing_offer_marker_slot: ActivitySlotId,
    path_blessing_count_slot: ActivitySlotId,
    ability_projection_slot: ActivitySlotId,
    curio_bindings: crate::curio_activity::CurioActivityBindings,
    formation_capability_slot: ActivitySlotId,
    formation_inventory: ActivityInventoryId,
    occurrence_interactions: &OccurrenceInteractionRuntimeCatalog,
    service_interactions: &ServiceInteractionRuntimeCatalog,
    external_outcome_slot: ActivitySlotId,
    occurrence_effect_slot: ActivitySlotId,
    occurrence_battle_active_slot: ActivitySlotId,
    occurrence_battle_reward_count_slot: ActivitySlotId,
    path_edge: ActivityEdgeId,
    trailblaze_bonus_edge: ActivityEdgeId,
    topology_entry_edges: &[(TopologyId, ActivityEdgeId)],
    topology_edges: &[(TopologyNodeId, TopologyNodeId, ActivityEdgeId)],
    exit_edges: &[(TopologyNodeId, ActivityEdgeId)],
    hub_edges: &[HubEdges],
    hubs: &[DomainHubDefinition],
) -> Result<CompiledPrograms, UniverseTopologyCompileError> {
    let path_options = catalog
        .paths()
        .iter()
        .enumerate()
        .map(|(priority, path)| {
            ActivityOptionDefinition::new(
                path_option(path.id().get()),
                priority as i32,
                always(),
                vec![
                    set_optional(path_slot, u64::from(path.id().get())),
                    ActivityOperation::Traverse(path_edge),
                ],
            )
        })
        .collect();
    let topology_options = topology_entry_edges
        .iter()
        .enumerate()
        .map(|(priority, (topology, edge))| {
            ActivityOptionDefinition::new(
                topology_option(topology.get()),
                priority as i32,
                optional_equals(topology_slot, u64::from(topology.get())),
                vec![ActivityOperation::Traverse(*edge)],
            )
        })
        .collect();
    let mut trailblaze_bonus_options = Vec::new();
    let mut interactions = Vec::new();
    for (service, tier, position) in service_interactions.trailblaze_bonuses() {
        let compiled = service_interactions
            .compile_selection(*service, &ServiceInteractionSelection::Activate)
            .map_err(|_| UniverseTopologyCompileError::InvalidServiceInteraction)?;
        let id = trailblaze_bonus_option(service.get());
        trailblaze_bonus_options.push(ActivityOptionDefinition::new(
            id,
            i32::from((*tier as u8) * 3 + *position),
            trailblaze_bonus_condition(
                service_interactions.cosmic_fragments_slot(),
                compiled.required_fragments(),
                ability_projection_slot,
                *tier,
            ),
            vec![ActivityOperation::Traverse(trailblaze_bonus_edge)],
        ));
        interactions.push(AbstractInteractionBinding {
            node: node(TRAILBLAZE_BONUS_NODE),
            outcome: ActivityExternalOutcomeId::new(id.get())
                .expect("Trailblaze Bonus option is non-zero"),
            room: None,
            kind: None,
            source_content_id: catalog
                .service(*service)
                .ok_or(UniverseTopologyCompileError::InvalidServiceInteraction)?
                .stable_key()
                .into(),
            handler: SERVICE_INTERACTION_HANDLER_ID,
            payload: compiled.payload().into(),
            random_candidate_count: compiled.random_candidate_count(),
            random_label: compiled
                .random_candidate_count()
                .map(|_| ActivityRngLabel::Reward),
        });
    }
    let mut programs = vec![
        node_program(
            PATH_NODE,
            PATH_PROGRAM,
            ActivityDecisionKind::Choice,
            path_options,
        )?,
        node_program(
            TRAILBLAZE_BONUS_NODE,
            TRAILBLAZE_BONUS_PROGRAM,
            ActivityDecisionKind::ExternalOutcome,
            trailblaze_bonus_options,
        )?,
        node_program(
            TOPOLOGY_SELECTOR_NODE,
            TOPOLOGY_PROGRAM,
            ActivityDecisionKind::Checkpoint,
            topology_options,
        )?,
    ];
    let mut random_checkpoints = Vec::new();
    let mut random_offers = Vec::new();
    let mut encounter_options = Vec::new();
    let blessing_eligibility = BlessingOfferEligibility::fully_unlocked(vec![1, 2, 3])
        .map_err(|_| UniverseTopologyCompileError::InvalidBlessingRuntime)?;
    let eligible_blessings = blessing_runtime
        .eligible(&blessing_eligibility)
        .collect::<Vec<_>>();
    for (index, hub) in hubs.iter().enumerate() {
        let edges = hub_edges[index];
        let source = u64::from(hub.source_node.get());
        let room_options = hub
            .rooms
            .iter()
            .enumerate()
            .map(|(priority, room)| {
                let id = room_option(source, room.room);
                ActivityOptionDefinition::new(
                    id,
                    priority as i32,
                    always(),
                    vec![
                        set_optional(room_slot, u64::from(room.room.get())),
                        ActivityOperation::SetSlot {
                            slot: member_slot,
                            value: ActivityExpression::Literal(ActivityValue::OptionalId(None)),
                        },
                        ActivityOperation::Traverse(edges.resolution_content),
                    ],
                )
            })
            .collect::<Vec<_>>();
        random_checkpoints.push(
            ActivityRandomCheckpoint::new(
                hub.resolution_node,
                ActivityRngLabel::Encounter,
                ROOM_DRAW_PURPOSE,
                room_options.iter().map(|value| (value.id(), 1)).collect(),
            )
            .map_err(UniverseTopologyCompileError::RuntimeDefinition)?,
        );
        programs.push(node_program_id(
            hub.resolution_node,
            RESOLUTION_PROGRAM_OFFSET + hub.source_node.get(),
            ActivityDecisionKind::Checkpoint,
            room_options,
        )?);
        let mut content_options = Vec::new();
        let mut member_options = Vec::new();
        let mut member_weights = Vec::new();
        let mut battle_options = Vec::new();
        for (room_priority, room) in hub.rooms.iter().enumerate() {
            let room_condition = optional_equals(room_slot, u64::from(room.room.get()));
            if let Some(group_id) = room.encounter_group {
                let group = catalog.encounter_group(group_id).ok_or(
                    UniverseTopologyCompileError::MissingEncounterGroup(group_id),
                )?;
                for (member_priority, member) in group.members().iter().enumerate() {
                    let member_id = member_option(source, room.room, member.id());
                    member_options.push(ActivityOptionDefinition::new(
                        member_id,
                        member_priority as i32,
                        room_condition.clone(),
                        vec![
                            set_optional(member_slot, u64::from(member.id().get())),
                            ActivityOperation::Traverse(edges.member_battle),
                        ],
                    ));
                    member_weights.push((member_id, exact_weight(member.weight())?));
                    let engage = engage_option(source, room.room, member.id());
                    battle_options.push(ActivityOptionDefinition::new(
                        engage,
                        member_priority as i32,
                        ActivityCondition::All(
                            vec![
                                room_condition.clone(),
                                optional_equals(member_slot, u64::from(member.id().get())),
                            ]
                            .into_boxed_slice(),
                        ),
                        Vec::new(),
                    ));
                    encounter_options.push(EncounterOptionBinding {
                        option: engage,
                        member: member.id(),
                        room: room.room,
                        domain_kind: room.domain_kind,
                    });
                }
                content_options.push(ActivityOptionDefinition::new(
                    content_option(source, room.room),
                    room_priority as i32,
                    room_condition,
                    vec![ActivityOperation::Traverse(edges.content_member)],
                ));
            } else if let Some(service_options) =
                compile_room_services(catalog, service_interactions, participants, room.room)?
            {
                for (service_priority, compiled) in service_options.into_iter().enumerate() {
                    let id = service_interaction_option(
                        source,
                        room.room,
                        u32::try_from(service_priority)
                            .map_err(|_| UniverseTopologyCompileError::InvalidServiceInteraction)?,
                    );
                    content_options.push(ActivityOptionDefinition::new(
                        id,
                        room_priority
                            .saturating_mul(256)
                            .saturating_add(service_priority) as i32,
                        service_option_condition(
                            room_condition.clone(),
                            service_interactions.cosmic_fragments_slot(),
                            compiled.required_fragments,
                            service_interactions.ability_projection_slot(),
                            compiled.required_ability,
                            compiled.required_defeated_participant,
                        ),
                        interaction_completion(
                            hub_clear_slot,
                            external_outcome_slot,
                            source,
                            edges.content_formation,
                        ),
                    ));
                    interactions.push(AbstractInteractionBinding {
                        node: hub.content_node,
                        outcome: ActivityExternalOutcomeId::new(id.get())
                            .expect("derived service option is non-zero"),
                        room: Some(room.room),
                        kind: Some(room.kind),
                        source_content_id: compiled.source_content_id,
                        handler: compiled.handler,
                        payload: compiled.payload,
                        random_candidate_count: compiled.random_candidate_count,
                        random_label: compiled.random_label,
                    });
                }
            } else if let Some(occurrence) = occurrence_for_source(catalog, &room.source_content_id)
            {
                let mut choice_ids = occurrence
                    .variants()
                    .iter()
                    .filter_map(|id| {
                        catalog
                            .occurrence_variants()
                            .iter()
                            .find(|value| value.id() == *id)
                    })
                    .flat_map(|variant| variant.choices().iter().copied())
                    .collect::<Vec<_>>();
                choice_ids.sort_unstable();
                choice_ids.dedup();
                if choice_ids.is_empty() {
                    return Err(UniverseTopologyCompileError::InvalidOccurrence);
                }
                for (choice_priority, choice_id) in choice_ids.iter().enumerate() {
                    let choice = catalog
                        .occurrence_choices()
                        .iter()
                        .find(|value| value.id() == *choice_id)
                        .ok_or(UniverseTopologyCompileError::InvalidOccurrence)?;
                    let compiled = occurrence_interactions
                        .compile_choice(choice.id())
                        .ok_or(UniverseTopologyCompileError::InvalidOccurrenceInteraction)?;
                    if compiled.external_results().is_empty() {
                        let id = occurrence_choice_option(source, room.room, choice.id().get());
                        let completion = if let Some(member) = compiled.battle_member() {
                            let member_id = member_option(source, room.room, member);
                            member_options.push(ActivityOptionDefinition::new(
                                member_id,
                                choice_priority as i32,
                                room_condition.clone(),
                                vec![
                                    set_optional(member_slot, u64::from(member.get())),
                                    ActivityOperation::Traverse(edges.member_battle),
                                ],
                            ));
                            member_weights.push((member_id, 1));
                            let engage = engage_option(source, room.room, member);
                            battle_options.push(ActivityOptionDefinition::new(
                                engage,
                                choice_priority as i32,
                                ActivityCondition::All(
                                    vec![
                                        room_condition.clone(),
                                        optional_equals(member_slot, u64::from(member.get())),
                                    ]
                                    .into_boxed_slice(),
                                ),
                                Vec::new(),
                            ));
                            encounter_options.push(EncounterOptionBinding {
                                option: engage,
                                member,
                                room: room.room,
                                domain_kind: room.domain_kind,
                            });
                            let finish = vec![
                                ActivityOperation::AddCounter {
                                    slot: external_outcome_slot,
                                    key: source,
                                    delta: integer(1),
                                },
                                set_optional(member_slot, u64::from(member.get())),
                                ActivityOperation::SetSlot {
                                    slot: occurrence_battle_active_slot,
                                    value: integer(1),
                                },
                                ActivityOperation::SetSlot {
                                    slot: occurrence_battle_reward_count_slot,
                                    value: integer(0),
                                },
                                ActivityOperation::Traverse(edges.content_member),
                            ];
                            progressive_battle_completion(
                                occurrence_effect_slot,
                                compiled.repeat_key(),
                                edges.content_repeat,
                                finish,
                            )
                        } else {
                            interaction_completion_with_repeat(
                                hub_clear_slot,
                                external_outcome_slot,
                                occurrence_effect_slot,
                                compiled.repeat_key(),
                                source,
                                edges.content_repeat,
                                edges.content_formation,
                            )
                        };
                        push_occurrence_interaction(
                            &mut content_options,
                            &mut interactions,
                            id,
                            room_priority,
                            choice_priority,
                            room_condition.clone(),
                            hub,
                            room,
                            completion,
                            choice.stable_key(),
                            compiled.payload(),
                            compiled.random_candidate_count(),
                        );
                    } else {
                        for (result_priority, result) in
                            compiled.external_results().iter().enumerate()
                        {
                            let id = occurrence_external_result_option(
                                source,
                                room.room,
                                choice.id().get(),
                                result.content(),
                            );
                            push_occurrence_interaction(
                                &mut content_options,
                                &mut interactions,
                                id,
                                room_priority,
                                choice_priority
                                    .saturating_mul(1_024)
                                    .saturating_add(result_priority),
                                room_condition.clone(),
                                hub,
                                room,
                                interaction_completion_with_repeat(
                                    hub_clear_slot,
                                    external_outcome_slot,
                                    occurrence_effect_slot,
                                    compiled.repeat_key(),
                                    source,
                                    edges.content_repeat,
                                    edges.content_formation,
                                ),
                                choice.stable_key(),
                                result.payload(),
                                result.random_candidate_count(),
                            );
                        }
                    }
                }
            } else {
                let id = interaction_option(source, room.room);
                content_options.push(ActivityOptionDefinition::new(
                    id,
                    room_priority as i32,
                    room_condition.clone(),
                    interaction_completion(
                        hub_clear_slot,
                        external_outcome_slot,
                        source,
                        edges.content_formation,
                    ),
                ));
                interactions.push(AbstractInteractionBinding {
                    node: hub.content_node,
                    outcome: ActivityExternalOutcomeId::new(id.get())
                        .expect("derived interaction option is non-zero"),
                    room: Some(room.room),
                    kind: Some(room.kind),
                    source_content_id: room.source_content_id.clone(),
                    handler: STANDARD_UNIVERSE_EXTERNAL_INTERACTION_HANDLER_ID,
                    payload: room.source_content_id.as_bytes().into(),
                    random_candidate_count: None,
                    random_label: None,
                });
            }
        }
        programs.push(node_program_id(
            hub.content_node,
            CONTENT_PROGRAM_OFFSET + hub.source_node.get(),
            ActivityDecisionKind::ExternalOutcome,
            content_options,
        )?);
        random_checkpoints.push(
            ActivityRandomCheckpoint::new(
                hub.member_node,
                ActivityRngLabel::Encounter,
                MEMBER_DRAW_PURPOSE,
                member_weights,
            )
            .map_err(UniverseTopologyCompileError::RuntimeDefinition)?,
        );
        programs.push(node_program_id(
            hub.member_node,
            MEMBER_PROGRAM_OFFSET + hub.source_node.get(),
            ActivityDecisionKind::Checkpoint,
            member_options,
        )?);
        programs.push(node_program_id(
            hub.battle_node,
            BATTLE_PROGRAM_OFFSET + hub.source_node.get(),
            ActivityDecisionKind::Encounter,
            battle_options,
        )?);
        let reward_completion = edges.reward_resolution.map_or(
            BlessingRewardCompletion::Inline {
                reward_formation: edges.reward_formation,
                hub_clear_slot,
                ability_projection_slot,
            },
            BlessingRewardCompletion::Resolution,
        );
        let reward = compile_blessing_reward(
            source,
            reward_completion,
            path_blessing_count_slot,
            blessing_offer_marker_slot,
            curio_bindings,
            blessing_inventory,
            occurrence_effect_slot,
            &eligible_blessings,
        )?;
        let random_offer = compile_blessing_offer_policy(
            catalog,
            hub.reward_node,
            source,
            reward.weights,
            blessing_reroll_slot,
            blessing_offer_marker_slot,
            curio_bindings,
            &eligible_blessings,
        )?;
        random_offers.push(random_offer);
        programs.push(reward_program::reward_node_program_id(
            hub.reward_node,
            REWARD_PROGRAM_OFFSET + hub.source_node.get(),
            reward.options,
            curio_bindings,
            hub_clear_slot,
            source,
            edges.reward_formation,
            occurrence_battle_active_slot,
            occurrence_battle_reward_count_slot,
        )?);
        if let (Some(node), Some(repeat), Some(finish)) = (
            hub.reward_resolution_node,
            edges.resolution_reward,
            edges.resolution_formation,
        ) {
            programs.push(compile_reward_resolution_program(
                node,
                REWARD_RESOLUTION_PROGRAM_OFFSET + hub.source_node.get(),
                source,
                hub_clear_slot,
                ability_projection_slot,
                occurrence_battle_active_slot,
                occurrence_battle_reward_count_slot,
                repeat,
                finish,
            )?);
        }
        let formation_options = path_runtime.formation_selection_options(
            FormationSelectionBindings {
                selected_path_slot: path_slot,
                path_blessing_count_slot,
                formation_capability_slot,
                formation_inventory,
            },
            formation_skip_option(source),
            |formation| formation_option(source, formation),
            &[
                ActivityOperation::SetSlot {
                    slot: occurrence_battle_active_slot,
                    value: integer(0),
                },
                ActivityOperation::SetSlot {
                    slot: occurrence_battle_reward_count_slot,
                    value: integer(0),
                },
                ActivityOperation::Traverse(edges.formation_route),
            ],
        );
        programs.push(node_program_id(
            hub.formation_node,
            FORMATION_PROGRAM_OFFSET + hub.source_node.get(),
            ActivityDecisionKind::Choice,
            formation_options,
        )?);
        programs.push(compile_route_program(
            hub,
            hub_clear_slot,
            topology_edges,
            exit_edges,
            curio_bindings,
        )?);
    }
    encounter_options.sort_by_key(|item| item.option);
    Ok(CompiledPrograms {
        programs,
        random_checkpoints,
        random_offers,
        encounter_options,
        interactions,
    })
}
