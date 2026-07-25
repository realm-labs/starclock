//! Deterministic queued-action ordering independent from presentation timing.

use crate::{
    AbilityId, ActionOrigin, CommandId, EventId, RuleId, RuleInstanceId, SourceDefinitionId,
    TriggerId, UnitId,
    battle::spec::{FormationIndex, TeamSide},
    catalog::action::ReactionBoundary,
    id::SpawnSequence,
    target::model::TargetCommitment,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ReactionOrder {
    pub(crate) boundary: ReactionBoundary,
    pub(crate) tier: ReactionTier,
    pub(crate) priority: i16,
    pub(crate) side: TeamSide,
    pub(crate) formation: FormationIndex,
    pub(crate) spawn: SpawnSequence,
    pub(crate) source: SourceDefinitionId,
    pub(crate) rule: Option<RuleId>,
    pub(crate) instance: Option<RuleInstanceId>,
    pub(crate) trigger: Option<TriggerId>,
    pub(crate) actor: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) insertion: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(crate) enum ReactionTier {
    ForcedFollowUp = 0,
    Ultimate = 1,
    ExtraAction = 2,
    ExtraTurnAction = 3,
}

impl ReactionTier {
    pub(crate) const fn for_origin(origin: ActionOrigin) -> Self {
        match origin {
            ActionOrigin::FollowUp | ActionOrigin::Counter => Self::ForcedFollowUp,
            ActionOrigin::UltimateInterrupt => Self::Ultimate,
            ActionOrigin::ExtraTurn => Self::ExtraTurnAction,
            ActionOrigin::NormalTurn
            | ActionOrigin::Forced
            | ActionOrigin::ExtraAction
            | ActionOrigin::DelayedAction
            | ActionOrigin::SummonAction
            | ActionOrigin::MemospriteAction
            | ActionOrigin::Countdown => Self::ExtraAction,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueuedAction {
    pub(crate) order: ReactionOrder,
    pub(crate) root: CommandId,
    pub(crate) parent: EventId,
    pub(crate) actor: UnitId,
    pub(crate) owner: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) origin: ActionOrigin,
    pub(crate) targets: TargetCommitment,
    pub(crate) payment: Option<crate::catalog::action::SkillPointPaymentPolicy>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ReactionQueue {
    entries: Vec<QueuedAction>,
}

impl ReactionQueue {
    pub(crate) fn push(&mut self, entry: QueuedAction) {
        let index = self
            .entries
            .binary_search_by_key(&entry.order, |candidate| candidate.order)
            .unwrap_or_else(|index| index);
        self.entries.insert(index, entry);
    }

    pub(crate) fn pop_ready(&mut self, boundary: ReactionBoundary) -> Option<QueuedAction> {
        self.entries
            .first()
            .is_some_and(|entry| entry.order.boundary <= boundary)
            .then(|| self.entries.remove(0))
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn entries(&self) -> &[QueuedAction] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::action::{
        TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
    };

    fn runtime<I: TryFrom<u64>>(raw: u64) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    fn definition<I: TryFrom<u32>>(raw: u32) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    fn entry(boundary: ReactionBoundary, priority: i16, insertion: u64) -> QueuedAction {
        let actor = runtime(1);
        let ability = definition(1);
        QueuedAction {
            order: ReactionOrder {
                boundary,
                tier: ReactionTier::ForcedFollowUp,
                priority,
                side: TeamSide::Player,
                formation: FormationIndex::new(0).unwrap(),
                spawn: runtime(1),
                source: definition(1),
                rule: None,
                instance: None,
                trigger: None,
                actor,
                ability,
                insertion,
            },
            root: runtime(1),
            parent: runtime(1),
            actor,
            owner: actor,
            ability,
            origin: ActionOrigin::FollowUp,
            targets: TargetCommitment {
                selector: UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single)
                    .unwrap(),
                invalidation: TargetInvalidationPolicy::CancelRemainingForTarget,
                primary: Some(runtime(2)),
                targets: vec![runtime(2)].into_boxed_slice(),
            },
            payment: None,
        }
    }

    #[test]
    fn semantic_tiers_cannot_be_inverted_by_authored_priority() {
        let mut follow_up = entry(ReactionBoundary::AfterAction, 100, 2);
        follow_up.order.tier = ReactionTier::for_origin(ActionOrigin::FollowUp);
        let mut extra_action = entry(ReactionBoundary::AfterAction, -100, 1);
        extra_action.order.tier = ReactionTier::for_origin(ActionOrigin::ExtraAction);
        extra_action.origin = ActionOrigin::ExtraAction;
        let mut queue = ReactionQueue::default();
        queue.push(extra_action);
        queue.push(follow_up);
        assert_eq!(
            queue
                .pop_ready(ReactionBoundary::AfterAction)
                .unwrap()
                .origin,
            ActionOrigin::FollowUp
        );
    }

    #[test]
    fn boundary_priority_and_insertion_form_a_complete_stable_order() {
        let mut queue = ReactionQueue::default();
        queue.push(entry(ReactionBoundary::AfterAction, -10, 1));
        queue.push(entry(ReactionBoundary::AfterHit, 10, 3));
        queue.push(entry(ReactionBoundary::AfterHit, -10, 2));
        queue.push(entry(ReactionBoundary::AfterHit, -10, 1));
        assert_eq!(
            queue
                .entries()
                .iter()
                .map(|entry| (
                    entry.order.boundary,
                    entry.order.tier,
                    entry.order.priority,
                    entry.order.insertion
                ))
                .collect::<Vec<_>>(),
            [
                (
                    ReactionBoundary::AfterHit,
                    ReactionTier::ForcedFollowUp,
                    -10,
                    1
                ),
                (
                    ReactionBoundary::AfterHit,
                    ReactionTier::ForcedFollowUp,
                    -10,
                    2
                ),
                (
                    ReactionBoundary::AfterHit,
                    ReactionTier::ForcedFollowUp,
                    10,
                    3
                ),
                (
                    ReactionBoundary::AfterAction,
                    ReactionTier::ForcedFollowUp,
                    -10,
                    1
                ),
            ]
        );
        assert_eq!(
            queue
                .pop_ready(ReactionBoundary::AfterHit)
                .unwrap()
                .order
                .insertion,
            1
        );
        assert_eq!(queue.len(), 3);
    }
}
