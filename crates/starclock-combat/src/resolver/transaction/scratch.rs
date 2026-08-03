use core::mem;

use crate::battle::state::BattleState;

#[cfg(feature = "benchmark-instrumentation")]
use crate::resolver::journal::JournalMetrics;
use crate::resolver::journal::MutationJournal;

#[derive(Debug)]
pub(crate) struct ResolutionScratch {
    pub(super) working: BattleState,
    pub(super) journal: MutationJournal,
    #[cfg(feature = "benchmark-instrumentation")]
    last_metrics: JournalMetrics,
    #[cfg(test)]
    preparations: u64,
}

impl ResolutionScratch {
    pub(crate) fn from_state(state: &BattleState) -> Self {
        Self {
            working: state.semantic_clone(),
            journal: MutationJournal::default(),
            #[cfg(feature = "benchmark-instrumentation")]
            last_metrics: JournalMetrics::default(),
            #[cfg(test)]
            preparations: 1,
        }
    }

    pub(crate) fn prepare(&mut self, state: &BattleState) {
        self.working.clone_from_semantics(state);
        self.journal.clear();
        #[cfg(test)]
        {
            self.preparations += 1;
        }
    }

    pub(crate) fn commit_into(&mut self, authoritative: &mut BattleState) {
        mem::swap(&mut self.working, authoritative);
        #[cfg(feature = "benchmark-instrumentation")]
        {
            self.last_metrics = self.journal.metrics();
        }
        self.journal.release_bounded();
    }

    #[cfg(feature = "benchmark-instrumentation")]
    pub(crate) const fn last_metrics(&self) -> JournalMetrics {
        self.last_metrics
    }

    #[cfg(test)]
    pub(crate) const fn preparations(&self) -> u64 {
        self.preparations
    }
}
