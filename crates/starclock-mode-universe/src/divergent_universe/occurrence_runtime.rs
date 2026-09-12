//! Occurrence identities and explicit missing-graph external-result boundaries.

use std::sync::Arc;

use starclock_activity::{ActivityStateHash, GraphActivity};
use starclock_data::divergent_universe_service_catalog::{
    DivergentUniverseOccurrenceId, DivergentUniverseOccurrenceVariantId,
};

use super::DivergentUniverseRuntimeFactory;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseOccurrenceAccuracy {
    ExactReleasedHandbookIdentityVariantAndUnlockBindings,
    VersionedProjectPolicyRejectMissingOccurrenceGraphsWithoutMutation,
    ExplicitTypedExternalResultBoundaryWithoutGameplayParityClaim,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseOccurrenceExternalResultKind {
    DialogueInteraction,
    Minigame,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceExternalResult {
    kind: DivergentUniverseOccurrenceExternalResultKind,
    result_id: Box<str>,
}

impl DivergentUniverseOccurrenceExternalResult {
    pub fn new(
        kind: DivergentUniverseOccurrenceExternalResultKind,
        result_id: impl Into<Box<str>>,
    ) -> Result<Self, DivergentUniverseOccurrenceRuntimeError> {
        let result_id = result_id.into();
        if result_id.is_empty() {
            return Err(DivergentUniverseOccurrenceRuntimeError::InvalidExternalResult);
        }
        Ok(Self { kind, result_id })
    }

    #[must_use]
    pub const fn kind(&self) -> DivergentUniverseOccurrenceExternalResultKind {
        self.kind
    }

    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceUnlockRuntime {
    ordinal: u16,
    source_field: Box<str>,
    variant: DivergentUniverseOccurrenceVariantId,
}

impl DivergentUniverseOccurrenceUnlockRuntime {
    #[must_use]
    pub const fn ordinal(&self) -> u16 {
        self.ordinal
    }

    #[must_use]
    pub fn source_field(&self) -> &str {
        &self.source_field
    }

    #[must_use]
    pub const fn variant(&self) -> &DivergentUniverseOccurrenceVariantId {
        &self.variant
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceRuntimeDefinition {
    id: DivergentUniverseOccurrenceId,
    variants: Box<[DivergentUniverseOccurrenceVariantId]>,
    handbook_priority: u16,
    handbook_used: bool,
    selection_policy: Box<str>,
    unlock_rules: Box<[DivergentUniverseOccurrenceUnlockRuntime]>,
    unresolved_offer_behavior: Box<str>,
}

impl DivergentUniverseOccurrenceRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseOccurrenceId {
        &self.id
    }

    #[must_use]
    pub fn variants(&self) -> &[DivergentUniverseOccurrenceVariantId] {
        &self.variants
    }

    #[must_use]
    pub const fn handbook_priority(&self) -> u16 {
        self.handbook_priority
    }

    #[must_use]
    pub const fn handbook_used(&self) -> bool {
        self.handbook_used
    }

    #[must_use]
    pub fn selection_policy(&self) -> &str {
        &self.selection_policy
    }

    #[must_use]
    pub fn unlock_rules(&self) -> &[DivergentUniverseOccurrenceUnlockRuntime] {
        &self.unlock_rules
    }

    #[must_use]
    pub fn unresolved_offer_behavior(&self) -> &str {
        &self.unresolved_offer_behavior
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceEntryConditionRuntime {
    kind: Box<str>,
    occurrence: DivergentUniverseOccurrenceId,
}

impl DivergentUniverseOccurrenceEntryConditionRuntime {
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub const fn occurrence(&self) -> &DivergentUniverseOccurrenceId {
        &self.occurrence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceVariantRuntimeDefinition {
    id: DivergentUniverseOccurrenceVariantId,
    occurrence: DivergentUniverseOccurrenceId,
    occurrences: Box<[DivergentUniverseOccurrenceId]>,
    entry_conditions: Box<[DivergentUniverseOccurrenceEntryConditionRuntime]>,
    graph_path: Box<str>,
    graph_resolution: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseOccurrenceVariantRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseOccurrenceVariantId {
        &self.id
    }

    #[must_use]
    pub const fn occurrence(&self) -> &DivergentUniverseOccurrenceId {
        &self.occurrence
    }

    #[must_use]
    pub fn occurrences(&self) -> &[DivergentUniverseOccurrenceId] {
        &self.occurrences
    }

    #[must_use]
    pub fn entry_conditions(&self) -> &[DivergentUniverseOccurrenceEntryConditionRuntime] {
        &self.entry_conditions
    }

    #[must_use]
    pub fn graph_path(&self) -> &str {
        &self.graph_path
    }

    #[must_use]
    pub fn graph_resolution(&self) -> &str {
        &self.graph_resolution
    }

    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceChoiceProgram {
    choice_id: Box<str>,
    cost: Box<str>,
    ordered_outcomes: Box<[Box<str>]>,
}

impl DivergentUniverseOccurrenceChoiceProgram {
    #[must_use]
    pub fn choice_id(&self) -> &str {
        &self.choice_id
    }

    #[must_use]
    pub fn cost(&self) -> &str {
        &self.cost
    }

    #[must_use]
    pub fn ordered_outcomes(&self) -> &[Box<str>] {
        &self.ordered_outcomes
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseOccurrenceRuntime {
    occurrences: Arc<[DivergentUniverseOccurrenceRuntimeDefinition]>,
    variants: Arc<[DivergentUniverseOccurrenceVariantRuntimeDefinition]>,
    choice_programs: Arc<[DivergentUniverseOccurrenceChoiceProgram]>,
}

impl DivergentUniverseRuntimeFactory {
    pub fn occurrence_runtime(
        &self,
    ) -> Result<DivergentUniverseOccurrenceRuntime, DivergentUniverseOccurrenceRuntimeError> {
        DivergentUniverseOccurrenceRuntime::compile(self.bundle.service_catalog())
    }
}

impl DivergentUniverseOccurrenceRuntime {
    fn compile(
        catalog: &starclock_data::divergent_universe_service_catalog::DivergentUniverseServiceCatalog,
    ) -> Result<Self, DivergentUniverseOccurrenceRuntimeError> {
        let occurrences = catalog
            .occurrences()
            .iter()
            .map(|value| {
                if value.runtime_lowered
                    || value.variants.len() != 1
                    || !value.choice_ids.is_empty()
                    || !value.handbook_used
                    || value.selection_policy.as_ref() != "OwningDomainOrServiceBindingRequired"
                    || value.unresolved_offer_behavior.as_ref() != "FailClosed"
                    || value.unlock_rules.is_empty()
                {
                    return Err(DivergentUniverseOccurrenceRuntimeError::InvalidCatalog);
                }
                Ok(DivergentUniverseOccurrenceRuntimeDefinition {
                    id: value.id.clone(),
                    variants: value.variants.clone(),
                    handbook_priority: value.handbook_priority,
                    handbook_used: value.handbook_used,
                    selection_policy: value.selection_policy.clone(),
                    unlock_rules: value
                        .unlock_rules
                        .iter()
                        .map(|rule| DivergentUniverseOccurrenceUnlockRuntime {
                            ordinal: rule.ordinal,
                            source_field: rule.source_field.clone(),
                            variant: rule.variant.clone(),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    unresolved_offer_behavior: value.unresolved_offer_behavior.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let variants = catalog
            .variants()
            .iter()
            .map(|value| {
                if value.runtime_lowered
                    || !value.choice_ids.is_empty()
                    || value.occurrences.is_empty()
                    || value.occurrences[0] != value.occurrence
                    || value.graph_resolution.as_ref() != "MissingAtPinnedRevision"
                    || value.fallback.as_ref() != "RejectWithoutMutation"
                {
                    return Err(DivergentUniverseOccurrenceRuntimeError::InvalidCatalog);
                }
                Ok(DivergentUniverseOccurrenceVariantRuntimeDefinition {
                    id: value.id.clone(),
                    occurrence: value.occurrence.clone(),
                    occurrences: value.occurrences.clone(),
                    entry_conditions: value
                        .entry_conditions
                        .iter()
                        .map(
                            |condition| DivergentUniverseOccurrenceEntryConditionRuntime {
                                kind: condition.kind.clone(),
                                occurrence: condition.occurrence.clone(),
                            },
                        )
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    graph_path: value.graph_path.clone(),
                    graph_resolution: value.graph_resolution.clone(),
                    fallback: value.fallback.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if occurrences.len() != 118 || variants.len() != 97 {
            return Err(DivergentUniverseOccurrenceRuntimeError::InvalidCatalog);
        }
        for occurrence in &occurrences {
            let variant = lookup(&variants, |value| &value.id, &occurrence.variants[0])?;
            if !variant.occurrences.contains(&occurrence.id)
                || occurrence
                    .unlock_rules
                    .iter()
                    .any(|rule| rule.variant != variant.id)
            {
                return Err(DivergentUniverseOccurrenceRuntimeError::InvalidCatalog);
            }
        }
        Ok(Self {
            occurrences: occurrences.into(),
            variants: variants.into(),
            choice_programs: Arc::from([]),
        })
    }

    #[must_use]
    pub const fn accuracies(&self) -> [DivergentUniverseOccurrenceAccuracy; 3] {
        [
            DivergentUniverseOccurrenceAccuracy::ExactReleasedHandbookIdentityVariantAndUnlockBindings,
            DivergentUniverseOccurrenceAccuracy::VersionedProjectPolicyRejectMissingOccurrenceGraphsWithoutMutation,
            DivergentUniverseOccurrenceAccuracy::ExplicitTypedExternalResultBoundaryWithoutGameplayParityClaim,
        ]
    }

    #[must_use]
    pub fn occurrences(&self) -> &[DivergentUniverseOccurrenceRuntimeDefinition] {
        &self.occurrences
    }

    #[must_use]
    pub fn variants(&self) -> &[DivergentUniverseOccurrenceVariantRuntimeDefinition] {
        &self.variants
    }

    #[must_use]
    pub fn choice_programs(&self) -> &[DivergentUniverseOccurrenceChoiceProgram] {
        &self.choice_programs
    }

    pub fn reject_unresolved_occurrence_offer(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        occurrence: &DivergentUniverseOccurrenceId,
    ) -> Result<(), DivergentUniverseOccurrenceRuntimeError> {
        validate_activity(activity, expected)?;
        let _ = lookup(&self.occurrences, |value| &value.id, occurrence)?;
        Err(DivergentUniverseOccurrenceRuntimeError::UnresolvedOffer)
    }

    pub fn submit_external_result(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        occurrence: &DivergentUniverseOccurrenceId,
        variant: &DivergentUniverseOccurrenceVariantId,
        result: &DivergentUniverseOccurrenceExternalResult,
    ) -> Result<(), DivergentUniverseOccurrenceRuntimeError> {
        validate_activity(activity, expected)?;
        if result.result_id.is_empty() {
            return Err(DivergentUniverseOccurrenceRuntimeError::InvalidExternalResult);
        }
        let occurrence = lookup(&self.occurrences, |value| &value.id, occurrence)?;
        let variant = lookup(&self.variants, |value| &value.id, variant)?;
        if !occurrence.variants.contains(&variant.id)
            || !variant.occurrences.contains(&occurrence.id)
        {
            return Err(DivergentUniverseOccurrenceRuntimeError::VariantMismatch);
        }
        Err(DivergentUniverseOccurrenceRuntimeError::ExternalResultUnavailable(result.kind))
    }
}

fn validate_activity(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseOccurrenceRuntimeError> {
    if activity.state_hash() != expected {
        return Err(DivergentUniverseOccurrenceRuntimeError::StaleStateHash);
    }
    if activity.player_view().terminal().is_some() {
        return Err(DivergentUniverseOccurrenceRuntimeError::ActivityCompleted);
    }
    Ok(())
}

fn lookup<'a, T, I: Ord>(
    values: &'a [T],
    id: impl Fn(&T) -> &I,
    expected: &I,
) -> Result<&'a T, DivergentUniverseOccurrenceRuntimeError> {
    values
        .binary_search_by(|value| id(value).cmp(expected))
        .ok()
        .map(|index| &values[index])
        .ok_or(DivergentUniverseOccurrenceRuntimeError::UnknownIdentity)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseOccurrenceRuntimeError {
    InvalidCatalog,
    UnknownIdentity,
    VariantMismatch,
    StaleStateHash,
    ActivityCompleted,
    InvalidExternalResult,
    UnresolvedOffer,
    ExternalResultUnavailable(DivergentUniverseOccurrenceExternalResultKind),
}

impl core::fmt::Display for DivergentUniverseOccurrenceRuntimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Occurrence runtime error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseOccurrenceRuntimeError {}
