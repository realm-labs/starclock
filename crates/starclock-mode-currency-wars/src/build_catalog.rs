use std::collections::BTreeSet;

use starclock_build::{
    ability::AbilityInvestment,
    spec::CombatantBuildSpec,
    substitution::{
        BuildSubstitutionError, OwnedBuildMinimumFacts, SubstitutedBuild, substitute_owned_or_trial,
    },
};
#[cfg(test)]
use starclock_build::{
    ability::AbilityLevel,
    light_cone::{LightConeLevel, Superimposition},
    spec::{EidolonLevel, LightConeLoadout, PromotionStage},
};
use starclock_combat::{AbilityId, ResolvedCombatantSpec};
#[cfg(test)]
use starclock_combat::{
    CombatantSpecDigest, Hp, ResolvedDefinitionBindings, Speed, UnitDefinitionId, UnitLevel,
};

#[cfg(test)]
use crate::CurrencyWarsEquipmentDressRule;
use crate::equipment::resolve_off_field_contributions;
use crate::{
    CurrencyWarsEquipmentCategory, CurrencyWarsEquipmentCategoryLimit, CurrencyWarsEquipmentId,
    CurrencyWarsOffFieldContributionSnapshot, CurrencyWarsOffFieldEligibility,
    CurrencyWarsOffFieldPayload, CurrencyWarsPositionKind, CurrencyWarsRoleId,
    CurrencyWarsRuntimeEquipment,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBuildMinimum {
    AccountOrModeMinimum,
    AccountOrMappedMinimum,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBuildMapping {
    pub stable_key: Box<str>,
    pub role: CurrencyWarsRoleId,
    pub avatar_id: u32,
    pub special_avatar_id: u32,
    pub level: CurrencyWarsBuildMinimum,
    pub trace_state: CurrencyWarsBuildMinimum,
    pub light_cone: CurrencyWarsBuildMinimum,
    pub relics: CurrencyWarsBuildMinimum,
    pub mutates_account: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBuildReference {
    pub stable_key: Box<str>,
    pub role: CurrencyWarsRoleId,
    pub avatar_id: u32,
    pub owned_build_id: Box<str>,
    pub trial_build_id: Box<str>,
    pub in_pool: bool,
}

/// Exact immutable mapped minimum compiled from one released special-avatar row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsTrialBuild {
    pub stable_key: Box<str>,
    pub role: CurrencyWarsRoleId,
    pub avatar_id: u32,
    pub special_avatar_id: u32,
    pub world_level: u8,
    pub skill_tree_key: Box<str>,
    pub relic_property_type: u32,
    pub relic_property_type_extra: u32,
    pub relic_main_value: u32,
    pub relic_sub_value: u32,
    pub relic_sets: Box<[CurrencyWarsRelicSetThreshold]>,
    pub source_ability_bindings: Box<[CurrencyWarsSourceAbilityBinding]>,
    pub effective_ability_levels: Box<[AbilityInvestment]>,
    pub technique_ability: AbilityId,
    pub spec: CombatantBuildSpec,
    pub combatant: ResolvedCombatantSpec,
}

/// Explicit released source-skill locator to shared stable Ability identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSourceAbilityBinding {
    pub source_skill_id: u32,
    pub shared_ability: AbilityId,
}

/// Released relic-set threshold retained for later battle-program lowering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRelicSetThreshold {
    pub property_type: u32,
    pub set_id: u32,
    pub piece_count: u8,
    pub ability_name: Box<str>,
    pub ability_parameters: Box<[Box<str>]>,
}

impl CurrencyWarsTrialBuild {
    /// Resolves an account snapshot over this immutable mapped minimum.
    ///
    /// The caller owns account lookup and snapshots it before this boundary;
    /// neither Activity nor combat is queried or mutated here.
    pub fn substitute_owned(
        &self,
        owned: Option<(&CombatantBuildSpec, OwnedBuildMinimumFacts)>,
    ) -> Result<SubstitutedBuild, BuildSubstitutionError> {
        substitute_owned_or_trial(owned, &self.spec)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBuildSourceRole {
    SharedBuildCandidate,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBuildSourceDisposition {
    PendingExplicitRoleRowJoin,
    ExplicitRoleRowJoin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBuildSource {
    pub stable_key: Box<str>,
    pub path: Box<str>,
    pub sha256: Box<str>,
    pub role: CurrencyWarsBuildSourceRole,
    pub disposition: CurrencyWarsBuildSourceDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBuildSubstitutionRule {
    pub stable_key: Box<str>,
    pub selection_timing: Box<str>,
    pub owned_trial_policy: Box<str>,
    pub refresh_timing: Box<str>,
    pub teardown: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEquipmentDefinition {
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub slot: Box<str>,
    pub eligibility_json: Box<str>,
    pub effect_ids: Box<[Box<str>]>,
    pub parameters_json: Box<str>,
    pub replacement_rule: Box<str>,
    pub runtime: Option<CurrencyWarsRuntimeEquipment>,
    pub category_limit: Option<CurrencyWarsEquipmentCategoryLimit>,
    pub character_slot_limit: Option<u8>,
    pub character_implant_limit: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEquipmentRecommendation {
    pub equipment: CurrencyWarsEquipmentId,
    pub roles: Box<[CurrencyWarsRoleId]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOffFieldSourceKind {
    BackEquipment,
    BackRoleRank,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOffFieldDestination {
    BackEquipmentContribution,
    BackPositionContribution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOffFieldConversion {
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub source_kind: CurrencyWarsOffFieldSourceKind,
    pub eligibility: CurrencyWarsOffFieldEligibility,
    pub payload: CurrencyWarsOffFieldPayload,
    pub destination: CurrencyWarsOffFieldDestination,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBuildCatalog {
    mappings: Box<[CurrencyWarsBuildMapping]>,
    references: Box<[CurrencyWarsBuildReference]>,
    trial_builds: Box<[CurrencyWarsTrialBuild]>,
    sources: Box<[CurrencyWarsBuildSource]>,
    substitution_rules: Box<[CurrencyWarsBuildSubstitutionRule]>,
    equipment: Box<[CurrencyWarsEquipmentDefinition]>,
    recommendations: Box<[CurrencyWarsEquipmentRecommendation]>,
    off_field_conversions: Box<[CurrencyWarsOffFieldConversion]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBuildCatalogParts {
    pub mappings: Vec<CurrencyWarsBuildMapping>,
    pub references: Vec<CurrencyWarsBuildReference>,
    pub trial_builds: Vec<CurrencyWarsTrialBuild>,
    pub sources: Vec<CurrencyWarsBuildSource>,
    pub substitution_rules: Vec<CurrencyWarsBuildSubstitutionRule>,
    pub equipment: Vec<CurrencyWarsEquipmentDefinition>,
    pub recommendations: Vec<CurrencyWarsEquipmentRecommendation>,
    pub off_field_conversions: Vec<CurrencyWarsOffFieldConversion>,
}

impl CurrencyWarsBuildCatalog {
    pub fn new(
        mut parts: CurrencyWarsBuildCatalogParts,
    ) -> Result<Self, CurrencyWarsBuildCatalogError> {
        parts.mappings.sort_by_key(|value| value.role);
        parts.references.sort_by_key(|value| value.role);
        parts.trial_builds.sort_by_key(|value| value.role);
        parts
            .sources
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        parts
            .substitution_rules
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        parts
            .equipment
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        parts.recommendations.sort_by_key(|value| value.equipment);
        parts
            .off_field_conversions
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        validate(&parts)?;
        Ok(Self {
            mappings: parts.mappings.into_boxed_slice(),
            references: parts.references.into_boxed_slice(),
            trial_builds: parts.trial_builds.into_boxed_slice(),
            sources: parts.sources.into_boxed_slice(),
            substitution_rules: parts.substitution_rules.into_boxed_slice(),
            equipment: parts.equipment.into_boxed_slice(),
            recommendations: parts.recommendations.into_boxed_slice(),
            off_field_conversions: parts.off_field_conversions.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn mappings(&self) -> &[CurrencyWarsBuildMapping] {
        &self.mappings
    }
    #[must_use]
    pub fn references(&self) -> &[CurrencyWarsBuildReference] {
        &self.references
    }
    #[must_use]
    pub fn trial_builds(&self) -> &[CurrencyWarsTrialBuild] {
        &self.trial_builds
    }
    #[must_use]
    pub fn trial_build(&self, role: CurrencyWarsRoleId) -> Option<&CurrencyWarsTrialBuild> {
        self.trial_builds
            .binary_search_by_key(&role, |value| value.role)
            .ok()
            .map(|index| &self.trial_builds[index])
    }

    /// Resolves a role through its exact role/avatar/special-avatar join.
    pub fn resolve_role_build(
        &self,
        role: CurrencyWarsRoleId,
        owned: Option<(&CombatantBuildSpec, OwnedBuildMinimumFacts)>,
    ) -> Result<SubstitutedBuild, CurrencyWarsBuildResolutionError> {
        self.trial_build(role)
            .ok_or(CurrencyWarsBuildResolutionError::UnknownRole)?
            .substitute_owned(owned)
            .map_err(CurrencyWarsBuildResolutionError::Substitution)
    }
    #[must_use]
    pub fn sources(&self) -> &[CurrencyWarsBuildSource] {
        &self.sources
    }
    #[must_use]
    pub fn substitution_rules(&self) -> &[CurrencyWarsBuildSubstitutionRule] {
        &self.substitution_rules
    }
    #[must_use]
    pub fn equipment(&self) -> &[CurrencyWarsEquipmentDefinition] {
        &self.equipment
    }
    #[must_use]
    pub fn recommendations(&self) -> &[CurrencyWarsEquipmentRecommendation] {
        &self.recommendations
    }
    pub fn recommended_for_role(
        &self,
        role: CurrencyWarsRoleId,
    ) -> impl Iterator<Item = CurrencyWarsEquipmentId> + '_ {
        self.recommendations
            .iter()
            .filter(move |value| value.roles.contains(&role))
            .map(|value| value.equipment)
    }
    #[must_use]
    pub fn runtime_equipment(
        &self,
        id: CurrencyWarsEquipmentId,
    ) -> Option<&CurrencyWarsRuntimeEquipment> {
        self.equipment
            .iter()
            .filter_map(|definition| definition.runtime.as_ref())
            .find(|definition| definition.id == id)
    }
    #[must_use]
    pub fn runtime_equipment_definition(
        &self,
        id: CurrencyWarsEquipmentId,
    ) -> Option<&CurrencyWarsEquipmentDefinition> {
        self.equipment.iter().find(|definition| {
            definition
                .runtime
                .as_ref()
                .is_some_and(|value| value.id == id)
        })
    }
    #[must_use]
    pub fn equipment_category_limit(&self, category: CurrencyWarsEquipmentCategory) -> Option<u8> {
        self.equipment
            .iter()
            .filter_map(|definition| definition.category_limit)
            .find(|limit| limit.category == category)
            .and_then(|limit| limit.maximum)
    }
    #[must_use]
    pub fn character_equipment_slot_limit(&self) -> u8 {
        self.equipment
            .iter()
            .find_map(|definition| definition.character_slot_limit)
            .expect("validated Currency Wars equipment slot limit exists")
    }
    #[must_use]
    pub fn character_implant_slot_limit(&self) -> u8 {
        self.equipment
            .iter()
            .find_map(|definition| definition.character_implant_limit)
            .expect("validated Currency Wars equipment implant limit exists")
    }
    #[must_use]
    pub fn off_field_conversions(&self) -> &[CurrencyWarsOffFieldConversion] {
        &self.off_field_conversions
    }

    #[must_use]
    pub fn resolve_off_field_contributions(
        &self,
        role: CurrencyWarsRoleId,
        position: CurrencyWarsPositionKind,
        build: &CombatantBuildSpec,
    ) -> CurrencyWarsOffFieldContributionSnapshot {
        resolve_off_field_contributions(
            role,
            position,
            build,
            self.off_field_conversions.iter().map(|conversion| {
                (
                    conversion.stable_key.as_ref(),
                    &conversion.eligibility,
                    &conversion.payload,
                )
            }),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyWarsBuildResolutionError {
    UnknownRole,
    Substitution(BuildSubstitutionError),
}

impl std::fmt::Display for CurrencyWarsBuildResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Currency Wars Build resolution failed: {self:?}")
    }
}

impl std::error::Error for CurrencyWarsBuildResolutionError {}

#[cfg(test)]
impl CurrencyWarsBuildCatalog {
    pub(crate) fn test_fixture(role: CurrencyWarsRoleId) -> Self {
        Self::new(CurrencyWarsBuildCatalogParts {
            mappings: vec![CurrencyWarsBuildMapping {
                stable_key: "build.role.1001".into(),
                role,
                avatar_id: role.get(),
                special_avatar_id: 3_701_001,
                level: CurrencyWarsBuildMinimum::AccountOrModeMinimum,
                trace_state: CurrencyWarsBuildMinimum::AccountOrModeMinimum,
                light_cone: CurrencyWarsBuildMinimum::AccountOrMappedMinimum,
                relics: CurrencyWarsBuildMinimum::AccountOrMappedMinimum,
                mutates_account: false,
            }],
            references: vec![CurrencyWarsBuildReference {
                stable_key: "build-reference.role.1001".into(),
                role,
                avatar_id: role.get(),
                owned_build_id: "account-avatar:1001".into(),
                trial_build_id: "gridfight-special-avatar:3701001".into(),
                in_pool: true,
            }],
            trial_builds: vec![CurrencyWarsTrialBuild {
                stable_key: "trial-build.role.1001".into(),
                role,
                avatar_id: role.get(),
                special_avatar_id: 3_701_001,
                world_level: 6,
                skill_tree_key: "W5_Standard_70-80".into(),
                relic_property_type: 610_315,
                relic_property_type_extra: 630_415,
                relic_main_value: 3_362_515,
                relic_sub_value: 515,
                relic_sets: Box::new([]),
                source_ability_bindings: Box::new([CurrencyWarsSourceAbilityBinding {
                    source_skill_id: 1,
                    shared_ability: AbilityId::new(1).expect("fixture ability is nonzero"),
                }]),
                effective_ability_levels: Box::new([AbilityInvestment::new(
                    AbilityId::new(1).expect("fixture ability is nonzero"),
                    AbilityLevel::new(1).expect("fixture ability level is valid"),
                )]),
                technique_ability: AbilityId::new(1).expect("fixture ability is nonzero"),
                spec: CombatantBuildSpec::new(
                    UnitDefinitionId::new(role.get()).expect("fixture form is nonzero"),
                    UnitLevel::new(80).expect("fixture level is valid"),
                    PromotionStage::new(6).expect("fixture promotion is valid"),
                )
                .with_ability_levels(vec![AbilityInvestment::new(
                    AbilityId::new(1).expect("fixture ability is nonzero"),
                    AbilityLevel::new(1).expect("fixture ability level is valid"),
                )])
                .expect("fixture ability selection is unique")
                .with_eidolon(EidolonLevel::new(3).expect("fixture Eidolon is valid"))
                .with_light_cone(LightConeLoadout::new(
                    starclock_build::id::LightConeId::new(1)
                        .expect("fixture Light Cone is nonzero"),
                    LightConeLevel::new(80).expect("fixture Light Cone level is valid"),
                    PromotionStage::new(6).expect("fixture promotion is valid"),
                    Superimposition::new(1).expect("fixture superimposition is valid"),
                )),
                combatant: ResolvedCombatantSpec::new(
                    UnitDefinitionId::new(role.get()).expect("fixture form is nonzero"),
                    UnitLevel::new(80).expect("fixture level is valid"),
                    Hp::new(1_000).expect("fixture HP is positive"),
                    Speed::from_scaled(100_000_000).expect("fixture Speed is positive"),
                    ResolvedDefinitionBindings::new(
                        vec![AbilityId::new(1).expect("fixture ability is nonzero")],
                        vec![],
                        vec![],
                    )
                    .expect("fixture bindings are canonical"),
                    CombatantSpecDigest::new([1; 32]).expect("fixture digest is nonzero"),
                )
                .expect("fixture combatant is valid"),
            }],
            sources: vec![CurrencyWarsBuildSource {
                stable_key: "build-source.fixture".into(),
                path: "fixture.json".into(),
                sha256: "00".repeat(32).into_boxed_str(),
                role: CurrencyWarsBuildSourceRole::SharedBuildCandidate,
                disposition: CurrencyWarsBuildSourceDisposition::PendingExplicitRoleRowJoin,
            }],
            substitution_rules: vec![CurrencyWarsBuildSubstitutionRule {
                stable_key: "build-substitution.fixture".into(),
                selection_timing: "fixture".into(),
                owned_trial_policy: "fixture".into(),
                refresh_timing: "fixture".into(),
                teardown: "fixture".into(),
            }],
            equipment: vec![
                CurrencyWarsEquipmentDefinition {
                    stable_key: "equipment.fixture.1".into(),
                    source_id: "1".into(),
                    slot: "Front".into(),
                    eligibility_json: "{}".into(),
                    effect_ids: Box::new([]),
                    parameters_json: "[]".into(),
                    replacement_rule: "fixture".into(),
                    runtime: Some(CurrencyWarsRuntimeEquipment {
                        id: CurrencyWarsEquipmentId::new(1).unwrap(),
                        category: CurrencyWarsEquipmentCategory::Basic,
                        tags: Box::new([10]),
                        dress_rule: CurrencyWarsEquipmentDressRule::Any,
                        properties: Box::new([]),
                        ability_name: None,
                        parameters: Box::new([]),
                    }),
                    category_limit: Some(CurrencyWarsEquipmentCategoryLimit {
                        category: CurrencyWarsEquipmentCategory::Basic,
                        maximum: Some(1),
                    }),
                    character_slot_limit: Some(3),
                    character_implant_limit: Some(1),
                },
                CurrencyWarsEquipmentDefinition {
                    stable_key: "equipment.fixture.2".into(),
                    source_id: "2".into(),
                    slot: "Front".into(),
                    eligibility_json: "{}".into(),
                    effect_ids: Box::new([]),
                    parameters_json: "[]".into(),
                    replacement_rule: "fixture".into(),
                    runtime: Some(CurrencyWarsRuntimeEquipment {
                        id: CurrencyWarsEquipmentId::new(2).unwrap(),
                        category: CurrencyWarsEquipmentCategory::Basic,
                        tags: Box::new([11]),
                        dress_rule: CurrencyWarsEquipmentDressRule::Any,
                        properties: Box::new([]),
                        ability_name: None,
                        parameters: Box::new([]),
                    }),
                    category_limit: None,
                    character_slot_limit: None,
                    character_implant_limit: None,
                },
            ],
            recommendations: vec![
                CurrencyWarsEquipmentRecommendation {
                    equipment: CurrencyWarsEquipmentId::new(1).unwrap(),
                    roles: Box::new([role]),
                },
                CurrencyWarsEquipmentRecommendation {
                    equipment: CurrencyWarsEquipmentId::new(2).unwrap(),
                    roles: Box::new([role]),
                },
            ],
            off_field_conversions: vec![CurrencyWarsOffFieldConversion {
                stable_key: "conversion.fixture".into(),
                source_id: "1".into(),
                source_kind: CurrencyWarsOffFieldSourceKind::BackEquipment,
                eligibility: CurrencyWarsOffFieldEligibility::Eidolon {
                    role,
                    rank_id: 100_101,
                    rank: 1,
                },
                payload: CurrencyWarsOffFieldPayload {
                    owner_properties: Box::new([]),
                    all_member_properties: Box::new([]),
                    modified_skills: Box::new([]),
                    rank_abilities: Box::new([]),
                    parameters: Box::new([]),
                },
                destination: CurrencyWarsOffFieldDestination::BackEquipmentContribution,
            }],
        })
        .expect("test Build catalog is valid")
    }
}

fn validate(parts: &CurrencyWarsBuildCatalogParts) -> Result<(), CurrencyWarsBuildCatalogError> {
    if parts.mappings.is_empty()
        || parts.references.is_empty()
        || parts.trial_builds.is_empty()
        || parts.sources.is_empty()
        || parts.substitution_rules.is_empty()
        || parts.equipment.is_empty()
        || parts.recommendations.is_empty()
        || parts.off_field_conversions.is_empty()
    {
        return Err(error("Currency Wars Build catalog is empty"));
    }
    if parts.recommendations.iter().any(|recommendation| {
        !parts.equipment.iter().any(|definition| {
            definition.runtime.as_ref().map(|value| value.id) == Some(recommendation.equipment)
        }) || recommendation
            .roles
            .iter()
            .any(|role| !parts.mappings.iter().any(|mapping| mapping.role == *role))
    }) {
        return Err(error(
            "Currency Wars equipment recommendation relationship is invalid",
        ));
    }
    let mappings = parts
        .mappings
        .iter()
        .map(|value| value.role)
        .collect::<BTreeSet<_>>();
    let references = parts
        .references
        .iter()
        .map(|value| value.role)
        .collect::<BTreeSet<_>>();
    let trials = parts
        .trial_builds
        .iter()
        .map(|value| value.role)
        .collect::<BTreeSet<_>>();
    if mappings.len() != parts.mappings.len()
        || references.len() != parts.references.len()
        || mappings != references
        || mappings != trials
        || trials.len() != parts.trial_builds.len()
        || parts.mappings.iter().any(|mapping| mapping.mutates_account)
    {
        return Err(error("Currency Wars Build mapping closure is invalid"));
    }
    for mapping in &parts.mappings {
        let reference = parts
            .references
            .iter()
            .find(|value| value.role == mapping.role)
            .ok_or_else(|| error("Currency Wars Build reference is missing"))?;
        if reference.avatar_id != mapping.avatar_id {
            return Err(error("Currency Wars Build avatar reference is invalid"));
        }
        let trial = parts
            .trial_builds
            .iter()
            .find(|value| value.role == mapping.role)
            .ok_or_else(|| error("Currency Wars trial Build is missing"))?;
        if trial.avatar_id != mapping.avatar_id
            || trial.special_avatar_id != mapping.special_avatar_id
        {
            return Err(error("Currency Wars trial Build mapping is invalid"));
        }
    }
    let runtime_equipment = parts
        .equipment
        .iter()
        .filter_map(|definition| definition.runtime.as_ref())
        .collect::<Vec<_>>();
    let equipment_ids = runtime_equipment
        .iter()
        .map(|definition| definition.id)
        .collect::<BTreeSet<_>>();
    let category_limits = parts
        .equipment
        .iter()
        .filter_map(|definition| definition.category_limit)
        .map(|limit| limit.category)
        .collect::<BTreeSet<_>>();
    let character_limits = parts
        .equipment
        .iter()
        .filter_map(|definition| definition.character_slot_limit)
        .collect::<Vec<_>>();
    let implant_limits = parts
        .equipment
        .iter()
        .filter_map(|definition| definition.character_implant_limit)
        .collect::<Vec<_>>();
    if equipment_ids.len() != runtime_equipment.len()
        || runtime_equipment.is_empty()
        || category_limits.is_empty()
        || character_limits.as_slice() != [3]
        || implant_limits.as_slice() != [1]
    {
        return Err(error("Currency Wars runtime equipment closure is invalid"));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBuildCatalogError {
    message: Box<str>,
}
impl std::fmt::Display for CurrencyWarsBuildCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for CurrencyWarsBuildCatalogError {}
fn error(message: &'static str) -> CurrencyWarsBuildCatalogError {
    CurrencyWarsBuildCatalogError {
        message: message.into(),
    }
}
