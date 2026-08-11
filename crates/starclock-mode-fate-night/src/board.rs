use std::collections::BTreeSet;
use std::num::NonZeroU32;

use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityTerminalOutcome, NodeId, SectionId,
    TerminalOutcome,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FateBoardNodeKind {
    Choice,
    Battle,
    Reward,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FateBoardNode {
    pub id: u32,
    pub kind: FateBoardNodeKind,
    pub maximum_visits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FateBoardEdge {
    pub id: u32,
    pub from: u32,
    pub to: u32,
    pub priority: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateBoard {
    id: NonZeroU32,
    entry: u32,
    nodes: Box<[FateBoardNode]>,
    edges: Box<[FateBoardEdge]>,
}

impl FateBoard {
    pub fn new(
        id: u32,
        entry: u32,
        mut nodes: Vec<FateBoardNode>,
        mut edges: Vec<FateBoardEdge>,
    ) -> Result<Self, FateBoardError> {
        let id = NonZeroU32::new(id).ok_or(FateBoardError::ZeroIdentity)?;
        nodes.sort_by_key(|node| node.id);
        edges.sort_by_key(|edge| edge.id);
        if nodes.is_empty()
            || nodes
                .iter()
                .any(|node| node.id == 0 || node.maximum_visits == 0)
            || nodes.windows(2).any(|pair| pair[0].id == pair[1].id)
            || nodes.binary_search_by_key(&entry, |node| node.id).is_err()
        {
            return Err(FateBoardError::InvalidNodes);
        }
        if edges.iter().any(|edge| {
            edge.id == 0
                || nodes
                    .binary_search_by_key(&edge.from, |node| node.id)
                    .is_err()
                || nodes
                    .binary_search_by_key(&edge.to, |node| node.id)
                    .is_err()
        }) || edges.windows(2).any(|pair| pair[0].id == pair[1].id)
        {
            return Err(FateBoardError::InvalidEdges);
        }
        let mut reachable = BTreeSet::from([entry]);
        loop {
            let before = reachable.len();
            for edge in &edges {
                if reachable.contains(&edge.from) {
                    reachable.insert(edge.to);
                }
            }
            if before == reachable.len() {
                break;
            }
        }
        if reachable.len() != nodes.len() {
            return Err(FateBoardError::UnreachableNode);
        }
        Ok(Self {
            id,
            entry,
            nodes: nodes.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id.get()
    }

    #[must_use]
    pub fn nodes(&self) -> &[FateBoardNode] {
        &self.nodes
    }

    pub fn compile(&self, section: SectionId) -> Result<ActivityGraphDefinition, FateBoardError> {
        let failed_id = self
            .nodes
            .iter()
            .map(|node| node.id)
            .max()
            .and_then(|value| value.checked_add(1))
            .ok_or(FateBoardError::IdentityOverflow)?;
        let faulted_id = failed_id
            .checked_add(1)
            .ok_or(FateBoardError::IdentityOverflow)?;
        let mut nodes = self
            .nodes
            .iter()
            .map(|node| {
                ActivityNodeDefinition::new(
                    NodeId::new(node.id).ok_or(FateBoardError::ZeroIdentity)?,
                    section,
                    match node.kind {
                        FateBoardNodeKind::Choice => ActivityNodeKind::Choice,
                        FateBoardNodeKind::Battle => ActivityNodeKind::Battle,
                        FateBoardNodeKind::Reward => ActivityNodeKind::Reward,
                        FateBoardNodeKind::Completed => {
                            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed)
                        }
                    },
                    node.maximum_visits,
                )
                .map_err(|_| FateBoardError::InvalidNodes)
            })
            .collect::<Result<Vec<_>, _>>()?;
        nodes.extend([
            ActivityNodeDefinition::new(
                NodeId::new(failed_id).ok_or(FateBoardError::ZeroIdentity)?,
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
                1,
            )
            .map_err(|_| FateBoardError::InvalidNodes)?,
            ActivityNodeDefinition::new(
                NodeId::new(faulted_id).ok_or(FateBoardError::ZeroIdentity)?,
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
                1,
            )
            .map_err(|_| FateBoardError::InvalidNodes)?,
        ]);
        let mut edges = self
            .edges
            .iter()
            .map(|edge| {
                let source = self
                    .nodes
                    .binary_search_by_key(&edge.from, |node| node.id)
                    .ok()
                    .map(|index| self.nodes[index].kind)
                    .ok_or(FateBoardError::InvalidEdges)?;
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(edge.id).ok_or(FateBoardError::ZeroIdentity)?,
                    NodeId::new(edge.from).ok_or(FateBoardError::ZeroIdentity)?,
                    NodeId::new(edge.to).ok_or(FateBoardError::ZeroIdentity)?,
                    match source {
                        FateBoardNodeKind::Battle => {
                            ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete)
                        }
                        FateBoardNodeKind::Choice | FateBoardNodeKind::Reward => {
                            ActivityEdgeCondition::OptionSelected
                        }
                        FateBoardNodeKind::Completed => {
                            return Err(FateBoardError::InvalidEdges);
                        }
                    },
                    edge.priority,
                    1,
                )
                .map_err(|_| FateBoardError::InvalidEdges)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut next_edge = self
            .edges
            .iter()
            .map(|edge| edge.id)
            .max()
            .unwrap_or_default()
            .checked_add(1)
            .ok_or(FateBoardError::IdentityOverflow)?;
        for battle in self
            .nodes
            .iter()
            .filter(|node| node.kind == FateBoardNodeKind::Battle)
        {
            for (target, outcome) in [
                (failed_id, TerminalOutcome::Failed),
                (faulted_id, TerminalOutcome::Faulted),
            ] {
                edges.push(
                    ActivityEdgeDefinition::new(
                        ActivityEdgeId::new(next_edge).ok_or(FateBoardError::ZeroIdentity)?,
                        NodeId::new(battle.id).ok_or(FateBoardError::ZeroIdentity)?,
                        NodeId::new(target).ok_or(FateBoardError::ZeroIdentity)?,
                        ActivityEdgeCondition::BattleOutcome(outcome),
                        i32::try_from(next_edge).map_err(|_| FateBoardError::IdentityOverflow)?,
                        1,
                    )
                    .map_err(|_| FateBoardError::InvalidEdges)?,
                );
                next_edge = next_edge
                    .checked_add(1)
                    .ok_or(FateBoardError::IdentityOverflow)?;
            }
        }
        let maximum_visits = self
            .nodes
            .iter()
            .try_fold(0_u32, |total, node| total.checked_add(node.maximum_visits))
            .and_then(|total| total.checked_add(2))
            .ok_or(FateBoardError::VisitLimitOverflow)?;
        ActivityGraphDefinition::new(
            NodeId::new(self.entry).ok_or(FateBoardError::ZeroIdentity)?,
            nodes,
            edges,
            maximum_visits,
        )
        .map_err(|_| FateBoardError::InvalidGraph)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FateBoardError {
    ZeroIdentity,
    InvalidNodes,
    InvalidEdges,
    UnreachableNode,
    VisitLimitOverflow,
    IdentityOverflow,
    InvalidGraph,
}

#[cfg(test)]
mod tests {
    use starclock_activity::SectionId;

    use super::{FateBoard, FateBoardEdge, FateBoardNode, FateBoardNodeKind};

    #[test]
    fn case_board_compiles_to_shared_activity_graph() {
        let board = FateBoard::new(
            1,
            1,
            vec![
                FateBoardNode {
                    id: 1,
                    kind: FateBoardNodeKind::Choice,
                    maximum_visits: 1,
                },
                FateBoardNode {
                    id: 2,
                    kind: FateBoardNodeKind::Battle,
                    maximum_visits: 1,
                },
                FateBoardNode {
                    id: 3,
                    kind: FateBoardNodeKind::Completed,
                    maximum_visits: 1,
                },
            ],
            vec![
                FateBoardEdge {
                    id: 1,
                    from: 1,
                    to: 2,
                    priority: 1,
                },
                FateBoardEdge {
                    id: 2,
                    from: 2,
                    to: 3,
                    priority: 1,
                },
            ],
        )
        .unwrap();
        let graph = board.compile(SectionId::new(1).unwrap()).unwrap();
        assert_eq!(graph.nodes().len(), 5);
        assert_eq!(graph.edges().len(), 4);
    }
}
