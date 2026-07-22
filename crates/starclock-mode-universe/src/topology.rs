//! Spatial-free Standard Universe topology and domain-hub compilation.

use std::sync::Arc;

use starclock_activity::{
    ActivityBootstrapSelection, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel,
    ActivitySlotId, ActivityStateDefinition, ActivityTerminalOutcome, ActivityValue,
    GraphActivityDefinition, GraphActivityDefinitionError, GraphActivityNodeProgram, NodeId,
    ParticipantLock, SectionId,
};

use crate::{
    catalog::UniverseCatalog,
    id::{RoomId, TopologyId, TopologyNodeId},
};

pub const STANDARD_UNIVERSE_TOPOLOGY_REVISION: &str = "standard-universe-topology-v1";

const PATH_NODE: u32 = 1;
const TOPOLOGY_SELECTOR_NODE: u32 = 2;
const HUB_NODE_OFFSET: u32 = 1_000;
const COMPLETED_NODE: u32 = 4_000;
const PATH_PROGRAM: u32 = 1;
const TOPOLOGY_PROGRAM: u32 = 2;
const HUB_PROGRAM_OFFSET: u32 = 1_000;
const PATH_OPTION_OFFSET: u64 = 1_000_000;
const TOPOLOGY_OPTION_OFFSET: u64 = 2_000_000;
const INTERACTION_OPTION_OFFSET: u64 = 3_000_000;
const ROUTE_OPTION_OFFSET: u64 = 4_000_000;
const EXIT_OPTION_OFFSET: u64 = 5_000_000;
const TOPOLOGY_DRAW_PURPOSE: u16 = 1;

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

/// One abstract hub. P3-B3 resolves one eligible room into concrete content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainHubDefinition {
    node: NodeId,
    topology: TopologyId,
    source_node: TopologyNodeId,
    section_index: u32,
    eligible_rooms: Box<[RoomId]>,
    interaction: ActivityOptionId,
    routes: Box<[DomainRouteDefinition]>,
}

impl DomainHubDefinition {
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
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
    pub fn eligible_rooms(&self) -> &[RoomId] {
        &self.eligible_rooms
    }
    #[must_use]
    pub const fn interaction(&self) -> ActivityOptionId {
        self.interaction
    }
    #[must_use]
    pub fn routes(&self) -> &[DomainRouteDefinition] {
        &self.routes
    }
}

pub(crate) struct CompiledUniverseTopology {
    pub(crate) runtime: Arc<GraphActivityDefinition>,
    pub(crate) hubs: Box<[DomainHubDefinition]>,
    pub(crate) candidates: Box<[TopologyId]>,
}

pub(crate) fn compile(
    catalog: &UniverseCatalog,
    identity: starclock_activity::ActivityDefinitionIdentity,
    state: ActivityStateDefinition,
    participants: Arc<ParticipantLock>,
    path_slot: ActivitySlotId,
    topology_slot: ActivitySlotId,
    hub_clear_slot: ActivitySlotId,
) -> Result<CompiledUniverseTopology, UniverseTopologyCompileError> {
    let path_node = node(PATH_NODE);
    let selector_node = node(TOPOLOGY_SELECTOR_NODE);
    let completed_node = node(COMPLETED_NODE);
    let mut nodes = vec![
        ActivityNodeDefinition::new(path_node, section(1), ActivityNodeKind::Choice, 1)
            .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
        ActivityNodeDefinition::new(selector_node, section(1), ActivityNodeKind::Checkpoint, 1)
            .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
        ActivityNodeDefinition::new(
            completed_node,
            section(1),
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    ];
    let mut edges = Vec::new();
    let path_edge = push_edge(&mut edges, path_node, selector_node)?;
    let mut topology_entry_edges = Vec::new();
    let mut topology_edges = Vec::new();
    let mut exit_edges = Vec::new();
    let mut hubs = Vec::new();

    for topology in catalog.topologies() {
        let section_id = section(topology.source_map_id());
        for source in topology.nodes() {
            nodes.push(
                ActivityNodeDefinition::new(
                    hub_node(source.id()),
                    section_id,
                    ActivityNodeKind::Choice,
                    1,
                )
                .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
            );
        }
        topology_entry_edges.push((
            topology.id(),
            push_edge(&mut edges, selector_node, hub_node(topology.start()))?,
        ));
        for source in topology.nodes() {
            let mut routes = Vec::new();
            if source.is_terminal() {
                let edge = push_edge(&mut edges, hub_node(source.id()), completed_node)?;
                exit_edges.push((source.id(), edge));
                routes.push(DomainRouteDefinition {
                    option: option(EXIT_OPTION_OFFSET + u64::from(source.id().get())),
                    target: None,
                });
            } else {
                for target in source.outgoing() {
                    let edge = push_edge(&mut edges, hub_node(source.id()), hub_node(*target))?;
                    topology_edges.push((source.id(), *target, edge));
                    routes.push(DomainRouteDefinition {
                        option: option(ROUTE_OPTION_OFFSET + u64::from(edge.get())),
                        target: Some(*target),
                    });
                }
            }
            let eligible_rooms = catalog
                .rooms()
                .iter()
                .filter(|room| room_is_eligible(room.section_ids(), source.source_node_id()))
                .map(|room| room.id())
                .collect::<Vec<_>>();
            if eligible_rooms.is_empty() {
                return Err(UniverseTopologyCompileError::NoEligibleRoom(source.id()));
            }
            hubs.push(DomainHubDefinition {
                node: hub_node(source.id()),
                topology: topology.id(),
                source_node: source.id(),
                section_index: source.source_node_id(),
                eligible_rooms: eligible_rooms.into_boxed_slice(),
                interaction: option(INTERACTION_OPTION_OFFSET + u64::from(source.id().get())),
                routes: routes.into_boxed_slice(),
            });
        }
    }

    let graph = ActivityGraphDefinition::new(
        path_node,
        nodes,
        edges,
        u32::try_from(
            catalog
                .topologies()
                .iter()
                .map(|value| value.nodes().len())
                .sum::<usize>()
                + 3,
        )
        .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    )
    .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?;
    let programs = compile_programs(
        catalog,
        path_slot,
        topology_slot,
        hub_clear_slot,
        path_edge,
        &topology_entry_edges,
        &topology_edges,
        &exit_edges,
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
    )
    .map_err(UniverseTopologyCompileError::RuntimeDefinition)?;
    Ok(CompiledUniverseTopology {
        runtime: Arc::new(runtime),
        hubs: hubs.into_boxed_slice(),
        candidates: candidates.into_boxed_slice(),
    })
}

#[allow(clippy::too_many_arguments)]
fn compile_programs(
    catalog: &UniverseCatalog,
    path_slot: ActivitySlotId,
    topology_slot: ActivitySlotId,
    hub_clear_slot: ActivitySlotId,
    path_edge: ActivityEdgeId,
    topology_entry_edges: &[(TopologyId, ActivityEdgeId)],
    topology_edges: &[(TopologyNodeId, TopologyNodeId, ActivityEdgeId)],
    exit_edges: &[(TopologyNodeId, ActivityEdgeId)],
    hubs: &[DomainHubDefinition],
) -> Result<Vec<GraphActivityNodeProgram>, UniverseTopologyCompileError> {
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
                    ActivityOperation::SetSlot {
                        slot: path_slot,
                        value: ActivityExpression::Literal(ActivityValue::OptionalId(Some(
                            u64::from(path.id().get()),
                        ))),
                    },
                    ActivityOperation::Traverse(path_edge),
                ],
            )
        })
        .collect::<Vec<_>>();
    let path_program = program(
        PATH_PROGRAM,
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Choice,
            options: path_options.into_boxed_slice(),
        }],
    )?;

    let topology_options = topology_entry_edges
        .iter()
        .enumerate()
        .map(|(priority, (topology, edge))| {
            ActivityOptionDefinition::new(
                option(TOPOLOGY_OPTION_OFFSET + u64::from(topology.get())),
                priority as i32,
                ActivityCondition::Equal(
                    ActivityExpression::Slot(topology_slot),
                    ActivityExpression::Literal(ActivityValue::OptionalId(Some(u64::from(
                        topology.get(),
                    )))),
                ),
                vec![ActivityOperation::Traverse(*edge)],
            )
        })
        .collect::<Vec<_>>();
    let topology_program = program(
        TOPOLOGY_PROGRAM,
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Checkpoint,
            options: topology_options.into_boxed_slice(),
        }],
    )?;

    let mut programs = vec![
        GraphActivityNodeProgram::new(node(PATH_NODE), path_program),
        GraphActivityNodeProgram::new(node(TOPOLOGY_SELECTOR_NODE), topology_program),
    ];
    for hub in hubs {
        let counter = ActivityExpression::CounterValue {
            slot: hub_clear_slot,
            key: u64::from(hub.source_node.get()),
        };
        let mut options = vec![ActivityOptionDefinition::new(
            hub.interaction,
            0,
            ActivityCondition::LessThan(
                counter.clone(),
                ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
            ),
            vec![ActivityOperation::AddCounter {
                slot: hub_clear_slot,
                key: u64::from(hub.source_node.get()),
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
            }],
        )];
        for (priority, route) in hub.routes.iter().enumerate() {
            let edge = match route.target {
                Some(target) => topology_edges
                    .iter()
                    .find(|(source, candidate, _)| {
                        *source == hub.source_node && *candidate == target
                    })
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
                i32::try_from(priority + 1).unwrap_or(i32::MAX),
                ActivityCondition::Equal(
                    counter.clone(),
                    ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                ),
                operations,
            ));
        }
        programs.push(GraphActivityNodeProgram::new(
            hub.node,
            program(
                HUB_PROGRAM_OFFSET + hub.source_node.get(),
                vec![ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Route,
                    options: options.into_boxed_slice(),
                }],
            )?,
        ));
    }
    Ok(programs)
}

fn room_is_eligible(section_ids: &[u32], source_node: u32) -> bool {
    section_ids.is_empty() || section_ids.contains(&0) || section_ids.contains(&source_node)
}

fn push_edge(
    edges: &mut Vec<ActivityEdgeDefinition>,
    from: NodeId,
    to: NodeId,
) -> Result<ActivityEdgeId, UniverseTopologyCompileError> {
    let id = ActivityEdgeId::new(
        u32::try_from(edges.len() + 1).map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    )
    .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
    edges.push(
        ActivityEdgeDefinition::new(id, from, to, ActivityEdgeCondition::OptionSelected, 0, 1)
            .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
    );
    Ok(id)
}

fn program(
    id: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseTopologyCompileError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(id).ok_or(UniverseTopologyCompileError::InvalidProgram)?,
        operations,
    )
    .map_err(|_| UniverseTopologyCompileError::InvalidProgram)
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

const fn hub_node(source: TopologyNodeId) -> NodeId {
    node(HUB_NODE_OFFSET + source.get())
}
const fn node(raw: u32) -> NodeId {
    match NodeId::new(raw) {
        Some(value) => value,
        None => panic!("static node identity is non-zero"),
    }
}
const fn section(raw: u32) -> SectionId {
    match SectionId::new(raw) {
        Some(value) => value,
        None => panic!("source map identity is non-zero"),
    }
}
const fn option(raw: u64) -> ActivityOptionId {
    match ActivityOptionId::new(raw) {
        Some(value) => value,
        None => panic!("static option identity is non-zero"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseTopologyCompileError {
    InvalidGraph,
    InvalidProgram,
    NoEligibleRoom(TopologyNodeId),
    RuntimeDefinition(GraphActivityDefinitionError),
}
