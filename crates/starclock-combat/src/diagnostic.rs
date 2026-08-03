//! Optional, bounded and non-authoritative battle-resolution diagnostics.
//!
//! These records explain transient resolver work that cannot be reconstructed
//! from a stable [`crate::BattleView`]. They never participate in battle state,
//! replay identity, event ordering, RNG, or canonical hashing.

use crate::reaction::queue::ReactionOrder;
use crate::reaction::queue::ReactionTier;
use crate::target::model::TargetCommitment;
use crate::{
    AbilityId, ActionOrigin, BattleStateHash, CommandId, EventId, FormationIndex, RuleId,
    RuleInstanceId, SourceDefinitionId, SpawnSequence, TriggerId, UnitId,
    battle::spec::TeamSide,
    catalog::action::{ReactionBoundary, TargetInvalidationPolicy, UnitTargetSelector},
};

/// Hard bound for one accepted command's diagnostic records.
pub const MAX_DIAGNOSTIC_RECORDS_PER_COMMAND: usize = 8_192;

/// Owned diagnostic batch reused by callers across commands.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BattleDiagnostics {
    root_command: Option<CommandId>,
    committed_revision: Option<u64>,
    state_hash: Option<BattleStateHash>,
    records: Vec<DiagnosticRecord>,
    truncated: bool,
}

impl BattleDiagnostics {
    /// Creates an empty bounded diagnostic batch.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            root_command: None,
            committed_revision: None,
            state_hash: None,
            records: Vec::new(),
            truncated: false,
        }
    }

    /// Clears prior-command records while retaining bounded allocation capacity.
    pub fn clear(&mut self) {
        self.root_command = None;
        self.committed_revision = None;
        self.state_hash = None;
        self.records.clear();
        self.truncated = false;
        if self.records.capacity() > MAX_DIAGNOSTIC_RECORDS_PER_COMMAND {
            self.records.shrink_to(MAX_DIAGNOSTIC_RECORDS_PER_COMMAND);
        }
    }

    /// Returns records in exact resolver-observation order.
    #[must_use]
    pub fn records(&self) -> &[DiagnosticRecord] {
        &self.records
    }

    /// Returns whether additional records were dropped at the hard bound.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the accepted command associated with this batch.
    #[must_use]
    pub const fn root_command(&self) -> Option<CommandId> {
        self.root_command
    }

    /// Returns the committed boundary revision associated with this batch.
    #[must_use]
    pub const fn committed_revision(&self) -> Option<u64> {
        self.committed_revision
    }

    /// Returns the committed boundary hash associated with this batch.
    #[must_use]
    pub const fn state_hash(&self) -> Option<BattleStateHash> {
        self.state_hash
    }

    pub(crate) fn finish(
        &mut self,
        root_command: CommandId,
        committed_revision: u64,
        state_hash: BattleStateHash,
    ) {
        self.root_command = Some(root_command);
        self.committed_revision = Some(committed_revision);
        self.state_hash = Some(state_hash);
    }

    pub(crate) fn record(&mut self, record: impl FnOnce() -> DiagnosticRecord) {
        if self.records.len() == MAX_DIAGNOSTIC_RECORDS_PER_COMMAND {
            self.truncated = true;
            return;
        }
        self.records.push(record());
    }
}

/// Non-authoritative explanation of transient resolver work.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticRecord {
    /// One trigger-produced action entered the deterministic reaction queue.
    ReactionQueued {
        event: EventId,
        actor: UnitId,
        owner: UnitId,
        ability: AbilityId,
        origin: ActionOrigin,
        order: ReactionOrderDiagnostic,
        targets: CommittedTargetsDiagnostic,
    },
    /// The next ready action left the queue for execution-time validation.
    ReactionDequeued {
        insertion: u64,
        actor: UnitId,
        ability: AbilityId,
        boundary: ReactionBoundary,
    },
    /// Execution-time target validation completed for a queued action.
    ReactionTargetsValidated {
        insertion: u64,
        targets: CommittedTargetsDiagnostic,
        accepted: bool,
    },
    /// A queued action was deterministically cancelled before execution.
    ReactionCancelled {
        insertion: u64,
        actor: UnitId,
        ability: AbilityId,
        reason: ActionCancellationReason,
    },
}

/// Complete deterministic order key copied only while diagnostics are enabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReactionOrderDiagnostic {
    pub boundary: ReactionBoundary,
    pub tier: ReactionTierDiagnostic,
    pub priority: i16,
    pub side: TeamSide,
    pub formation: FormationIndex,
    pub spawn: SpawnSequence,
    pub source: SourceDefinitionId,
    pub rule: Option<RuleId>,
    pub instance: Option<RuleInstanceId>,
    pub trigger: Option<TriggerId>,
    pub actor: UnitId,
    pub ability: AbilityId,
    pub insertion: u64,
}

/// Public diagnostic projection of the internal reaction family order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionTierDiagnostic {
    ForcedFollowUp,
    Ultimate,
    ExtraAction,
    ExtraTurnAction,
}

/// Target commitment retained by a queued action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedTargetsDiagnostic {
    pub selector: UnitTargetSelector,
    pub invalidation: TargetInvalidationPolicy,
    pub primary: Option<UnitId>,
    pub targets: Box<[UnitId]>,
}

/// Stable reason that a queued action did not execute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionCancellationReason {
    ActorUnavailable,
    AbilityUnavailable,
    FollowUpBlocked,
    MissingActionDefinition,
    ResourceUnavailable,
    TargetInvalid,
}

impl From<ReactionTier> for ReactionTierDiagnostic {
    fn from(value: ReactionTier) -> Self {
        match value {
            ReactionTier::ForcedFollowUp => Self::ForcedFollowUp,
            ReactionTier::Ultimate => Self::Ultimate,
            ReactionTier::ExtraAction => Self::ExtraAction,
            ReactionTier::ExtraTurnAction => Self::ExtraTurnAction,
        }
    }
}

impl From<ReactionOrder> for ReactionOrderDiagnostic {
    fn from(value: ReactionOrder) -> Self {
        Self {
            boundary: value.boundary,
            tier: value.tier.into(),
            priority: value.priority,
            side: value.side,
            formation: value.formation,
            spawn: value.spawn,
            source: value.source,
            rule: value.rule,
            instance: value.instance,
            trigger: value.trigger,
            actor: value.actor,
            ability: value.ability,
            insertion: value.insertion,
        }
    }
}

impl From<&TargetCommitment> for CommittedTargetsDiagnostic {
    fn from(value: &TargetCommitment) -> Self {
        Self {
            selector: value.selector,
            invalidation: value.invalidation,
            primary: value.primary,
            targets: value.targets.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime<I: TryFrom<u64>>(raw: u64) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).expect("test runtime ID")
    }

    fn definition<I: TryFrom<u32>>(raw: u32) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).expect("test definition ID")
    }

    #[test]
    fn diagnostics_are_hard_bounded_and_reusable() {
        let mut diagnostics = BattleDiagnostics::new();
        for insertion in 1..=MAX_DIAGNOSTIC_RECORDS_PER_COMMAND as u64 + 1 {
            diagnostics.record(|| DiagnosticRecord::ReactionDequeued {
                insertion,
                actor: runtime(1),
                ability: definition(1),
                boundary: ReactionBoundary::AfterAction,
            });
        }
        assert_eq!(
            diagnostics.records().len(),
            MAX_DIAGNOSTIC_RECORDS_PER_COMMAND
        );
        assert!(diagnostics.truncated());
        diagnostics.clear();
        assert!(diagnostics.records().is_empty());
        assert!(!diagnostics.truncated());
    }
}
