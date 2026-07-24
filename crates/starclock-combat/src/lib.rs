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
mod effect;
mod event;
pub mod formula;
mod id;
pub mod modifier;
mod numeric;
mod operation;
mod reaction;
mod resolver;
mod resource;
pub mod rng;
pub mod rule;
mod target;
mod timeline;
mod toughness;

// This is the deliberate small crate facade. The defining modules remain
// private so representation/backend details have one canonical external path.
pub use id::{
    AbilityId, ActionId, AiCandidateId, AiGraphId, AiStateId, AiTransitionId, CommandId,
    DecisionId, EffectDefinitionId, EffectInstanceId, EncounterId, EncounterWaveId,
    EnemyDefinitionId, EnemyPhaseId, EventId, HitId, HitPlanDefinitionId, ModifierDefinitionId,
    ModifierInstanceId, ModifierStackingGroupId, NativeHandlerId, OperationId, PhaseId, ProgramId,
    RuleBundleId, RuleId, RuleInstanceId, SelectorId, ShieldInstanceId, SourceDefinitionId,
    SpawnSequence, StateSlotDefinitionId, TimelineActorId, TriggerId, UnitDefinitionId, UnitId,
    WaveInstanceId, ZeroIdError,
};
pub use numeric::domain::{
    ActionGauge, DamageAmount, Energy, HealingAmount, Hp, Probability, RawToughness, ShieldAmount,
    Speed, StatValue,
};
pub use numeric::rounding::{NumericError, Rounding};
pub use numeric::scalar::{Ratio, Scalar};

// Deliberate stable battle facade over private aggregate/store modules.
pub use action::model::ActionOrigin;
pub use actor::link::{
    CountdownCatalogDefinition, CountdownDefinition, LinkedEntity, LinkedEntityKind,
    LinkedOwnerScaling, LinkedStatScaling, LinkedUnitCatalogDefinition, LinkedUnitDefinition,
    OwnerLinkPolicy, ReviveDefinition, ReviveGaugePolicy, TransformEndPolicy,
    TransformationDefinition, WaveLinkPolicy,
};
pub use actor::model::{LifeState, PresenceState};
pub use battle::aggregate::Battle;
pub use battle::build::{BattleBuildError, BattleBuildErrorKind};
pub use battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};
pub use battle::model::{BattlePhase, Resolution};
pub use battle::spec::{
    AssemblyDigest, BattleSeed, BattleSpec, BattleSpecDigest, BattleSpecError, CombatInputDigest,
    CombatantSpecDigest, CombatantSpecError, ConcedePolicy, FormationIndex, KeyedTeamResourceSpec,
    ParticipantInitialState, ParticipantSource, ParticipantSpec, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, ResolvedModifierBinding, TeamResourceSpec, TeamResourceWavePolicy,
    TeamSide, UnitLevel,
};
pub use battle::view::{
    ActiveTurnView, BattleIdentityView, BattleView, BreakEffectView, EffectView, EncounterView,
    FormationView, InterruptWindowView, LinkView, ModifierInstanceView, RuleInstanceView,
    ShieldView, TeamView, TimelineActorView, ToughnessLayerView, UnitView,
};
pub use codec::BattleStateHash;
pub use command::model::{
    Command, CommandError, CommandErrorKind, DecisionKind, DecisionOwner, DecisionPoint,
};
pub use effect::model::{
    ControlledAction, DispelCategory, DotDefinition, DotDetonationDefinition, DurationClock,
    EffectApplicationDefinition, EffectCategory, EffectChancePolicy, EffectRemovalDefinition,
    EffectRuntimeDefinition, EffectRuntimeTemplate, EffectSnapshotPolicy, EffectStackPolicy,
    EffectTeardownPolicy, EffectTickPhase,
};
pub use event::cause::{Cause, CauseActor};
pub use event::model::{
    ActionEventData, BattleEvent, BattleEventData, BattleEventKind, BreakDamageEventData,
    BreakDamageKind, DamageEventData, DamageKind, DecisionEventData, EffectEventData,
    EnemyPhaseEventData, FaultEventData, HealEventData, HitEventData, HpConsumptionEventData,
    PhaseEventData, ResourceEventData, RuleSignalEventData, RuleStateEventData, ShieldEventData,
    SkillPointPayer, ToughnessEventData, TurnEventData, UnitEventData, WaveEventData,
};
pub use timeline::state::InterruptWindowKind;
pub use toughness::model::{
    BreakCreditPolicy, ToughnessLayerKind, ToughnessLayerSpec, ToughnessReductionDefinition,
    ToughnessWeaknessPolicy,
};

/// Compatibility identifier for authoritative numeric representation and rounding.
pub const NUMERIC_POLICY_REVISION: &str = "fixed-i64-6dp-v1";
/// Compatibility identifier for canonical battle-state hashing.
pub const STATE_HASH_REVISION: &str = "sha256-v4";
/// Compatibility identifier for canonical combat-input encoding and hashing.
pub const COMBAT_INPUT_CODEC_REVISION: &str = "combat-input-v1";
