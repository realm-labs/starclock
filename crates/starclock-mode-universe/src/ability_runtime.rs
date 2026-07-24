//! Closed, deterministic execution model for Standard Universe Ability Tree effects.

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivitySlotId, ActivityValue as ActivityStateValue,
};

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::AbilityTreeNodeId,
    path::ExactParameter,
    progression::{AbilityEffectClass, AbilityOperation, AbilityValueUnit},
};

pub const ABILITY_RUNTIME_REVISION: &str = "standard-universe-ability-runtime-v1";
const SIX_DECIMAL_SCALE: i64 = 1_000_000;

/// Generic execution boundary at which an Ability Tree projection is requested.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityBoundary {
    RunStart = 0,
    BattleStart = 1,
    EnterEliteOrBossDomain = 2,
    AfterBattle = 3,
}

/// Whether the caller is compiling Activity-owned or Battle-owned values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityProjectionScope {
    Run = 0,
    Battle = 1,
}

/// Complete deterministic inputs for evaluating authored Ability Tree conditions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityExecutionContext {
    scope: AbilityProjectionScope,
    boundary: AbilityBoundary,
    chosen_path_blessings: u8,
    first_battle_won: bool,
}

impl AbilityExecutionContext {
    #[must_use]
    pub const fn new(
        scope: AbilityProjectionScope,
        boundary: AbilityBoundary,
        chosen_path_blessings: u8,
        first_battle_won: bool,
    ) -> Self {
        Self {
            scope,
            boundary,
            chosen_path_blessings,
            first_battle_won,
        }
    }

    #[must_use]
    pub const fn run_start() -> Self {
        Self::new(
            AbilityProjectionScope::Run,
            AbilityBoundary::RunStart,
            0,
            false,
        )
    }

    #[must_use]
    pub const fn scope(self) -> AbilityProjectionScope {
        self.scope
    }

    #[must_use]
    pub const fn boundary(self) -> AbilityBoundary {
        self.boundary
    }
}

/// Closed set of Version 4.4 Ability Tree targets.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityTarget {
    RunPathSelection = 0,
    BattlePathResonance = 1,
    PartyAttackFlat = 2,
    PartyDefenseFlat = 3,
    PartyMaximumHpFlat = 4,
    RunPathResonance = 5,
    PathResonanceDamageRatio = 6,
    ServiceReviver = 7,
    ServiceReviverRestoredHpRatio = 8,
    BlessingChoiceResetCount = 9,
    EnhancedTrailblazeBonus = 10,
    InitialCosmicFragments = 11,
    FirstBattleBlessingCount = 12,
    PartyCritRateRatio = 13,
    PathResonanceInitialEnergy = 14,
    PartySpeedRatio = 15,
    PartyInitialEnergy = 16,
    PartyEnergy = 17,
    PartyCritDamageRatio = 18,
    PartyDamageTakenReductionRatio = 19,
    PartyEffectHitRateRatio = 20,
    RunConsumableUse = 21,
}

impl AbilityTarget {
    const ALL: [Self; 22] = [
        Self::RunPathSelection,
        Self::BattlePathResonance,
        Self::PartyAttackFlat,
        Self::PartyDefenseFlat,
        Self::PartyMaximumHpFlat,
        Self::RunPathResonance,
        Self::PathResonanceDamageRatio,
        Self::ServiceReviver,
        Self::ServiceReviverRestoredHpRatio,
        Self::BlessingChoiceResetCount,
        Self::EnhancedTrailblazeBonus,
        Self::InitialCosmicFragments,
        Self::FirstBattleBlessingCount,
        Self::PartyCritRateRatio,
        Self::PathResonanceInitialEnergy,
        Self::PartySpeedRatio,
        Self::PartyInitialEnergy,
        Self::PartyEnergy,
        Self::PartyCritDamageRatio,
        Self::PartyDamageTakenReductionRatio,
        Self::PartyEffectHitRateRatio,
        Self::RunConsumableUse,
    ];

    #[must_use]
    pub const fn activity_key(self) -> u64 {
        self as u64 + 1
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "run.path_selection" => Self::RunPathSelection,
            "battle.path_resonance" => Self::BattlePathResonance,
            "party.atk_flat" => Self::PartyAttackFlat,
            "party.def_flat" => Self::PartyDefenseFlat,
            "party.max_hp_flat" => Self::PartyMaximumHpFlat,
            "run.path_resonance" => Self::RunPathResonance,
            "path_resonance.damage_ratio" => Self::PathResonanceDamageRatio,
            "service.reviver" => Self::ServiceReviver,
            "service.reviver.restored_hp_ratio" => Self::ServiceReviverRestoredHpRatio,
            "reward.blessing_choice.reset_count" => Self::BlessingChoiceResetCount,
            "run.trailblaze_bonus.enhanced" => Self::EnhancedTrailblazeBonus,
            "universe.currency.cosmic-fragments.initial" => Self::InitialCosmicFragments,
            "reward.first_battle.blessing_count" => Self::FirstBattleBlessingCount,
            "party.crit_rate_ratio" => Self::PartyCritRateRatio,
            "path_resonance.initial_energy" => Self::PathResonanceInitialEnergy,
            "party.speed_ratio" => Self::PartySpeedRatio,
            "party.initial_energy" => Self::PartyInitialEnergy,
            "party.energy" => Self::PartyEnergy,
            "party.crit_damage_ratio" => Self::PartyCritDamageRatio,
            "party.damage_taken_reduction_ratio" => Self::PartyDamageTakenReductionRatio,
            "party.effect_hit_rate_ratio" => Self::PartyEffectHitRateRatio,
            "run.consumable_use" => Self::RunConsumableUse,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn stable_key(self) -> &'static str {
        match self {
            Self::RunPathSelection => "run.path_selection",
            Self::BattlePathResonance => "battle.path_resonance",
            Self::PartyAttackFlat => "party.atk_flat",
            Self::PartyDefenseFlat => "party.def_flat",
            Self::PartyMaximumHpFlat => "party.max_hp_flat",
            Self::RunPathResonance => "run.path_resonance",
            Self::PathResonanceDamageRatio => "path_resonance.damage_ratio",
            Self::ServiceReviver => "service.reviver",
            Self::ServiceReviverRestoredHpRatio => "service.reviver.restored_hp_ratio",
            Self::BlessingChoiceResetCount => "reward.blessing_choice.reset_count",
            Self::EnhancedTrailblazeBonus => "run.trailblaze_bonus.enhanced",
            Self::InitialCosmicFragments => "universe.currency.cosmic-fragments.initial",
            Self::FirstBattleBlessingCount => "reward.first_battle.blessing_count",
            Self::PartyCritRateRatio => "party.crit_rate_ratio",
            Self::PathResonanceInitialEnergy => "path_resonance.initial_energy",
            Self::PartySpeedRatio => "party.speed_ratio",
            Self::PartyInitialEnergy => "party.initial_energy",
            Self::PartyEnergy => "party.energy",
            Self::PartyCritDamageRatio => "party.crit_damage_ratio",
            Self::PartyDamageTakenReductionRatio => "party.damage_taken_reduction_ratio",
            Self::PartyEffectHitRateRatio => "party.effect_hit_rate_ratio",
            Self::RunConsumableUse => "run.consumable_use",
        }
    }

    const fn expected_unit(self) -> AbilityValueUnit {
        match self {
            Self::RunPathSelection
            | Self::BattlePathResonance
            | Self::ServiceReviver
            | Self::EnhancedTrailblazeBonus
            | Self::RunConsumableUse => AbilityValueUnit::Boolean,
            Self::RunPathResonance
            | Self::BlessingChoiceResetCount
            | Self::InitialCosmicFragments
            | Self::FirstBattleBlessingCount => AbilityValueUnit::Count,
            Self::PartyAttackFlat
            | Self::PartyDefenseFlat
            | Self::PartyMaximumHpFlat
            | Self::PathResonanceInitialEnergy => AbilityValueUnit::Flat,
            Self::PathResonanceDamageRatio
            | Self::ServiceReviverRestoredHpRatio
            | Self::PartyCritRateRatio
            | Self::PartySpeedRatio
            | Self::PartyInitialEnergy
            | Self::PartyEnergy
            | Self::PartyCritDamageRatio
            | Self::PartyDamageTakenReductionRatio
            | Self::PartyEffectHitRateRatio => AbilityValueUnit::Ratio,
        }
    }

    const fn scope(self) -> AbilityProjectionScope {
        match self {
            Self::RunPathSelection
            | Self::RunPathResonance
            | Self::ServiceReviver
            | Self::ServiceReviverRestoredHpRatio
            | Self::BlessingChoiceResetCount
            | Self::EnhancedTrailblazeBonus
            | Self::InitialCosmicFragments
            | Self::FirstBattleBlessingCount
            | Self::RunConsumableUse => AbilityProjectionScope::Run,
            Self::BattlePathResonance
            | Self::PartyAttackFlat
            | Self::PartyDefenseFlat
            | Self::PartyMaximumHpFlat
            | Self::PathResonanceDamageRatio
            | Self::PartyCritRateRatio
            | Self::PathResonanceInitialEnergy
            | Self::PartySpeedRatio
            | Self::PartyInitialEnergy
            | Self::PartyEnergy
            | Self::PartyCritDamageRatio
            | Self::PartyDamageTakenReductionRatio
            | Self::PartyEffectHitRateRatio => AbilityProjectionScope::Battle,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum AbilityTrigger {
    Always,
    ChosenPathBlessingsAtLeast(u8),
    FirstBattleWon,
    BattleStart,
    EnterEliteOrBossDomain,
}

impl AbilityTrigger {
    fn parse(value: Option<&str>) -> Option<Self> {
        Some(match value.unwrap_or("") {
            "" => Self::Always,
            "chosen_path_blessing_count>=3" => Self::ChosenPathBlessingsAtLeast(3),
            "chosen_path_blessing_count>=6" => Self::ChosenPathBlessingsAtLeast(6),
            "chosen_path_blessing_count>=10" => Self::ChosenPathBlessingsAtLeast(10),
            "chosen_path_blessing_count>=14" => Self::ChosenPathBlessingsAtLeast(14),
            "first_battle_won" => Self::FirstBattleWon,
            "battle_start" => Self::BattleStart,
            "enter_elite_or_boss_domain" => Self::EnterEliteOrBossDomain,
            _ => return None,
        })
    }

    const fn matches(self, context: AbilityExecutionContext) -> bool {
        match self {
            Self::Always => true,
            Self::ChosenPathBlessingsAtLeast(required) => context.chosen_path_blessings >= required,
            Self::FirstBattleWon => context.first_battle_won,
            Self::BattleStart => matches!(context.boundary, AbilityBoundary::BattleStart),
            Self::EnterEliteOrBossDomain => {
                matches!(context.boundary, AbilityBoundary::EnterEliteOrBossDomain)
            }
        }
    }
}

/// Signed six-decimal domain value; no floating-point backend is exposed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AbilityValue(i64);

impl AbilityValue {
    pub const ZERO: Self = Self(0);

    fn from_exact(value: ExactParameter) -> Result<Self, AbilityRuntimeError> {
        if value.scale() > 6 {
            return Err(AbilityRuntimeError::InvalidValue);
        }
        let factor = 10_i64
            .checked_pow(u32::from(6 - value.scale()))
            .ok_or(AbilityRuntimeError::Overflow)?;
        value
            .coefficient()
            .checked_mul(factor)
            .map(Self)
            .ok_or(AbilityRuntimeError::Overflow)
    }

    fn checked_add(self, other: Self) -> Result<Self, AbilityRuntimeError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(AbilityRuntimeError::Overflow)
    }

    #[must_use]
    pub const fn raw_six_decimal(self) -> i64 {
        self.0
    }

    #[must_use]
    pub const fn integral(self) -> Option<i64> {
        if self.0 % SIX_DECIMAL_SCALE == 0 {
            Some(self.0 / SIX_DECIMAL_SCALE)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledAbilityEffect {
    source: AbilityTreeNodeId,
    class: AbilityEffectClass,
    operation: AbilityOperation,
    target: AbilityTarget,
    value: AbilityValue,
    unit: AbilityValueUnit,
    trigger: AbilityTrigger,
}

/// One source-attributed effect that participated in a projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppliedAbilityEffect {
    source: AbilityTreeNodeId,
    operation: AbilityOperation,
    target: AbilityTarget,
    value: AbilityValue,
}

impl AppliedAbilityEffect {
    #[must_use]
    pub const fn source(self) -> AbilityTreeNodeId {
        self.source
    }
    #[must_use]
    pub const fn operation(self) -> AbilityOperation {
        self.operation
    }
    #[must_use]
    pub const fn target(self) -> AbilityTarget {
        self.target
    }
    #[must_use]
    pub const fn value(self) -> AbilityValue {
        self.value
    }
}

/// Final value of one generic target after ordered Ability operations execute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectedAbilityValue {
    target: AbilityTarget,
    unit: AbilityValueUnit,
    value: AbilityValue,
}

impl ProjectedAbilityValue {
    #[must_use]
    pub const fn target(self) -> AbilityTarget {
        self.target
    }
    #[must_use]
    pub const fn unit(self) -> AbilityValueUnit {
        self.unit
    }
    #[must_use]
    pub const fn value(self) -> AbilityValue {
        self.value
    }
}

/// Canonical result of executing selected Ability Tree effects at one boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityRuntimeProjection {
    context: AbilityExecutionContext,
    values: Box<[ProjectedAbilityValue]>,
    applied: Box<[AppliedAbilityEffect]>,
    digest: [u8; 32],
}

/// One projection plus the ordinary Activity operations that replace the
/// authoritative values for its scope at a declared boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActivityProjection {
    projection: AbilityRuntimeProjection,
    operations: Box<[ActivityOperation]>,
}

impl AbilityActivityProjection {
    #[must_use]
    pub const fn projection(&self) -> &AbilityRuntimeProjection {
        &self.projection
    }

    #[must_use]
    pub fn operations(&self) -> &[ActivityOperation] {
        &self.operations
    }
}

impl AbilityRuntimeProjection {
    #[must_use]
    pub const fn context(&self) -> AbilityExecutionContext {
        self.context
    }
    #[must_use]
    pub fn values(&self) -> &[ProjectedAbilityValue] {
        &self.values
    }
    #[must_use]
    pub fn applied_effects(&self) -> &[AppliedAbilityEffect] {
        &self.applied
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn value(&self, target: AbilityTarget) -> Option<AbilityValue> {
        self.values
            .binary_search_by_key(&target, |entry| entry.target)
            .ok()
            .map(|index| self.values[index].value)
    }
}

/// Immutable compiler/executor for all released Version 4.4 Ability Tree effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityRuntimeCatalog {
    effects: Box<[CompiledAbilityEffect]>,
    digest: [u8; 32],
}

impl AbilityRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, AbilityRuntimeError> {
        let mut effects = Vec::new();
        for node in catalog.ability_tree_nodes() {
            for effect in node.effects() {
                let target = AbilityTarget::parse(effect.target_key())
                    .ok_or(AbilityRuntimeError::UnknownTarget(node.id()))?;
                let trigger = AbilityTrigger::parse(effect.condition())
                    .ok_or(AbilityRuntimeError::UnknownCondition(node.id()))?;
                validate_shape(effect.operation(), target, effect.unit(), effect.value())?;
                if !class_allows(node.effect_class(), target.scope()) {
                    return Err(AbilityRuntimeError::ClassMismatch(node.id()));
                }
                effects.push(CompiledAbilityEffect {
                    source: node.id(),
                    class: node.effect_class(),
                    operation: effect.operation(),
                    target,
                    value: AbilityValue::from_exact(effect.value())?,
                    unit: effect.unit(),
                    trigger,
                });
            }
        }
        if catalog.ability_tree_nodes().len() != 42 || effects.len() != 50 {
            return Err(AbilityRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&effects);
        Ok(Self {
            effects: effects.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub const fn effect_count(&self) -> usize {
        self.effects.len()
    }

    pub fn project(
        &self,
        selected: &[AbilityTreeNodeId],
        context: AbilityExecutionContext,
    ) -> Result<AbilityRuntimeProjection, AbilityRuntimeError> {
        if selected.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(AbilityRuntimeError::NonCanonicalSelection);
        }
        let mut values: Vec<ProjectedAbilityValue> = Vec::new();
        let mut applied = Vec::new();
        for effect in self.effects.iter().filter(|effect| {
            selected.binary_search(&effect.source).is_ok()
                && effect.target.scope() == context.scope
                && effect.trigger.matches(context)
        }) {
            let entry = match values.binary_search_by_key(&effect.target, |entry| entry.target) {
                Ok(index) => &mut values[index],
                Err(index) => {
                    values.insert(
                        index,
                        ProjectedAbilityValue {
                            target: effect.target,
                            unit: effect.unit,
                            value: AbilityValue::ZERO,
                        },
                    );
                    &mut values[index]
                }
            };
            entry.value = execute(effect.operation, entry.value, effect.value)?;
            applied.push(AppliedAbilityEffect {
                source: effect.source,
                operation: effect.operation,
                target: effect.target,
                value: effect.value,
            });
        }
        let digest = projection_digest(context, &values, &applied);
        Ok(AbilityRuntimeProjection {
            context,
            values: values.into_boxed_slice(),
            applied: applied.into_boxed_slice(),
            digest,
        })
    }

    pub fn project_activity_operations(
        &self,
        selected: &[AbilityTreeNodeId],
        context: AbilityExecutionContext,
        slot: ActivitySlotId,
    ) -> Result<AbilityActivityProjection, AbilityRuntimeError> {
        let projection = self.project(selected, context)?;
        let operations = AbilityTarget::ALL
            .into_iter()
            .filter(|target| target.scope() == context.scope())
            .map(|target| {
                let value = projection.value(target).unwrap_or(AbilityValue::ZERO);
                ActivityOperation::AddCounter {
                    slot,
                    key: target.activity_key(),
                    delta: ActivityExpression::Subtract(
                        Box::new(ActivityExpression::Literal(
                            ActivityStateValue::BoundedInteger(value.raw_six_decimal()),
                        )),
                        Box::new(ActivityExpression::CounterValue {
                            slot,
                            key: target.activity_key(),
                        }),
                    ),
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(AbilityActivityProjection {
            projection,
            operations,
        })
    }
}

fn class_allows(class: AbilityEffectClass, scope: AbilityProjectionScope) -> bool {
    matches!(
        (class, scope),
        (AbilityEffectClass::Run, AbilityProjectionScope::Run)
            | (AbilityEffectClass::Battle, AbilityProjectionScope::Battle)
            | (AbilityEffectClass::RunAndBattle, _)
    )
}

fn execute(
    operation: AbilityOperation,
    current: AbilityValue,
    authored: AbilityValue,
) -> Result<AbilityValue, AbilityRuntimeError> {
    match operation {
        AbilityOperation::Unlock | AbilityOperation::Enable => Ok(authored),
        AbilityOperation::Set | AbilityOperation::SetRatio => Ok(authored),
        AbilityOperation::AddStat
        | AbilityOperation::UnlockFormationSlot
        | AbilityOperation::AddLimit
        | AbilityOperation::AddCurrency
        | AbilityOperation::AddChoice
        | AbilityOperation::AddResource => current.checked_add(authored),
    }
}

fn validate_shape(
    operation: AbilityOperation,
    target: AbilityTarget,
    unit: AbilityValueUnit,
    value: ExactParameter,
) -> Result<(), AbilityRuntimeError> {
    if unit != target.expected_unit() {
        return Err(AbilityRuntimeError::UnitMismatch);
    }
    let operation_unit = match operation {
        AbilityOperation::Unlock | AbilityOperation::Enable => AbilityValueUnit::Boolean,
        AbilityOperation::UnlockFormationSlot
        | AbilityOperation::AddLimit
        | AbilityOperation::AddCurrency
        | AbilityOperation::AddChoice => AbilityValueUnit::Count,
        AbilityOperation::AddResource => AbilityValueUnit::Flat,
        AbilityOperation::SetRatio => AbilityValueUnit::Ratio,
        AbilityOperation::AddStat | AbilityOperation::Set => unit,
    };
    if operation_unit != unit
        || matches!(unit, AbilityValueUnit::Boolean)
            && (value.coefficient(), value.scale()) != (1, 0)
    {
        return Err(AbilityRuntimeError::OperationMismatch);
    }
    Ok(())
}

fn catalog_digest(effects: &[CompiledAbilityEffect]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-ability-runtime-catalog-v1");
    encoder.text(ABILITY_RUNTIME_REVISION);
    encoder.u32(effects.len() as u32);
    for effect in effects {
        encoder.u32(effect.source.get());
        encoder.u8(effect.class as u8);
        encoder.u8(effect.operation as u8);
        encoder.u8(effect.target as u8);
        encoder.i64(effect.value.0);
        encoder.u8(effect.unit as u8);
        encode_trigger(&mut encoder, effect.trigger);
    }
    encoder.finish()
}

fn projection_digest(
    context: AbilityExecutionContext,
    values: &[ProjectedAbilityValue],
    applied: &[AppliedAbilityEffect],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-ability-runtime-projection-v1");
    encoder.text(ABILITY_RUNTIME_REVISION);
    encoder.u8(context.scope as u8);
    encoder.u8(context.boundary as u8);
    encoder.u8(context.chosen_path_blessings);
    encoder.u8(u8::from(context.first_battle_won));
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.u8(value.target as u8);
        encoder.u8(value.unit as u8);
        encoder.i64(value.value.0);
    }
    encoder.u32(applied.len() as u32);
    for effect in applied {
        encoder.u32(effect.source.get());
        encoder.u8(effect.operation as u8);
        encoder.u8(effect.target as u8);
        encoder.i64(effect.value.0);
    }
    encoder.finish()
}

fn encode_trigger(encoder: &mut Encoder, trigger: AbilityTrigger) {
    match trigger {
        AbilityTrigger::Always => encoder.u8(0),
        AbilityTrigger::ChosenPathBlessingsAtLeast(value) => {
            encoder.u8(1);
            encoder.u8(value);
        }
        AbilityTrigger::FirstBattleWon => encoder.u8(2),
        AbilityTrigger::BattleStart => encoder.u8(3),
        AbilityTrigger::EnterEliteOrBossDomain => encoder.u8(4),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityRuntimeError {
    InvalidDenominator,
    UnknownTarget(AbilityTreeNodeId),
    UnknownCondition(AbilityTreeNodeId),
    ClassMismatch(AbilityTreeNodeId),
    UnitMismatch,
    OperationMismatch,
    InvalidValue,
    Overflow,
    NonCanonicalSelection,
}
