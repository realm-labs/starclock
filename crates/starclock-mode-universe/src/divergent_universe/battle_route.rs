//! Mode-owned physical battle addresses and graph-validated routing.

use super::{DivergentUniverseEntryFlowError, DivergentUniverseFlowInstance};
use starclock_activity::{
    ActivityEdgeCondition, ActivityNodeKind, ActivityPlayerView, NodeId, SectionId, TerminalOutcome,
};
use starclock_data::divergent_universe_decisions::BattleRewardDomain;

pub(super) struct LayerBattleAddress {
    pub(super) battle: NodeId,
    pub(super) reward: NodeId,
    pub(super) section: SectionId,
}

/// Automatic initialization precedes the later layer's sole random offer.
pub(super) fn layer_entry_node(
    layer_count: u32,
    ordinal: u32,
) -> Result<NodeId, DivergentUniverseEntryFlowError> {
    let invalid = || DivergentUniverseEntryFlowError::InvalidActivityDefinition;
    if ordinal == 0 || ordinal > layer_count {
        return Err(invalid());
    }
    let raw = if ordinal == 1 {
        Some(1)
    } else {
        layer_count
            .checked_mul(3)
            .and_then(|value| value.checked_add(ordinal))
            .and_then(|value| value.checked_add(4))
    }
    .ok_or_else(invalid)?;
    NodeId::new(raw).ok_or_else(invalid)
}

impl LayerBattleAddress {
    pub(super) fn new(
        layer_count: u32,
        ordinal: u32,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let invalid = || DivergentUniverseEntryFlowError::InvalidActivityDefinition;
        if ordinal == 0 || ordinal > layer_count {
            return Err(invalid());
        }
        let raw = if ordinal == 1 {
            layer_count.checked_add(3)
        } else {
            ordinal
                .checked_sub(2)
                .and_then(|value| value.checked_mul(2))
                .and_then(|value| value.checked_add(8))
                .and_then(|offset| layer_count.checked_add(offset))
        }
        .ok_or_else(invalid)?;
        let reward = raw
            .checked_add(if ordinal == 1 { 3 } else { 1 })
            .ok_or_else(invalid)?;
        Ok(Self {
            battle: NodeId::new(raw).ok_or_else(invalid)?,
            reward: NodeId::new(reward).ok_or_else(invalid)?,
            section: SectionId::new(ordinal).ok_or_else(invalid)?,
        })
    }
}

impl DivergentUniverseFlowInstance {
    /// Only the graph-bound victory node can resolve a settled domain. Domain
    /// selection is carried through the shared logical-node scope, not stages.
    pub(super) fn battle_reward_domain(
        &self,
        view: &ActivityPlayerView,
    ) -> Option<BattleRewardDomain> {
        if !self.has_runtime_battle_route() || !self.is_battle_reward(view.current_node()) {
            return None;
        }
        self.domain_from_view(view).ok().flatten()
    }

    pub(super) fn encounter_destination(&self, current: NodeId) -> Option<(NodeId, SectionId)> {
        let graph = self.definition().graph();
        let mut targets = graph
            .outgoing(current)
            .filter_map(|edge| graph.node(edge.to()))
            .filter(|node| node.kind() == ActivityNodeKind::Battle);
        let target = targets.next()?;
        if targets.next().is_some() {
            return None;
        }
        Some((target.id(), target.section()))
    }

    pub(super) fn is_battle_reward(&self, current: NodeId) -> bool {
        let graph = self.definition().graph();
        graph
            .node(current)
            .is_some_and(|node| node.kind() == ActivityNodeKind::Reward)
            && graph.edges().iter().any(|edge| {
                edge.to() == current
                    && edge.condition()
                        == ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete)
                    && graph
                        .node(edge.from())
                        .is_some_and(|node| node.kind() == ActivityNodeKind::Battle)
            })
    }
}

pub(super) fn domain_choice_node(
    layer_count: u32,
    ordinal: u32,
) -> Result<NodeId, DivergentUniverseEntryFlowError> {
    if ordinal < 2 || ordinal > layer_count {
        return Err(DivergentUniverseEntryFlowError::InvalidActivityDefinition);
    }
    layer_count
        .checked_mul(4)
        .and_then(|value| value.checked_add(ordinal))
        .and_then(|value| value.checked_add(5))
        .and_then(NodeId::new)
        .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
