use super::*;

pub(super) fn compile_route_program(
    hub: &DomainHubDefinition,
    hub_clear_slot: ActivitySlotId,
    topology_edges: &[(TopologyNodeId, TopologyNodeId, ActivityEdgeId)],
    exit_edges: &[(TopologyNodeId, ActivityEdgeId)],
    curio_bindings: crate::curio_activity::CurioActivityBindings,
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
        let finish = if route.target.is_none() {
            vec![
                ActivityOperation::Traverse(edge),
                ActivityOperation::Terminal(ActivityTerminalOutcome::Completed),
            ]
        } else {
            vec![ActivityOperation::Traverse(edge)]
        };
        let operations = if route.target.is_some() {
            let with_cogwheel =
                crate::curio_activity::cogwheel_domain_entry_settlement(curio_bindings, &finish);
            vec![ActivityOperation::Conditional {
                condition: crate::curio_activity::cogwheel_condition(curio_bindings),
                if_true: with_cogwheel.into_boxed_slice(),
                if_false: finish.into_boxed_slice(),
            }]
        } else {
            finish
        };
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
