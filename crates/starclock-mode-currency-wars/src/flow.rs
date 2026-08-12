use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityTerminalOutcome, NodeId, SectionId,
    TerminalOutcome,
};

use crate::{CurrencyWarsNodeKind, CurrencyWarsRoute};

const SECTION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsFlow {
    graph: ActivityGraphDefinition,
    route_nodes: Box<[NodeId]>,
    loss_nodes: Box<[Option<NodeId>]>,
    completed: NodeId,
    failed: NodeId,
    faulted: NodeId,
}

impl CurrencyWarsFlow {
    pub fn compile(route: &CurrencyWarsRoute) -> Result<Self, CurrencyWarsFlowError> {
        if route.nodes.is_empty() {
            return Err(error("Currency Wars route is empty"));
        }
        let count = u32::try_from(route.nodes.len()).map_err(debug_error)?;
        let terminal_base = count
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| error("Currency Wars flow node identity overflow"))?;
        let completed = node_id(terminal_base)?;
        let failed = node_id(terminal_base + 1)?;
        let faulted = node_id(terminal_base + 2)?;
        let mut nodes = Vec::with_capacity(route.nodes.len() * 2 + 3);
        let mut edges = Vec::new();
        let mut route_nodes = Vec::with_capacity(route.nodes.len());
        let mut loss_nodes = Vec::with_capacity(route.nodes.len());
        let mut edge_raw = 1_u32;

        for (index, route_node) in route.nodes.iter().enumerate() {
            let action = action_node(index)?;
            let next = route
                .nodes
                .get(index + 1)
                .map(|_| action_node(index + 1))
                .transpose()?
                .unwrap_or(completed);
            route_nodes.push(action);
            nodes.push(node(action, node_kind(route_node.kind))?);
            if route_node.kind.battle() {
                let loss = loss_node(index)?;
                loss_nodes.push(Some(loss));
                nodes.push(node(loss, ActivityNodeKind::Checkpoint)?);
                edges.push(edge(
                    &mut edge_raw,
                    action,
                    next,
                    ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete),
                )?);
                edges.push(edge(
                    &mut edge_raw,
                    action,
                    loss,
                    ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Failed),
                )?);
                edges.push(edge(
                    &mut edge_raw,
                    action,
                    faulted,
                    ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Faulted),
                )?);
                edges.push(edge(
                    &mut edge_raw,
                    loss,
                    next,
                    ActivityEdgeCondition::OptionSelected,
                )?);
                edges.push(edge(
                    &mut edge_raw,
                    loss,
                    failed,
                    ActivityEdgeCondition::OptionSelected,
                )?);
            } else {
                loss_nodes.push(None);
                edges.push(edge(
                    &mut edge_raw,
                    action,
                    next,
                    ActivityEdgeCondition::OptionSelected,
                )?);
            }
        }
        nodes.extend([
            node(
                completed,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            )?,
            node(
                failed,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
            )?,
            node(
                faulted,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
            )?,
        ]);
        let maximum_visits = u32::try_from(nodes.len()).map_err(debug_error)?;
        let graph = ActivityGraphDefinition::new(route_nodes[0], nodes, edges, maximum_visits)
            .map_err(debug_error)?;
        Ok(Self {
            graph,
            route_nodes: route_nodes.into_boxed_slice(),
            loss_nodes: loss_nodes.into_boxed_slice(),
            completed,
            failed,
            faulted,
        })
    }

    #[must_use]
    pub const fn graph(&self) -> &ActivityGraphDefinition {
        &self.graph
    }

    #[must_use]
    pub fn into_graph(self) -> ActivityGraphDefinition {
        self.graph
    }

    #[must_use]
    pub fn activity_node(&self, index: usize) -> Option<NodeId> {
        self.route_nodes.get(index).copied()
    }

    #[must_use]
    pub fn loss_node(&self, index: usize) -> Option<NodeId> {
        self.loss_nodes.get(index).copied().flatten()
    }

    #[must_use]
    pub fn route_index(&self, node: NodeId) -> Option<usize> {
        self.route_nodes
            .iter()
            .position(|candidate| *candidate == node)
    }

    #[must_use]
    pub const fn completed(&self) -> NodeId {
        self.completed
    }

    #[must_use]
    pub const fn failed(&self) -> NodeId {
        self.failed
    }

    #[must_use]
    pub const fn faulted(&self) -> NodeId {
        self.faulted
    }
}

fn node_kind(kind: CurrencyWarsNodeKind) -> ActivityNodeKind {
    match kind {
        CurrencyWarsNodeKind::Supply => ActivityNodeKind::Shop,
        CurrencyWarsNodeKind::Monster
        | CurrencyWarsNodeKind::CampMonster
        | CurrencyWarsNodeKind::EliteBranch
        | CurrencyWarsNodeKind::Boss => ActivityNodeKind::Battle,
    }
}

fn action_node(index: usize) -> Result<NodeId, CurrencyWarsFlowError> {
    indexed_node(index, 1)
}

fn loss_node(index: usize) -> Result<NodeId, CurrencyWarsFlowError> {
    indexed_node(index, 2)
}

fn indexed_node(index: usize, offset: u32) -> Result<NodeId, CurrencyWarsFlowError> {
    u32::try_from(index)
        .ok()
        .and_then(|value| value.checked_mul(2))
        .and_then(|value| value.checked_add(offset))
        .and_then(NodeId::new)
        .ok_or_else(|| error("Currency Wars flow node identity overflow"))
}

fn node(
    id: NodeId,
    kind: ActivityNodeKind,
) -> Result<ActivityNodeDefinition, CurrencyWarsFlowError> {
    ActivityNodeDefinition::new(id, section(), kind, 1).map_err(debug_error)
}

fn edge(
    next_id: &mut u32,
    from: NodeId,
    to: NodeId,
    condition: ActivityEdgeCondition,
) -> Result<ActivityEdgeDefinition, CurrencyWarsFlowError> {
    let id = ActivityEdgeId::new(*next_id).ok_or_else(|| error("Currency Wars edge ID is zero"))?;
    *next_id = next_id
        .checked_add(1)
        .ok_or_else(|| error("Currency Wars edge identity overflow"))?;
    ActivityEdgeDefinition::new(
        id,
        from,
        to,
        condition,
        i32::try_from(id.get()).map_err(debug_error)?,
        1,
    )
    .map_err(debug_error)
}

fn node_id(raw: u32) -> Result<NodeId, CurrencyWarsFlowError> {
    NodeId::new(raw).ok_or_else(|| error("Currency Wars node ID is zero"))
}

fn section() -> SectionId {
    SectionId::new(SECTION).expect("Currency Wars section ID is non-zero")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsFlowError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsFlowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsFlowError {}

fn error(message: &str) -> CurrencyWarsFlowError {
    CurrencyWarsFlowError {
        message: message.into(),
    }
}

fn debug_error(value: impl std::fmt::Debug) -> CurrencyWarsFlowError {
    CurrencyWarsFlowError {
        message: format!("{value:?}").into_boxed_str(),
    }
}

#[cfg(test)]
mod tests {
    use starclock_activity::ActivityNodeKind;

    use super::CurrencyWarsFlow;
    use crate::catalog::tests_support;

    #[test]
    fn loss_checkpoint_exists_only_after_battles() {
        let catalog = tests_support::catalog();
        let flow = CurrencyWarsFlow::compile(&catalog.routes()[0]).unwrap();

        assert!(flow.loss_node(0).is_some());
        assert!(flow.loss_node(1).is_none());
        assert_eq!(
            flow.graph()
                .nodes()
                .iter()
                .filter(|node| node.kind() == ActivityNodeKind::Battle)
                .count(),
            1
        );
    }
}
