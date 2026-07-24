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

pub(super) fn apply_option_with_prefix(
    state: &mut ActivityTransactionState,
    option: ActivityOptionId,
    prefix: &[crate::ActivityOperation],
    cause: ActivityCause,
    graph: &ActivityGraphDefinition,
) -> ActivityTransactionOutcome {
    let Some(pending) = &state.pending else {
        return ActivityTransactionOutcome::Rejected(
            ActivityTransactionRejection::DecisionNotOffered,
        );
    };
    if cause.node != state.current_node
        || cause.command_sequence != state.command_sequence.saturating_add(1)
    {
        return ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::CauseMismatch);
    }
    let Some(selected) = pending.options.iter().find(|item| item.id() == option) else {
        return ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::UnknownOption);
    };
    let operation_count = prefix
        .len()
        .checked_add(selected.operations().len())
        .expect("validated Activity operation bounds fit usize");
    let mut operations = Vec::with_capacity(operation_count);
    operations.extend_from_slice(prefix);
    operations.extend_from_slice(selected.operations());
    let cause = cause.with_option(option);
    let mut working = state.transaction_copy();
    working.pending = None;
    let mut events = Vec::new();
    match working.execute(&operations, cause, graph, &mut events) {
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
