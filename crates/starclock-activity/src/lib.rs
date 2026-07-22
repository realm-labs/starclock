//! Generic cross-battle activity orchestration boundary.
//!
//! Activities own flow, scoped state, participant locks and declared battle
//! result projections while treating resolved battle input as opaque handoff
//! data.

#![forbid(unsafe_code)]

mod aggregate;
mod codec;
mod graph;
mod id;
mod participant;
mod projection;
mod scope;
mod slot;
mod spec;
mod state_definition;

pub use aggregate::{
    Activity, ActivityCommand, ActivityCommandError, ActivityCommandErrorKind, ActivityDecision,
    ActivityEvent, ActivityPhase, ActivityResolution, BattleHandoff, ResultIdentityField,
};
pub use codec::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityGraphDigest, ActivityStateHash,
    BattleResultDigest, BuildDigest, EventDigest, ParticipantLockDigest,
};
pub use graph::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityGraphDefinition,
    ActivityGraphDefinitionError, ActivityNodeDefinition, ActivityNodeKind,
    ActivityTerminalOutcome, MAX_ACTIVITY_EDGES, MAX_ACTIVITY_NODES, MAX_ACTIVITY_TOTAL_VISITS,
    MAX_EDGE_TRAVERSALS, MAX_NODE_VISITS,
};
pub use id::{
    ActivityDefinitionId, ActivityEdgeId, ActivityInstanceId, ActivityInventoryId,
    ActivityModifierId, ActivitySlotId, AttemptId, BattleSequence, NodeId, ParticipantId,
    ProjectionId, SectionId,
};
pub use participant::{
    LoadoutLockScope, OpaqueParticipantBuild, ParticipantLock, ParticipantLockEntry,
    ParticipantLockError, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
pub use projection::{
    BattleOutcome, BattleResult, BattleResultConfiguration, BattleResultIdentity,
    BattleResultProjection, BattleResultProjectionError, MetricValue, MetricValueKind,
    ProjectedValue, ProjectionField,
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
