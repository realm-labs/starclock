//! Spatial-free Standard Universe topology, room and encounter compilation.

use std::sync::Arc;

use starclock_activity::{
    ActivityBootstrapSelection, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityExternalOutcomeId,
    ActivityGraphDefinition, ActivityInventoryId, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomCheckpoint, ActivityRandomOffer, ActivityRandomPolicies,
    ActivityRngLabel, ActivitySlotId, ActivityStateDefinition, ActivityTerminalOutcome,
    ActivityValue, GraphActivityDefinition, GraphActivityDefinitionError, GraphActivityNodeProgram,
    LogicalScopeAddress, LogicalScopeClassDefinition, LogicalScopeClassId, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId, ParticipantLock, SectionId, TerminalOutcome,
};

use crate::{
    blessing_runtime::{BlessingOfferEligibility, BlessingRuntimeCatalog},
    catalog::UniverseCatalog,
    encounter::RoomContentKind,
    id::{EncounterGroupId, EncounterMemberId, RoomId, TopologyId, TopologyNodeId},
    path::ExactParameter,
    path_runtime::{FormationSelectionBindings, PathRuntimeCatalog},
};

pub const STANDARD_UNIVERSE_TOPOLOGY_REVISION: &str = "standard-universe-topology-v4";
pub const STANDARD_UNIVERSE_DOMAIN_VISIT_CLASS: u32 = 1;

const PATH_NODE: u32 = 1;
const TOPOLOGY_SELECTOR_NODE: u32 = 2;
const COMPLETED_NODE: u32 = 3;
const FAILED_NODE: u32 = 4;
const FAULTED_NODE: u32 = 6;
const RESOLUTION_NODE_OFFSET: u32 = 10_000;
const CONTENT_NODE_OFFSET: u32 = 20_000;
const MEMBER_NODE_OFFSET: u32 = 30_000;
const BATTLE_NODE_OFFSET: u32 = 40_000;
const REWARD_NODE_OFFSET: u32 = 50_000;
const FORMATION_NODE_OFFSET: u32 = 55_000;
const ROUTE_NODE_OFFSET: u32 = 60_000;
const PATH_PROGRAM: u32 = 1;
const TOPOLOGY_PROGRAM: u32 = 2;
const RESOLUTION_PROGRAM_OFFSET: u32 = 10_000;
const CONTENT_PROGRAM_OFFSET: u32 = 20_000;
const MEMBER_PROGRAM_OFFSET: u32 = 30_000;
const BATTLE_PROGRAM_OFFSET: u32 = 40_000;
const REWARD_PROGRAM_OFFSET: u32 = 50_000;
const FORMATION_PROGRAM_OFFSET: u32 = 55_000;
const ROUTE_PROGRAM_OFFSET: u32 = 60_000;
const PATH_OPTION_OFFSET: u64 = 1_000_000;
const TOPOLOGY_OPTION_OFFSET: u64 = 2_000_000;
const ROOM_OPTION_OFFSET: u64 = 1_000_000_000_000;
const CONTENT_OPTION_OFFSET: u64 = 2_000_000_000_000;
const MEMBER_OPTION_OFFSET: u64 = 3_000_000_000_000;
const ENGAGE_OPTION_OFFSET: u64 = 4_000_000_000_000;
const INTERACTION_OPTION_OFFSET: u64 = 4_500_000_000_000;
const REWARD_OPTION_OFFSET: u64 = 5_000_000_000_000;
const FORMATION_OPTION_OFFSET: u64 = 5_500_000_000_000;
const FORMATION_SKIP_OPTION_OFFSET: u64 = 5_900_000_000_000;
const ROUTE_OPTION_OFFSET: u64 = 6_000_000_000_000;
const EXIT_OPTION_OFFSET: u64 = 7_000_000_000_000;
const TOPOLOGY_DRAW_PURPOSE: u16 = 1;
const ROOM_DRAW_PURPOSE: u16 = 2;
const MEMBER_DRAW_PURPOSE: u16 = 3;
const BLESSING_DRAW_PURPOSE: u16 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainRouteDefinition {
    option: ActivityOptionId,
    target: Option<TopologyNodeId>,
}

impl DomainRouteDefinition {
    #[must_use]
    pub const fn option(&self) -> ActivityOptionId {
        self.option
    }
    #[must_use]
    pub const fn target(&self) -> Option<TopologyNodeId> {
        self.target
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRoomContent {
    room: RoomId,
    kind: RoomContentKind,
    encounter_group: Option<EncounterGroupId>,
    source_content_id: Box<str>,
}

impl ResolvedRoomContent {
    #[must_use]
    pub const fn room(&self) -> RoomId {
        self.room
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbstractInteractionBinding {
    outcome: ActivityExternalOutcomeId,
    room: RoomId,
    kind: RoomContentKind,
    source_content_id: Box<str>,
}

impl AbstractInteractionBinding {
    #[must_use]
    pub const fn outcome(&self) -> ActivityExternalOutcomeId {
        self.outcome
    }
    #[must_use]
    pub const fn room(&self) -> RoomId {
        self.room
    }
    #[must_use]
    pub const fn kind(&self) -> RoomContentKind {
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
    path_blessing_count_slot: ActivitySlotId,
    formation_inventory: ActivityInventoryId,
    external_outcome_slot: ActivitySlotId,
) -> Result<CompiledUniverseTopology, UniverseTopologyCompileError> {
    let mut nodes = terminal_nodes()?;
    nodes.push(activity_node(PATH_NODE, 1, ActivityNodeKind::Choice)?);
    nodes.push(activity_node(
        TOPOLOGY_SELECTOR_NODE,
        1,
        ActivityNodeKind::Checkpoint,
    )?);
    let mut edges = Vec::new();
    let path_edge = push_edge(&mut edges, node(PATH_NODE), node(TOPOLOGY_SELECTOR_NODE))?;
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
                nodes.push(activity_node(node_id.get(), section_id, kind)?);
            }
            hub_edges.push(build_hub_edges(&mut edges, source.id())?);
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
                    option: option(EXIT_OPTION_OFFSET + u64::from(source.id().get())),
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
                        option: option(ROUTE_OPTION_OFFSET + u64::from(edge.get())),
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
        u32::try_from(hubs.len().saturating_mul(7).saturating_add(5))
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
        path_slot,
        topology_slot,
        hub_clear_slot,
        room_slot,
        member_slot,
        blessing_runtime,
        path_runtime,
        blessing_inventory,
        blessing_reroll_slot,
        path_blessing_count_slot,
        formation_inventory,
        external_outcome_slot,
        path_edge,
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
    let runtime = GraphActivityDefinition::new(
        identity,
        graph,
        state,
        participants,
        programs,
        Some(bootstrap),
        ActivityRandomPolicies::new(random_checkpoints, random_offers),
    )
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

fn domain_logical_scopes(
    graph: &ActivityGraphDefinition,
    hubs: &[DomainHubDefinition],
) -> Result<LogicalScopeDefinitions, UniverseTopologyCompileError> {
    let class = LogicalScopeClassId::new(STANDARD_UNIVERSE_DOMAIN_VISIT_CLASS)
        .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
    let class_definition =
        LogicalScopeClassDefinition::new(class, None, graph.maximum_total_visits())
            .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
    let mut bindings = Vec::with_capacity(hubs.len().saturating_mul(7));
    for hub in hubs {
        let address = LogicalScopeAddress::new(class, u64::from(hub.source_node.get()))
            .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
        for node in [
            hub.resolution_node,
            hub.content_node,
            hub.member_node,
            hub.battle_node,
            hub.reward_node,
            hub.formation_node,
            hub.route_node,
        ] {
            bindings.push(
                LogicalScopeNodeBinding::new(node, vec![address])
                    .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
            );
        }
    }
    LogicalScopeDefinitions::new(vec![class_definition], bindings)
        .map_err(|_| UniverseTopologyCompileError::InvalidGraph)
}

#[derive(Clone, Copy)]
struct HubEdges {
    resolution_content: ActivityEdgeId,
    content_member: ActivityEdgeId,
    content_formation: ActivityEdgeId,
    member_battle: ActivityEdgeId,
    reward_formation: ActivityEdgeId,
    formation_route: ActivityEdgeId,
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
    path_slot: ActivitySlotId,
    topology_slot: ActivitySlotId,
    hub_clear_slot: ActivitySlotId,
    room_slot: ActivitySlotId,
    member_slot: ActivitySlotId,
    blessing_runtime: &BlessingRuntimeCatalog,
    path_runtime: &PathRuntimeCatalog,
    blessing_inventory: ActivityInventoryId,
    blessing_reroll_slot: ActivitySlotId,
    path_blessing_count_slot: ActivitySlotId,
    formation_inventory: ActivityInventoryId,
    external_outcome_slot: ActivitySlotId,
    path_edge: ActivityEdgeId,
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
                option(PATH_OPTION_OFFSET + u64::from(path.id().get())),
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
                option(TOPOLOGY_OPTION_OFFSET + u64::from(topology.get())),
                priority as i32,
                optional_equals(topology_slot, u64::from(topology.get())),
                vec![ActivityOperation::Traverse(*edge)],
            )
        })
        .collect();
    let mut programs = vec![
        node_program(
            PATH_NODE,
            PATH_PROGRAM,
            ActivityDecisionKind::Choice,
            path_options,
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
    let mut interactions = Vec::new();
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
                    });
                }
                content_options.push(ActivityOptionDefinition::new(
                    content_option(source, room.room),
                    room_priority as i32,
                    room_condition,
                    vec![ActivityOperation::Traverse(edges.content_member)],
                ));
            } else {
                let id = interaction_option(source, room.room);
                content_options.push(ActivityOptionDefinition::new(
                    id,
                    room_priority as i32,
                    room_condition.clone(),
                    vec![
                        ActivityOperation::AddCounter {
                            slot: hub_clear_slot,
                            key: source,
                            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                        },
                        ActivityOperation::AddCounter {
                            slot: external_outcome_slot,
                            key: source,
                            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                        },
                        ActivityOperation::Traverse(edges.content_formation),
                    ],
                ));
                interactions.push(AbstractInteractionBinding {
                    outcome: ActivityExternalOutcomeId::new(id.get())
                        .expect("derived interaction option is non-zero"),
                    room: room.room,
                    kind: room.kind,
                    source_content_id: room.source_content_id.clone(),
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
        let mut reward_options = Vec::with_capacity(eligible_blessings.len());
        let mut reward_weights = Vec::with_capacity(eligible_blessings.len());
        for (priority, blessing) in eligible_blessings.iter().enumerate() {
            let id = blessing_option(source, blessing.blessing());
            let settlement = vec![
                ActivityOperation::AddCounter {
                    slot: path_blessing_count_slot,
                    key: u64::from(blessing.path().get()),
                    delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                },
                ActivityOperation::AddCounter {
                    slot: hub_clear_slot,
                    key: source,
                    delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                },
                ActivityOperation::Traverse(edges.reward_formation),
            ];
            reward_options.push(
                blessing_runtime
                    .acquisition_option(
                        blessing.blessing(),
                        id,
                        priority as i32,
                        blessing_inventory,
                        settlement,
                    )
                    .ok_or(UniverseTopologyCompileError::InvalidBlessingRuntime)?,
            );
            reward_weights.push((id, 1));
        }
        random_offers.push(
            ActivityRandomOffer::new(
                hub.reward_node,
                ActivityRngLabel::Reward,
                BLESSING_DRAW_PURPOSE,
                3,
                reward_weights,
                Some((blessing_reroll_slot, 1)),
            )
            .map_err(UniverseTopologyCompileError::RuntimeDefinition)?,
        );
        programs.push(node_program_id(
            hub.reward_node,
            REWARD_PROGRAM_OFFSET + hub.source_node.get(),
            ActivityDecisionKind::Reward,
            reward_options,
        )?);
        let formation_options = path_runtime.formation_selection_options(
            FormationSelectionBindings {
                selected_path_slot: path_slot,
                path_blessing_count_slot,
                formation_inventory,
            },
            formation_skip_option(source),
            |formation| formation_option(source, formation),
            &[ActivityOperation::Traverse(edges.formation_route)],
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

fn compile_route_program(
    hub: &DomainHubDefinition,
    hub_clear_slot: ActivitySlotId,
    topology_edges: &[(TopologyNodeId, TopologyNodeId, ActivityEdgeId)],
    exit_edges: &[(TopologyNodeId, ActivityEdgeId)],
) -> Result<GraphActivityNodeProgram, UniverseTopologyCompileError> {
    let cleared = ActivityCondition::Equal(
        ActivityExpression::CounterValue {
            slot: hub_clear_slot,
            key: u64::from(hub.source_node.get()),
        },
        ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
    );
    let mut options = Vec::new();
    for (priority, route) in hub.routes.iter().enumerate() {
        let edge = match route.target {
            Some(target) => topology_edges
                .iter()
                .find(|(source, candidate, _)| *source == hub.source_node && *candidate == target)
                .map(|(_, _, edge)| *edge),
            None => exit_edges
                .iter()
                .find(|(source, _)| *source == hub.source_node)
                .map(|(_, edge)| *edge),
        }
        .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
        let mut operations = vec![ActivityOperation::Traverse(edge)];
        if route.target.is_none() {
            operations.push(ActivityOperation::Terminal(
                ActivityTerminalOutcome::Completed,
            ));
        }
        options.push(ActivityOptionDefinition::new(
            route.option,
            priority as i32,
            cleared.clone(),
            operations,
        ));
    }
    node_program_id(
        hub.route_node,
        ROUTE_PROGRAM_OFFSET + hub.source_node.get(),
        ActivityDecisionKind::Route,
        options,
    )
}

fn terminal_nodes() -> Result<Vec<ActivityNodeDefinition>, UniverseTopologyCompileError> {
    [
        (COMPLETED_NODE, ActivityTerminalOutcome::Completed),
        (FAILED_NODE, ActivityTerminalOutcome::Failed),
        (FAULTED_NODE, ActivityTerminalOutcome::Faulted),
    ]
    .into_iter()
    .map(|(id, outcome)| activity_node(id, 1, ActivityNodeKind::Terminal(outcome)))
    .collect()
}

fn build_hub_edges(
    edges: &mut Vec<ActivityEdgeDefinition>,
    source: TopologyNodeId,
) -> Result<HubEdges, UniverseTopologyCompileError> {
    let resolution_content = push_edge(edges, resolution_node(source), content_node(source))?;
    let content_member = push_edge(edges, content_node(source), member_node(source))?;
    let content_formation = push_edge(edges, content_node(source), formation_node(source))?;
    let member_battle = push_edge(edges, member_node(source), battle_node(source))?;
    push_condition_edge(
        edges,
        battle_node(source),
        reward_node(source),
        ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete),
    )?;
    push_condition_edge(
        edges,
        battle_node(source),
        node(FAILED_NODE),
        ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Failed),
    )?;
    push_condition_edge(
        edges,
        battle_node(source),
        node(FAULTED_NODE),
        ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Faulted),
    )?;
    let reward_formation = push_edge(edges, reward_node(source), formation_node(source))?;
    let formation_route = push_edge(edges, formation_node(source), route_node(source))?;
    Ok(HubEdges {
        resolution_content,
        content_member,
        content_formation,
        member_battle,
        reward_formation,
        formation_route,
    })
}

fn resolve_rooms(
    catalog: &UniverseCatalog,
    source_node: u32,
) -> Result<Box<[ResolvedRoomContent]>, UniverseTopologyCompileError> {
    let mut resolved = Vec::new();
    for room in catalog
        .rooms()
        .iter()
        .filter(|room| room_is_eligible(room.section_ids(), source_node))
    {
        let mut bindings = catalog.room_content().iter().filter(|binding| {
            binding.room() == room.id() && binding.condition_key() == room.source_group_id()
        });
        let binding =
            bindings
                .next()
                .ok_or(UniverseTopologyCompileError::MissingPrimaryRoomContent(
                    room.id(),
                ))?;
        if bindings.next().is_some() {
            return Err(UniverseTopologyCompileError::AmbiguousPrimaryRoomContent(
                room.id(),
            ));
        }
        resolved.push(ResolvedRoomContent {
            room: room.id(),
            kind: binding.kind(),
            encounter_group: binding.encounter_group(),
            source_content_id: binding.source_content_id().into(),
        });
    }
    if resolved.is_empty() {
        return Err(UniverseTopologyCompileError::NoEligibleRoom(
            TopologyNodeId::new(source_node).ok_or(UniverseTopologyCompileError::InvalidGraph)?,
        ));
    }
    Ok(resolved.into_boxed_slice())
}

fn exact_weight(value: ExactParameter) -> Result<u64, UniverseTopologyCompileError> {
    if value.coefficient() <= 0 || value.scale() > 6 {
        return Err(UniverseTopologyCompileError::InvalidEncounterWeight);
    }
    let multiplier = 10_u64
        .checked_pow(u32::from(6 - value.scale()))
        .ok_or(UniverseTopologyCompileError::InvalidEncounterWeight)?;
    u64::try_from(value.coefficient())
        .ok()
        .and_then(|coefficient| coefficient.checked_mul(multiplier))
        .ok_or(UniverseTopologyCompileError::InvalidEncounterWeight)
}

fn node_program(
    node_id: u32,
    program_id: u32,
    kind: ActivityDecisionKind,
    options: Vec<ActivityOptionDefinition>,
) -> Result<GraphActivityNodeProgram, UniverseTopologyCompileError> {
    node_program_id(node(node_id), program_id, kind, options)
}

fn node_program_id(
    node_id: NodeId,
    program_id: u32,
    kind: ActivityDecisionKind,
    mut options: Vec<ActivityOptionDefinition>,
) -> Result<GraphActivityNodeProgram, UniverseTopologyCompileError> {
    options.sort_by_key(|option| (option.priority(), option.id()));
    Ok(GraphActivityNodeProgram::new(
        node_id,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(program_id).ok_or(UniverseTopologyCompileError::InvalidGraph)?,
            vec![ActivityOperation::Offer {
                kind,
                options: options.into_boxed_slice(),
            }],
        )
        .map_err(|_| UniverseTopologyCompileError::InvalidProgram)?,
    ))
}

fn optional_equals(slot: ActivitySlotId, value: u64) -> ActivityCondition {
    ActivityCondition::Equal(
        ActivityExpression::Slot(slot),
        ActivityExpression::Literal(ActivityValue::OptionalId(Some(value))),
    )
}

fn set_optional(slot: ActivitySlotId, value: u64) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(ActivityValue::OptionalId(Some(value))),
    }
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

fn activity_node(
    id: u32,
    section_id: u32,
    kind: ActivityNodeKind,
) -> Result<ActivityNodeDefinition, UniverseTopologyCompileError> {
    ActivityNodeDefinition::new(node(id), section(section_id), kind, 1)
        .map_err(|_| UniverseTopologyCompileError::InvalidGraph)
}

fn push_edge(
    edges: &mut Vec<ActivityEdgeDefinition>,
    from: NodeId,
    to: NodeId,
) -> Result<ActivityEdgeId, UniverseTopologyCompileError> {
    push_condition_edge(edges, from, to, ActivityEdgeCondition::Always)
}

fn push_condition_edge(
    edges: &mut Vec<ActivityEdgeDefinition>,
    from: NodeId,
    to: NodeId,
    condition: ActivityEdgeCondition,
) -> Result<ActivityEdgeId, UniverseTopologyCompileError> {
    let id = ActivityEdgeId::new(
        u32::try_from(edges.len() + 1).map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    )
    .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
    edges.push(
        ActivityEdgeDefinition::new(id, from, to, condition, 0, 1)
            .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    );
    Ok(id)
}

const fn resolution_node(source: TopologyNodeId) -> NodeId {
    node(RESOLUTION_NODE_OFFSET + source.get())
}
const fn content_node(source: TopologyNodeId) -> NodeId {
    node(CONTENT_NODE_OFFSET + source.get())
}
const fn member_node(source: TopologyNodeId) -> NodeId {
    node(MEMBER_NODE_OFFSET + source.get())
}
const fn battle_node(source: TopologyNodeId) -> NodeId {
    node(BATTLE_NODE_OFFSET + source.get())
}
const fn reward_node(source: TopologyNodeId) -> NodeId {
    node(REWARD_NODE_OFFSET + source.get())
}
const fn formation_node(source: TopologyNodeId) -> NodeId {
    node(FORMATION_NODE_OFFSET + source.get())
}
const fn route_node(source: TopologyNodeId) -> NodeId {
    node(ROUTE_NODE_OFFSET + source.get())
}

fn room_option(source: u64, room: RoomId) -> ActivityOptionId {
    option(ROOM_OPTION_OFFSET + source * 1_000 + u64::from(room.get()))
}
fn content_option(source: u64, room: RoomId) -> ActivityOptionId {
    option(CONTENT_OPTION_OFFSET + source * 1_000 + u64::from(room.get()))
}
fn member_option(source: u64, room: RoomId, member: EncounterMemberId) -> ActivityOptionId {
    option(
        MEMBER_OPTION_OFFSET
            + source * 1_000_000
            + u64::from(room.get()) * 1_000
            + u64::from(member.get()),
    )
}
fn engage_option(source: u64, room: RoomId, member: EncounterMemberId) -> ActivityOptionId {
    option(
        ENGAGE_OPTION_OFFSET
            + source * 1_000_000
            + u64::from(room.get()) * 1_000
            + u64::from(member.get()),
    )
}
fn interaction_option(source: u64, room: RoomId) -> ActivityOptionId {
    option(INTERACTION_OPTION_OFFSET + source * 10_000_000 + u64::from(room.get()))
}
fn blessing_option(source: u64, blessing: crate::id::BlessingId) -> ActivityOptionId {
    option(REWARD_OPTION_OFFSET + source * 1_000_000 + u64::from(blessing.get()))
}
fn formation_option(source: u64, formation: crate::id::ResonanceId) -> ActivityOptionId {
    option(FORMATION_OPTION_OFFSET + source * 1_000_000 + u64::from(formation.get()))
}
fn formation_skip_option(source: u64) -> ActivityOptionId {
    option(FORMATION_SKIP_OPTION_OFFSET + source)
}

fn room_is_eligible(section_ids: &[u32], source_node: u32) -> bool {
    section_ids.is_empty() || section_ids.contains(&0) || section_ids.contains(&source_node)
}
const fn node(raw: u32) -> NodeId {
    match NodeId::new(raw) {
        Some(value) => value,
        None => panic!("static node ID must be non-zero"),
    }
}
const fn section(raw: u32) -> SectionId {
    match SectionId::new(raw) {
        Some(value) => value,
        None => panic!("static section ID must be non-zero"),
    }
}
fn option(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).expect("derived option ID is non-zero")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseTopologyCompileError {
    InvalidGraph,
    InvalidProgram,
    InvalidEncounterWeight,
    InvalidBlessingRuntime,
    NoEligibleRoom(TopologyNodeId),
    MissingPrimaryRoomContent(RoomId),
    AmbiguousPrimaryRoomContent(RoomId),
    MissingEncounterGroup(EncounterGroupId),
    RuntimeDefinition(GraphActivityDefinitionError),
}
