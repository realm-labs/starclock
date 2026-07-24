use crate::{
    ActivityBattleSettlementError, ActivityFault, ActivityHandlerFaultKind,
    ActivityInteractionBindingError, ActivityPreparationError, ActivityRngError,
    ActivityTransactionRejection, NodeId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityDefinitionError {
    InvalidBootstrapSelection,
    InvalidBootstrapSlot,
    DuplicateNodeProgram,
    MissingNodeProgram(NodeId),
    TerminalNodeProgram(NodeId),
    InvalidProgramBinding(NodeId),
    InvalidRandomCheckpoint,
    DuplicateRandomCheckpoint,
    InvalidRandomOffer,
    DuplicateRandomOffer,
    IncompatibleStateShape,
    InvalidLogicalScopes,
    InvalidInteractionBindings(ActivityInteractionBindingError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityStartError {
    Rng(ActivityRngError),
    State(ActivityFault),
    Runtime(GraphActivityRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityRuntimeError {
    MissingNodeProgram,
    InvalidCause,
    Rejected(ActivityTransactionRejection),
    AutomaticStepLimit,
    InvalidRandomCheckpoint,
    InvalidRandomOffer,
    Rng(ActivityRngError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityRandomOfferError {
    StaleStateHash,
    NotOffered,
    RerollDisabled,
    RerollLimitReached,
    InvalidCounter,
    InvalidProgram,
    Rejected(ActivityTransactionRejection),
    Faulted(ActivityFault),
    Runtime(GraphActivityRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityCommandError {
    StaleStateHash,
    DecisionNotOffered,
    InteractionNotBound,
    HandlerUnavailable,
    HandlerFault(ActivityHandlerFaultKind),
    InteractionOperationLimit,
    Rejected(ActivityTransactionRejection),
    Runtime(GraphActivityRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityEncounterError {
    StaleStateHash,
    DecisionNotOffered,
    Rejected(ActivityTransactionRejection),
    Faulted(ActivityFault),
    Preparation(ActivityPreparationError),
    Runtime(GraphActivityRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityBattleError {
    Settlement(ActivityBattleSettlementError),
    Runtime(GraphActivityRuntimeError),
}
