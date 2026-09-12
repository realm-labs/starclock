//! Immutable Arithmetic Mapping definitions for Divergent Universe.
//!
//! These definitions describe temporary mode-owned patches. They neither read
//! account state nor mutate account characters, Light Cones or Relics.

use std::collections::BTreeMap;

macro_rules! stable_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);

        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseMappingError> {
                let value = value.into();
                if !value.starts_with($prefix) || value.len() == $prefix.len() {
                    return Err(error(concat!(stringify!($name), " namespace mismatch")));
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

stable_id!(
    DivergentUniverseMappingEligibilityId,
    "divergent-universe.mapping-eligibility."
);
stable_id!(
    DivergentUniverseMappingBuildId,
    "divergent-universe.mapping-build."
);
stable_id!(
    DivergentUniverseMappingRuleId,
    "divergent-universe.mapping-rule."
);

/// Upstream avatar source locator. This is not a Starclock content identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseAvatarLocator(Box<str>);

impl DivergentUniverseAvatarLocator {
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseMappingError> {
        let value = value.into();
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(error("avatar locator must contain only ASCII digits"));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact decimal text retained without floating-point conversion.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseMappingDecimal(Box<str>);

impl DivergentUniverseMappingDecimal {
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseMappingError> {
        let value = value.into();
        if !canonical_decimal(&value) {
            return Err(error("noncanonical Arithmetic Mapping decimal"));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappingEligibilityKind {
    ExplicitBuildReferenceCatalog,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseAccountComparisonPolicy {
    ApplyOnlyWhenCorrespondingBelowThresholdConditionIsTrue,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniversePublicIdentityResolution {
    ResolvedAvatarConfig,
    MissingReleasedAvatarConfig,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappedLevelPatch {
    EquilibriumLevelCapWhenBelow,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappedTracePatch {
    ActivateOrRaiseWhenInactiveOrBelowRequirement,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappedLightConePatch {
    UnspecifiedConditionAndTemporaryIdentity,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappedRelicPatch {
    ReplaceWhenTotalEnhancementBelowRequirement,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseExactTemporaryLoadout {
    Unspecified,
}

/// Generic temporary build patch; application belongs to a later runtime batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTemporaryBuildPatch {
    pub level: DivergentUniverseMappedLevelPatch,
    pub traces: DivergentUniverseMappedTracePatch,
    pub light_cone: DivergentUniverseMappedLightConePatch,
    pub relics: DivergentUniverseMappedRelicPatch,
    pub exact_loadout: DivergentUniverseExactTemporaryLoadout,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappingRuleKind {
    Scope,
    CharacterLevel,
    Traces,
    Relics,
    LightCone,
    Refresh,
    Teardown,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappingTiming {
    ModeBoundary,
    RunEntryOrRefresh,
    Unspecified,
    RunEntryAndAcceptedPartyChange,
    RunFinalization,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappingCondition {
    InsideDivergentUniverse,
    CharacterLevelBelowEquilibriumCap,
    UnlockedTraceInactiveOrBelowRequirement,
    RelicTotalEnhancementBelowRequirement,
    Unspecified,
    MappingInputChanged,
    LeavingDivergentUniverse,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseMappingOperation {
    ApplyTemporaryMappingState,
    RaiseCharacterToEquilibriumCap,
    ActivateOrRaiseTrace,
    ReplaceWithCompatibleTemporaryRelics,
    Unspecified,
    ReevaluateOnlyBelowThresholdFields,
    RemoveTemporaryMappingState,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseStrongerBuildRule {
    PreserveWhenConditionIsFalse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingEligibilityDefinition {
    pub id: DivergentUniverseMappingEligibilityId,
    pub avatar: DivergentUniverseAvatarLocator,
    pub sort_weight: u32,
    pub eligibility: DivergentUniverseMappingEligibilityKind,
    pub has_special_avatar_mapping: bool,
    pub has_role_buff: bool,
    pub account_comparison: DivergentUniverseAccountComparisonPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingBuildDefinition {
    pub id: DivergentUniverseMappingBuildId,
    pub avatar: DivergentUniverseAvatarLocator,
    pub public_identity: DivergentUniversePublicIdentityResolution,
    pub eligible_catalog_entry: bool,
    pub special_avatar: Option<Box<str>>,
    pub role_buff_id: Box<str>,
    pub role_buff_binding_key: Box<str>,
    pub role_buff_modifier_name: Box<str>,
    pub role_buff_parameters: Box<[DivergentUniverseMappingDecimal]>,
    pub patch: DivergentUniverseTemporaryBuildPatch,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingRuleDefinition {
    pub id: DivergentUniverseMappingRuleId,
    pub kind: DivergentUniverseMappingRuleKind,
    pub timing: DivergentUniverseMappingTiming,
    pub condition: DivergentUniverseMappingCondition,
    pub operations: Box<[DivergentUniverseMappingOperation]>,
    pub stronger_build_rule: DivergentUniverseStrongerBuildRule,
    pub account_mutation: bool,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingCatalogParts {
    pub eligibility: Vec<DivergentUniverseMappingEligibilityDefinition>,
    pub builds: Vec<DivergentUniverseMappingBuildDefinition>,
    pub rules: Vec<DivergentUniverseMappingRuleDefinition>,
    pub source_obligations: usize,
}

/// Validated immutable Arithmetic Mapping catalog with no account-state access.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingCatalog {
    eligibility: Box<[DivergentUniverseMappingEligibilityDefinition]>,
    builds: Box<[DivergentUniverseMappingBuildDefinition]>,
    rules: Box<[DivergentUniverseMappingRuleDefinition]>,
    source_obligations: usize,
}

impl DivergentUniverseMappingCatalog {
    pub fn new(
        mut parts: DivergentUniverseMappingCatalogParts,
    ) -> Result<Self, DivergentUniverseMappingError> {
        require_count(&parts.eligibility, 84, "mapping eligibility")?;
        require_count(&parts.builds, 95, "mapping builds")?;
        require_count(&parts.rules, 7, "mapping lifecycle rules")?;
        if parts.source_obligations != 258 {
            return Err(error("Arithmetic Mapping source obligation closure drift"));
        }
        parts
            .eligibility
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .builds
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .rules
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        ensure_unique(&parts.eligibility, |value| &value.id, "mapping eligibility")?;
        ensure_unique(&parts.builds, |value| &value.id, "mapping build")?;
        ensure_unique(&parts.rules, |value| &value.id, "mapping rule")?;
        validate_mapping(&parts)?;
        Ok(Self {
            eligibility: parts.eligibility.into_boxed_slice(),
            builds: parts.builds.into_boxed_slice(),
            rules: parts.rules.into_boxed_slice(),
            source_obligations: parts.source_obligations,
        })
    }

    #[must_use]
    pub const fn eligibility(&self) -> &[DivergentUniverseMappingEligibilityDefinition] {
        &self.eligibility
    }

    #[must_use]
    pub const fn builds(&self) -> &[DivergentUniverseMappingBuildDefinition] {
        &self.builds
    }

    #[must_use]
    pub const fn rules(&self) -> &[DivergentUniverseMappingRuleDefinition] {
        &self.rules
    }

    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.source_obligations
    }

    #[must_use]
    pub fn build(
        &self,
        avatar: &DivergentUniverseAvatarLocator,
    ) -> Option<&DivergentUniverseMappingBuildDefinition> {
        self.builds
            .binary_search_by(|value| value.avatar.cmp(avatar))
            .ok()
            .map(|index| &self.builds[index])
    }

    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseMappingCatalogParts {
        DivergentUniverseMappingCatalogParts {
            eligibility: self.eligibility.into_vec(),
            builds: self.builds.into_vec(),
            rules: self.rules.into_vec(),
            source_obligations: self.source_obligations,
        }
    }
}

fn validate_mapping(
    parts: &DivergentUniverseMappingCatalogParts,
) -> Result<(), DivergentUniverseMappingError> {
    let eligibility = parts
        .eligibility
        .iter()
        .map(|value| (&value.avatar, value))
        .collect::<BTreeMap<_, _>>();
    let builds = parts
        .builds
        .iter()
        .map(|value| (&value.avatar, value))
        .collect::<BTreeMap<_, _>>();
    if eligibility.len() != parts.eligibility.len() || builds.len() != parts.builds.len() {
        return Err(error("duplicate Arithmetic Mapping avatar locator"));
    }
    for value in &parts.eligibility {
        let Some(build) = builds.get(&value.avatar) else {
            return Err(error("eligibility has no mapping build"));
        };
        if value.id.as_str()
            != format!(
                "divergent-universe.mapping-eligibility.{}",
                value.avatar.as_str()
            )
            || !value.has_role_buff
            || value.has_role_buff != !build.role_buff_id.is_empty()
            || value.has_special_avatar_mapping != build.special_avatar.is_some()
        {
            return Err(error("mapping eligibility/build join drift"));
        }
    }
    for value in &parts.builds {
        if value.id.as_str()
            != format!("divergent-universe.mapping-build.{}", value.avatar.as_str())
            || value.eligible_catalog_entry != eligibility.contains_key(&value.avatar)
            || value.role_buff_id.is_empty()
            || value.role_buff_binding_key.is_empty()
            || value.role_buff_modifier_name.is_empty()
            || !(4..=7).contains(&value.role_buff_parameters.len())
            || value.runtime_lowered
        {
            return Err(error("mapping build closure drift"));
        }
    }
    let arities = parts
        .builds
        .iter()
        .fold(BTreeMap::new(), |mut counts, value| {
            *counts
                .entry(value.role_buff_parameters.len())
                .or_insert(0usize) += 1;
            counts
        });
    if arities != BTreeMap::from([(4, 41), (5, 34), (6, 18), (7, 2)])
        || parts
            .builds
            .iter()
            .filter(|value| value.special_avatar.is_some())
            .count()
            != 79
        || parts
            .builds
            .iter()
            .filter(|value| {
                value.public_identity
                    == DivergentUniversePublicIdentityResolution::ResolvedAvatarConfig
            })
            .count()
            != 91
    {
        return Err(error("mapping build denominator drift"));
    }
    validate_rules(&parts.rules)
}

fn validate_rules(
    rules: &[DivergentUniverseMappingRuleDefinition],
) -> Result<(), DivergentUniverseMappingError> {
    let by_kind = rules
        .iter()
        .map(|value| (value.kind, value))
        .collect::<BTreeMap<_, _>>();
    if by_kind.len() != 7
        || rules.iter().any(|value| {
            value.account_mutation
                || value.runtime_lowered
                || value.operations.len() != 1
                || value.stronger_build_rule
                    != DivergentUniverseStrongerBuildRule::PreserveWhenConditionIsFalse
        })
    {
        return Err(error("mapping lifecycle boundary drift"));
    }
    let expected = [
        (
            DivergentUniverseMappingRuleKind::Scope,
            DivergentUniverseMappingTiming::ModeBoundary,
            DivergentUniverseMappingCondition::InsideDivergentUniverse,
            DivergentUniverseMappingOperation::ApplyTemporaryMappingState,
        ),
        (
            DivergentUniverseMappingRuleKind::CharacterLevel,
            DivergentUniverseMappingTiming::RunEntryOrRefresh,
            DivergentUniverseMappingCondition::CharacterLevelBelowEquilibriumCap,
            DivergentUniverseMappingOperation::RaiseCharacterToEquilibriumCap,
        ),
        (
            DivergentUniverseMappingRuleKind::Traces,
            DivergentUniverseMappingTiming::RunEntryOrRefresh,
            DivergentUniverseMappingCondition::UnlockedTraceInactiveOrBelowRequirement,
            DivergentUniverseMappingOperation::ActivateOrRaiseTrace,
        ),
        (
            DivergentUniverseMappingRuleKind::Relics,
            DivergentUniverseMappingTiming::RunEntryOrRefresh,
            DivergentUniverseMappingCondition::RelicTotalEnhancementBelowRequirement,
            DivergentUniverseMappingOperation::ReplaceWithCompatibleTemporaryRelics,
        ),
        (
            DivergentUniverseMappingRuleKind::LightCone,
            DivergentUniverseMappingTiming::Unspecified,
            DivergentUniverseMappingCondition::Unspecified,
            DivergentUniverseMappingOperation::Unspecified,
        ),
        (
            DivergentUniverseMappingRuleKind::Refresh,
            DivergentUniverseMappingTiming::RunEntryAndAcceptedPartyChange,
            DivergentUniverseMappingCondition::MappingInputChanged,
            DivergentUniverseMappingOperation::ReevaluateOnlyBelowThresholdFields,
        ),
        (
            DivergentUniverseMappingRuleKind::Teardown,
            DivergentUniverseMappingTiming::RunFinalization,
            DivergentUniverseMappingCondition::LeavingDivergentUniverse,
            DivergentUniverseMappingOperation::RemoveTemporaryMappingState,
        ),
    ];
    if expected
        .into_iter()
        .any(|(kind, timing, condition, operation)| {
            by_kind.get(&kind).is_none_or(|value| {
                value.timing != timing
                    || value.condition != condition
                    || value.operations.as_ref() != [operation]
            })
        })
    {
        return Err(error("mapping lifecycle rule closure drift"));
    }
    Ok(())
}

fn canonical_decimal(value: &str) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    if unsigned.is_empty() || unsigned.starts_with('+') {
        return false;
    }
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
    {
        return false;
    }
    fraction.is_none_or(|digits| {
        !digits.is_empty()
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && !digits.ends_with('0')
    })
}

fn require_count<T>(
    values: &[T],
    expected: usize,
    label: &str,
) -> Result<(), DivergentUniverseMappingError> {
    if values.len() != expected {
        return Err(error(&format!(
            "expected {expected} {label}, got {}",
            values.len()
        )));
    }
    Ok(())
}

fn ensure_unique<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> &K,
    label: &str,
) -> Result<(), DivergentUniverseMappingError> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        return Err(error(&format!("duplicate {label} identity")));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingError {
    message: Box<str>,
}

impl std::fmt::Display for DivergentUniverseMappingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DivergentUniverseMappingError {}

fn error(message: &str) -> DivergentUniverseMappingError {
    DivergentUniverseMappingError {
        message: message.into(),
    }
}
