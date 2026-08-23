//! Snapshot-resolved Affix multipliers lowered to generic combat modifiers.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use starclock_combat::{
    ModifierDefinitionId, ModifierStackingGroupId, Ratio, ResolvedCombatantSpec, Rounding, Scalar,
    SourceDefinitionId,
    catalog::builder::CombatCatalogBuilder,
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{Comparison, ConditionExpr, RuleSource, RuleValue, SourceClass, ValueExpr},
};

use crate::{
    CurrencyWarsContributionSnapshot, CurrencyWarsEnemyAffixBehavior,
    CurrencyWarsEnemyAffixSemantic, CurrencyWarsRoleContribution, CurrencyWarsRoleId,
    battle_assembly::{
        CurrencyWarsBattleAssemblyError,
        combatant_overlay::{attach_modifier, attach_modifier_to_linked_subjects},
        debug_error, error,
    },
};

const DEFINITION_BASE: u32 = 0x7d70_0000;
pub(crate) const BOND_OR_INVESTMENT_DAMAGE_TAG: &str = "currency-wars.damage.bond-or-investment";
const DAMAGE_PURPOSES: [FormulaPurpose; 7] = [
    FormulaPurpose::OrdinaryDamage,
    FormulaPurpose::Dot,
    FormulaPurpose::Break,
    FormulaPurpose::SuperBreak,
    FormulaPurpose::AdditionalDamage,
    FormulaPurpose::JointDamage,
    FormulaPurpose::ElationDamage,
];

#[derive(Clone, Debug, Default)]
pub(crate) struct EnemyAffixOverlays {
    enemy: Vec<CompiledModifier>,
}

impl EnemyAffixOverlays {
    pub(crate) fn apply_enemy(
        &self,
        mut combatant: ResolvedCombatantSpec,
    ) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
        for modifier in &self.enemy {
            combatant = modifier.attach(&combatant)?;
        }
        Ok(combatant)
    }
}

#[derive(Clone, Debug)]
struct CompiledModifier {
    definitions: Box<[ModifierDefinitionId]>,
    source: RuleSource,
    digest: [u8; 32],
}

impl CompiledModifier {
    fn attach(
        &self,
        base: &ResolvedCombatantSpec,
    ) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
        self.definitions.iter().try_fold(base.clone(), |value, id| {
            attach_modifier(
                &value,
                *id,
                self.source.clone(),
                b"starclock.currency-wars.enemy-affix-modifier.v1",
                self.digest,
            )
        })
    }

    fn attach_to_linked_subjects(
        &self,
        base: &ResolvedCombatantSpec,
    ) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
        self.definitions.iter().try_fold(base.clone(), |value, id| {
            attach_modifier_to_linked_subjects(
                &value,
                *id,
                self.source.clone(),
                b"starclock.currency-wars.enemy-affix-modifier.v1",
                self.digest,
            )
        })
    }
}

pub(crate) fn install_static_modifiers(
    builder: &mut CombatCatalogBuilder,
    snapshot: &CurrencyWarsContributionSnapshot,
    combatants: &mut BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<EnemyAffixOverlays, CurrencyWarsBattleAssemblyError> {
    let mut compiler = Compiler::new(builder, snapshot.digest.bytes());
    let speed_alternation = speed_alternation_values(snapshot)?;
    for role in &snapshot.roles {
        let Some(combatant) = combatants.get(&role.role.id) else {
            continue;
        };
        let mut replacement = combatant.clone();
        if role.equipment.len() < 3
            && let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::Enervation)
        {
            replacement = compiler
                .damage_multiplier(
                    Ratio::from_scaled(scalar_parameter(behavior, 0)?.scaled()),
                    FormulaSubject::Source,
                    Vec::new(),
                    u64::from(role.role.id.get()),
                )?
                .attach_to_linked_subjects(&replacement)?;
        }
        let outgoing = player_outgoing_multiplier(snapshot, role)?;
        let mut compiled = Vec::new();
        if outgoing != Ratio::ONE {
            compiled.push(compiler.damage_multiplier(
                outgoing,
                FormulaSubject::Source,
                Vec::new(),
                u64::from(role.role.id.get()),
            )?);
        }
        if let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::ConnectionsFirst)
        {
            compiled.push(compiler.damage_multiplier(
                Ratio::from_scaled(scalar_parameter(behavior, 1)?.scaled()),
                FormulaSubject::Source,
                vec![ModifierFilter::DamageTag(
                    BOND_OR_INVESTMENT_DAMAGE_TAG.into(),
                )],
                u64::from(role.role.id.get()),
            )?);
        }
        if role.role_state.star() == 1
            && let Some(behavior) =
                behavior(snapshot, CurrencyWarsEnemyAffixSemantic::PreyOnTheWeak)
        {
            compiled.push(compiler.formula_addition(
                FormulaStage::Vulnerability,
                scalar_parameter(behavior, 0)?,
                FormulaSubject::Target,
                vec![ModifierFilter::Source(SourceClass::Enemy)],
                u64::from(role.role.id.get()),
            )?);
        }
        if let Some(value) = speed_alternation.get(&role.role.id) {
            compiled.push(compiler.stat_addition(
                StatKind::Spd,
                FormulaStage::PercentOfBase,
                *value,
                u64::from(role.role.id.get()),
            )?);
        }
        if let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::LeadByExample) {
            let highest = ConditionExpr::HighestDamageDealer(StatQuerySubject::Owner);
            let weaken = Scalar::ONE
                .checked_sub(scalar_parameter(behavior, 0)?)
                .map_err(debug_error)?;
            let boost = scalar_parameter(behavior, 1)?
                .checked_sub(Scalar::ONE)
                .map_err(debug_error)?;
            compiled.push(compiler.formula_expression(
                FormulaStage::Weaken,
                ValueExpr::Choose {
                    condition: Box::new(highest.clone()),
                    when_true: Box::new(ValueExpr::Literal(RuleValue::Scalar(weaken))),
                    when_false: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
                },
                FormulaSubject::Source,
                Vec::new(),
                u64::from(role.role.id.get()),
            )?);
            compiled.push(compiler.formula_expression(
                FormulaStage::DamageBoost,
                ValueExpr::Choose {
                    condition: Box::new(ConditionExpr::Not(Box::new(highest))),
                    when_true: Box::new(ValueExpr::Literal(RuleValue::Scalar(boost))),
                    when_false: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
                },
                FormulaSubject::Source,
                Vec::new(),
                u64::from(role.role.id.get()),
            )?);
        }
        if let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::LostLuck) {
            compiled.push(
                compiler.formula_purpose_addition(
                    FormulaStage::Probability,
                    FormulaPurpose::SecondaryProcChance,
                    scalar_parameter(behavior, 0)?
                        .checked_neg()
                        .map_err(debug_error)?,
                    FormulaSubject::Source,
                    u64::from(role.role.id.get()),
                )?,
            );
        }
        if let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::BluntTheEdge) {
            compiled.push(compiler.conditional_weaken_below_mitigation(
                scalar_parameter(behavior, 0)?,
                Ratio::from_scaled(scalar_parameter(behavior, 1)?.scaled()),
                u64::from(role.role.id.get()),
            )?);
        }
        let replacement = compiled
            .iter()
            .try_fold(replacement, |value, modifier| modifier.attach(&value))?;
        combatants.insert(role.role.id, replacement);
    }

    let mut enemy = Vec::new();
    if let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::RustingTreasury) {
        let maximum = integer_parameter(behavior, 2)?;
        let count = snapshot.unequipped_equipment_count.min(maximum);
        if count > 0 {
            let count = Scalar::checked_from_integer(i64::from(count)).map_err(debug_error)?;
            let outgoing = Ratio::ONE
                .checked_add(Ratio::from_scaled(
                    scalar_parameter(behavior, 0)?
                        .checked_mul(count, Rounding::NearestTiesAway)
                        .map_err(debug_error)?
                        .scaled(),
                ))
                .map_err(debug_error)?;
            enemy.push(compiler.damage_multiplier(
                outgoing,
                FormulaSubject::Source,
                Vec::new(),
                0,
            )?);
            enemy.push(
                compiler.formula_addition(
                    FormulaStage::Mitigation,
                    scalar_parameter(behavior, 1)?
                        .checked_mul(count, Rounding::NearestTiesAway)
                        .map_err(debug_error)?,
                    FormulaSubject::Target,
                    Vec::new(),
                    1,
                )?,
            );
        }
    }
    if let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::ThickSkinned) {
        enemy.push(compiler.formula_expression(
            FormulaStage::Mitigation,
            ValueExpr::Choose {
                condition: Box::new(ConditionExpr::CurrentTargetIsBroken),
                when_true: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
                when_false: Box::new(ValueExpr::Literal(RuleValue::Scalar(scalar_parameter(
                    behavior, 0,
                )?))),
            },
            FormulaSubject::Target,
            Vec::new(),
            2,
        )?);
    }
    Ok(EnemyAffixOverlays { enemy })
}

fn speed_alternation_values(
    snapshot: &CurrencyWarsContributionSnapshot,
) -> Result<BTreeMap<CurrencyWarsRoleId, Scalar>, CurrencyWarsBattleAssemblyError> {
    let Some(behavior) = behavior(snapshot, CurrencyWarsEnemyAffixSemantic::SpeedAlternation)
    else {
        return Ok(BTreeMap::new());
    };
    let mut ordered = snapshot
        .roles
        .iter()
        .map(|role| (role.role.id, role.build.relic_stats().base_ratios()[3]))
        .collect::<Vec<_>>();
    ordered.sort_unstable_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    let fast_count = ordered.len().min(2);
    let slow_count = ordered.len().saturating_sub(fast_count).min(2);
    let mut values = BTreeMap::new();
    for (role, _) in ordered.iter().take(fast_count) {
        values.insert(*role, scalar_parameter(behavior, 0)?);
    }
    for (role, _) in ordered.iter().rev().take(slow_count) {
        values.insert(*role, scalar_parameter(behavior, 1)?);
    }
    Ok(values)
}

fn player_outgoing_multiplier(
    snapshot: &CurrencyWarsContributionSnapshot,
    role: &CurrencyWarsRoleContribution,
) -> Result<Ratio, CurrencyWarsBattleAssemblyError> {
    let mut multiplier = Ratio::ONE;
    for behavior in &snapshot.enemy_affix_behaviors {
        let value = match behavior.semantic {
            CurrencyWarsEnemyAffixSemantic::ConnectionsFirst => {
                Some(scalar_parameter(behavior, 0)?)
            }
            CurrencyWarsEnemyAffixSemantic::DifferentialTreatment => Some(scalar_parameter(
                behavior,
                usize::from(role.role_state.star().saturating_sub(1).min(2)),
            )?),
            CurrencyWarsEnemyAffixSemantic::ExpensiveTaste => Some(scalar_parameter(
                behavior,
                usize::from(role.role.rarity >= 4),
            )?),
            CurrencyWarsEnemyAffixSemantic::CheapTaste => Some(scalar_parameter(
                behavior,
                usize::from(role.role.rarity >= 4),
            )?),
            CurrencyWarsEnemyAffixSemantic::MeAlone if role.inactive_bond_count > 0 => {
                Some(scalar_parameter(
                    behavior,
                    usize::from(role.inactive_bond_count.saturating_sub(1).min(2)),
                )?)
            }
            _ => None,
        };
        if let Some(value) = value {
            multiplier = multiplier
                .checked_mul(
                    Ratio::from_scaled(value.scaled()),
                    Rounding::NearestTiesAway,
                )
                .map_err(debug_error)?;
        }
    }
    Ok(multiplier)
}

struct Compiler<'a> {
    builder: &'a mut CombatCatalogBuilder,
    root_digest: [u8; 32],
    next: u32,
}

impl<'a> Compiler<'a> {
    const fn new(builder: &'a mut CombatCatalogBuilder, root_digest: [u8; 32]) -> Self {
        Self {
            builder,
            root_digest,
            next: DEFINITION_BASE,
        }
    }

    fn damage_multiplier(
        &mut self,
        multiplier: Ratio,
        subject: FormulaSubject,
        filters: Vec<ModifierFilter>,
        discriminator: u64,
    ) -> Result<CompiledModifier, CurrencyWarsBattleAssemblyError> {
        let (stage, value) = if multiplier >= Ratio::ONE {
            (
                FormulaStage::DamageBoost,
                Scalar::from_scaled(
                    multiplier
                        .scaled()
                        .checked_sub(Ratio::ONE.scaled())
                        .ok_or_else(|| error("Currency Wars Affix multiplier underflow"))?,
                ),
            )
        } else {
            (
                FormulaStage::Weaken,
                Scalar::from_scaled(
                    Ratio::ONE
                        .scaled()
                        .checked_sub(multiplier.scaled())
                        .ok_or_else(|| error("Currency Wars Affix multiplier underflow"))?,
                ),
            )
        };
        self.formula_addition(stage, value, subject, filters, discriminator)
    }

    fn conditional_weaken_below_mitigation(
        &mut self,
        threshold: Scalar,
        multiplier: Ratio,
        discriminator: u64,
    ) -> Result<CompiledModifier, CurrencyWarsBattleAssemblyError> {
        let weaken = Ratio::ONE
            .scaled()
            .checked_sub(multiplier.scaled())
            .map(Scalar::from_scaled)
            .ok_or_else(|| error("Currency Wars Blunt the Edge multiplier exceeds one"))?;
        let group = ModifierStackingGroupId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix modifier group ID is invalid"))?;
        let source_id = SourceDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix modifier source ID is invalid"))?;
        let digest = modifier_digest(
            self.root_digest,
            FormulaStage::Weaken,
            Scalar::ZERO,
            discriminator,
        );
        let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), digest);
        self.builder.add_modifier_group(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        let definitions = DAMAGE_PURPOSES
            .into_iter()
            .map(|purpose| {
                let id = ModifierDefinitionId::new(self.take_id()?)
                    .ok_or_else(|| error("Currency Wars Affix modifier ID is invalid"))?;
                self.builder.add_modifier(ModifierDefinition {
                    id,
                    stat: StatKind::Atk,
                    stage: FormulaStage::Weaken,
                    purpose,
                    value: ValueExpr::Choose {
                        condition: Box::new(ConditionExpr::Compare {
                            lhs: Box::new(ValueExpr::QueryFormulaStage {
                                subject: StatQuerySubject::Owner,
                                stage: FormulaStage::Mitigation,
                                purpose,
                            }),
                            operator: Comparison::Less,
                            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(threshold))),
                        }),
                        when_true: Box::new(ValueExpr::Literal(RuleValue::Scalar(weaken))),
                        when_false: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
                    },
                    stacking_group: group,
                    priority: 0,
                    floor: None,
                    cap: None,
                    cap_stage: FormulaStage::Weaken,
                    snapshot: SnapshotPolicy::Dynamic,
                    source_stack_slot: None,
                    filters: Box::new([ModifierFilter::FormulaSubject(FormulaSubject::Source)]),
                });
                Ok(id)
            })
            .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
        Ok(CompiledModifier {
            definitions: definitions.into_boxed_slice(),
            source,
            digest,
        })
    }

    fn formula_addition(
        &mut self,
        stage: FormulaStage,
        value: Scalar,
        subject: FormulaSubject,
        filters: Vec<ModifierFilter>,
        discriminator: u64,
    ) -> Result<CompiledModifier, CurrencyWarsBattleAssemblyError> {
        self.formula_expression(
            stage,
            ValueExpr::Literal(RuleValue::Scalar(value)),
            subject,
            filters,
            discriminator,
        )
    }

    fn formula_expression(
        &mut self,
        stage: FormulaStage,
        value: ValueExpr,
        subject: FormulaSubject,
        mut filters: Vec<ModifierFilter>,
        discriminator: u64,
    ) -> Result<CompiledModifier, CurrencyWarsBattleAssemblyError> {
        let group = ModifierStackingGroupId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix modifier group ID is invalid"))?;
        let source_id = SourceDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix modifier source ID is invalid"))?;
        let digest = modifier_digest(self.root_digest, stage, Scalar::ZERO, discriminator);
        let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), digest);
        self.builder.add_modifier_group(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        filters.push(ModifierFilter::FormulaSubject(subject));
        let definitions = DAMAGE_PURPOSES
            .into_iter()
            .map(|purpose| {
                let id = ModifierDefinitionId::new(self.take_id()?)
                    .ok_or_else(|| error("Currency Wars Affix modifier ID is invalid"))?;
                self.builder.add_modifier(ModifierDefinition {
                    id,
                    stat: StatKind::Atk,
                    stage,
                    purpose,
                    value: value.clone(),
                    stacking_group: group,
                    priority: 0,
                    floor: None,
                    cap: None,
                    cap_stage: stage,
                    snapshot: SnapshotPolicy::Dynamic,
                    source_stack_slot: None,
                    filters: filters.clone().into_boxed_slice(),
                });
                Ok(id)
            })
            .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
        Ok(CompiledModifier {
            definitions: definitions.into_boxed_slice(),
            source,
            digest,
        })
    }

    fn stat_addition(
        &mut self,
        stat: StatKind,
        stage: FormulaStage,
        value: Scalar,
        discriminator: u64,
    ) -> Result<CompiledModifier, CurrencyWarsBattleAssemblyError> {
        let group = ModifierStackingGroupId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix stat modifier group ID is invalid"))?;
        let source_id = SourceDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix stat modifier source ID is invalid"))?;
        let digest = modifier_digest(self.root_digest, stage, value, discriminator);
        let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), digest);
        let definition = ModifierDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix stat modifier ID is invalid"))?;
        self.builder.add_modifier_group(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        self.builder.add_modifier(ModifierDefinition {
            id: definition,
            stat,
            stage,
            purpose: FormulaPurpose::Stat,
            value: ValueExpr::Literal(RuleValue::Scalar(value)),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
        Ok(CompiledModifier {
            definitions: Box::new([definition]),
            source,
            digest,
        })
    }

    fn formula_purpose_addition(
        &mut self,
        stage: FormulaStage,
        purpose: FormulaPurpose,
        value: Scalar,
        subject: FormulaSubject,
        discriminator: u64,
    ) -> Result<CompiledModifier, CurrencyWarsBattleAssemblyError> {
        let group = ModifierStackingGroupId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix formula group ID is invalid"))?;
        let source_id = SourceDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix formula source ID is invalid"))?;
        let digest = modifier_digest(self.root_digest, stage, value, discriminator);
        let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), digest);
        let definition = ModifierDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars Affix formula modifier ID is invalid"))?;
        self.builder.add_modifier_group(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        self.builder.add_modifier(ModifierDefinition {
            id: definition,
            stat: StatKind::Atk,
            stage,
            purpose,
            value: ValueExpr::Literal(RuleValue::Scalar(value)),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([ModifierFilter::FormulaSubject(subject)]),
        });
        Ok(CompiledModifier {
            definitions: Box::new([definition]),
            source,
            digest,
        })
    }

    fn take_id(&mut self) -> Result<u32, CurrencyWarsBattleAssemblyError> {
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| error("Currency Wars Affix definition ID overflow"))?;
        Ok(id)
    }
}

fn behavior(
    snapshot: &CurrencyWarsContributionSnapshot,
    semantic: CurrencyWarsEnemyAffixSemantic,
) -> Option<&CurrencyWarsEnemyAffixBehavior> {
    snapshot
        .enemy_affix_behaviors
        .iter()
        .find(|behavior| behavior.semantic == semantic)
}

fn scalar_parameter(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    index: usize,
) -> Result<Scalar, CurrencyWarsBattleAssemblyError> {
    behavior
        .parameters
        .get(index)
        .copied()
        .ok_or_else(|| error("Currency Wars Affix scalar parameter is missing"))
}

fn integer_parameter(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    index: usize,
) -> Result<u32, CurrencyWarsBattleAssemblyError> {
    let scaled = scalar_parameter(behavior, index)?.scaled();
    if scaled < 0 || scaled % 1_000_000 != 0 {
        return Err(error("Currency Wars Affix integer parameter is invalid"));
    }
    u32::try_from(scaled / 1_000_000).map_err(debug_error)
}

fn modifier_digest(
    root: [u8; 32],
    stage: FormulaStage,
    value: Scalar,
    discriminator: u64,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.enemy-affix-static-modifier.v1");
    hash.update(root);
    hash.update([stage as u8]);
    hash.update(value.scaled().to_le_bytes());
    hash.update(discriminator.to_le_bytes());
    hash.finalize().into()
}
