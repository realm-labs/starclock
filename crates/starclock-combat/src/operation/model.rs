use crate::catalog::action::HitCritPolicy;
use crate::catalog::action::QueueActionDefinition;
use crate::catalog::action::ReactionBoundary;
use crate::catalog::action::SkillPointPaymentPolicy;
use crate::catalog::action::TeamResourceChangeDefinition;
use crate::catalog::action::WeaknessApplicationDefinition;
use crate::formula::model::CombatElement;
use crate::formula::toughness::SuperBreakDefinition;
use crate::rng::types::DrawPurpose;
use crate::rule::model::RuleSlotMutationDefinition;
use crate::{
    AbilityId, ActionOrigin, CountdownDefinition, DotDetonationDefinition,
    EffectApplicationDefinition, EffectChancePolicy, EffectDefinitionId, EffectRemovalDefinition,
    EffectRuntimeDefinition, EnemyPhaseId, LinkedUnitDefinition, PresenceState, RawToughness,
    ReviveDefinition, RuleId, RuleInstanceId, SourceDefinitionId, ToughnessReductionDefinition,
    TransformationDefinition, TriggerId, UnitId,
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
    AddWeaknessFromAlliedElements(AddWeaknessFromAlliedElementsOp),
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
            Self::AddWeaknessFromAlliedElements(operation) => operation.id,
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
    pub(crate) phase: EnemyPhaseId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EncounterLifecycleOp {
    pub(crate) id: OperationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SummonLinkedOp {
    pub(crate) id: OperationId,
    pub(crate) owners: Box<[UnitId]>,
    pub(crate) definition: LinkedUnitDefinition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CreateCountdownOp {
    pub(crate) id: OperationId,
    pub(crate) owner: UnitId,
    pub(crate) definition: CountdownDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangePresenceOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) presence: PresenceState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransformOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: TransformationDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviveOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: ReviveDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnitLifecycleOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueueActionOp {
    pub(crate) id: OperationId,
    pub(crate) definition: QueueActionDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueueRuleActionOp {
    pub(crate) id: OperationId,
    pub(crate) actors: Box<[UnitId]>,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) owner: UnitId,
    pub(crate) ability: AbilityId,
    pub(crate) origin: ActionOrigin,
    pub(crate) priority: i16,
    pub(crate) boundary: ReactionBoundary,
    pub(crate) payment: Option<SkillPointPaymentPolicy>,
    pub(crate) source: SourceDefinitionId,
    pub(crate) rule: Option<RuleId>,
    pub(crate) instance: Option<RuleInstanceId>,
    pub(crate) trigger: Option<TriggerId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModifyStateSlotOp {
    pub(crate) id: OperationId,
    pub(crate) owner: UnitId,
    pub(crate) instance: Option<RuleInstanceId>,
    pub(crate) definition: RuleSlotMutationDefinition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ModifyTeamResourceOp {
    pub(crate) id: OperationId,
    pub(crate) actor: UnitId,
    pub(crate) definition: TeamResourceChangeDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyEffectOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: EffectApplicationDefinition,
    pub(crate) rng_purpose: Option<DrawPurpose>,
    pub(crate) resolved_chances: Option<Box<[EffectChancePolicy]>>,
    pub(crate) resolved_runtime: Option<Box<[EffectRuntimeDefinition]>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoveEffectsOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: EffectRemovalDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DetonateDotsOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: DotDetonationDefinition,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HitOperationScratch {
    pub(crate) effective_reductions: BTreeMap<UnitId, RawToughness>,
    pub(crate) critical_by_target: BTreeMap<UnitId, bool>,
    pub(crate) shared_critical_draw: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AddWeaknessOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: WeaknessApplicationDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AddWeaknessFromAlliedElementsOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) count: u8,
    pub(crate) duration_turns: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReduceToughnessOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: ToughnessReductionDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ForceBreakOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) element: CombatElement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SuperBreakOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: SuperBreakDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DamageOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) formula: OrdinaryDamageDefinition,
    pub(crate) element: Option<CombatElement>,
    pub(crate) crit_policy: HitCritPolicy,
    pub(crate) apply_source_modifiers: bool,
    /// Replaces inherited action tags with Attack + Ultimate for formula
    /// modifier queries without creating a new action lifecycle.
    pub(crate) ultimate_semantics: bool,
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
    pub(crate) source_effect: Option<EffectDefinitionId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemoveShieldsOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) effect: EffectDefinitionId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConsumeHpOp {
    pub(crate) id: OperationId,
    pub(crate) targets: Box<[UnitId]>,
    pub(crate) definition: HpConsumptionDefinition,
}
