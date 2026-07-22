use super::*;

pub(super) fn replace_pending_with_program(
    state: &mut ActivityTransactionState,
    program: &ActivityProgramDefinition,
    cause: ActivityCause,
    graph: &ActivityGraphDefinition,
) -> ActivityTransactionOutcome {
    if state.pending.is_none() || state.terminal.is_some() {
        return ActivityTransactionOutcome::Rejected(
            ActivityTransactionRejection::DecisionNotOffered,
        );
    }
    if cause.program != program.id()
        || cause.node != state.current_node
        || cause.command_sequence != state.command_sequence.saturating_add(1)
    {
        return ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::CauseMismatch);
    }
    let mut working = state.transaction_copy();
    working.pending = None;
    let mut events = Vec::new();
    match working.execute(program.operations(), cause, graph, &mut events) {
        Ok(()) => {
            working.command_sequence = cause.command_sequence;
            *state = working;
            ActivityTransactionOutcome::Committed(events.into_boxed_slice())
        }
        Err(ExecutionFailure::Rejected(error)) => ActivityTransactionOutcome::Rejected(error),
        Err(ExecutionFailure::Fault(fault)) => {
            let mut faulted = state.transaction_copy();
            faulted.command_sequence = cause.command_sequence;
            faulted.pending = None;
            faulted.terminal = Some(ActivityTerminalOutcome::Faulted);
            events.clear();
            events.push(ActivityTransactionEvent {
                cause,
                kind: ActivityTransactionEventKind::Faulted(fault),
            });
            *state = faulted;
            ActivityTransactionOutcome::Faulted(events.into_boxed_slice(), fault)
        }
    }
}
