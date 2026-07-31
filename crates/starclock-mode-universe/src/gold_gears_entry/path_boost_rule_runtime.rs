//! Terminal dispatch for frozen Path-boost and inherited Blessing rules.

use starclock_activity::ActivityTransactionState;
use starclock_combat::{
    ModifierDefinitionId, ModifierStackingGroupId, Scalar, SourceDefinitionId,
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};

use crate::{
    blessing_runtime::BlessingRuntimeCatalog, catalog::UniverseCatalog, digest::Encoder,
    gold_gears_content::GoldAndGearsContentCatalog, gold_gears_unique::GoldAndGearsUniqueCatalog,
    rule::MechanicRuleKind,
};

use super::{
    GoldAndGearsEntryError, GoldAndGearsPathBoostStat,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
};

pub const GOLD_AND_GEARS_PATH_BOOST_EXECUTION_REVISION: &str =
    "gold-and-gears-path-boost-execution-v1";

const MODIFIER_BASE: u32 = 0x7f20_0000;
const GROUP_BASE: u32 = 0x7f21_0000;
const SOURCE_BASE: u32 = 0x7f22_0000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsPathBoostRuleKind {
    PathBoost = 0,
    BlessingDefinition = 1,
    BlessingLevel = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsPathBoostRuleOwnership {
    GoldAndGears = 0,
    Shared = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsPathBoostRuleBinding {
    rule_id: Box<str>,
    owner_id: Box<str>,
    kind: GoldAndGearsPathBoostRuleKind,
    ownership: GoldAndGearsPathBoostRuleOwnership,
}

impl GoldAndGearsPathBoostRuleBinding {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn kind(&self) -> GoldAndGearsPathBoostRuleKind {
        self.kind
    }

    #[must_use]
    pub const fn ownership(&self) -> GoldAndGearsPathBoostRuleOwnership {
        self.ownership
    }

    #[must_use]
    pub const fn executor(&self) -> &'static str {
        match self.ownership {
            GoldAndGearsPathBoostRuleOwnership::GoldAndGears => "CombatRuleIr",
            GoldAndGearsPathBoostRuleOwnership::Shared => "ReleasedSharedExecutor",
        }
    }

    #[must_use]
    pub const fn accuracy(&self) -> &'static str {
        "ExactPublic"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsPathBoostCombatBinding {
    source_rule_id: Box<str>,
    owner_id: Box<str>,
    stat: GoldAndGearsPathBoostStat,
    ratio_scaled: i64,
    groups: Box<[ModifierStackingGroup]>,
    definitions: Box<[ModifierDefinition]>,
    source: RuleSource,
}

impl GoldAndGearsPathBoostCombatBinding {
    #[must_use]
    pub fn source_rule_id(&self) -> &str {
        &self.source_rule_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn stat(&self) -> GoldAndGearsPathBoostStat {
        self.stat
    }

    #[must_use]
    pub const fn ratio_scaled(&self) -> i64 {
        self.ratio_scaled
    }

    #[must_use]
    pub fn groups(&self) -> &[ModifierStackingGroup] {
        &self.groups
    }

    #[must_use]
    pub fn definitions(&self) -> &[ModifierDefinition] {
        &self.definitions
    }

    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsPathBoostCombatSet {
    binding: GoldAndGearsPathBoostCombatBinding,
    digest: [u8; 32],
}

impl GoldAndGearsPathBoostCombatSet {
    #[must_use]
    pub const fn binding(&self) -> &GoldAndGearsPathBoostCombatBinding {
        &self.binding
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsPathBoostRuleRuntimeCatalog {
    bindings: Box<[GoldAndGearsPathBoostRuleBinding]>,
    digest: [u8; 32],
}

impl GoldAndGearsPathBoostRuleRuntimeCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
        unique: &GoldAndGearsUniqueCatalog,
        standard: &UniverseCatalog,
        blessings: &BlessingRuntimeCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut bindings = Vec::with_capacity(495);
        for boost in &unique.path_boosts {
            if !boost
                .rule_contribution
                .starts_with("gold-gears.rule.path-boost.")
            {
                return Err(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime);
            }
            bindings.push(binding(
                &boost.rule_contribution,
                &boost.identity.stable_key,
                GoldAndGearsPathBoostRuleKind::PathBoost,
                GoldAndGearsPathBoostRuleOwnership::GoldAndGears,
            )?);
        }
        for authored in &content.blessings {
            let released = standard
                .blessings()
                .iter()
                .find(|candidate| candidate.stable_key() == authored.key.as_str())
                .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
            if authored.inherited_rules.len() != 1
                || authored.inherited_rules[0].as_str() != released.rule_key()
                || blessings.definition(released.id()).is_none()
                || !released_rule(
                    standard,
                    released.rule_key(),
                    released.stable_key(),
                    MechanicRuleKind::BlessingDefinition,
                )
            {
                return Err(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime);
            }
            bindings.push(binding(
                released.rule_key(),
                released.stable_key(),
                GoldAndGearsPathBoostRuleKind::BlessingDefinition,
                GoldAndGearsPathBoostRuleOwnership::Shared,
            )?);
        }
        for authored in &content.blessing_levels {
            let released = standard
                .blessing_levels()
                .iter()
                .find(|candidate| candidate.stable_key() == authored.key.as_str())
                .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
            let runtime = blessings
                .definition(released.blessing())
                .and_then(|definition| definition.level(released.level()))
                .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
            if authored.inherited_rules.len() != 1
                || authored.inherited_rules[0].as_str() != released.rule_key()
                || runtime.rule_key() != released.rule_key()
                || !released_rule(
                    standard,
                    released.rule_key(),
                    released.stable_key(),
                    MechanicRuleKind::BlessingLevel,
                )
            {
                return Err(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime);
            }
            bindings.push(binding(
                released.rule_key(),
                released.stable_key(),
                GoldAndGearsPathBoostRuleKind::BlessingLevel,
                GoldAndGearsPathBoostRuleOwnership::Shared,
            )?);
        }
        bindings.sort_unstable_by(|left, right| left.rule_id.cmp(&right.rule_id));
        if bindings.len() != 495
            || bindings
                .windows(2)
                .any(|pair| pair[0].rule_id >= pair[1].rule_id)
            || count_kind(&bindings, GoldAndGearsPathBoostRuleKind::PathBoost) != 9
            || count_kind(&bindings, GoldAndGearsPathBoostRuleKind::BlessingDefinition) != 162
            || count_kind(&bindings, GoldAndGearsPathBoostRuleKind::BlessingLevel) != 324
        {
            return Err(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime);
        }
        let digest = execution_digest(&bindings);
        Ok(Self {
            bindings: bindings.into_boxed_slice(),
            digest,
        })
    }

    pub(super) fn bindings(&self) -> &[GoldAndGearsPathBoostRuleBinding] {
        &self.bindings
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn path_boost_rule_bindings(&self) -> &[GoldAndGearsPathBoostRuleBinding] {
        self.content_runtime.path_boost_rules.bindings()
    }

    #[must_use]
    pub fn path_boost_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.path_boost_rules.digest()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn path_boost_rule_bindings(&self) -> &[GoldAndGearsPathBoostRuleBinding] {
        self.content_runtime.path_boost_rules.bindings()
    }

    #[must_use]
    pub fn path_boost_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.path_boost_rules.digest()
    }

    pub fn compile_path_boost_combat_set(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<GoldAndGearsPathBoostCombatSet, GoldAndGearsEntryError> {
        let contribution = self.path_boost_contribution(state)?;
        let terminal = self
            .path_boost_rule_bindings()
            .iter()
            .find(|binding| {
                binding.kind == GoldAndGearsPathBoostRuleKind::PathBoost
                    && binding.owner_id.as_ref() == contribution.source_boost()
            })
            .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
        let ordinal = terminal
            .rule_id
            .strip_prefix("gold-gears.rule.path-boost.")
            .and_then(|value| value.parse::<u32>().ok())
            .and_then(|value| value.checked_sub(650_099))
            .filter(|value| (1..=9).contains(value))
            .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
        let source_id = SourceDefinitionId::new(SOURCE_BASE + ordinal)
            .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
        let specs = modifier_specs(contribution.stat());
        let mut groups = Vec::with_capacity(specs.len());
        let mut definitions = Vec::with_capacity(specs.len());
        for (index, spec) in specs.into_iter().enumerate() {
            let offset = ordinal
                .checked_mul(32)
                .and_then(|value| value.checked_add(u32::try_from(index).ok()?))
                .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
            let group_id = ModifierStackingGroupId::new(GROUP_BASE + offset)
                .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?;
            groups.push(ModifierStackingGroup {
                id: group_id,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            });
            definitions.push(ModifierDefinition {
                id: ModifierDefinitionId::new(MODIFIER_BASE + offset)
                    .ok_or(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime)?,
                stat: spec.stat,
                stage: spec.stage,
                purpose: spec.purpose,
                value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                    contribution.ratio_scaled(),
                ))),
                stacking_group: group_id,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: spec.stage,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: spec.filters,
            });
        }
        let source = RuleSource::new(
            source_id,
            SourceClass::Mode,
            vec![],
            source_digest(terminal, contribution.stat(), contribution.ratio_scaled()),
        );
        let binding = GoldAndGearsPathBoostCombatBinding {
            source_rule_id: terminal.rule_id.clone(),
            owner_id: terminal.owner_id.clone(),
            stat: contribution.stat(),
            ratio_scaled: contribution.ratio_scaled(),
            groups: groups.into_boxed_slice(),
            definitions: definitions.into_boxed_slice(),
            source,
        };
        let digest = combat_set_digest(&binding);
        Ok(GoldAndGearsPathBoostCombatSet { binding, digest })
    }
}

struct ModifierSpec {
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    filters: Box<[ModifierFilter]>,
}

fn modifier_specs(stat: GoldAndGearsPathBoostStat) -> Vec<ModifierSpec> {
    let stat_spec = |stat, stage, purpose| ModifierSpec {
        stat,
        stage,
        purpose,
        filters: Box::new([]),
    };
    let formula_spec = |purpose, tag: Option<&str>| {
        let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)];
        if let Some(tag) = tag {
            filters.push(ModifierFilter::AbilityTag(tag.into()));
        }
        ModifierSpec {
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose,
            filters: filters.into_boxed_slice(),
        }
    };
    match stat {
        GoldAndGearsPathBoostStat::ShieldGain => vec![ModifierSpec {
            stat: StatKind::ShieldStrength,
            stage: FormulaStage::Shield,
            purpose: FormulaPurpose::Shield,
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)]
                .into_boxed_slice(),
        }],
        GoldAndGearsPathBoostStat::EffectHitRate => vec![stat_spec(
            StatKind::EffectHitRate,
            FormulaStage::Flat,
            FormulaPurpose::Stat,
        )],
        GoldAndGearsPathBoostStat::DamageOverTime => {
            vec![formula_spec(FormulaPurpose::Dot, None)]
        }
        GoldAndGearsPathBoostStat::OutgoingHealing => vec![stat_spec(
            StatKind::OutgoingHealing,
            FormulaStage::Flat,
            FormulaPurpose::Stat,
        )],
        GoldAndGearsPathBoostStat::CriticalDamage => vec![stat_spec(
            StatKind::CritDamage,
            FormulaStage::Flat,
            FormulaPurpose::Stat,
        )],
        GoldAndGearsPathBoostStat::DamageDealt => [
            FormulaPurpose::OrdinaryDamage,
            FormulaPurpose::Dot,
            FormulaPurpose::Break,
            FormulaPurpose::SuperBreak,
            FormulaPurpose::AdditionalDamage,
            FormulaPurpose::JointDamage,
            FormulaPurpose::ElationDamage,
            FormulaPurpose::TrueDamage,
        ]
        .into_iter()
        .map(|purpose| formula_spec(purpose, None))
        .collect(),
        GoldAndGearsPathBoostStat::FollowUpAttackDamage => {
            vec![formula_spec(
                FormulaPurpose::OrdinaryDamage,
                Some("follow_up"),
            )]
        }
        GoldAndGearsPathBoostStat::BasicAttackDamage => {
            vec![formula_spec(FormulaPurpose::OrdinaryDamage, Some("basic"))]
        }
        GoldAndGearsPathBoostStat::UltimateDamage => {
            vec![formula_spec(
                FormulaPurpose::OrdinaryDamage,
                Some("ultimate"),
            )]
        }
    }
}

fn binding(
    rule_id: &str,
    owner_id: &str,
    kind: GoldAndGearsPathBoostRuleKind,
    ownership: GoldAndGearsPathBoostRuleOwnership,
) -> Result<GoldAndGearsPathBoostRuleBinding, GoldAndGearsEntryError> {
    if rule_id.is_empty()
        || !owner_id.starts_with("universe.") && !owner_id.starts_with("gold-gears.")
    {
        return Err(GoldAndGearsEntryError::InvalidPathBoostRuleRuntime);
    }
    Ok(GoldAndGearsPathBoostRuleBinding {
        rule_id: rule_id.into(),
        owner_id: owner_id.into(),
        kind,
        ownership,
    })
}

fn released_rule(
    catalog: &UniverseCatalog,
    rule_id: &str,
    owner_id: &str,
    kind: MechanicRuleKind,
) -> bool {
    catalog.mechanic_rules().iter().any(|rule| {
        rule.stable_key() == rule_id && rule.source_record_key() == owner_id && rule.kind() == kind
    })
}

fn count_kind(
    bindings: &[GoldAndGearsPathBoostRuleBinding],
    kind: GoldAndGearsPathBoostRuleKind,
) -> usize {
    bindings
        .iter()
        .filter(|binding| binding.kind == kind)
        .count()
}

fn execution_digest(bindings: &[GoldAndGearsPathBoostRuleBinding]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-path-boost-execution-v1");
    encoder.text(GOLD_AND_GEARS_PATH_BOOST_EXECUTION_REVISION);
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.text(&binding.rule_id);
        encoder.text(&binding.owner_id);
        encoder.u8(binding.kind as u8);
        encoder.u8(binding.ownership as u8);
    }
    encoder.finish()
}

fn source_digest(
    binding: &GoldAndGearsPathBoostRuleBinding,
    stat: GoldAndGearsPathBoostStat,
    ratio_scaled: i64,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-path-boost-source-v1");
    encoder.text(&binding.rule_id);
    encoder.text(&binding.owner_id);
    encoder.u8(stat as u8);
    encoder.i64(ratio_scaled);
    encoder.finish()
}

fn combat_set_digest(binding: &GoldAndGearsPathBoostCombatBinding) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-path-boost-combat-set-v1");
    encoder.text(&binding.source_rule_id);
    encoder.text(&binding.owner_id);
    encoder.u8(binding.stat as u8);
    encoder.i64(binding.ratio_scaled);
    encoder.u32(binding.definitions.len() as u32);
    for definition in &binding.definitions {
        encoder.u32(definition.id.get());
        encoder.u32(definition.stacking_group.get());
        encoder.u8(definition.stage as u8);
        encoder.u8(definition.purpose as u8);
        encoder.u32(definition.filters.len() as u32);
    }
    encoder.finish()
}
