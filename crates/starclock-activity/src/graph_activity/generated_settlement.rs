//! Verified battle settlement and RNG-backed pre-offer state operations.

use crate::{
    ActivityBattleResultSubmission, ActivityBattleSettlement, ActivityCause, ActivityOperation,
    ActivityPlayerView, ActivityProgramDefinition, ActivityProgramId, ActivityRngStreams,
    ActivityStateHash, ActivityTerminalOutcome, ActivityTransactionOutcome, BattleResult,
    GraphActivity, GraphActivityBattleError, GraphActivityBattleResolution,
    GraphActivityCommandError, GraphActivityRuntimeError,
};

use super::contains_boundary_operation;

// The mode supplies a static, configuration-bound sequence, not an unbounded
// callback loop. Each stage retains the ordinary operation/program limits.
const MAX_SETTLEMENT_STAGES: usize = 32;

impl GraphActivity {
    /// Verifies and settles a battle, generates ordered state-only stages, then
    /// resumes the destination node, all within one atomic boundary.
    ///
    /// This trusted mode-executor hook is not an adapter command. The generator
    /// observes verified carry, metrics and the destination node before its
    /// program runs or offers a decision. The nonempty sequence contains at most
    /// 32 distinct program IDs, in mode-authored order. Each generator observes
    /// all preceding stages' committed operations and receives its program ID,
    /// the original typed settlement and labeled Activity RNG. The settlement's
    /// hash describes the pre-stage state, not later stage views. The mode must
    /// bind this order and its generators to configuration identity; callbacks
    /// must not perform external effects. Invalid,
    /// stale or duplicate results reject before calling the generator.
    ///
    /// Generated operations cannot offer, traverse, relocate or terminate. Any
    /// generation, validation, mutation or pump failure restores the pending
    /// battle, state, events and RNG to their pre-submission snapshot. A fault
    /// caused by generated operations or the destination program also rolls
    /// back; a verified battle outcome routed to a fault terminal is retained.
    pub fn submit_pending_battle_result_with_generated_boundary(
        &mut self,
        expected_state_hash: ActivityStateHash,
        result: BattleResult,
        program_ids: &[ActivityProgramId],
        mut generate: impl FnMut(
            ActivityProgramId,
            &ActivityPlayerView,
            ActivityBattleSettlement,
            &mut ActivityRngStreams,
        ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError>,
    ) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
        if program_ids.is_empty()
            || program_ids.len() > MAX_SETTLEMENT_STAGES
            || program_ids
                .iter()
                .enumerate()
                .any(|(index, id)| program_ids[..index].contains(id))
        {
            return Err(invalid_boundary());
        }
        let original_state = self.state.transaction_copy();
        let original_rng = self.rng.transaction_copy();
        let outcome = (|| {
            let settlement = self
                .state
                .submit_pending_battle_result(
                    self.definition.identity,
                    &self.definition.graph,
                    self.instance,
                    &self.rng,
                    ActivityBattleResultSubmission::new(expected_state_hash, result),
                )
                .map_err(GraphActivityBattleError::Settlement)?;
            let mut events = Vec::new();
            for &program_id in program_ids {
                let operations =
                    generate(program_id, &self.player_view(), settlement, &mut self.rng)
                        .map_err(GraphActivityBattleError::GeneratedBoundary)?;
                if operations.is_empty() {
                    continue;
                }
                let program = ActivityProgramDefinition::new(program_id, operations)
                    .map_err(|_| invalid_boundary())?;
                program
                    .validate_against(&self.definition.state, &self.definition.graph)
                    .map_err(|_| invalid_boundary())?;
                if contains_boundary_operation(program.operations()) {
                    return Err(invalid_boundary());
                }
                let sequence = self
                    .state
                    .command_sequence()
                    .checked_add(1)
                    .ok_or_else(invalid_boundary)?;
                let cause = ActivityCause::new(sequence, program_id, self.state.current_node())
                    .ok_or_else(invalid_boundary)?;
                events.extend(
                    match self.state.apply_settlement_extension_program(
                        &program,
                        cause,
                        &self.definition.graph,
                    ) {
                        ActivityTransactionOutcome::Committed(events) => events,
                        ActivityTransactionOutcome::Rejected(rejection) => {
                            return Err(GraphActivityBattleError::GeneratedBoundary(
                                GraphActivityCommandError::Rejected(rejection),
                            ));
                        }
                        ActivityTransactionOutcome::Faulted(_, fault) => {
                            return Err(GraphActivityBattleError::GeneratedBoundary(
                                GraphActivityCommandError::InteractionFault(fault),
                            ));
                        }
                    },
                );
            }
            let settled_terminal = self.state.terminal();
            events.extend(self.pump().map_err(GraphActivityBattleError::Runtime)?);
            if settled_terminal != Some(ActivityTerminalOutcome::Faulted)
                && self.state.terminal() == Some(ActivityTerminalOutcome::Faulted)
            {
                return Err(invalid_boundary());
            }
            Ok(GraphActivityBattleResolution {
                settlement,
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

fn invalid_boundary() -> GraphActivityBattleError {
    GraphActivityBattleError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
