use crate::{
    battle::spec::{FormationIndex, TeamSide},
    id::{AbilityId, SpawnSequence, UnitId},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(
    dead_code,
    reason = "B3 freezes interrupt priority ordering; B4 introduces executable interrupt offers"
)]
pub(crate) enum InterruptPriority {
    ForcedFollowUp = 0,
    Ultimate = 1,
    ExtraAction = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PendingInterrupt {
    pub(crate) priority: InterruptPriority,
    pub(crate) side: TeamSide,
    pub(crate) formation: FormationIndex,
    pub(crate) spawn: SpawnSequence,
    pub(crate) actor: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) insertion: u64,
}

impl PendingInterrupt {
    #[allow(
        dead_code,
        reason = "B3 queue tests freeze the total key; B4 constructs runtime entries"
    )]
    fn key(
        self,
    ) -> (
        InterruptPriority,
        TeamSide,
        FormationIndex,
        SpawnSequence,
        UnitId,
        AbilityId,
        u64,
    ) {
        (
            self.priority,
            self.side,
            self.formation,
            self.spawn,
            self.actor,
            self.ability,
            self.insertion,
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct InterruptQueue {
    entries: Vec<PendingInterrupt>,
}

impl InterruptQueue {
    #[allow(
        dead_code,
        reason = "B3 freezes queue insertion/removal; B4 creates executable entries"
    )]
    pub(crate) fn push(&mut self, entry: PendingInterrupt) {
        let index = self
            .entries
            .binary_search_by(|candidate| candidate.key().cmp(&entry.key()))
            .unwrap_or_else(|index| index);
        self.entries.insert(index, entry);
    }

    #[allow(
        dead_code,
        reason = "B3 freezes queue insertion/removal; B4 creates executable entries"
    )]
    pub(crate) fn pop(&mut self) -> Option<PendingInterrupt> {
        (!self.entries.is_empty()).then(|| self.entries.remove(0))
    }

    pub(crate) fn entries(&self) -> &[PendingInterrupt] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition<I: TryFrom<u32>>(raw: u32) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    fn runtime<I: TryFrom<u64>>(raw: u64) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    #[test]
    fn interrupt_order_has_a_complete_stable_tie_key() {
        let base = PendingInterrupt {
            priority: InterruptPriority::Ultimate,
            side: TeamSide::Player,
            formation: FormationIndex::new(0).unwrap(),
            spawn: runtime(1),
            actor: runtime(1),
            ability: definition(1),
            insertion: 2,
        };
        let mut queue = InterruptQueue::default();
        queue.push(base);
        queue.push(PendingInterrupt {
            priority: InterruptPriority::ForcedFollowUp,
            insertion: 3,
            ..base
        });
        queue.push(PendingInterrupt {
            priority: InterruptPriority::ExtraAction,
            insertion: 1,
            ..base
        });
        queue.push(PendingInterrupt {
            insertion: 1,
            ..base
        });
        assert_eq!(
            queue
                .entries()
                .iter()
                .map(|entry| (entry.priority, entry.insertion))
                .collect::<Vec<_>>(),
            [
                (InterruptPriority::ForcedFollowUp, 3),
                (InterruptPriority::Ultimate, 1),
                (InterruptPriority::Ultimate, 2),
                (InterruptPriority::ExtraAction, 1),
            ]
        );
        assert_eq!(
            queue.pop().unwrap().priority,
            InterruptPriority::ForcedFollowUp
        );
        assert_eq!(queue.pop().unwrap().insertion, 1);
    }
}
