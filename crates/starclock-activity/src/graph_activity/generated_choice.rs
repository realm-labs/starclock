//! RNG-backed option prefixes use the ordinary pending-option transaction.

use crate::{
    ActivityCause, ActivityDecisionId, ActivityDecisionKind, ActivityGeneratedBoundaryResolution,
    ActivityOperation, ActivityOptionId, ActivityPlayerView, ActivityProgramDefinition,
    ActivityRngStreams, ActivityStateHash, ActivityTerminalOutcome, ActivityTransactionOutcome,
    ActivityTransactionRejection, GraphActivity, GraphActivityCommandError,
    GraphActivityRuntimeError, MAX_ACTIVITY_PROGRAM_OPERATIONS,
};

use super::contains_boundary_operation;

impl GraphActivity {
    /// Generates state-only operations and commits them with one offered choice.
    ///
    /// This is a trusted mode-executor boundary, not permission for an adapter
    /// to supply arbitrary rewards. The generator reads the pre-command player
    /// view and labeled Activity RNG only; it must not perform external effects.
    /// Stale hashes, wrong decisions/options and external outcomes reject before
    /// generation. Authored random-offer selection operations execute first,
    /// followed by the generated prefix and the selected option's operations.
    /// The combined program is validated, including its nested operation budget.
    /// Generated operations cannot offer, traverse, relocate or terminate.
    ///
    /// State changes, option consumption, events, RNG and automatic graph advance
    /// share one commit boundary. Any generation, validation, rejection, fault or
    /// pump error restores the exact pre-command state and RNG; no result escapes
    /// on failure. The caller-owned value is returned only after successful commit.
    pub fn choose_option_with_generated_prefix<T>(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
        generate: impl FnOnce(
            &ActivityPlayerView,
            &mut ActivityRngStreams,
        ) -> Result<(Vec<ActivityOperation>, T), GraphActivityCommandError>,
    ) -> Result<ActivityGeneratedBoundaryResolution<T>, GraphActivityCommandError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        let view = self.player_view();
        let offered = view
            .decision()
            .filter(|offered| offered.id() == decision)
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        if offered.kind() == ActivityDecisionKind::ExternalOutcome {
            return Err(GraphActivityCommandError::InteractionNotBound);
        }
        let selected = self
            .state
            .pending_option_definition(option)
            .ok_or(GraphActivityCommandError::Rejected(
                ActivityTransactionRejection::UnknownOption,
            ))?
            .operations()
            .to_vec();
        let program_id = self
            .definition
            .program(self.state.current_node())
            .ok_or_else(invalid_prefix)?
            .id();
        let sequence = self
            .state
            .command_sequence()
            .checked_add(1)
            .ok_or_else(invalid_prefix)?;
        let cause = ActivityCause::new(sequence, program_id, self.state.current_node())
            .ok_or_else(invalid_prefix)?;
        let mut prefix = self
            .definition
            .random_offer(self.state.current_node())
            .map_or_else(Vec::new, |policy| policy.selection_prefix.to_vec());
        let original_state = self.state.transaction_copy();
        let original_rng = self.rng.transaction_copy();
        let outcome = (|| {
            let (generated, value) = generate(&view, &mut self.rng)?;
            let count = generated
                .len()
                .checked_add(prefix.len())
                .and_then(|count| count.checked_add(selected.len()))
                .ok_or_else(invalid_prefix)?;
            if count > MAX_ACTIVITY_PROGRAM_OPERATIONS {
                return Err(invalid_prefix());
            }
            // Validate depth before recursively checking forbidden boundaries.
            let generated_program = ActivityProgramDefinition::new(program_id, generated)
                .map_err(|_| invalid_prefix())?;
            if contains_boundary_operation(generated_program.operations()) {
                return Err(invalid_prefix());
            }
            prefix.extend_from_slice(generated_program.operations());
            let mut combined = prefix.clone();
            combined.extend(selected);
            let program = ActivityProgramDefinition::new(program_id, combined)
                .map_err(|_| invalid_prefix())?;
            program
                .validate_against(&self.definition.state, &self.definition.graph)
                .map_err(|_| invalid_prefix())?;
            let mut events = match self.state.apply_option_with_prefix(
                option,
                &prefix,
                cause,
                &self.definition.graph,
            ) {
                ActivityTransactionOutcome::Committed(events) => events.into_vec(),
                ActivityTransactionOutcome::Rejected(rejection) => {
                    return Err(GraphActivityCommandError::Rejected(rejection));
                }
                ActivityTransactionOutcome::Faulted(_, fault) => {
                    return Err(GraphActivityCommandError::InteractionFault(fault));
                }
            };
            events.extend(self.pump().map_err(GraphActivityCommandError::Runtime)?);
            if self.state.terminal() == Some(ActivityTerminalOutcome::Faulted) {
                return Err(invalid_prefix());
            }
            Ok(ActivityGeneratedBoundaryResolution {
                value,
                events: events.into_boxed_slice(),
                state_hash: self.state_hash(),
            })
        })();
        if outcome.is_err() {
            self.state = original_state;
            self.rng = original_rng;
        }
        outcome
    }
}

fn invalid_prefix() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
