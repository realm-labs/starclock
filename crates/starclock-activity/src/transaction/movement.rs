use super::*;

impl ActivityTransactionState {
    pub(crate) fn traverse_edge(
        &mut self,
        edge: ActivityEdgeId,
        graph: &ActivityGraphDefinition,
    ) -> Result<(NodeId, Vec<(ActivitySlotId, SlotResetPoint)>), ActivityFault> {
        let edge_def = graph
            .edges()
            .iter()
            .find(|item| item.id() == edge)
            .ok_or(ActivityFault::InvalidGraphEdge(edge))?;
        if edge_def.from() != self.current_node {
            return Err(ActivityFault::InvalidGraphEdge(edge));
        }
        let next_edge_count = self
            .edge_traversals
            .get(&edge)
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ActivityFault::VisitLimitExceeded)?;
        if next_edge_count > edge_def.maximum_traversals() {
            return Err(ActivityFault::VisitLimitExceeded);
        }
        let resets = self.transition_to_node(edge_def.to(), graph)?;
        self.edge_traversals.insert(edge, next_edge_count);
        Ok((edge_def.to(), resets))
    }

    pub(super) fn relocate_node(
        &mut self,
        target: NodeId,
        graph: &ActivityGraphDefinition,
    ) -> Result<Vec<(ActivitySlotId, SlotResetPoint)>, ActivityFault> {
        self.transition_to_node(target, graph)
    }

    fn transition_to_node(
        &mut self,
        target: NodeId,
        graph: &ActivityGraphDefinition,
    ) -> Result<Vec<(ActivitySlotId, SlotResetPoint)>, ActivityFault> {
        let next_node = graph
            .node(target)
            .ok_or(ActivityFault::InvalidGraphNode(target))?;
        let next_node_count = self
            .node_visits
            .get(&target)
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ActivityFault::VisitLimitExceeded)?;
        let next_total = self
            .total_visits
            .checked_add(1)
            .ok_or(ActivityFault::VisitLimitExceeded)?;
        if next_node_count > next_node.maximum_visits() || next_total > graph.maximum_total_visits()
        {
            return Err(ActivityFault::VisitLimitExceeded);
        }
        self.node_visits.insert(target, next_node_count);
        self.total_visits = next_total;
        self.logical_scopes
            .transition(self.definition.logical_scopes(), target)
            .map_err(|_| ActivityFault::LogicalScopeLimitExceeded)?;
        let current_section = graph
            .node(self.current_node)
            .ok_or(ActivityFault::InvalidGraphNode(self.current_node))?
            .section();
        let section_changed = current_section != next_node.section();
        let mut resets = Vec::new();
        for point in [
            section_changed.then_some(SlotResetPoint::SectionStart),
            Some(SlotResetPoint::NodeStart),
        ]
        .into_iter()
        .flatten()
        {
            let values = self
                .definition
                .slots()
                .iter()
                .filter(|definition| definition.resets().contains(&point))
                .map(|definition| (definition.id(), definition.initial().clone()))
                .collect::<Vec<_>>();
            for (slot, initial) in values {
                self.slots.insert(slot, initial);
                resets.push((slot, point));
            }
        }
        self.current_node = target;
        Ok(resets)
    }
}
