//! Canonical Standard Universe snapshot-to-combat contribution compilation.

use std::{collections::BTreeSet, sync::Arc};

use starclock_combat::{
    ModifierDefinitionId, ModifierStackingGroupId, RuleBundleId, RuleId, Scalar,
    SourceDefinitionId,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};

use crate::{
    ability_runtime::{AbilityRuntimeProjection, AbilityTarget, AbilityValue},
    blessing_runtime::BlessingContributionSet,
    catalog::UniverseCatalog,
    curio_runtime::CurioContributionSet,
    digest::Encoder,
    id::PathId,
    path::ResonanceKind,
    path_runtime::PathContributionSet,
    progression::AbilityEffectClass,
    rule::{MechanicRuleDefinition, MechanicRuleKind},
    run_runtime::AbilityTreeContributionSet,
};

pub const UNIVERSE_BATTLE_CONTRIBUTION_REVISION: &str = "standard-universe-battle-contribution-v1";

const RULE_ID_BASE: u32 = 0x7000_0000;
const BUNDLE_ID_BASE: u32 = 0x7100_0000;
const RULE_SOURCE_ID_BASE: u32 = 0x7200_0000;
const MODIFIER_SOURCE_ID_BASE: u32 = 0x7280_0000;
const MODIFIER_ID_BASE: u32 = 0x7300_0000;
const MODIFIER_GROUP_ID_BASE: u32 = 0x7400_0000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum UniverseBattleRuleRole {
    BlessingDefinition = 0,
    BlessingLevel = 1,
    Resonance = 2,
    Formation = 3,
    CurioDefinition = 4,
    CurioState = 5,
    AbilityTree = 6,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseBattleRuleBinding {
    role: UniverseBattleRuleRole,
    rule: RuleId,
    bundle: RuleBundleId,
    source: RuleSource,
    stable_key: Box<str>,
    source_record_key: Box<str>,
    source_binding_key: Option<Box<str>>,
    mechanic_tags: Box<[Box<str>]>,
}

impl UniverseBattleRuleBinding {
    #[must_use]
    pub const fn role(&self) -> UniverseBattleRuleRole {
        self.role
    }

    #[must_use]
    pub const fn rule(&self) -> RuleId {
        self.rule
    }

    #[must_use]
    pub const fn bundle(&self) -> RuleBundleId {
        self.bundle
    }

    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub fn source_record_key(&self) -> &str {
        &self.source_record_key
    }

    #[must_use]
    pub fn source_binding_key(&self) -> Option<&str> {
        self.source_binding_key.as_deref()
    }

    #[must_use]
    pub fn mechanic_tags(&self) -> &[Box<str>] {
        &self.mechanic_tags
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseBattleModifierBinding {
    target: AbilityTarget,
    value: AbilityValue,
    group: ModifierStackingGroup,
    definition: ModifierDefinition,
    source: RuleSource,
}

impl UniverseBattleModifierBinding {
    #[must_use]
    pub const fn target(&self) -> AbilityTarget {
        self.target
    }

    #[must_use]
    pub const fn value(&self) -> AbilityValue {
        self.value
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UniverseBattleBoundaryValue {
    target: AbilityTarget,
    value: AbilityValue,
}

impl UniverseBattleBoundaryValue {
    #[must_use]
    pub const fn target(self) -> AbilityTarget {
        self.target
    }

    #[must_use]
    pub const fn value(self) -> AbilityValue {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseBattleContributionSet {
    selected_path: PathId,
    selected_path_blessings: u8,
    path_digest: [u8; 32],
    rules: Box<[UniverseBattleRuleBinding]>,
    modifiers: Box<[UniverseBattleModifierBinding]>,
    boundary_values: Box<[UniverseBattleBoundaryValue]>,
    digest: [u8; 32],
}

impl UniverseBattleContributionSet {
    #[must_use]
    pub const fn selected_path(&self) -> PathId {
        self.selected_path
    }

    #[must_use]
    pub const fn selected_path_blessings(&self) -> u8 {
        self.selected_path_blessings
    }

    #[must_use]
    pub const fn path_digest(&self) -> [u8; 32] {
        self.path_digest
    }

    #[must_use]
    pub fn rules(&self) -> &[UniverseBattleRuleBinding] {
        &self.rules
    }

    #[must_use]
    pub fn modifiers(&self) -> &[UniverseBattleModifierBinding] {
        &self.modifiers
    }

    #[must_use]
    pub fn boundary_values(&self) -> &[UniverseBattleBoundaryValue] {
        &self.boundary_values
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub struct UniverseBattleContributionCompiler {
    catalog: Arc<UniverseCatalog>,
    rules: Box<[MechanicRuleDefinition]>,
    digest: [u8; 32],
}

impl UniverseBattleContributionCompiler {
    pub fn compile(catalog: Arc<UniverseCatalog>) -> Result<Self, UniverseBattleContributionError> {
        let mut rules = catalog.mechanic_rules().to_vec();
        rules.sort_unstable_by_key(MechanicRuleDefinition::id);
        if rules.len() != 786
            || rules.windows(2).any(|pair| pair[0].id() >= pair[1].id())
            || rules
                .iter()
                .any(|rule| rule.stable_key().is_empty() || rule.source_record_key().is_empty())
        {
            return Err(UniverseBattleContributionError::InvalidCatalog);
        }
        let mut source_keys = BTreeSet::new();
        if rules
            .iter()
            .any(|rule| !source_keys.insert((rule.kind(), rule.source_record_key().to_owned())))
        {
            return Err(UniverseBattleContributionError::DuplicateSourceRecord);
        }
        validate_denominators(&rules)?;
        let combat = catalog.simulation_catalog().combat_catalog();
        for rule in &rules {
            let raw = rule.id().get();
            let rule_id =
                RuleId::new(RULE_ID_BASE + raw).expect("reserved rule identity is non-zero");
            let bundle_id = RuleBundleId::new(BUNDLE_ID_BASE + raw)
                .expect("reserved bundle identity is non-zero");
            if combat.rule(rule_id).is_some() || combat.rule_bundle(bundle_id).is_some() {
                return Err(UniverseBattleContributionError::IdentityCollision);
            }
        }
        for raw in 1..=22 {
            let modifier = ModifierDefinitionId::new(MODIFIER_ID_BASE + raw)
                .expect("reserved modifier identity is non-zero");
            if combat.modifier(modifier).is_some() {
                return Err(UniverseBattleContributionError::IdentityCollision);
            }
        }
        let digest = compiler_digest(&rules);
        Ok(Self {
            catalog,
            rules: rules.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compile_snapshot(
        &self,
        path: &PathContributionSet,
        blessings: &BlessingContributionSet,
        curios: &CurioContributionSet,
        abilities: &AbilityTreeContributionSet,
        ability_projection: &AbilityRuntimeProjection,
    ) -> Result<UniverseBattleContributionSet, UniverseBattleContributionError> {
        let selected_path = path.passive().path();
        if blessings
            .entries()
            .iter()
            .filter(|value| value.path() == selected_path)
            .count()
            != usize::from(path.selected_path_blessings())
        {
            return Err(UniverseBattleContributionError::SnapshotMismatch);
        }
        let mut rules = Vec::new();
        for blessing in blessings.entries() {
            let definition = self
                .catalog
                .blessing(blessing.blessing())
                .ok_or(UniverseBattleContributionError::MissingContent)?;
            let level = self
                .catalog
                .blessing_level(blessing.level().id())
                .ok_or(UniverseBattleContributionError::MissingContent)?;
            self.push_binding(
                &mut rules,
                MechanicRuleKind::BlessingDefinition,
                definition.stable_key(),
                UniverseBattleRuleRole::BlessingDefinition,
            )?;
            self.push_binding(
                &mut rules,
                MechanicRuleKind::BlessingLevel,
                level.stable_key(),
                UniverseBattleRuleRole::BlessingLevel,
            )?;
        }
        if let Some(resonance) = path.resonance() {
            self.push_resonance(&mut rules, resonance.id(), resonance.kind())?;
        }
        for formation in path.formations() {
            self.push_resonance(&mut rules, formation.id(), formation.kind())?;
        }
        for curio in curios.entries() {
            let definition = self
                .catalog
                .curio(curio.curio())
                .ok_or(UniverseBattleContributionError::MissingContent)?;
            let state = self
                .catalog
                .curio_state(curio.state().id())
                .ok_or(UniverseBattleContributionError::MissingContent)?;
            self.push_binding(
                &mut rules,
                MechanicRuleKind::CurioDefinition,
                definition.stable_key(),
                UniverseBattleRuleRole::CurioDefinition,
            )?;
            self.push_binding(
                &mut rules,
                MechanicRuleKind::CurioState,
                state.stable_key(),
                UniverseBattleRuleRole::CurioState,
            )?;
        }
        for ability in abilities.entries().iter().filter(|ability| {
            matches!(
                ability.effect_class(),
                AbilityEffectClass::Battle | AbilityEffectClass::RunAndBattle
            )
        }) {
            let definition = self
                .catalog
                .ability_tree_node(ability.id())
                .ok_or(UniverseBattleContributionError::MissingContent)?;
            self.push_binding(
                &mut rules,
                MechanicRuleKind::AbilityTreeContribution,
                definition.stable_key(),
                UniverseBattleRuleRole::AbilityTree,
            )?;
        }
        rules.sort_unstable_by_key(UniverseBattleRuleBinding::bundle);
        if rules
            .windows(2)
            .any(|pair| pair[0].bundle >= pair[1].bundle)
        {
            return Err(UniverseBattleContributionError::DuplicateBinding);
        }

        let boundary_values = ability_projection
            .values()
            .iter()
            .map(|value| UniverseBattleBoundaryValue {
                target: value.target(),
                value: value.value(),
            })
            .collect::<Vec<_>>();
        let modifiers = boundary_values
            .iter()
            .filter_map(|value| modifier_binding(*value, ability_projection.digest()))
            .collect::<Result<Vec<_>, UniverseBattleContributionError>>()?;
        let digest = contribution_digest(
            selected_path,
            path,
            blessings,
            curios,
            abilities,
            ability_projection,
            &rules,
            &modifiers,
            &boundary_values,
        );
        Ok(UniverseBattleContributionSet {
            selected_path,
            selected_path_blessings: path.selected_path_blessings(),
            path_digest: path.digest(),
            rules: rules.into_boxed_slice(),
            modifiers: modifiers.into_boxed_slice(),
            boundary_values: boundary_values.into_boxed_slice(),
            digest,
        })
    }

    fn push_resonance(
        &self,
        output: &mut Vec<UniverseBattleRuleBinding>,
        id: crate::id::ResonanceId,
        kind: ResonanceKind,
    ) -> Result<(), UniverseBattleContributionError> {
        let definition = self
            .catalog
            .resonance(id)
            .ok_or(UniverseBattleContributionError::MissingContent)?;
        let role = match kind {
            ResonanceKind::Resonance => UniverseBattleRuleRole::Resonance,
            ResonanceKind::Formation => UniverseBattleRuleRole::Formation,
        };
        self.push_binding(
            output,
            MechanicRuleKind::PathResonance,
            definition.stable_key(),
            role,
        )
    }

    fn push_binding(
        &self,
        output: &mut Vec<UniverseBattleRuleBinding>,
        kind: MechanicRuleKind,
        source_record_key: &str,
        role: UniverseBattleRuleRole,
    ) -> Result<(), UniverseBattleContributionError> {
        let rule = self
            .rules
            .iter()
            .find(|rule| rule.kind() == kind && rule.source_record_key() == source_record_key)
            .ok_or(UniverseBattleContributionError::MissingRule)?;
        output.push(binding(rule, role)?);
        Ok(())
    }
}

fn binding(
    definition: &MechanicRuleDefinition,
    role: UniverseBattleRuleRole,
) -> Result<UniverseBattleRuleBinding, UniverseBattleContributionError> {
    let raw = definition.id().get();
    let rule = RuleId::new(
        RULE_ID_BASE
            .checked_add(raw)
            .ok_or(UniverseBattleContributionError::IdentityOverflow)?,
    )
    .ok_or(UniverseBattleContributionError::IdentityOverflow)?;
    let bundle = RuleBundleId::new(
        BUNDLE_ID_BASE
            .checked_add(raw)
            .ok_or(UniverseBattleContributionError::IdentityOverflow)?,
    )
    .ok_or(UniverseBattleContributionError::IdentityOverflow)?;
    let source = SourceDefinitionId::new(
        RULE_SOURCE_ID_BASE
            .checked_add(raw)
            .ok_or(UniverseBattleContributionError::IdentityOverflow)?,
    )
    .ok_or(UniverseBattleContributionError::IdentityOverflow)?;
    Ok(UniverseBattleRuleBinding {
        role,
        rule,
        bundle,
        source: RuleSource::new(
            source,
            source_class(role),
            vec![],
            mechanic_rule_digest(definition),
        ),
        stable_key: definition.stable_key().into(),
        source_record_key: definition.source_record_key().into(),
        source_binding_key: definition.source_binding_key().map(Into::into),
        mechanic_tags: definition.mechanic_tags().to_vec().into_boxed_slice(),
    })
}

fn modifier_binding(
    value: UniverseBattleBoundaryValue,
    projection_digest: [u8; 32],
) -> Option<Result<UniverseBattleModifierBinding, UniverseBattleContributionError>> {
    if value.value == AbilityValue::ZERO {
        return None;
    }
    let (stat, stage) = match value.target {
        AbilityTarget::PartyAttackFlat => (StatKind::Atk, FormulaStage::Flat),
        AbilityTarget::PartyDefenseFlat => (StatKind::Def, FormulaStage::Flat),
        AbilityTarget::PartyMaximumHpFlat => (StatKind::Hp, FormulaStage::Flat),
        AbilityTarget::PartyCritRateRatio => (StatKind::CritRate, FormulaStage::Flat),
        AbilityTarget::PartySpeedRatio => (StatKind::Spd, FormulaStage::PercentOfBase),
        AbilityTarget::PartyCritDamageRatio => (StatKind::CritDamage, FormulaStage::Flat),
        AbilityTarget::PartyEffectHitRateRatio => (StatKind::EffectHitRate, FormulaStage::Flat),
        _ => return None,
    };
    let raw = value.target as u32 + 1;
    let definition_id = ModifierDefinitionId::new(MODIFIER_ID_BASE + raw)?;
    let group_id = ModifierStackingGroupId::new(MODIFIER_GROUP_ID_BASE + raw)?;
    let source_id = SourceDefinitionId::new(MODIFIER_SOURCE_ID_BASE + raw)?;
    Some(Ok(UniverseBattleModifierBinding {
        target: value.target,
        value: value.value,
        group: ModifierStackingGroup {
            id: group_id,
            aggregation: ModifierAggregation::ReplaceGroup,
        },
        definition: ModifierDefinition {
            id: definition_id,
            stat,
            stage,
            purpose: FormulaPurpose::Stat,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                value.value.raw_six_decimal(),
            ))),
            stacking_group: group_id,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            filters: Box::new([]),
        },
        source: RuleSource::new(
            source_id,
            SourceClass::Progression,
            vec![],
            projection_digest,
        ),
    }))
}

fn source_class(role: UniverseBattleRuleRole) -> SourceClass {
    match role {
        UniverseBattleRuleRole::AbilityTree => SourceClass::Progression,
        _ => SourceClass::Mode,
    }
}

fn validate_denominators(
    rules: &[MechanicRuleDefinition],
) -> Result<(), UniverseBattleContributionError> {
    let expected = [
        (MechanicRuleKind::AbilityTreeContribution, 42),
        (MechanicRuleKind::BlessingDefinition, 162),
        (MechanicRuleKind::BlessingLevel, 324),
        (MechanicRuleKind::CurioDefinition, 61),
        (MechanicRuleKind::CurioState, 67),
        (MechanicRuleKind::RunService, 94),
        (MechanicRuleKind::PathResonance, 36),
    ];
    if expected
        .iter()
        .any(|(kind, count)| rules.iter().filter(|rule| rule.kind() == *kind).count() != *count)
    {
        Err(UniverseBattleContributionError::InvalidCatalog)
    } else {
        Ok(())
    }
}

fn compiler_digest(rules: &[MechanicRuleDefinition]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-contribution.compiler.v1");
    encoder.text(UNIVERSE_BATTLE_CONTRIBUTION_REVISION);
    encoder.u32(u32::try_from(rules.len()).expect("frozen rule count fits u32"));
    for rule in rules {
        encoder.digest(mechanic_rule_digest(rule));
    }
    encoder.finish()
}

fn mechanic_rule_digest(rule: &MechanicRuleDefinition) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-rule-binding.v1");
    encoder.u32(rule.id().get());
    encoder.u8(rule.kind() as u8);
    encoder.text(rule.stable_key());
    encoder.text(rule.source_record_key());
    encoder.text(rule.source_file());
    encoder.optional_text(rule.native_handler_key());
    encoder.optional_text(rule.source_binding_key());
    encoder.u32(u32::try_from(rule.parameters().len()).expect("bounded parameters"));
    for parameter in rule.parameters() {
        match (parameter.index(), parameter.key()) {
            (Some(index), None) => {
                encoder.u8(0);
                encoder.u32(index);
            }
            (None, Some(key)) => {
                encoder.u8(1);
                encoder.text(key);
            }
            _ => encoder.u8(u8::MAX),
        }
        encoder.text(parameter.value());
    }
    encoder.u32(u32::try_from(rule.mechanic_tags().len()).expect("bounded tags"));
    for tag in rule.mechanic_tags() {
        encoder.text(tag);
    }
    encoder.optional_text(rule.approximation_replacement_condition());
    encoder.finish()
}

#[allow(clippy::too_many_arguments)]
fn contribution_digest(
    selected_path: PathId,
    path: &PathContributionSet,
    blessings: &BlessingContributionSet,
    curios: &CurioContributionSet,
    abilities: &AbilityTreeContributionSet,
    projection: &AbilityRuntimeProjection,
    rules: &[UniverseBattleRuleBinding],
    modifiers: &[UniverseBattleModifierBinding],
    boundary_values: &[UniverseBattleBoundaryValue],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-contribution.set.v1");
    encoder.text(UNIVERSE_BATTLE_CONTRIBUTION_REVISION);
    encoder.u32(selected_path.get());
    encoder.digest(path.digest());
    encoder.digest(blessings.digest());
    encoder.digest(curios.digest());
    encoder.digest(abilities.digest());
    encoder.digest(projection.digest());
    encoder.u32(u32::try_from(rules.len()).expect("bounded rules"));
    for rule in rules {
        encoder.u32(rule.bundle.get());
        encoder.u32(rule.rule.get());
        encoder.u32(rule.source.definition().get());
        encoder.digest(rule.source.digest());
        encoder.u8(rule.role as u8);
    }
    encoder.u32(u32::try_from(modifiers.len()).expect("bounded modifiers"));
    for modifier in modifiers {
        encoder.u32(modifier.definition.id.get());
        encoder.u32(modifier.group.id.get());
        encoder.u32(modifier.source.definition().get());
        encoder.u8(modifier.target as u8);
        encoder.i64(modifier.value.raw_six_decimal());
    }
    encoder.u32(u32::try_from(boundary_values.len()).expect("bounded boundary values"));
    for value in boundary_values {
        encoder.u8(value.target as u8);
        encoder.i64(value.value.raw_six_decimal());
    }
    encoder.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseBattleContributionError {
    InvalidCatalog,
    DuplicateSourceRecord,
    MissingContent,
    MissingRule,
    SnapshotMismatch,
    DuplicateBinding,
    IdentityOverflow,
    IdentityCollision,
}

impl core::fmt::Display for UniverseBattleContributionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Universe battle contribution error: {self:?}")
    }
}

impl std::error::Error for UniverseBattleContributionError {}
