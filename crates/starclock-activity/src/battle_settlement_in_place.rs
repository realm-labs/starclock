//! Atomic verified battle settlement for graphs whose room routing remains
//! separate from the battle-result boundary.

use crate::{
    ActivityBattleResultSubmission, ActivityBattleSettlementError, ActivityCause,
    ActivityDefinitionIdentity, ActivityGraphDefinition, ActivityInstanceId,
    ActivityProgramBindingError, ActivityProgramDefinition, ActivityRngStreams, ActivityStateHash,
    ActivityTerminalOutcome, ActivityTransactionEvent, ActivityTransactionOutcome,
    ActivityTransactionRejection, ActivityTransactionState, BattleOutcome, BattleResultDigest,
    battle_settlement::{
        MAX_COMPLETED_ACTIVITY_BATTLES, apply_carry, metric_value, validate_participant_results,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBattleInPlaceSettlement {
    outcome: BattleOutcome,
    result_digest: BattleResultDigest,
    terminal: Option<ActivityTerminalOutcome>,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl ActivityBattleInPlaceSettlement {
    #[must_use]
    pub const fn outcome(&self) -> BattleOutcome {
        self.outcome
    }
    #[must_use]
    pub const fn result_digest(&self) -> BattleResultDigest {
        self.result_digest
    }
    #[must_use]
    pub const fn terminal(&self) -> Option<ActivityTerminalOutcome> {
        self.terminal
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityBattleInPlaceSettlementError {
    InvalidProgram(ActivityProgramBindingError),
    Settlement(ActivityBattleSettlementError),
    Rejected(ActivityTransactionRejection),
}

impl ActivityTransactionState {
    /// Verifies and settles a nested result without consuming a graph edge,
    /// then atomically applies one optional post-battle program.
    ///
    /// Victory leaves routing to the mode's next ordinary command. Defeat and
    /// combat faults enter the generic failed/faulted Activity terminal even
    /// when a mode graph intentionally has no corresponding room node.
    pub fn submit_pending_battle_result_in_place(
        &mut self,
        identity: ActivityDefinitionIdentity,
        graph: &ActivityGraphDefinition,
        instance: ActivityInstanceId,
        rng: &ActivityRngStreams,
        submission: ActivityBattleResultSubmission,
        post_battle: Option<&ActivityProgramDefinition>,
    ) -> Result<ActivityBattleInPlaceSettlement, ActivityBattleInPlaceSettlementError> {
        if let Some(program) = post_battle {
            program
                .validate_against(self.state_definition(), graph)
                .map_err(ActivityBattleInPlaceSettlementError::InvalidProgram)?;
        }
        if self.state_hash(identity, graph, instance, rng) != submission.expected_state_hash {
            return Err(settlement(ActivityBattleSettlementError::StaleState));
        }
        let awaiting = self
            .awaiting_battle
            .as_ref()
            .ok_or_else(|| settlement(ActivityBattleSettlementError::BattleNotStarted))?;
        let result = submission.result.as_ref();
        if result.identity() != awaiting.identity {
            return Err(settlement(
                ActivityBattleSettlementError::ResultIdentityMismatch,
            ));
        }
        let actual_digest = result.actual_digest();
        if actual_digest != result.claimed_digest() {
            return Err(settlement(
                ActivityBattleSettlementError::ResultDigestMismatch,
            ));
        }
        if !awaiting.contract.projection.matches(result.values()) {
            return Err(settlement(
                ActivityBattleSettlementError::ResultProjectionMismatch,
            ));
        }
        let outcome = result
            .outcome()
            .ok_or_else(|| settlement(ActivityBattleSettlementError::ResultProjectionMismatch))?;
        let fault = result
            .terminal_fault()
            .ok_or_else(|| settlement(ActivityBattleSettlementError::ResultProjectionMismatch))?;
        if (outcome == BattleOutcome::Faulted) != fault.is_some() {
            return Err(settlement(
                ActivityBattleSettlementError::FaultOutcomeMismatch,
            ));
        }
        let attempt = self
            .attempt
            .as_ref()
            .ok_or_else(|| settlement(ActivityBattleSettlementError::MissingPendingBattle))?;
        validate_participant_results(attempt, result).map_err(settlement)?;
        if self.completed_battles.len() >= MAX_COMPLETED_ACTIVITY_BATTLES {
            return Err(settlement(
                ActivityBattleSettlementError::CompletedBattleLimit,
            ));
        }
        let contract = awaiting.contract.clone();
        let mut working = self.transaction_copy();
        for definition in contract.carry.iter().copied() {
            let state = result
                .participant_states()
                .find(|state| state.participant() == definition.participant())
                .ok_or_else(|| {
                    settlement(ActivityBattleSettlementError::ParticipantResultMismatch)
                })?;
            working
                .carry
                .insert(apply_carry(definition, state).map_err(settlement)?);
        }
        for binding in contract.metrics.iter() {
            let value = result
                .metrics()
                .find(|(key, _)| *key == binding.key())
                .map(|(_, value)| value)
                .ok_or_else(|| {
                    settlement(ActivityBattleSettlementError::ResultProjectionMismatch)
                })?;
            working
                .settle_metric(binding.slot(), metric_value(value), binding.policy())
                .map_err(|error| settlement(ActivityBattleSettlementError::ActivityFault(error)))?;
        }
        let terminal = match outcome {
            BattleOutcome::Won | BattleOutcome::Finalized => None,
            BattleOutcome::Lost => Some(ActivityTerminalOutcome::Failed),
            BattleOutcome::Faulted => Some(ActivityTerminalOutcome::Faulted),
        };
        if let Some(terminal) = terminal {
            working.settle_terminal(terminal);
        }
        working.awaiting_battle = None;
        working
            .attempt
            .as_mut()
            .expect("validated attempt exists")
            .mark_settled();
        working.completed_battles.push(actual_digest);
        let mut events = Vec::new();
        if let Some(program) = post_battle {
            let cause = ActivityCause::new(
                working.command_sequence().saturating_add(1),
                program.id(),
                working.current_node(),
            )
            .expect("a successor command sequence is non-zero");
            match working.apply_program(program, cause, graph) {
                ActivityTransactionOutcome::Committed(committed)
                | ActivityTransactionOutcome::Faulted(committed, _) => events.extend(committed),
                ActivityTransactionOutcome::Rejected(error) => {
                    return Err(ActivityBattleInPlaceSettlementError::Rejected(error));
                }
            }
        }
        let state_hash = working.state_hash(identity, graph, instance, rng);
        *self = working;
        Ok(ActivityBattleInPlaceSettlement {
            outcome,
            result_digest: actual_digest,
            terminal,
            events: events.into_boxed_slice(),
            state_hash,
        })
    }
}

const fn settlement(error: ActivityBattleSettlementError) -> ActivityBattleInPlaceSettlementError {
    ActivityBattleInPlaceSettlementError::Settlement(error)
}
