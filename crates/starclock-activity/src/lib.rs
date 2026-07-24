//! Generic cross-battle activity orchestration boundary.
//!
//! Activities own flow, scoped state, participant locks and declared battle
//! result projections while treating resolved battle input as opaque handoff
//! data.

#![forbid(unsafe_code)]

mod activity_rng;
mod aggregate;
mod battle_preparation;
mod battle_settlement;
mod codec;
mod graph;
mod graph_activity;
mod graph_command;
mod handler_registry;
mod id;
mod participant;
mod program;
mod projection;
mod scope;
mod slot;
mod spec;
mod state_definition;
mod transaction;
mod view;

pub use activity_rng::{
    ACTIVITY_RNG_REVISION, ActivityRngContext, ActivityRngDraw, ActivityRngError, ActivityRngLabel,
    ActivityRngStreamSnapshot, ActivityRngStreams,
};
pub use aggregate::{
    Activity, ActivityCommand, ActivityCommandError, ActivityCommandErrorKind, ActivityDecision,
    ActivityEvent, ActivityPhase, ActivityResolution, BattleHandoff, ResultIdentityField,
};
pub use battle_preparation::{
    ActivityBattlePreparationRequest, ActivityPendingBattleView, ActivityPreparationBoundary,
    ActivityPreparationError, ActivityPreparationOptionKind, ActivityPreparationOptionView,
    ActivityPreparationView, ActivityRosterLock, ActivityRosterLockError,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EncounterPreparationDefinitionError,
    MAX_PREPARATION_TECHNIQUES, MAX_PREPARED_BATTLE_VARIANTS, PendingBattleSpec,
    PreparedBattleVariant, TechniqueEngagement, TechniqueOptionDefinition,
};
pub use battle_settlement::{
    ActivityBattleHandoff, ActivityBattleResultContract, ActivityBattleResultContractError,
    ActivityBattleResultSubmission, ActivityBattleSettlement, ActivityBattleSettlementError,
    ActivityBattleStartRequest, ActivityMetricProjectionBinding,
    ActivityParticipantCarryDefinition, ActivityParticipantCarryState, EnergyCarryPolicy,
    HpCarryPolicy, LifeCarryPolicy, MAX_COMPLETED_ACTIVITY_BATTLES, MetricSettlementPolicy,
    PresenceCarryPolicy,
};
pub use codec::{
    ACTIVITY_STATE_CODEC_REVISION, ACTIVITY_STATE_HASH_REVISION, ActivityConfigDigest,
    ActivityDefinitionDigest, ActivityGraphDigest, ActivityStateHash, BattleProjectionDigest,
    BattleResultDigest, BattleSettlementContractDigest, BuildDigest, EncounterPreparationDigest,
    EventDigest, ParticipantLockDigest, TechniqueContributionDigest,
};
pub use graph::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityGraphDefinition,
    ActivityGraphDefinitionError, ActivityNodeDefinition, ActivityNodeKind,
    ActivityTerminalOutcome, MAX_ACTIVITY_EDGES, MAX_ACTIVITY_NODES, MAX_ACTIVITY_TOTAL_VISITS,
    MAX_EDGE_TRAVERSALS, MAX_NODE_VISITS,
};
pub use graph_activity::{
    ActivityBootstrapSelection, ActivityRandomCheckpoint, ActivityRandomOffer,
    ActivityRandomPolicies, GraphActivity, GraphActivityBattleError, GraphActivityBattleResolution,
    GraphActivityCommandError, GraphActivityDefinition, GraphActivityDefinitionError,
    GraphActivityEncounterError, GraphActivityNodeProgram, GraphActivityPreparationResolution,
    GraphActivityRandomOfferError, GraphActivityResolution, GraphActivityRuntimeError,
    GraphActivityStartError,
};
pub use graph_command::{
    GRAPH_ACTIVITY_API_REVISION, GraphActivityCommand, GraphActivityCommandKind,
};
pub use handler_registry::{
    ACTIVITY_HANDLER_REGISTRY_REVISION, ActivityHandler, ActivityHandlerBundle,
    ActivityHandlerFault, ActivityHandlerFaultKind, ActivityHandlerInput, ActivityHandlerOutput,
    ActivityHandlerRegistration, ActivityHandlerRegistry, ActivityHandlerRegistryDigest,
    ActivityHandlerRegistryError, MAX_ACTIVITY_HANDLER_BUNDLES, MAX_ACTIVITY_HANDLER_PAYLOAD_BYTES,
    MAX_ACTIVITY_HANDLERS, core_activity_handler_bundle,
};
pub use id::{
    ActivityBattleHandoffId, ActivityDecisionId, ActivityDefinitionId, ActivityEdgeId,
    ActivityExternalOutcomeId, ActivityHandlerId, ActivityInstanceId, ActivityInventoryId,
    ActivityModifierId, ActivityOptionId, ActivityProgramId, ActivitySlotId, AttemptId,
    BattleSequence, NodeId, ParticipantId, ProjectionId, SectionId,
};
pub use participant::{
    LoadoutLockScope, OpaqueParticipantBuild, ParticipantLock, ParticipantLockEntry,
    ParticipantLockError, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
pub use program::{
    ActivityCondition, ActivityDecisionKind, ActivityExpression, ActivityOperation,
    ActivityOptionDefinition, ActivityProgramBindingError, ActivityProgramDefinition,
    ActivityProgramDefinitionError, ActivityValueType, MAX_ACTIVITY_OPTIONS,
    MAX_ACTIVITY_PROGRAM_DEPTH, MAX_ACTIVITY_PROGRAM_OPERATIONS,
};
pub use projection::{
    BattleOutcome, BattleResult, BattleResultConfiguration, BattleResultIdentity,
    BattleResultProjection, BattleResultProjectionError, MetricValue, MetricValueKind,
    ParticipantBattleState, ProjectedValue, ProjectionField,
};
pub use scope::{ActivityScope, OneBattleFlow, OneBattleFlowError, ScopeIdentity, TerminalOutcome};
pub use slot::{
    ActivitySlotDefinition, ActivityValue, MAX_SLOT_COLLECTION_ENTRIES, SlotDefinitionError,
    SlotResetPoint, SlotValueKind,
};
pub use spec::{
    ActivityDefinitionIdentity, ActivityMasterSeed, ActivitySpec, ActivitySpecError, BattleBinding,
    BattleBindingError,
};
pub use state_definition::{
    ActivityAccumulationPolicy, ActivityInventoryDefinition, ActivityModifierDefinition,
    ActivityModifierOwner, ActivityScopeIdentity, ActivityScopePath, ActivityScopePathError,
    ActivitySnapshotBoundary, ActivityStateDefinition, ActivityStateDefinitionError,
    ActivityStateSource, ActivityStateVisibility, MAX_ACTIVITY_INVENTORIES, MAX_ACTIVITY_MODIFIERS,
    MAX_ACTIVITY_STATE_SLOTS, MAX_INVENTORY_ENTRIES, MAX_INVENTORY_STACK, SlotCarryPolicy,
};
pub use transaction::{
    ActivityCause, ActivityFault, ActivityTransactionEvent, ActivityTransactionEventKind,
    ActivityTransactionOutcome, ActivityTransactionRejection, ActivityTransactionState,
};
pub use view::{
    ActivityDebugView, ActivityDecisionView, ActivityInventoryView, ActivityOptionView,
    ActivityPlayerView, ActivitySlotView,
};
