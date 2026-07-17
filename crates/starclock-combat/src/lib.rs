//! Deterministic, engine-agnostic ownership boundary for exactly one battle.
//!
//! This crate accepts only generic resolved combat input. Build selections,
//! generated data rows, activities, modes, controllers, replay transport and
//! engines remain outside this dependency root.

#![forbid(unsafe_code)]

mod action;
mod actor;
mod battle;
#[cfg(feature = "benchmark-instrumentation")]
pub mod benchmark;
pub mod catalog;
mod codec;
mod command;
mod event;
mod formula;
mod id;
mod numeric;
mod operation;
mod resolver;
mod resource;
pub mod rng;
mod target;
mod timeline;

// This is the deliberate small crate facade. The defining modules remain
// private so representation/backend details have one canonical external path.
pub use id::{
    AbilityId, ActionId, CommandId, DecisionId, EffectDefinitionId, EffectInstanceId, EncounterId,
    EnemyDefinitionId, EventId, HitId, HitPlanDefinitionId, ModifierDefinitionId,
    ModifierInstanceId, NativeHandlerId, OperationId, PhaseId, ProgramId, RuleBundleId, RuleId,
    RuleInstanceId, SelectorId, ShieldInstanceId, SourceDefinitionId, SpawnSequence,
    StateSlotDefinitionId, TimelineActorId, TriggerId, UnitDefinitionId, UnitId, WaveInstanceId,
    ZeroIdError,
};
pub use numeric::domain::{
    ActionGauge, DamageAmount, Energy, HealingAmount, Hp, Probability, RawToughness, ShieldAmount,
    Speed, StatValue,
};
pub use numeric::rounding::{NumericError, Rounding};
pub use numeric::scalar::{Ratio, Scalar};

// Deliberate stable battle facade over private aggregate/store modules.
pub use action::model::ActionOrigin;
pub use actor::model::{LifeState, PresenceState};
pub use battle::aggregate::Battle;
pub use battle::build::{BattleBuildError, BattleBuildErrorKind};
pub use battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};
pub use battle::model::{BattlePhase, Resolution};
pub use battle::spec::{
    BattleSeed, BattleSpec, BattleSpecDigest, BattleSpecError, CombatantSpecDigest,
    CombatantSpecError, ConcedePolicy, FormationIndex, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, TeamResourceSpec, TeamSide, UnitLevel,
};
pub use battle::view::{
    ActiveTurnView, BattleIdentityView, BattleView, EncounterView, FormationView,
    InterruptWindowView, TeamView, TimelineActorView, UnitView,
};
pub use codec::BattleStateHash;
pub use command::model::{
    Command, CommandError, CommandErrorKind, DecisionKind, DecisionOwner, DecisionPoint,
};
pub use event::cause::{Cause, CauseActor};
pub use event::model::{
    ActionEventData, BattleEvent, BattleEventData, BattleEventKind, DamageEventData,
    DecisionEventData, FaultEventData, HealEventData, HitEventData, PhaseEventData,
    ResourceEventData, TurnEventData, UnitEventData, WaveEventData,
};
pub use timeline::state::InterruptWindowKind;

/// Compatibility identifier for authoritative numeric representation and rounding.
pub const NUMERIC_POLICY_REVISION: &str = "fixed-i64-6dp-v1";
/// Compatibility identifier for canonical battle-state hashing.
pub const STATE_HASH_REVISION: &str = "sha256-v1";
