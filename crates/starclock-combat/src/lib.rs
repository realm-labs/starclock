//! Deterministic, engine-agnostic ownership boundary for exactly one battle.
//!
//! This crate accepts only generic resolved combat input. Build selections,
//! generated data rows, activities, modes, controllers, replay transport and
//! engines remain outside this dependency root.

#![forbid(unsafe_code)]

mod actor;
mod battle;
pub mod catalog;
mod command;
mod id;
mod numeric;
pub mod rng;

// This is the deliberate small crate facade. The defining modules remain
// private so representation/backend details have one canonical external path.
pub use id::{
    AbilityId, ActionId, DecisionId, EffectDefinitionId, EffectInstanceId, EncounterId,
    EnemyDefinitionId, EventId, HitId, HitPlanDefinitionId, ModifierDefinitionId,
    ModifierInstanceId, NativeHandlerId, OperationId, PhaseId, ProgramId, RuleBundleId, RuleId,
    RuleInstanceId, SelectorId, ShieldInstanceId, SourceDefinitionId, SpawnSequence,
    StateSlotDefinitionId, TimelineActorId, TriggerId, UnitDefinitionId, UnitId, WaveInstanceId,
    ZeroIdError,
};
pub use numeric::domain::{
    ActionGauge, DamageAmount, HealingAmount, Hp, Probability, RawToughness, ShieldAmount, Speed,
    StatValue,
};
pub use numeric::rounding::{NumericError, Rounding};
pub use numeric::scalar::{Ratio, Scalar};

// Deliberate stable battle facade over private aggregate/store modules.
pub use actor::model::{LifeState, PresenceState};
pub use battle::aggregate::Battle;
pub use battle::build::{BattleBuildError, BattleBuildErrorKind};
pub use battle::model::{BattlePhase, Resolution};
pub use battle::spec::{
    BattleSeed, BattleSpec, BattleSpecDigest, BattleSpecError, CombatantSpecDigest,
    CombatantSpecError, ConcedePolicy, FormationIndex, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, TeamResourceSpec, TeamSide, UnitLevel,
};
pub use battle::view::{
    BattleIdentityView, BattleView, EncounterView, FormationView, TeamView, TimelineActorView,
    UnitView,
};
pub use command::model::{
    Command, CommandError, CommandErrorKind, DecisionKind, DecisionOwner, DecisionPoint,
};

/// Compatibility identifier for authoritative numeric representation and rounding.
pub const NUMERIC_POLICY_REVISION: &str = "fixed-i64-6dp-v1";
