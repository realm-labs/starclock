//! Production combat-modifier lowering for the six Stats Conundrum rules.

use starclock_combat::{
    EffectDefinitionId, ModifierDefinitionId, ModifierStackingGroupId, Rounding, Scalar,
    SourceDefinitionId, StateSlotDefinitionId,
    catalog::definition::EffectDefinition,
    formula::toughness::EnemyRank,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, RuleValueKind, SourceClass, ValueExpr},
};

use crate::digest::Encoder;

use super::{
    GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY, GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION,
    GoldAndGearsConundrumEffect, GoldAndGearsEntryError, GoldAndGearsRuntimeInstance,
};

/// Revision of the six-rule production combat-modifier projection.
pub const GOLD_AND_GEARS_STATS_CONUNDRUM_MODIFIER_REVISION: &str =
    "gold-and-gears-stats-conundrum-combat-modifier-v1";

const MODIFIER_BASE: u32 = 0x7f10_0000;
const GROUP_BASE: u32 = 0x7f11_0000;
const SOURCE_BASE: u32 = 0x7f12_0000;
const BERSERK_EFFECT_ID: EffectDefinitionId =
    EffectDefinitionId::new(0x7f14_0001).expect("reserved effect identity is non-zero");
const BERSERK_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x7f13_0001).expect("reserved slot identity is non-zero");

/// Semantic role of one generic combat modifier emitted by a frozen rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsStatsConundrumModifierRole {
    EnemyAttackRatio = 0,
    EnemyMaximumHpRatio = 1,
    EnemySpeedRatio = 2,
    BerserkAttackRatioPerStack = 3,
    BerserkSpeedRatioPerStack = 4,
    EliteBossToughnessRatio = 5,
    EliteBossReceivedAttackAdvanceRatio = 6,
}

/// Caller-visible activation condition retained beside the generic modifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsStatsConundrumActivation {
    EveryEnemy,
    EliteOrBossWhileBerserk,
    EliteOrBossAfterReceivedAttackWhileBerserk,
}

impl GoldAndGearsStatsConundrumActivation {
    /// Whether a battle assembler should attach this modifier for the supplied
    /// immutable combat snapshot.
    #[must_use]
    pub fn is_active(self, rank: EnemyRank, berserk_stacks: u8, received_attack: bool) -> bool {
        match self {
            Self::EveryEnemy => true,
            Self::EliteOrBossWhileBerserk => rank == EnemyRank::EliteOrBoss && berserk_stacks > 0,
            Self::EliteOrBossAfterReceivedAttackWhileBerserk => {
                rank == EnemyRank::EliteOrBoss && berserk_stacks > 0 && received_attack
            }
        }
    }
}

/// One source-attributed modifier definition selected by Stats composition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsStatsConundrumModifierBinding {
    source_rule_id: Box<str>,
    owner_id: Box<str>,
    role: GoldAndGearsStatsConundrumModifierRole,
    activation: GoldAndGearsStatsConundrumActivation,
    ratio_scaled: i64,
    group: ModifierStackingGroup,
    definition: ModifierDefinition,
    source: RuleSource,
}

impl GoldAndGearsStatsConundrumModifierBinding {
    #[must_use]
    pub fn source_rule_id(&self) -> &str {
        &self.source_rule_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn role(&self) -> GoldAndGearsStatsConundrumModifierRole {
        self.role
    }

    #[must_use]
    pub const fn activation(&self) -> GoldAndGearsStatsConundrumActivation {
        self.activation
    }

    /// Signed six-decimal ratio in millionths.
    #[must_use]
    pub const fn ratio_scaled(&self) -> i64 {
        self.ratio_scaled
    }

    #[must_use]
    pub const fn group(&self) -> &ModifierStackingGroup {
        &self.group
    }

    #[must_use]
    pub const fn definition(&self) -> &ModifierDefinition {
        &self.definition
    }

    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
}

/// Canonically ordered generic modifier set for one selected Stats level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsStatsConundrumModifierSet {
    selected_level: u8,
    bindings: Box<[GoldAndGearsStatsConundrumModifierBinding]>,
    digest: [u8; 32],
}

impl GoldAndGearsStatsConundrumModifierSet {
    #[must_use]
    pub const fn selected_level(&self) -> u8 {
        self.selected_level
    }

    #[must_use]
    pub fn bindings(&self) -> &[GoldAndGearsStatsConundrumModifierBinding] {
        &self.bindings
    }

    #[must_use]
    pub const fn berserk_stack_slot(&self) -> StateSlotDefinitionId {
        BERSERK_STACK_SLOT
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(super) fn source_stack_effect(&self) -> Option<EffectDefinition> {
        let modifiers = self
            .bindings
            .iter()
            .filter(|binding| binding.definition.source_stack_slot.is_some())
            .map(|binding| binding.definition.id)
            .collect::<Vec<_>>();
        (!modifiers.is_empty())
            .then(|| EffectDefinition::new(BERSERK_EFFECT_ID, Vec::new(), modifiers))
    }
}

impl GoldAndGearsRuntimeInstance {
    /// Lowers the selected Stats Conundrum composition into combat-owned
    /// modifier definitions.
    ///
    /// The battle assembler remains responsible for evaluating each binding's
    /// activation against an immutable enemy snapshot and attaching it through
    /// the ordinary combat source/modifier boundary.
    pub fn compile_stats_conundrum_modifiers(
        &self,
    ) -> Result<GoldAndGearsStatsConundrumModifierSet, GoldAndGearsEntryError> {
        let mut bindings = Vec::new();
        for contribution in self.conundrum_contributions() {
            let Some(level) = stats_level(contribution.source_level()) else {
                continue;
            };
            let rule_id = format!("gold-gears.rule.conundrum.stats.{level}");
            match contribution.effect() {
                GoldAndGearsConundrumEffect::EnemyStat(policy) => {
                    push_binding(
                        &mut bindings,
                        level,
                        &rule_id,
                        contribution.source_level(),
                        GoldAndGearsStatsConundrumModifierRole::EnemyAttackRatio,
                        GoldAndGearsStatsConundrumActivation::EveryEnemy,
                        policy.attack_ratio_scaled(),
                    )?;
                    push_binding(
                        &mut bindings,
                        level,
                        &rule_id,
                        contribution.source_level(),
                        GoldAndGearsStatsConundrumModifierRole::EnemyMaximumHpRatio,
                        GoldAndGearsStatsConundrumActivation::EveryEnemy,
                        policy.maximum_hp_ratio_scaled(),
                    )?;
                    push_binding(
                        &mut bindings,
                        level,
                        &rule_id,
                        contribution.source_level(),
                        GoldAndGearsStatsConundrumModifierRole::EnemySpeedRatio,
                        GoldAndGearsStatsConundrumActivation::EveryEnemy,
                        policy.speed_ratio_scaled(),
                    )?;
                }
                GoldAndGearsConundrumEffect::EnhancedBerserk => {
                    let policy = self.conundrum_berserk_policy();
                    if !policy.enhanced() || level != 3 {
                        return Err(GoldAndGearsEntryError::InvalidStatsConundrumModifier);
                    }
                    push_binding(
                        &mut bindings,
                        level,
                        &rule_id,
                        contribution.source_level(),
                        GoldAndGearsStatsConundrumModifierRole::BerserkAttackRatioPerStack,
                        GoldAndGearsStatsConundrumActivation::EliteOrBossWhileBerserk,
                        policy.attack_ratio_per_stack_scaled(),
                    )?;
                    push_binding(
                        &mut bindings,
                        level,
                        &rule_id,
                        contribution.source_level(),
                        GoldAndGearsStatsConundrumModifierRole::BerserkSpeedRatioPerStack,
                        GoldAndGearsStatsConundrumActivation::EliteOrBossWhileBerserk,
                        policy.speed_ratio_per_stack_scaled(),
                    )?;
                }
                GoldAndGearsConundrumEffect::EliteBossResponse(policy) => {
                    if level != 5 {
                        return Err(GoldAndGearsEntryError::InvalidStatsConundrumModifier);
                    }
                    push_binding(
                        &mut bindings,
                        level,
                        &rule_id,
                        contribution.source_level(),
                        GoldAndGearsStatsConundrumModifierRole::EliteBossToughnessRatio,
                        GoldAndGearsStatsConundrumActivation::EliteOrBossWhileBerserk,
                        policy.toughness_ratio_scaled(),
                    )?;
                    push_binding(
                        &mut bindings,
                        level,
                        &rule_id,
                        contribution.source_level(),
                        GoldAndGearsStatsConundrumModifierRole::
                            EliteBossReceivedAttackAdvanceRatio,
                        GoldAndGearsStatsConundrumActivation::
                            EliteOrBossAfterReceivedAttackWhileBerserk,
                        policy.action_advance_ratio_scaled(),
                    )?;
                }
                _ => {}
            }
        }
        bindings.sort_unstable_by_key(|binding| binding.definition.id);
        if bindings.windows(2).any(|pair| {
            pair[0].definition.id >= pair[1].definition.id
                || pair[0].group.id >= pair[1].group.id
                || pair[0].source.definition() >= pair[1].source.definition()
        }) {
            return Err(GoldAndGearsEntryError::InvalidStatsConundrumModifier);
        }
        let expected = [0_usize, 3, 3, 5, 5, 7, 7];
        if bindings.len()
            != expected
                .get(usize::from(self.stats_conundrum()))
                .copied()
                .ok_or(GoldAndGearsEntryError::InvalidStatsConundrumModifier)?
        {
            return Err(GoldAndGearsEntryError::InvalidStatsConundrumModifier);
        }
        let digest = set_digest(self.stats_conundrum(), &bindings);
        Ok(GoldAndGearsStatsConundrumModifierSet {
            selected_level: self.stats_conundrum(),
            bindings: bindings.into_boxed_slice(),
            digest,
        })
    }
}

fn stats_level(source: &str) -> Option<u8> {
    source
        .strip_prefix("gold-gears.conundrum-level.stats.")?
        .parse()
        .ok()
}

fn push_binding(
    output: &mut Vec<GoldAndGearsStatsConundrumModifierBinding>,
    level: u8,
    rule_id: &str,
    owner_id: &str,
    role: GoldAndGearsStatsConundrumModifierRole,
    activation: GoldAndGearsStatsConundrumActivation,
    ratio_scaled: i64,
) -> Result<(), GoldAndGearsEntryError> {
    if !(1..=6).contains(&level) || !(0..=1_000_000).contains(&ratio_scaled) {
        return Err(GoldAndGearsEntryError::InvalidStatsConundrumModifier);
    }
    let ordinal = u32::from(level)
        .checked_mul(16)
        .and_then(|value| value.checked_add(u32::from(role as u8) + 1))
        .ok_or(GoldAndGearsEntryError::InvalidStatsConundrumModifier)?;
    let definition_id = ModifierDefinitionId::new(MODIFIER_BASE + ordinal)
        .ok_or(GoldAndGearsEntryError::InvalidStatsConundrumModifier)?;
    let group_id = ModifierStackingGroupId::new(GROUP_BASE + ordinal)
        .ok_or(GoldAndGearsEntryError::InvalidStatsConundrumModifier)?;
    let source_id = SourceDefinitionId::new(SOURCE_BASE + ordinal)
        .ok_or(GoldAndGearsEntryError::InvalidStatsConundrumModifier)?;
    let (stat, stage, value, stack_slot) = modifier_value(role, ratio_scaled)?;
    let group = ModifierStackingGroup {
        id: group_id,
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    };
    let definition = ModifierDefinition {
        id: definition_id,
        stat,
        stage,
        purpose: FormulaPurpose::Stat,
        value,
        stacking_group: group_id,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: stack_slot,
        filters: Box::new([]),
    };
    let source = RuleSource::new(
        source_id,
        SourceClass::Mode,
        vec![],
        source_digest(rule_id, owner_id, role, activation, ratio_scaled),
    );
    output.push(GoldAndGearsStatsConundrumModifierBinding {
        source_rule_id: rule_id.into(),
        owner_id: owner_id.into(),
        role,
        activation,
        ratio_scaled,
        group,
        definition,
        source,
    });
    Ok(())
}

fn modifier_value(
    role: GoldAndGearsStatsConundrumModifierRole,
    ratio_scaled: i64,
) -> Result<
    (
        StatKind,
        FormulaStage,
        ValueExpr,
        Option<StateSlotDefinitionId>,
    ),
    GoldAndGearsEntryError,
> {
    let scalar = Scalar::from_scaled(ratio_scaled);
    let literal = || ValueExpr::Literal(RuleValue::Scalar(scalar));
    Ok(match role {
        GoldAndGearsStatsConundrumModifierRole::EnemyAttackRatio => {
            (StatKind::Atk, FormulaStage::PercentOfBase, literal(), None)
        }
        GoldAndGearsStatsConundrumModifierRole::EnemyMaximumHpRatio => {
            (StatKind::Hp, FormulaStage::PercentOfBase, literal(), None)
        }
        GoldAndGearsStatsConundrumModifierRole::EnemySpeedRatio => {
            (StatKind::Spd, FormulaStage::PercentOfBase, literal(), None)
        }
        GoldAndGearsStatsConundrumModifierRole::BerserkAttackRatioPerStack
        | GoldAndGearsStatsConundrumModifierRole::BerserkSpeedRatioPerStack => (
            if role == GoldAndGearsStatsConundrumModifierRole::BerserkAttackRatioPerStack {
                StatKind::Atk
            } else {
                StatKind::Spd
            },
            FormulaStage::PercentOfBase,
            ValueExpr::Multiply {
                lhs: Box::new(ValueExpr::Convert {
                    value: Box::new(ValueExpr::Slot(BERSERK_STACK_SLOT)),
                    target: RuleValueKind::Scalar,
                    rounding: Rounding::NearestTiesEven,
                }),
                rhs: Box::new(literal()),
                rounding: Rounding::NearestTiesEven,
            },
            Some(BERSERK_STACK_SLOT),
        ),
        GoldAndGearsStatsConundrumModifierRole::EliteBossToughnessRatio => (
            StatKind::MaximumToughness,
            FormulaStage::PercentOfBase,
            literal(),
            None,
        ),
        GoldAndGearsStatsConundrumModifierRole::EliteBossReceivedAttackAdvanceRatio => (
            StatKind::ReceivedAttackActionAdvance,
            FormulaStage::Flat,
            literal(),
            None,
        ),
    })
}

fn source_digest(
    rule_id: &str,
    owner_id: &str,
    role: GoldAndGearsStatsConundrumModifierRole,
    activation: GoldAndGearsStatsConundrumActivation,
    ratio_scaled: i64,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-gears.stats-conundrum-modifier-source.v1");
    encoder.text(GOLD_AND_GEARS_STATS_CONUNDRUM_MODIFIER_REVISION);
    encoder.text(GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION);
    encoder.text(GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY);
    encoder.text(rule_id);
    encoder.text(owner_id);
    encoder.u8(role as u8);
    encoder.u8(match activation {
        GoldAndGearsStatsConundrumActivation::EveryEnemy => 0,
        GoldAndGearsStatsConundrumActivation::EliteOrBossWhileBerserk => 1,
        GoldAndGearsStatsConundrumActivation::EliteOrBossAfterReceivedAttackWhileBerserk => 2,
    });
    encoder.i64(ratio_scaled);
    encoder.finish()
}

fn set_digest(
    selected_level: u8,
    bindings: &[GoldAndGearsStatsConundrumModifierBinding],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-gears.stats-conundrum-modifier-set.v1");
    encoder.text(GOLD_AND_GEARS_STATS_CONUNDRUM_MODIFIER_REVISION);
    encoder.text(GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION);
    encoder.text(GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY);
    encoder.u8(selected_level);
    encoder.u32(u32::try_from(bindings.len()).expect("bounded modifier count"));
    for binding in bindings {
        encoder.text(binding.source_rule_id());
        encoder.text(binding.owner_id());
        encoder.u8(binding.role as u8);
        encoder.i64(binding.ratio_scaled);
        encoder.u32(binding.definition.id.get());
        encoder.u32(binding.group.id.get());
        encoder.u32(binding.source.definition().get());
        encoder.digest(binding.source.digest());
    }
    encoder.finish()
}
