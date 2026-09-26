//! Room-fragment isolation before composition into the owning Activity graph.

use std::collections::BTreeSet;

use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    domain_deck::DomainDeckSlots,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
};
use starclock_activity::{
    ActivityNodeKind, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngLabel, ActivityTerminalOutcome, NodeId,
};
use starclock_data::divergent_universe_domain_layout::DomainPositionKind;

impl DivergentUniverseRuntimeFactory {
    // Validate factual context joins, never original room-payload admission.
    pub(in crate::divergent_universe) fn room_context_matches(
        &self,
        context: &DomainRoomContext,
    ) -> bool {
        let Some(area) = self
            .bundle
            .catalog()
            .areas()
            .iter()
            .find(|area| area.id == context.area)
        else {
            return false;
        };
        let plane = context
            .plane_ordinal
            .checked_sub(1)
            .and_then(|value| usize::try_from(value).ok());
        let Some(position) = self
            .decision_catalog()
            .domain_layout()
            .iter()
            .find(|layout| layout.layer == context.layer)
            .and_then(|layout| {
                layout.positions.iter().find(|position| {
                    position.key == context.position_key
                        && position.ordinal == context.position_ordinal
                })
            })
        else {
            return false;
        };
        if plane.and_then(|index| area.layers.get(index)) != Some(&context.layer)
            || context.section.get() != context.plane_ordinal
        {
            return false;
        }
        match (&position.kind, context.composition) {
            (
                DomainPositionKind::Fixed {
                    kind,
                    preset_source,
                    level,
                },
                DomainRoomComposition::Fixed(actual),
            ) => {
                *kind == actual
                    && preset_source == &context.preset_source
                    && *level == context.level
            }
            (DomainPositionKind::Unspecified, DomainRoomComposition::Card(actual)) => self
                .decision_catalog()
                .domain_decks()
                .iter()
                .flat_map(|deck| deck.cards.iter())
                .any(|card| {
                    card.kind == actual
                        && card.preset_source == context.preset_source
                        && card.level == context.level
                }),
            _ => false,
        }
    }
}

pub(super) fn validate_fragment(
    context: &DomainRoomContext,
    fragment: &DomainRoomProgram,
    slots: DomainDeckSlots,
) -> Result<(), DomainRouteError> {
    let invalid = || DomainRouteError::InvalidFragment(context.preset_source.clone());
    let nodes = fragment
        .nodes
        .iter()
        .map(|node| node.id())
        .collect::<BTreeSet<_>>();
    let edges = fragment
        .edges
        .iter()
        .map(|edge| edge.id())
        .collect::<BTreeSet<_>>();
    let programs = fragment
        .programs
        .iter()
        .map(|program| program.node())
        .collect::<BTreeSet<_>>();
    if nodes.len() != fragment.nodes.len()
        || edges.len() != fragment.edges.len()
        || programs.len() != fragment.programs.len()
        || !nodes.contains(&context.entry_node())
        || !nodes.contains(&fragment.exit_node)
        || fragment.nodes.iter().any(|node| {
            node.id().get() < context.node_base
                || node.id().get() >= context.node_base + 64
                || node.section() != context.section
                || node.kind() == ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed)
                || (node.id() == fragment.exit_node
                    && matches!(node.kind(), ActivityNodeKind::Terminal(_)))
                || (!matches!(node.kind(), ActivityNodeKind::Terminal(_))
                    && !programs.contains(&node.id()))
        })
        || fragment.edges.iter().any(|edge| {
            edge.id().get() < context.edge_base
                || edge.id().get() >= context.edge_base + 127
                || !nodes.contains(&edge.from())
                || !nodes.contains(&edge.to())
        })
        || fragment
            .random_offers
            .iter()
            .any(|offer| !nodes.contains(&offer.node()))
    {
        return Err(invalid());
    }
    let mut program_ids = BTreeSet::new();
    for program in &fragment.programs {
        let id = program.program().id().get();
        if !nodes.contains(&program.node())
            || id < context.node_base
            || id >= context.node_base + 64
            || !program_ids.insert(id)
            || !operations_allowed(
                program.program().operations(),
                program.node(),
                context,
                fragment,
                slots,
            )
        {
            return Err(invalid());
        }
    }
    let protected = [slots.draw, slots.discard, slots.selected, slots.accepted];
    for offer in &fragment.random_offers {
        if (offer.label() == ActivityRngLabel::Graph && offer.purpose() == 24_101)
            || offer
                .reroll_counter()
                .is_some_and(|(slot, _)| protected.contains(&slot))
            || offer
                .selected_option_marker()
                .is_some_and(|(slot, _, _, _)| protected.contains(&slot))
        {
            return Err(invalid());
        }
        // Offer prefixes have not yet passed the shared graph validator. Bound
        // their operation depth before recursively inspecting protected writes.
        let prefix = ActivityProgramDefinition::new(
            ActivityProgramId::new(offer.node().get()).ok_or_else(invalid)?,
            offer.selection_prefix().to_vec(),
        )
        .map_err(DomainRouteError::Program)?;
        if !operations_allowed(prefix.operations(), offer.node(), context, fragment, slots) {
            return Err(invalid());
        }
    }
    Ok(())
}

fn operations_allowed(
    operations: &[ActivityOperation],
    node: NodeId,
    context: &DomainRoomContext,
    fragment: &DomainRoomProgram,
    slots: DomainDeckSlots,
) -> bool {
    let protected = [slots.draw, slots.discard, slots.selected, slots.accepted];
    operations.iter().all(|operation| match operation {
        ActivityOperation::SetSlot { slot, .. }
        | ActivityOperation::AddToSlot { slot, .. }
        | ActivityOperation::AddCounter { slot, .. }
        | ActivityOperation::SetCounter { slot, .. }
        | ActivityOperation::SetCounterMap { slot, .. }
        | ActivityOperation::SetOrderedIdSet { slot, .. }
        | ActivityOperation::InsertOrderedId { slot, .. }
        | ActivityOperation::RemoveOrderedId { slot, .. } => !protected.contains(slot),
        ActivityOperation::Conditional {
            if_true, if_false, ..
        } => {
            operations_allowed(if_true, node, context, fragment, slots)
                && operations_allowed(if_false, node, context, fragment, slots)
        }
        ActivityOperation::Offer { options, .. } => options
            .iter()
            .all(|option| operations_allowed(option.operations(), node, context, fragment, slots)),
        ActivityOperation::Traverse(edge) => {
            (*edge == context.exit_edge() && node == fragment.exit_node)
                || fragment
                    .edges
                    .iter()
                    .any(|candidate| candidate.id() == *edge && candidate.from() == node)
        }
        ActivityOperation::Relocate(target) => fragment
            .nodes
            .iter()
            .any(|candidate| candidate.id() == *target),
        ActivityOperation::Terminal(outcome) => *outcome != ActivityTerminalOutcome::Completed,
        ActivityOperation::AddInventory { .. }
        | ActivityOperation::RemoveInventory { .. }
        | ActivityOperation::SetInventoryCount { .. }
        | ActivityOperation::AddModifier { .. }
        | ActivityOperation::SetModifierStacks { .. }
        | ActivityOperation::RemoveModifier { .. }
        | ActivityOperation::RestoreParticipant { .. }
        | ActivityOperation::HealParticipantMaximumHpRatio { .. }
        | ActivityOperation::LoseParticipantCurrentHpRatio { .. }
        | ActivityOperation::SetParticipantEnergy { .. }
        | ActivityOperation::Require(_) => true,
    })
}
