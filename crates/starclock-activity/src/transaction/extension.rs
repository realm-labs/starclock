use super::*;
use crate::ActivityGraphDefinition;

impl ActivityTransactionState {
    /// Applies state-only extension work without consuming an already offered
    /// player decision. The caller must independently reject boundary
    /// operations before entering this transaction primitive.
    pub(crate) fn apply_extension_program(
        &mut self,
        program: &ActivityProgramDefinition,
        cause: ActivityCause,
        graph: &ActivityGraphDefinition,
    ) -> ActivityTransactionOutcome {
        self.apply_extension_program_at_settlement(program, cause, graph, false)
    }

    /// Applies state-only settlement work after the battle result has selected
    /// its continuation or terminal node.
    pub(crate) fn apply_settlement_extension_program(
        &mut self,
        program: &ActivityProgramDefinition,
        cause: ActivityCause,
        graph: &ActivityGraphDefinition,
    ) -> ActivityTransactionOutcome {
        self.apply_extension_program_at_settlement(program, cause, graph, true)
    }

    fn apply_extension_program_at_settlement(
        &mut self,
        program: &ActivityProgramDefinition,
        cause: ActivityCause,
        graph: &ActivityGraphDefinition,
        allow_terminal: bool,
    ) -> ActivityTransactionOutcome {
        if self.terminal.is_some() && !allow_terminal {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::StateAlreadyAtBoundary,
            );
        }
        if cause.program != program.id()
            || cause.node != self.current_node
            || cause.command_sequence != self.command_sequence.saturating_add(1)
        {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::CauseMismatch,
            );
        }
        let mut working = self.transaction_copy();
        let mut events = Vec::new();
        match working.execute(program.operations(), cause, graph, &mut events) {
            Ok(()) => {
                working.command_sequence = cause.command_sequence;
                *self = working;
                ActivityTransactionOutcome::Committed(events.into_boxed_slice())
            }
            Err(ExecutionFailure::Rejected(error)) => ActivityTransactionOutcome::Rejected(error),
            Err(ExecutionFailure::Fault(fault)) => {
                let mut faulted = self.transaction_copy();
                faulted.command_sequence = cause.command_sequence;
                faulted.pending = None;
                faulted.terminal = Some(ActivityTerminalOutcome::Faulted);
                events.clear();
                events.push(ActivityTransactionEvent {
                    cause,
                    kind: ActivityTransactionEventKind::Faulted(fault),
                });
                *self = faulted;
                ActivityTransactionOutcome::Faulted(events.into_boxed_slice(), fault)
            }
        }
    }
}
