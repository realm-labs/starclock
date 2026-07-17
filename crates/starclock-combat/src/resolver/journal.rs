use crate::{battle::model::BattlePhase, id::EventId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AllocationKind {
    Command,
    Decision,
    Event,
    Action,
    Phase,
    Hit,
    Operation,
    Wave,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "B2 freezes journal queue categories before B3 begins queue execution"
)]
pub(crate) enum QueueKind {
    Operation,
    Reaction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MutationField {
    Phase,
    Decision,
    CommittedRevision,
    Fault,
    Timeline,
    ActionGauge,
    TeamSkillPoints,
    UnitEnergy,
    UnitHp,
    UnitLife,
    UnitPresence,
    Encounter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "B2 freezes all required forward-journal fact families; B3 consumes the queue forms"
)]
pub(crate) enum JournalKind {
    Allocation {
        kind: AllocationKind,
        raw: u64,
    },
    Mutation {
        field: MutationField,
        before: u64,
        after: u64,
    },
    Event {
        id: EventId,
    },
    RngDraw {
        index: u64,
        purpose: u16,
    },
    Snapshot {
        sequence: u64,
    },
    QueueInsertion {
        queue: QueueKind,
        insertion: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JournalEntry {
    sequence: u32,
    kind: JournalKind,
}

#[derive(Debug, Default)]
pub(crate) struct MutationJournal {
    entries: Vec<JournalEntry>,
}

#[cfg(feature = "benchmark-instrumentation")]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct JournalMetrics {
    pub(crate) entries: u64,
    pub(crate) events: u64,
    pub(crate) operations: u64,
    pub(crate) retained_bytes: u64,
}

impl MutationJournal {
    #[cfg(feature = "benchmark-instrumentation")]
    pub(crate) fn metrics(&self) -> JournalMetrics {
        let mut metrics = JournalMetrics {
            entries: self.entries.len() as u64,
            retained_bytes: (self.entries.capacity() * core::mem::size_of::<JournalEntry>()) as u64,
            ..JournalMetrics::default()
        };
        for entry in &self.entries {
            match entry.kind {
                JournalKind::Event { .. } => metrics.events += 1,
                JournalKind::Allocation {
                    kind: AllocationKind::Operation,
                    ..
                } => metrics.operations += 1,
                _ => {}
            }
        }
        metrics
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn allocation(&mut self, kind: AllocationKind, raw: u64) {
        self.push(JournalKind::Allocation { kind, raw });
    }

    pub(crate) fn mutation(&mut self, field: MutationField, before: u64, after: u64) {
        self.push(JournalKind::Mutation {
            field,
            before,
            after,
        });
    }

    pub(crate) fn event(&mut self, id: EventId) {
        self.push(JournalKind::Event { id });
    }

    #[allow(dead_code, reason = "the first RNG-consuming transaction lands in B3")]
    pub(crate) fn rng_draw(&mut self, index: u64, purpose: u16) {
        self.push(JournalKind::RngDraw { index, purpose });
    }

    #[allow(dead_code, reason = "the first operation snapshot lands in B3")]
    pub(crate) fn snapshot(&mut self, sequence: u64) {
        self.push(JournalKind::Snapshot { sequence });
    }

    #[allow(dead_code, reason = "the first operation/reaction queues land in B3")]
    pub(crate) fn queue_insertion(&mut self, queue: QueueKind, insertion: u64) {
        self.push(JournalKind::QueueInsertion { queue, insertion });
    }

    pub(crate) fn verify(&self, emitted: &[EventId]) -> bool {
        let mut event_index = 0usize;
        for (index, entry) in self.entries.iter().enumerate() {
            if usize::try_from(entry.sequence).ok() != Some(index) {
                return false;
            }
            match entry.kind {
                JournalKind::Allocation { kind, raw } => {
                    let _stable_kind = kind;
                    if raw == 0 {
                        return false;
                    }
                }
                JournalKind::Mutation {
                    field,
                    before,
                    after,
                } => {
                    let _stable_field = field;
                    if before == after {
                        return false;
                    }
                }
                JournalKind::Event { id } => {
                    if emitted.get(event_index) != Some(&id) {
                        return false;
                    }
                    event_index += 1;
                }
                JournalKind::RngDraw { index, purpose } => {
                    if purpose == 0 || index == u64::MAX {
                        return false;
                    }
                }
                JournalKind::Snapshot { sequence } => {
                    if sequence == 0 {
                        return false;
                    }
                }
                JournalKind::QueueInsertion { queue, insertion } => {
                    let _stable_queue = queue;
                    if insertion == 0 {
                        return false;
                    }
                }
            }
        }
        event_index == emitted.len()
    }

    pub(crate) fn release_bounded(&mut self) {
        const MAX_RETAINED_ENTRIES: usize = 4_096;
        self.entries.clear();
        if self.entries.capacity() > MAX_RETAINED_ENTRIES {
            self.entries = Vec::new();
        }
    }

    fn push(&mut self, kind: JournalKind) {
        let sequence = u32::try_from(self.entries.len())
            .expect("rules-revision journal budget is below u32::MAX");
        self.entries.push(JournalEntry { sequence, kind });
    }
}

pub(crate) const fn phase_code(phase: BattlePhase) -> u64 {
    phase as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_journal_accepts_all_transaction_fact_families_in_append_order() {
        let mut journal = MutationJournal::default();
        journal.allocation(AllocationKind::Command, 1);
        journal.mutation(MutationField::Phase, 0, 2);
        journal.rng_draw(0, 1);
        journal.snapshot(1);
        journal.queue_insertion(QueueKind::Operation, 1);
        journal.queue_insertion(QueueKind::Reaction, 2);
        let event = EventId::new(1).unwrap();
        journal.allocation(AllocationKind::Event, event.get());
        journal.event(event);
        assert!(journal.verify(&[event]));
        journal.release_bounded();
        assert!(journal.verify(&[]));
    }
}
