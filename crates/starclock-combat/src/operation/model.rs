use crate::{
    UnitId,
    catalog::action::{
        HealingDefinition, HpConsumptionDefinition, OrdinaryDamageDefinition, ShieldDefinition,
    },
    id::OperationId,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Operation {
    Damage(DamageOp),
    Heal(HealOp),
    Shield(ShieldOp),
    RemoveShields(RemoveShieldsOp),
    ConsumeHp(ConsumeHpOp),
    AddWeakness(AddWeaknessOp),
    ReduceToughness(ReduceToughnessOp),
    ForceBreak(ForceBreakOp),
    SuperBreak(SuperBreakOp),
    ApplyEffect(ApplyEffectOp),
    RemoveEffects(RemoveEffectsOp),
    DetonateDots(DetonateDotsOp),
    ModifyStateSlot(ModifyStateSlotOp),
    ModifyTeamResource(ModifyTeamResourceOp),
    QueueAction(QueueActionOp),
    QueueRuleAction(QueueRuleActionOp),
    SummonLinked(SummonLinkedOp),
    CreateCountdown(CreateCountdownOp),
    ChangePresence(ChangePresenceOp),
    Transform(TransformOp),
    EndTransformation(UnitLifecycleOp),
    Revive(ReviveOp),
    DespawnLinked(UnitLifecycleOp),
    RequestWaveTransition(EncounterLifecycleOp),
    TransitionEnemyPhase(EnemyPhaseOp),
}

impl Operation {
    pub(crate) const fn id(&self) -> OperationId {
        match self {
            Self::Damage(operation) => operation.id,
            Self::Heal(operation) => operation.id,
            Self::Shield(operation) => operation.id,
            Self::RemoveShields(operation) => operation.id,
            Self::ConsumeHp(operation) => operation.id,
            Self::AddWeakness(operation) => operation.id,
            Self::ReduceToughness(operation) => operation.id,
            Self::ForceBreak(operation) => operation.id,
            Self::SuperBreak(operation) => operation.id,
            Self::ApplyEffect(operation) => operation.id,
            Self::RemoveEffects(operation) => operation.id,
            Self::DetonateDots(operation) => operation.id,
            Self::ModifyStateSlot(operation) => operation.id,
            Self::ModifyTeamResource(operation) => operation.id,
            Self::QueueAction(operation) => operation.id,
            Self::QueueRuleAction(operation) => operation.id,
            Self::SummonLinked(operation) => operation.id,
            Self::CreateCountdown(operation) => operation.id,
            Self::ChangePresence(operation) => operation.id,
            Self::Transform(operation) => operation.id,
            Self::EndTransformation(operation) => operation.id,
            Self::Revive(operation) => operation.id,
            Self::DespawnLinked(operation) => operation.id,
            Self::RequestWaveTransition(operation) => operation.id,
            Self::TransitionEnemyPhase(operation) => operation.id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnemyPhaseOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) phase: crate::EnemyPhaseId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EncounterLifecycleOp {
    pub(crate) id: OperationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SummonLinkedOp {
    pub(crate) id: OperationId,
    pub(crate) owners: Box<[UnitId]>,
    pub(crate) definition: crate::LinkedUnitDefinition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CreateCountdownOp {
    pub(crate) id: OperationId,
    pub(crate) owner: UnitId,
    pub(crate) definition: crate::CountdownDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangePresenceOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) presence: crate::PresenceState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransformOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::TransformationDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviveOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::ReviveDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnitLifecycleOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueueActionOp {
    pub(crate) id: OperationId,
    pub(crate) definition: crate::catalog::action::QueueActionDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueueRuleActionOp {
    pub(crate) id: OperationId,
    pub(crate) actors: Box<[UnitId]>,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) owner: UnitId,
    pub(crate) ability: crate::AbilityId,
    pub(crate) origin: crate::ActionOrigin,
    pub(crate) priority: i16,
    pub(crate) boundary: crate::catalog::action::ReactionBoundary,
    pub(crate) payment: Option<crate::catalog::action::SkillPointPaymentPolicy>,
    pub(crate) source: crate::SourceDefinitionId,
    pub(crate) rule: crate::RuleId,
    pub(crate) instance: crate::RuleInstanceId,
    pub(crate) trigger: crate::TriggerId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModifyStateSlotOp {
    pub(crate) id: OperationId,
    pub(crate) owner: UnitId,
    pub(crate) instance: Option<crate::RuleInstanceId>,
    pub(crate) definition: crate::rule::model::RuleSlotMutationDefinition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ModifyTeamResourceOp {
    pub(crate) id: OperationId,
    pub(crate) actor: UnitId,
    pub(crate) definition: crate::catalog::action::TeamResourceChangeDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyEffectOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::EffectApplicationDefinition,
    pub(crate) rng_purpose: Option<crate::rng::types::DrawPurpose>,
    pub(crate) resolved_chances: Option<Box<[crate::EffectChancePolicy]>>,
    pub(crate) resolved_runtime: Option<Box<[crate::EffectRuntimeDefinition]>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoveEffectsOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::EffectRemovalDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DetonateDotsOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::DotDetonationDefinition,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HitOperationScratch {
    pub(crate) effective_reductions: BTreeMap<UnitId, crate::RawToughness>,
    pub(crate) critical_by_target: BTreeMap<UnitId, bool>,
    pub(crate) shared_critical_draw: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AddWeaknessOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::catalog::action::WeaknessApplicationDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReduceToughnessOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::ToughnessReductionDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ForceBreakOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) element: crate::formula::model::CombatElement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SuperBreakOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: crate::formula::toughness::SuperBreakDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DamageOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) formula: OrdinaryDamageDefinition,
    pub(crate) element: Option<crate::formula::model::CombatElement>,
    pub(crate) crit_policy: crate::catalog::action::HitCritPolicy,
    pub(crate) minimum_hp: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HealOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) formula: HealingDefinition,
    pub(crate) apply_formula_modifiers: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShieldOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) formula: ShieldDefinition,
    pub(crate) source_effect: Option<crate::EffectDefinitionId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoveShieldsOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) effect: crate::EffectDefinitionId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConsumeHpOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: HpConsumptionDefinition,
}
