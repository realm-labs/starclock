use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityTerminalOutcome, NodeId, SectionId,
    TerminalOutcome,
};

use crate::{CurrencyWarsNodeKind, CurrencyWarsRoute};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsFlow {
    graph: ActivityGraphDefinition,
    route_nodes: Box<[NodeId]>,
    loss_nodes: Box<[Option<NodeId>]>,
    plane_transitions: Box<[CurrencyWarsPlaneTransition]>,
    completed: NodeId,
    failed: NodeId,
    faulted: NodeId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPlaneTransition {
    pub from_plane: u8,
    pub to_plane: u8,
    pub node: NodeId,
}

impl CurrencyWarsFlow {
    pub fn compile(route: &CurrencyWarsRoute) -> Result<Self, CurrencyWarsFlowError> {
        if route.nodes.is_empty() {
            return Err(error("Currency Wars route is empty"));
        }
        validate_route_order(route)?;
        let count = u32::try_from(route.nodes.len()).map_err(debug_error)?;
        let plane_boundaries = route
            .nodes
            .windows(2)
            .enumerate()
            .filter(|(_, pair)| pair[0].plane != pair[1].plane)
            .map(|(index, pair)| (index, pair[0].plane, pair[1].plane))
            .collect::<Vec<_>>();
        let transition_count = u32::try_from(plane_boundaries.len()).map_err(debug_error)?;
        let transition_base = count
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| error("Currency Wars flow node identity overflow"))?;
        let terminal_base = transition_base
            .checked_add(transition_count)
            .ok_or_else(|| error("Currency Wars flow node identity overflow"))?;
        let completed = node_id(terminal_base)?;
        let failed = node_id(terminal_base + 1)?;
        let faulted = node_id(terminal_base + 2)?;
        let mut nodes = Vec::with_capacity(route.nodes.len() * 2 + plane_boundaries.len() + 3);
        let mut edges = Vec::new();
        let mut route_nodes = Vec::with_capacity(route.nodes.len());
        let mut loss_nodes = Vec::with_capacity(route.nodes.len());
        let mut plane_transitions = Vec::with_capacity(plane_boundaries.len());
        let mut edge_raw = 1_u32;

        for (ordinal, (route_index, from_plane, to_plane)) in
            plane_boundaries.iter().copied().enumerate()
        {
            let transition = node_id(
                transition_base
                    .checked_add(u32::try_from(ordinal).map_err(debug_error)?)
                    .ok_or_else(|| error("Currency Wars flow node identity overflow"))?,
            )?;
            let next = action_node(route_index + 1)?;
            nodes.push(node(transition, ActivityNodeKind::Choice, from_plane)?);
            edges.push(edge(
                &mut edge_raw,
                transition,
                next,
                ActivityEdgeCondition::OptionSelected,
            )?);
            plane_transitions.push(CurrencyWarsPlaneTransition {
                from_plane,
                to_plane,
                node: transition,
            });
        }

        for (index, route_node) in route.nodes.iter().enumerate() {
            let action = action_node(index)?;
            let next = next_node(route, index, &plane_boundaries, transition_base, completed)?;
            route_nodes.push(action);
            nodes.push(node(action, node_kind(route_node.kind), route_node.plane)?);
            if route_node.kind.battle() {
                let loss = loss_node(index)?;
                loss_nodes.push(Some(loss));
                nodes.push(node(loss, ActivityNodeKind::Checkpoint, route_node.plane)?);
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
        let terminal_section = route
            .nodes
            .last()
            .and_then(|node| node.plane.checked_add(1))
            .ok_or_else(|| error("Currency Wars terminal section overflow"))?;
        nodes.extend([
            node(
                completed,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                terminal_section,
            )?,
            node(
                failed,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
                terminal_section,
            )?,
            node(
                faulted,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
                terminal_section,
            )?,
        ]);
        let maximum_visits = u32::try_from(nodes.len()).map_err(debug_error)?;
        let graph = ActivityGraphDefinition::new(route_nodes[0], nodes, edges, maximum_visits)
            .map_err(debug_error)?;
        Ok(Self {
            graph,
            route_nodes: route_nodes.into_boxed_slice(),
            loss_nodes: loss_nodes.into_boxed_slice(),
            plane_transitions: plane_transitions.into_boxed_slice(),
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
    pub fn plane_transitions(&self) -> &[CurrencyWarsPlaneTransition] {
        &self.plane_transitions
    }

    #[must_use]
    pub fn plane_transition(&self, node: NodeId) -> Option<CurrencyWarsPlaneTransition> {
        self.plane_transitions
            .iter()
            .copied()
            .find(|transition| transition.node == node)
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
    plane: u8,
) -> Result<ActivityNodeDefinition, CurrencyWarsFlowError> {
    ActivityNodeDefinition::new(id, section(plane)?, kind, 1).map_err(debug_error)
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

fn section(plane: u8) -> Result<SectionId, CurrencyWarsFlowError> {
    SectionId::new(u32::from(plane)).ok_or_else(|| error("Currency Wars Plane is zero"))
}

fn next_node(
    route: &CurrencyWarsRoute,
    index: usize,
    plane_boundaries: &[(usize, u8, u8)],
    transition_base: u32,
    completed: NodeId,
) -> Result<NodeId, CurrencyWarsFlowError> {
    let Some(next) = route.nodes.get(index + 1) else {
        return Ok(completed);
    };
    if route.nodes[index].plane == next.plane {
        return action_node(index + 1);
    }
    let ordinal = plane_boundaries
        .iter()
        .position(|(boundary, _, _)| *boundary == index)
        .ok_or_else(|| error("Currency Wars Plane boundary is missing"))?;
    transition_base
        .checked_add(u32::try_from(ordinal).map_err(debug_error)?)
        .and_then(NodeId::new)
        .ok_or_else(|| error("Currency Wars Plane transition identity overflow"))
}

fn validate_route_order(route: &CurrencyWarsRoute) -> Result<(), CurrencyWarsFlowError> {
    if route
        .nodes
        .first()
        .is_none_or(|node| node.plane != 1 || node.ordinal != 1)
        || route.nodes.last().is_none_or(|node| node.plane > 3)
        || route.nodes.windows(2).any(|pair| {
            pair[0].plane > pair[1].plane
                || (pair[0].plane == pair[1].plane && pair[0].ordinal >= pair[1].ordinal)
                || (pair[0].plane != pair[1].plane
                    && (pair[0].plane.checked_add(1) != Some(pair[1].plane)
                        || pair[1].ordinal != 1))
        })
    {
        return Err(error("Currency Wars route Plane order is invalid"));
    }
    Ok(())
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
    use starclock_activity::{ActivityNodeKind, SectionId};

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
            3
        );
        assert_eq!(flow.plane_transitions().len(), 2);
        assert_eq!(flow.graph().entry(), flow.activity_node(0).unwrap());
        assert_eq!(
            flow.graph()
                .node(flow.activity_node(0).unwrap())
                .unwrap()
                .section(),
            SectionId::new(1).unwrap(),
        );
        assert_eq!(
            flow.graph()
                .node(flow.activity_node(2).unwrap())
                .unwrap()
                .section(),
            SectionId::new(2).unwrap(),
        );
    }
}
