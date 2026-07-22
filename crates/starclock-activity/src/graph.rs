use std::collections::BTreeSet;

use crate::{
    ActivityEdgeId, ActivityGraphDigest, NodeId, OneBattleFlow, SectionId, TerminalOutcome,
    codec::ActivityV2Writer,
};

/// Hard definition bounds keep validation, hashing and future traversal finite.
pub const MAX_ACTIVITY_NODES: usize = 4_096;
pub const MAX_ACTIVITY_EDGES: usize = 16_384;
pub const MAX_ACTIVITY_TOTAL_VISITS: u32 = 1_000_000;
pub const MAX_NODE_VISITS: u32 = 65_535;
pub const MAX_EDGE_TRAVERSALS: u32 = 65_535;

/// Terminal settlement of a graph Activity. This is distinct from the legacy
/// one-battle outcome so adding abandonment cannot change legacy enum tags.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityTerminalOutcome {
    Completed = 0,
    Failed = 1,
    Abandoned = 2,
    Faulted = 3,
}

/// Generic node responsibility. Mode-specific vocabulary must compile to this set.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActivityNodeKind {
    Battle,
    Choice,
    Reward,
    Shop,
    Roster,
    ExternalOutcome,
    Checkpoint,
    ForkJoin,
    Terminal(ActivityTerminalOutcome),
}

impl ActivityNodeKind {
    const fn tag(self) -> u8 {
        match self {
            Self::Battle => 0,
            Self::Choice => 1,
            Self::Reward => 2,
            Self::Shop => 3,
            Self::Roster => 4,
            Self::ExternalOutcome => 5,
            Self::Checkpoint => 6,
            Self::ForkJoin => 7,
            Self::Terminal(_) => 8,
        }
    }

    #[must_use]
    pub const fn terminal(self) -> Option<ActivityTerminalOutcome> {
        match self {
            Self::Terminal(outcome) => Some(outcome),
            _ => None,
        }
    }
}

/// Initial closed branching vocabulary. Later expression batches may add a
/// validated predicate reference without changing authored edge identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActivityEdgeCondition {
    Always,
    BattleOutcome(TerminalOutcome),
    OptionSelected,
    ExternalOutcomeSubmitted,
}

impl ActivityEdgeCondition {
    const fn tag(self) -> u8 {
        match self {
            Self::Always => 0,
            Self::BattleOutcome(_) => 1,
            Self::OptionSelected => 2,
            Self::ExternalOutcomeSubmitted => 3,
        }
    }
}

/// One immutable graph node with an explicit per-node visit budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityNodeDefinition {
    id: NodeId,
    section: SectionId,
    kind: ActivityNodeKind,
    maximum_visits: u32,
}

impl ActivityNodeDefinition {
    pub fn new(
        id: NodeId,
        section: SectionId,
        kind: ActivityNodeKind,
        maximum_visits: u32,
    ) -> Result<Self, ActivityGraphDefinitionError> {
        if maximum_visits == 0 || maximum_visits > MAX_NODE_VISITS {
            return Err(ActivityGraphDefinitionError::InvalidNodeVisitLimit(id));
        }
        Ok(Self {
            id,
            section,
            kind,
            maximum_visits,
        })
    }

    #[must_use]
    pub const fn id(self) -> NodeId {
        self.id
    }
    #[must_use]
    pub const fn section(self) -> SectionId {
        self.section
    }
    #[must_use]
    pub const fn kind(self) -> ActivityNodeKind {
        self.kind
    }
    #[must_use]
    pub const fn maximum_visits(self) -> u32 {
        self.maximum_visits
    }

    fn encode(self, writer: &mut ActivityV2Writer) {
        writer.u32(self.id.get());
        writer.u32(self.section.get());
        writer.byte(self.kind.tag());
        if let ActivityNodeKind::Terminal(outcome) = self.kind {
            writer.byte(outcome as u8);
        }
        writer.u32(self.maximum_visits);
    }
}

/// One immutable directed edge with stable ordering and traversal budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityEdgeDefinition {
    id: ActivityEdgeId,
    from: NodeId,
    to: NodeId,
    condition: ActivityEdgeCondition,
    priority: i32,
    maximum_traversals: u32,
}

impl ActivityEdgeDefinition {
    pub fn new(
        id: ActivityEdgeId,
        from: NodeId,
        to: NodeId,
        condition: ActivityEdgeCondition,
        priority: i32,
        maximum_traversals: u32,
    ) -> Result<Self, ActivityGraphDefinitionError> {
        if maximum_traversals == 0 || maximum_traversals > MAX_EDGE_TRAVERSALS {
            return Err(ActivityGraphDefinitionError::InvalidEdgeTraversalLimit(id));
        }
        Ok(Self {
            id,
            from,
            to,
            condition,
            priority,
            maximum_traversals,
        })
    }

    #[must_use]
    pub const fn id(self) -> ActivityEdgeId {
        self.id
    }
    #[must_use]
    pub const fn from(self) -> NodeId {
        self.from
    }
    #[must_use]
    pub const fn to(self) -> NodeId {
        self.to
    }
    #[must_use]
    pub const fn condition(self) -> ActivityEdgeCondition {
        self.condition
    }
    #[must_use]
    pub const fn priority(self) -> i32 {
        self.priority
    }
    #[must_use]
    pub const fn maximum_traversals(self) -> u32 {
        self.maximum_traversals
    }

    fn encode(self, writer: &mut ActivityV2Writer) {
        writer.u32(self.id.get());
        writer.u32(self.from.get());
        writer.u32(self.to.get());
        writer.byte(self.condition.tag());
        if let ActivityEdgeCondition::BattleOutcome(outcome) = self.condition {
            writer.byte(outcome as u8);
        }
        writer.i32(self.priority);
        writer.u32(self.maximum_traversals);
    }
}

/// Canonical validated directed Activity graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityGraphDefinition {
    entry: NodeId,
    nodes: Box<[ActivityNodeDefinition]>,
    edges: Box<[ActivityEdgeDefinition]>,
    maximum_total_visits: u32,
    digest: ActivityGraphDigest,
}

impl ActivityGraphDefinition {
    pub fn new(
        entry: NodeId,
        mut nodes: Vec<ActivityNodeDefinition>,
        mut edges: Vec<ActivityEdgeDefinition>,
        maximum_total_visits: u32,
    ) -> Result<Self, ActivityGraphDefinitionError> {
        validate_collection_bounds(&nodes, &edges, maximum_total_visits)?;
        nodes.sort_by_key(|node| node.id());
        edges.sort_by_key(|edge| edge.id());
        validate_unique_ids(&nodes, &edges)?;
        let entry_index = node_index(&nodes, entry)
            .ok_or(ActivityGraphDefinitionError::MissingEntryNode(entry))?;
        validate_edges(&nodes, &edges)?;
        validate_reachability(&nodes, &edges, entry_index)?;
        let digest = digest_graph(entry, &nodes, &edges, maximum_total_visits);
        Ok(Self {
            entry,
            nodes: nodes.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
            maximum_total_visits,
            digest,
        })
    }

    pub(crate) fn one_battle(flow: OneBattleFlow) -> Self {
        let section = flow.section();
        let nodes = vec![
            node(flow.battle_node(), section, ActivityNodeKind::Battle),
            node(
                flow.terminal_node(TerminalOutcome::Complete),
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            ),
            node(
                flow.terminal_node(TerminalOutcome::Failed),
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
            ),
            node(
                flow.terminal_node(TerminalOutcome::Faulted),
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
            ),
        ];
        let outcomes = [
            TerminalOutcome::Complete,
            TerminalOutcome::Failed,
            TerminalOutcome::Faulted,
        ];
        let edges = outcomes
            .into_iter()
            .enumerate()
            .map(|(index, outcome)| {
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(index as u32 + 1).expect("one-based edge ID"),
                    flow.battle_node(),
                    flow.terminal_node(outcome),
                    ActivityEdgeCondition::BattleOutcome(outcome),
                    index as i32,
                    1,
                )
                .expect("one-battle edge bounds are valid")
            })
            .collect();
        Self::new(flow.battle_node(), nodes, edges, 2)
            .expect("OneBattleFlow already owns distinct nodes and bounded edges")
    }

    #[must_use]
    pub const fn entry(&self) -> NodeId {
        self.entry
    }
    #[must_use]
    pub fn nodes(&self) -> &[ActivityNodeDefinition] {
        &self.nodes
    }
    #[must_use]
    pub fn edges(&self) -> &[ActivityEdgeDefinition] {
        &self.edges
    }
    #[must_use]
    pub const fn maximum_total_visits(&self) -> u32 {
        self.maximum_total_visits
    }
    #[must_use]
    pub const fn digest(&self) -> ActivityGraphDigest {
        self.digest
    }
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&ActivityNodeDefinition> {
        node_index(&self.nodes, id).map(|index| &self.nodes[index])
    }
    pub fn outgoing(&self, id: NodeId) -> impl Iterator<Item = &ActivityEdgeDefinition> {
        self.edges.iter().filter(move |edge| edge.from == id)
    }
}

fn node(id: NodeId, section: SectionId, kind: ActivityNodeKind) -> ActivityNodeDefinition {
    ActivityNodeDefinition::new(id, section, kind, 1).expect("one is a valid visit limit")
}

fn validate_collection_bounds(
    nodes: &[ActivityNodeDefinition],
    edges: &[ActivityEdgeDefinition],
    maximum_total_visits: u32,
) -> Result<(), ActivityGraphDefinitionError> {
    if nodes.is_empty() {
        return Err(ActivityGraphDefinitionError::EmptyGraph);
    }
    if nodes.len() > MAX_ACTIVITY_NODES {
        return Err(ActivityGraphDefinitionError::TooManyNodes);
    }
    if edges.len() > MAX_ACTIVITY_EDGES {
        return Err(ActivityGraphDefinitionError::TooManyEdges);
    }
    if maximum_total_visits == 0 || maximum_total_visits > MAX_ACTIVITY_TOTAL_VISITS {
        return Err(ActivityGraphDefinitionError::InvalidTotalVisitLimit);
    }
    if nodes
        .iter()
        .any(|node| node.maximum_visits > maximum_total_visits)
    {
        return Err(ActivityGraphDefinitionError::NodeLimitExceedsTotal);
    }
    if edges
        .iter()
        .any(|edge| edge.maximum_traversals > maximum_total_visits)
    {
        return Err(ActivityGraphDefinitionError::EdgeLimitExceedsTotal);
    }
    Ok(())
}

fn validate_unique_ids(
    nodes: &[ActivityNodeDefinition],
    edges: &[ActivityEdgeDefinition],
) -> Result<(), ActivityGraphDefinitionError> {
    if let Some(pair) = nodes.windows(2).find(|pair| pair[0].id == pair[1].id) {
        return Err(ActivityGraphDefinitionError::DuplicateNode(pair[0].id));
    }
    if let Some(pair) = edges.windows(2).find(|pair| pair[0].id == pair[1].id) {
        return Err(ActivityGraphDefinitionError::DuplicateEdge(pair[0].id));
    }
    Ok(())
}

fn validate_edges(
    nodes: &[ActivityNodeDefinition],
    edges: &[ActivityEdgeDefinition],
) -> Result<(), ActivityGraphDefinitionError> {
    for edge in edges {
        if node_index(nodes, edge.from).is_none() {
            return Err(ActivityGraphDefinitionError::MissingEdgeSource(edge.id));
        }
        if node_index(nodes, edge.to).is_none() {
            return Err(ActivityGraphDefinitionError::MissingEdgeTarget(edge.id));
        }
        if nodes[node_index(nodes, edge.from).expect("checked above")]
            .kind
            .terminal()
            .is_some()
        {
            return Err(ActivityGraphDefinitionError::TerminalHasOutgoingEdge(
                edge.from,
            ));
        }
    }
    for node in nodes {
        let has_outgoing = edges.iter().any(|edge| edge.from == node.id);
        if node.kind.terminal().is_none() && !has_outgoing {
            return Err(ActivityGraphDefinitionError::NonTerminalHasNoExit(node.id));
        }
    }
    Ok(())
}

fn validate_reachability(
    nodes: &[ActivityNodeDefinition],
    edges: &[ActivityEdgeDefinition],
    entry_index: usize,
) -> Result<(), ActivityGraphDefinitionError> {
    let forward = walk(nodes, edges, [nodes[entry_index].id], false);
    if let Some(node) = nodes.iter().find(|node| !forward.contains(&node.id)) {
        return Err(ActivityGraphDefinitionError::UnreachableNode(node.id));
    }
    let terminals = nodes
        .iter()
        .filter(|node| node.kind.terminal().is_some())
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if terminals.is_empty() {
        return Err(ActivityGraphDefinitionError::MissingTerminal);
    }
    let reverse = walk(nodes, edges, terminals, true);
    if let Some(node) = nodes.iter().find(|node| !reverse.contains(&node.id)) {
        return Err(ActivityGraphDefinitionError::CannotReachTerminal(node.id));
    }
    Ok(())
}

fn walk(
    _nodes: &[ActivityNodeDefinition],
    edges: &[ActivityEdgeDefinition],
    starts: impl IntoIterator<Item = NodeId>,
    reverse: bool,
) -> BTreeSet<NodeId> {
    let mut seen = BTreeSet::new();
    let mut pending = starts.into_iter().collect::<Vec<_>>();
    while let Some(current) = pending.pop() {
        if !seen.insert(current) {
            continue;
        }
        for edge in edges {
            let (source, target) = if reverse {
                (edge.to, edge.from)
            } else {
                (edge.from, edge.to)
            };
            if source == current && !seen.contains(&target) {
                pending.push(target);
            }
        }
    }
    seen
}

fn node_index(nodes: &[ActivityNodeDefinition], id: NodeId) -> Option<usize> {
    nodes.binary_search_by_key(&id, |node| node.id).ok()
}

fn digest_graph(
    entry: NodeId,
    nodes: &[ActivityNodeDefinition],
    edges: &[ActivityEdgeDefinition],
    maximum_total_visits: u32,
) -> ActivityGraphDigest {
    let mut writer = ActivityV2Writer::new(*b"SCAG", 1, b"starclock-activity-graph-definition-v1");
    writer.u32(entry.get());
    writer.u32(maximum_total_visits);
    writer.u32(nodes.len() as u32);
    for node in nodes {
        node.encode(&mut writer);
    }
    writer.u32(edges.len() as u32);
    for edge in edges {
        edge.encode(&mut writer);
    }
    ActivityGraphDigest::new(writer.finish()).expect("SHA-256 graph digest is non-zero")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityGraphDefinitionError {
    EmptyGraph,
    TooManyNodes,
    TooManyEdges,
    InvalidTotalVisitLimit,
    InvalidNodeVisitLimit(NodeId),
    InvalidEdgeTraversalLimit(ActivityEdgeId),
    NodeLimitExceedsTotal,
    EdgeLimitExceedsTotal,
    DuplicateNode(NodeId),
    DuplicateEdge(ActivityEdgeId),
    MissingEntryNode(NodeId),
    MissingEdgeSource(ActivityEdgeId),
    MissingEdgeTarget(ActivityEdgeId),
    TerminalHasOutgoingEdge(NodeId),
    NonTerminalHasNoExit(NodeId),
    MissingTerminal,
    UnreachableNode(NodeId),
    CannotReachTerminal(NodeId),
}
