//! Snapshot-resolved static Currency Wars contributions installed into combat.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use starclock_combat::{
    Energy, ModifierDefinitionId, ModifierStackingGroupId, ResolvedCombatantSpec, Rounding, Scalar,
    SourceDefinitionId,
    catalog::builder::CombatCatalogBuilder,
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};

use crate::{
    CurrencyWarsAuthoredProperty, CurrencyWarsBondPropertyKind, CurrencyWarsContributionSnapshot,
    CurrencyWarsInvestmentKind, CurrencyWarsPositionKind, CurrencyWarsPropertyContribution,
    CurrencyWarsRoleContribution, CurrencyWarsRoleId,
};

use super::{
    CurrencyWarsBattleAssemblyError, combatant_overlay::attach_modifier, debug_error, error,
};

const DEFINITION_BASE: u32 = 0x7d90_0000;
const AUGMENT_CONTROLLER_POLICY_ID: &str =
    "currency-wars.augment-controller-contribution-policy.v1";
const AUGMENT_CONTROLLER_REPLACEMENT_CONDITION: &str = "Replace the one-percent all-damage contribution per selected Augment when reviewed released executable evidence identifies each Augment controller ability's typed battle semantics.";
const AUGMENT_ALL_DAMAGE_PER_SELECTION: i64 = 10_000;
const DAMAGE_PURPOSES: [FormulaPurpose; 7] = [
    FormulaPurpose::OrdinaryDamage,
    FormulaPurpose::Dot,
    FormulaPurpose::Break,
    FormulaPurpose::SuperBreak,
    FormulaPurpose::AdditionalDamage,
    FormulaPurpose::JointDamage,
    FormulaPurpose::ElationDamage,
];

/// Exact static contribution families consumed by one immutable battle assembly.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CurrencyWarsBattleContributionReceipt {
    pub front_role_count: u16,
    pub modifier_binding_count: u16,
    pub star_property_count: u16,
    pub equipment_property_count: u16,
    pub bond_property_count: u16,
    pub off_field_all_member_property_count: u16,
    pub entry_energy_role_count: u16,
    pub empowerment_skill_count: u16,
    pub character_override_count: u16,
    pub selected_investment_count: u16,
    pub augment_policy_modifier_count: u16,
}

impl CurrencyWarsBattleContributionReceipt {
    #[must_use]
    pub const fn augment_policy_id(self) -> Option<&'static str> {
        if self.augment_policy_modifier_count == 0 {
            None
        } else {
            Some(AUGMENT_CONTROLLER_POLICY_ID)
        }
    }

    #[must_use]
    pub const fn augment_policy_replacement_condition(self) -> Option<&'static str> {
        if self.augment_policy_modifier_count == 0 {
            None
        } else {
            Some(AUGMENT_CONTROLLER_REPLACEMENT_CONDITION)
        }
    }
}

pub(super) fn install_static_contributions(
    builder: &mut CombatCatalogBuilder,
    snapshot: &CurrencyWarsContributionSnapshot,
    combatants: &mut BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<CurrencyWarsBattleContributionReceipt, CurrencyWarsBattleAssemblyError> {
    let mut compiler = Compiler::new(builder, snapshot.digest.bytes());
    let mut receipt = CurrencyWarsBattleContributionReceipt {
        empowerment_skill_count: count(snapshot.roles.iter().map(|role| role.empowerment.len()))?,
        character_override_count: count(snapshot.roles.iter().map(|role| {
            usize::from(role.character_override.is_some()) + role.servant_overrides.len()
        }))?,
        selected_investment_count: count([
            snapshot.investments.len(),
            snapshot.typed_investments.len(),
            snapshot.enhancements.len(),
            snapshot.selected_enhancements.len(),
            snapshot.season_talents.len(),
        ])?,
        ..CurrencyWarsBattleContributionReceipt::default()
    };
    let shared_off_field = snapshot
        .roles
        .iter()
        .flat_map(|role| role.off_field.all_member_properties.iter())
        .collect::<Vec<_>>();
    let selected_augment_count = snapshot
        .investments
        .iter()
        .filter(|investment| investment.kind == CurrencyWarsInvestmentKind::Augment)
        .count();
    let augment_all_damage_scaled = i64::try_from(selected_augment_count)
        .ok()
        .and_then(|count| count.checked_mul(AUGMENT_ALL_DAMAGE_PER_SELECTION))
        .ok_or_else(|| error("Currency Wars selected Augment contribution overflow"))?;
    let augment_all_damage = Scalar::from_scaled(augment_all_damage_scaled);

    for role in snapshot
        .roles
        .iter()
        .filter(|role| role.position.kind() == CurrencyWarsPositionKind::Front)
    {
        receipt.front_role_count = receipt
            .front_role_count
            .checked_add(1)
            .ok_or_else(|| error("Currency Wars static contribution role count overflow"))?;
        let mut totals = RoleTotals::new(role)?;
        for property in &role.star_state.property_modifiers {
            if let Some(value) = property.value {
                totals.apply_authored(property, value.scalar().map_err(debug_error)?)?;
                receipt.star_property_count = increment(receipt.star_property_count)?;
            }
        }
        if let Some(rank) = role
            .star_state
            .rank_attachments
            .iter()
            .find(|rank| rank.rank == role.build.eidolon().get())
        {
            for property in &rank.properties {
                if let Some(value) = property.value {
                    totals.apply_authored(property, value.scalar().map_err(debug_error)?)?;
                    receipt.star_property_count = increment(receipt.star_property_count)?;
                }
            }
        }
        for equipment in &role.equipment {
            for property in &equipment.runtime.properties {
                totals.apply_named(property)?;
                receipt.equipment_property_count = increment(receipt.equipment_property_count)?;
            }
        }
        for property in snapshot
            .bonds
            .properties
            .iter()
            .filter(|property| property.targets.contains(&role.role.id))
        {
            totals.apply_bond(
                property.property.kind,
                Scalar::from_scaled(property.property.value.scaled()),
            )?;
            receipt.bond_property_count = increment(receipt.bond_property_count)?;
        }
        for property in &shared_off_field {
            totals.apply_named(property)?;
            receipt.off_field_all_member_property_count =
                increment(receipt.off_field_all_member_property_count)?;
        }
        if selected_augment_count > 0 {
            add(&mut totals.all_damage, augment_all_damage)?;
            receipt.augment_policy_modifier_count =
                increment(receipt.augment_policy_modifier_count)?;
        }

        let base = combatants
            .get(&role.role.id)
            .ok_or_else(|| error("Currency Wars static contribution role is missing"))?;
        let (replacement, bindings, energy_added) = compiler.compile(role, base, totals)?;
        receipt.modifier_binding_count = receipt
            .modifier_binding_count
            .checked_add(bindings)
            .ok_or_else(|| error("Currency Wars static modifier count overflow"))?;
        if energy_added {
            receipt.entry_energy_role_count = increment(receipt.entry_energy_role_count)?;
        }
        combatants.insert(role.role.id, replacement);
    }
    Ok(receipt)
}

struct RoleTotals {
    front_power_base: Scalar,
    front_power_bonus: Scalar,
    hp: Scalar,
    attack: Scalar,
    speed: Scalar,
    crit_rate: Scalar,
    crit_damage: Scalar,
    effect_resistance: Scalar,
    energy_regeneration: Scalar,
    entry_energy: Scalar,
    toughness_damage: Scalar,
    all_damage: Scalar,
    basic_damage: Scalar,
    skill_damage: Scalar,
    ultimate_damage: Scalar,
    dot_damage: Scalar,
    break_damage: Scalar,
    additional_damage: Scalar,
    elation_damage: Scalar,
    healing: Scalar,
    shield: Scalar,
    penetration: Scalar,
    mitigation: Scalar,
    luck_chance: Scalar,
    luck_damage: Scalar,
}

impl RoleTotals {
    fn new(role: &CurrencyWarsRoleContribution) -> Result<Self, CurrencyWarsBattleAssemblyError> {
        let front_power_base = role
            .star_state
            .front_power_base
            .ok_or_else(|| error("Currency Wars front role has no Front Power base"))?
            .scalar()
            .map_err(debug_error)?
            .checked_div_integer(100, Rounding::NearestTiesEven)
            .map_err(debug_error)?;
        Ok(Self {
            front_power_base,
            front_power_bonus: Scalar::ZERO,
            hp: Scalar::ZERO,
            attack: Scalar::ZERO,
            speed: Scalar::ZERO,
            crit_rate: Scalar::ZERO,
            crit_damage: Scalar::ZERO,
            effect_resistance: Scalar::ZERO,
            energy_regeneration: Scalar::ZERO,
            entry_energy: Scalar::ZERO,
            toughness_damage: Scalar::ZERO,
            all_damage: Scalar::ZERO,
            basic_damage: Scalar::ZERO,
            skill_damage: Scalar::ZERO,
            ultimate_damage: Scalar::ZERO,
            dot_damage: Scalar::ZERO,
            break_damage: Scalar::ZERO,
            additional_damage: Scalar::ZERO,
            elation_damage: Scalar::ZERO,
            healing: role
                .star_state
                .extra_heal_base
                .map_or(Ok(Scalar::ZERO), |value| value.scalar())
                .map_err(debug_error)?,
            shield: role
                .star_state
                .extra_shield_base
                .map_or(Ok(Scalar::ZERO), |value| value.scalar())
                .map_err(debug_error)?,
            penetration: Scalar::ZERO,
            mitigation: Scalar::ZERO,
            luck_chance: role
                .star_state
                .luck_chance
                .map_or(Ok(Scalar::ZERO), |value| value.scalar())
                .map_err(debug_error)?,
            luck_damage: role
                .star_state
                .luck_damage
                .map_or(Ok(Scalar::ZERO), |value| value.scalar())
                .map_err(debug_error)?,
        })
    }

    fn apply_authored(
        &mut self,
        property: &CurrencyWarsAuthoredProperty,
        value: Scalar,
    ) -> Result<(), CurrencyWarsBattleAssemblyError> {
        self.apply(property.property.as_ref(), value)
    }

    fn apply_named(
        &mut self,
        property: &CurrencyWarsPropertyContribution,
    ) -> Result<(), CurrencyWarsBattleAssemblyError> {
        self.apply(property.property.as_ref(), property.value)
    }

    fn apply_bond(
        &mut self,
        kind: CurrencyWarsBondPropertyKind,
        value: Scalar,
    ) -> Result<(), CurrencyWarsBattleAssemblyError> {
        match kind {
            CurrencyWarsBondPropertyKind::AllDamage
            | CurrencyWarsBondPropertyKind::AllDamageSecondary
            | CurrencyWarsBondPropertyKind::ElementDamage => add(&mut self.all_damage, value),
            CurrencyWarsBondPropertyKind::FrontPower => add(&mut self.front_power_bonus, value),
            CurrencyWarsBondPropertyKind::BackPower => Ok(()),
            CurrencyWarsBondPropertyKind::DamageOverTime => add(&mut self.dot_damage, value),
            CurrencyWarsBondPropertyKind::Hp => add(&mut self.hp, value),
            CurrencyWarsBondPropertyKind::Healing => add(&mut self.healing, value),
            CurrencyWarsBondPropertyKind::InsertDamage => add(&mut self.additional_damage, value),
            CurrencyWarsBondPropertyKind::LuckChance => add(&mut self.luck_chance, value),
            CurrencyWarsBondPropertyKind::LuckDamage => add(&mut self.luck_damage, value),
            CurrencyWarsBondPropertyKind::NormalDamage => add(&mut self.basic_damage, value),
            CurrencyWarsBondPropertyKind::Shield => add(&mut self.shield, value),
            CurrencyWarsBondPropertyKind::SkillDamage => add(&mut self.skill_damage, value),
            CurrencyWarsBondPropertyKind::Speed => add(&mut self.speed, value),
            CurrencyWarsBondPropertyKind::UltimateDamage => add(&mut self.ultimate_damage, value),
        }
    }

    fn apply(
        &mut self,
        property: &str,
        value: Scalar,
    ) -> Result<(), CurrencyWarsBattleAssemblyError> {
        match property {
            "ExtraFrontPowerAddedRatio1" => add(&mut self.front_power_bonus, value),
            "ExtraBackPowerAddedRatio1" => Ok(()),
            "ExtraHPAddedRatio1" | "ExtraHPAddedRatio2" | "HPAddedRatio" => {
                add(&mut self.hp, value)
            }
            "AttackAddedRatio" => add(&mut self.attack, value),
            "ExtraSpeedAddedRatio1" | "ExtraSpeedAddedRatio2" | "SpeedAddedRatio" => {
                add(&mut self.speed, value)
            }
            "CriticalChanceBase" => add(&mut self.crit_rate, value),
            "CriticalDamageBase" => add(&mut self.crit_damage, value),
            "StatusResistanceBase" => add(&mut self.effect_resistance, value),
            "SPRatioBase" => add(&mut self.energy_regeneration, value),
            "ExtraInitSP" => add(&mut self.entry_energy, value),
            "StanceBreakAddedRatio" => add(&mut self.toughness_damage, value),
            "ExtraAllDamageTypeAddedRatio1"
            | "ExtraAllDamageTypeAddedRatio3"
            | "ExtraAllDamageTypeAddedRatio4"
            | "AllDamageTypeAddedRatio" => add(&mut self.all_damage, value),
            "ExtraNormalDamageAddedRatio1" => add(&mut self.basic_damage, value),
            "ExtraSkillDamageAddedRatio1" => add(&mut self.skill_damage, value),
            "ExtraUltraDamageAddedRatio1" => add(&mut self.ultimate_damage, value),
            "ExtraDOTDamageAddedRatio1" => add(&mut self.dot_damage, value),
            "BreakDamageAddedRatioBase" | "BreakDamageExtraAddedRatio" => {
                add(&mut self.break_damage, value)
            }
            "ExtraInsertDamageAddedRatio1" => add(&mut self.additional_damage, value),
            "ElationDamageAddedRatioBase" => add(&mut self.elation_damage, value),
            "ExtraHealAddedRatio" | "HealRatioBase" => add(&mut self.healing, value),
            "ExtraShieldAddedRatio" => add(&mut self.shield, value),
            "AllDamageTypePenetrate" => add(&mut self.penetration, value),
            "ExtraAllDamageReduce" => add(&mut self.mitigation, value),
            "ExtraLuckChance" => add(&mut self.luck_chance, value),
            "ExtraLuckDamage" => add(&mut self.luck_damage, value),
            _ => Err(error("Currency Wars static property kind is unsupported")),
        }
    }
}

struct Compiler<'a> {
    builder: &'a mut CombatCatalogBuilder,
    root: [u8; 32],
    next: u32,
}

struct FormulaBinding<'a> {
    stage: FormulaStage,
    value: Scalar,
    purposes: &'a [FormulaPurpose],
    filters: Vec<ModifierFilter>,
    subject: FormulaSubject,
    discriminator: u64,
    include_identity: bool,
}

impl<'a> Compiler<'a> {
    const fn new(builder: &'a mut CombatCatalogBuilder, root: [u8; 32]) -> Self {
        Self {
            builder,
            root,
            next: DEFINITION_BASE,
        }
    }

    fn compile(
        &mut self,
        role: &CurrencyWarsRoleContribution,
        base: &ResolvedCombatantSpec,
        totals: RoleTotals,
    ) -> Result<(ResolvedCombatantSpec, u16, bool), CurrencyWarsBattleAssemblyError> {
        let mut replacement = base.clone();
        let mut bindings = 0_u16;
        let discriminator = u64::from(role.role.id.get()) << 8;
        let front_power = totals
            .front_power_base
            .checked_mul(
                Scalar::ONE
                    .checked_add(totals.front_power_bonus)
                    .map_err(debug_error)?,
                Rounding::NearestTiesEven,
            )
            .map_err(debug_error)?;
        for (stat, stage, value) in [
            (StatKind::Hp, FormulaStage::PercentOfBase, totals.hp),
            (StatKind::Atk, FormulaStage::PercentOfBase, totals.attack),
            (StatKind::Spd, FormulaStage::PercentOfBase, totals.speed),
            (StatKind::CritRate, FormulaStage::Flat, totals.crit_rate),
            (StatKind::CritDamage, FormulaStage::Flat, totals.crit_damage),
            (
                StatKind::EffectResistance,
                FormulaStage::Flat,
                totals.effect_resistance,
            ),
            (
                StatKind::EnergyRegenerationRate,
                FormulaStage::Flat,
                totals.energy_regeneration,
            ),
            (
                StatKind::ToughnessDamage,
                FormulaStage::Flat,
                totals.toughness_damage,
            ),
        ] {
            if value != Scalar::ZERO {
                replacement = self.attach_stat(&replacement, stat, stage, value, discriminator)?;
                bindings = increment(bindings)?;
            }
        }
        replacement = self.attach_formula_set(
            replacement,
            FormulaBinding {
                stage: FormulaStage::FinalMultiply,
                value: front_power,
                purposes: &DAMAGE_PURPOSES,
                filters: Vec::new(),
                subject: FormulaSubject::Source,
                discriminator,
                include_identity: true,
            },
            &mut bindings,
        )?;
        replacement = self.attach_formula_set(
            replacement,
            FormulaBinding {
                stage: FormulaStage::DamageBoost,
                value: totals.all_damage,
                purposes: &DAMAGE_PURPOSES,
                filters: Vec::new(),
                subject: FormulaSubject::Source,
                discriminator: discriminator + 1,
                include_identity: false,
            },
            &mut bindings,
        )?;
        for (value, purpose, tag, offset) in [
            (
                totals.basic_damage,
                FormulaPurpose::OrdinaryDamage,
                Some("basic"),
                2,
            ),
            (
                totals.skill_damage,
                FormulaPurpose::OrdinaryDamage,
                Some("skill"),
                3,
            ),
            (
                totals.ultimate_damage,
                FormulaPurpose::OrdinaryDamage,
                Some("ultimate"),
                4,
            ),
            (totals.dot_damage, FormulaPurpose::Dot, None, 5),
            (
                totals.additional_damage,
                FormulaPurpose::AdditionalDamage,
                None,
                6,
            ),
            (
                totals.elation_damage,
                FormulaPurpose::ElationDamage,
                None,
                7,
            ),
        ] {
            let filters = tag.map_or_else(Vec::new, |kind| {
                vec![ModifierFilter::AbilityTag(kind.into())]
            });
            replacement = self.attach_formula_set(
                replacement,
                FormulaBinding {
                    stage: FormulaStage::DamageBoost,
                    value,
                    purposes: &[purpose],
                    filters,
                    subject: FormulaSubject::Source,
                    discriminator: discriminator + offset,
                    include_identity: false,
                },
                &mut bindings,
            )?;
        }
        replacement = self.attach_formula_set(
            replacement,
            FormulaBinding {
                stage: FormulaStage::DamageBoost,
                value: totals.break_damage,
                purposes: &[FormulaPurpose::Break, FormulaPurpose::SuperBreak],
                filters: Vec::new(),
                subject: FormulaSubject::Source,
                discriminator: discriminator + 8,
                include_identity: false,
            },
            &mut bindings,
        )?;
        for (stage, purpose, value, subject, offset) in [
            (
                FormulaStage::Healing,
                FormulaPurpose::Healing,
                totals.healing,
                FormulaSubject::Source,
                9,
            ),
            (
                FormulaStage::Shield,
                FormulaPurpose::Shield,
                totals.shield,
                FormulaSubject::Source,
                10,
            ),
            (
                FormulaStage::Probability,
                FormulaPurpose::SecondaryProcChance,
                totals.luck_chance,
                FormulaSubject::Source,
                11,
            ),
            (
                FormulaStage::DamageBoost,
                FormulaPurpose::AdditionalDamage,
                totals.luck_damage,
                FormulaSubject::Source,
                12,
            ),
        ] {
            replacement = self.attach_formula_set(
                replacement,
                FormulaBinding {
                    stage,
                    value,
                    purposes: &[purpose],
                    filters: Vec::new(),
                    subject,
                    discriminator: discriminator + offset,
                    include_identity: false,
                },
                &mut bindings,
            )?;
        }
        replacement = self.attach_formula_set(
            replacement,
            FormulaBinding {
                stage: FormulaStage::Resistance,
                value: totals.penetration,
                purposes: &DAMAGE_PURPOSES,
                filters: Vec::new(),
                subject: FormulaSubject::Source,
                discriminator: discriminator + 13,
                include_identity: false,
            },
            &mut bindings,
        )?;
        replacement = self.attach_formula_set(
            replacement,
            FormulaBinding {
                stage: FormulaStage::Mitigation,
                value: totals.mitigation,
                purposes: &DAMAGE_PURPOSES,
                filters: Vec::new(),
                subject: FormulaSubject::Target,
                discriminator: discriminator + 14,
                include_identity: false,
            },
            &mut bindings,
        )?;
        let energy_added = totals.entry_energy != Scalar::ZERO;
        if energy_added {
            let current = replacement
                .current_energy()
                .scaled()
                .checked_add(totals.entry_energy.scaled())
                .ok_or_else(|| error("Currency Wars entry Energy overflow"))?
                .min(replacement.maximum_energy().scaled());
            let maximum = replacement.maximum_energy();
            replacement = replacement
                .with_energy(Energy::from_scaled(current).map_err(debug_error)?, maximum)
                .map_err(debug_error)?;
        }
        Ok((replacement, bindings, energy_added))
    }

    fn attach_stat(
        &mut self,
        base: &ResolvedCombatantSpec,
        stat: StatKind,
        stage: FormulaStage,
        value: Scalar,
        discriminator: u64,
    ) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
        let (group, source, digest) = self.identities(stage, value, discriminator)?;
        let id = self.modifier_id()?;
        self.builder.add_modifier_group(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        self.builder.add_modifier(ModifierDefinition {
            id,
            stat,
            stage,
            purpose: FormulaPurpose::Stat,
            value: literal(value),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
        attach_modifier(
            base,
            id,
            source,
            b"starclock.currency-wars.static-contribution.v1",
            digest,
        )
    }

    fn attach_formula_set(
        &mut self,
        mut base: ResolvedCombatantSpec,
        binding: FormulaBinding<'_>,
        bindings: &mut u16,
    ) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
        if binding.value == Scalar::ZERO || binding.include_identity && binding.value == Scalar::ONE
        {
            return Ok(base);
        }
        let (group, source, digest) =
            self.identities(binding.stage, binding.value, binding.discriminator)?;
        self.builder.add_modifier_group(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        let mut filters = binding.filters;
        filters.push(ModifierFilter::FormulaSubject(binding.subject));
        for purpose in binding.purposes {
            let id = self.modifier_id()?;
            self.builder.add_modifier(ModifierDefinition {
                id,
                stat: StatKind::Atk,
                stage: binding.stage,
                purpose: *purpose,
                value: literal(binding.value),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: binding.stage,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: filters.clone().into_boxed_slice(),
            });
            base = attach_modifier(
                &base,
                id,
                source.clone(),
                b"starclock.currency-wars.static-contribution.v1",
                digest,
            )?;
            *bindings = increment(*bindings)?;
        }
        Ok(base)
    }

    fn identities(
        &mut self,
        stage: FormulaStage,
        value: Scalar,
        discriminator: u64,
    ) -> Result<(ModifierStackingGroupId, RuleSource, [u8; 32]), CurrencyWarsBattleAssemblyError>
    {
        let group = ModifierStackingGroupId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars contribution group ID is invalid"))?;
        let source_id = SourceDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars contribution source ID is invalid"))?;
        let digest = contribution_digest(self.root, stage, value, discriminator);
        Ok((
            group,
            RuleSource::new(source_id, SourceClass::Mode, Vec::new(), digest),
            digest,
        ))
    }

    fn modifier_id(&mut self) -> Result<ModifierDefinitionId, CurrencyWarsBattleAssemblyError> {
        ModifierDefinitionId::new(self.take_id()?)
            .ok_or_else(|| error("Currency Wars contribution modifier ID is invalid"))
    }

    fn take_id(&mut self) -> Result<u32, CurrencyWarsBattleAssemblyError> {
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| error("Currency Wars contribution definition ID overflow"))?;
        Ok(id)
    }
}

fn add(target: &mut Scalar, value: Scalar) -> Result<(), CurrencyWarsBattleAssemblyError> {
    *target = target.checked_add(value).map_err(debug_error)?;
    Ok(())
}

fn literal(value: Scalar) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(value))
}

fn increment(value: u16) -> Result<u16, CurrencyWarsBattleAssemblyError> {
    value
        .checked_add(1)
        .ok_or_else(|| error("Currency Wars contribution receipt overflow"))
}

fn count(values: impl IntoIterator<Item = usize>) -> Result<u16, CurrencyWarsBattleAssemblyError> {
    values.into_iter().try_fold(0_u16, |total, value| {
        total
            .checked_add(u16::try_from(value).map_err(debug_error)?)
            .ok_or_else(|| error("Currency Wars contribution receipt overflow"))
    })
}

fn contribution_digest(
    root: [u8; 32],
    stage: FormulaStage,
    value: Scalar,
    discriminator: u64,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.static-contribution.v1");
    hash.update(root);
    hash.update([stage as u8]);
    hash.update(value.scaled().to_le_bytes());
    hash.update(discriminator.to_le_bytes());
    hash.finalize().into()
}
