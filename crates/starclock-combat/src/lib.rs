//! Deterministic, engine-agnostic ownership boundary for exactly one battle.
//!
//! This crate accepts only generic resolved combat input. Build selections,
//! generated data rows, activities, modes, controllers, replay transport and
//! engines remain outside this dependency root.

#![forbid(unsafe_code)]

pub mod catalog;
mod id;
mod numeric;

// This is the deliberate small crate facade. The defining modules remain
// private so representation/backend details have one canonical external path.
pub use id::{
    AbilityId, ActionId, EffectDefinitionId, EffectInstanceId, EncounterId, EnemyDefinitionId,
    EventId, HitId, HitPlanDefinitionId, ModifierDefinitionId, ModifierInstanceId, NativeHandlerId,
    OperationId, PhaseId, ProgramId, RuleBundleId, RuleId, RuleInstanceId, SelectorId,
    ShieldInstanceId, SourceDefinitionId, SpawnSequence, StateSlotDefinitionId, TimelineActorId,
    TriggerId, UnitDefinitionId, UnitId, WaveInstanceId, ZeroIdError,
};
pub use numeric::domain::{
    ActionGauge, DamageAmount, HealingAmount, Hp, Probability, RawToughness, ShieldAmount, Speed,
    StatValue,
};
pub use numeric::rounding::{NumericError, Rounding};
pub use numeric::scalar::{Ratio, Scalar};

/// Compatibility identifier for authoritative numeric representation and rounding.
pub const NUMERIC_POLICY_REVISION: &str = "fixed-i64-6dp-v1";
