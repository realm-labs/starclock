//! Per-instance destination binding and atomic mode-owned domain entry effects.

use super::{DomainCardId, DomainDeck, DomainDeckError, invalid};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId, ActivityGeneratedBoundaryResolution,
    ActivityOperation, ActivityOptionId, ActivityPlayerView, ActivityProgramDefinition,
    ActivityRngStreams, ActivitySlotId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError,
};

impl DomainDeck {
    /// Binds every card instance exactly once to an owning room's entry edge.
    /// Input order is irrelevant; the emitted offer uses stable card-ID order.
    /// Copies of a room preset remain separate instances and may share an edge.
    /// Missing, duplicate and foreign card bindings reject. The owning graph
    /// validates edge existence and source; this compiler does not infer rooms.
    pub fn offer_program_by_card(
        &self,
        destinations: &[(DomainCardId, ActivityEdgeId)],
    ) -> Result<Vec<ActivityOperation>, DomainDeckError> {
        let mut destinations = destinations.to_vec();
        destinations.sort_unstable_by_key(|(card, _)| *card);
        if destinations.len() != self.cards.len()
            || destinations
                .iter()
                .zip(self.cards.iter())
                .any(|((actual, _), expected)| actual != expected)
        {
            return Err(DomainDeckError::InvalidDestinations);
        }
        Ok(vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Route,
            options: destinations
                .into_iter()
                .map(|(card, edge)| self.card_option(card, edge))
                .collect(),
        }])
    }

    /// Settles the whole hand, applies generated domain-entry effects, then
    /// traverses the selected card's authored edge in one shared transaction.
    ///
    /// This is a trusted mode-executor boundary. Adapters supply offered IDs,
    /// never arbitrary operations. Generation sees the pre-command player view
    /// and project-owned RNG, and must have no external effects. The selected
    /// instance is authenticated before generation. Entry effects may update
    /// currencies, Curios or other room state, but cannot write this deck's four
    /// slots or offer/traverse/terminate. Shared Activity validates all operations
    /// and budgets. Rejection, including downstream room initialization failure,
    /// restores piles, offers, effects, events and RNG; no value escapes failure.
    pub fn choose_with_generated_entry<T>(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
        generate: impl FnOnce(
            &ActivityPlayerView,
            &mut ActivityRngStreams,
            DomainCardId,
        ) -> Result<(Vec<ActivityOperation>, T), GraphActivityCommandError>,
    ) -> Result<ActivityGeneratedBoundaryResolution<T>, GraphActivityCommandError> {
        self.validate_slots(activity)?;
        let policy = self
            .random_offer(activity.current_node())
            .map_err(|_| invalid())?;
        if !activity.definition().random_offers().contains(&policy) {
            return Err(invalid());
        }
        let program = activity
            .definition()
            .programs()
            .iter()
            .find(|program| program.node() == activity.current_node())
            .ok_or_else(invalid)?
            .program()
            .id();
        let protected = [
            self.slots.draw,
            self.slots.discard,
            self.slots.selected,
            self.slots.accepted,
        ];
        activity.choose_option_with_generated_prefix(expected, decision, selected, |view, rng| {
            let mut operations = self.selection_operations(view, selected)?;
            let card = DomainCardId::new(selected.get()).ok_or_else(invalid)?;
            let (entry, value) = generate(view, rng, card)?;
            // Validate nesting depth before walking branches. The shared engine
            // subsequently validates the complete combined program and bindings.
            let entry = ActivityProgramDefinition::new(program, entry).map_err(|_| invalid())?;
            if !entry_operations_allowed(entry.operations(), &protected) {
                return Err(invalid());
            }
            operations.extend_from_slice(entry.operations());
            Ok((operations, value))
        })
    }
}

fn entry_operations_allowed(
    operations: &[ActivityOperation],
    protected: &[ActivitySlotId],
) -> bool {
    operations.iter().all(|operation| match operation {
        ActivityOperation::SetSlot { slot, .. }
        | ActivityOperation::AddToSlot { slot, .. }
        | ActivityOperation::AddCounter { slot, .. }
        | ActivityOperation::SetCounter { slot, .. }
        | ActivityOperation::RemoveCounter { slot, .. }
        | ActivityOperation::SetCounterMap { slot, .. }
        | ActivityOperation::SetOrderedIdSet { slot, .. }
        | ActivityOperation::InsertOrderedId { slot, .. }
        | ActivityOperation::RemoveOrderedId { slot, .. } => !protected.contains(slot),
        ActivityOperation::Conditional {
            if_true, if_false, ..
        } => {
            entry_operations_allowed(if_true, protected)
                && entry_operations_allowed(if_false, protected)
        }
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
        ActivityOperation::Traverse(_)
        | ActivityOperation::Relocate(_)
        | ActivityOperation::Offer { .. }
        | ActivityOperation::Terminal(_) => false,
    })
}
