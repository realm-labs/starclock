//! Immutable Occurrence choice-graph definitions.

use crate::definition::LocalizedText;
use crate::id::{OccurrenceChoiceId, OccurrenceId, OccurrenceVariantId};
use crate::path::ExactParameter;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OccurrenceOperation {
    Battle = 0,
    Consume = 1,
    Discard = 2,
    Enhance = 3,
    Lose = 4,
    Obtain = 5,
    Repair = 6,
    Restore = 7,
    Special = 8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OccurrenceTarget {
    Blessing = 0,
    Character = 1,
    CosmicFragments = 2,
    Curio = 3,
    Hp = 4,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AuthoredScalarUnit {
    Scalar = 0,
    Percent = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AuthoredScalar {
    value: ExactParameter,
    unit: AuthoredScalarUnit,
}

impl AuthoredScalar {
    pub(crate) const fn new(value: ExactParameter, unit: AuthoredScalarUnit) -> Self {
        Self { value, unit }
    }
    #[must_use]
    pub const fn value(self) -> ExactParameter {
        self.value
    }
    #[must_use]
    pub const fn unit(self) -> AuthoredScalarUnit {
        self.unit
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum RandomOutcomePolicy {
    StableUniformOrderedCandidates = 0,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceParameterVector {
    source_option_id: Box<str>,
    values: Box<[ExactParameter]>,
}

impl OccurrenceParameterVector {
    pub(crate) fn new(source_option_id: &str, values: Box<[ExactParameter]>) -> Self {
        Self {
            source_option_id: source_option_id.into(),
            values,
        }
    }
    #[must_use]
    pub fn source_option_id(&self) -> &str {
        &self.source_option_id
    }
    #[must_use]
    pub fn values(&self) -> &[ExactParameter] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceCost {
    operation: OccurrenceOperation,
    targets: Box<[OccurrenceTarget]>,
}

impl OccurrenceCost {
    pub(crate) fn new(operation: OccurrenceOperation, targets: Box<[OccurrenceTarget]>) -> Self {
        Self { operation, targets }
    }
    #[must_use]
    pub const fn operation(&self) -> OccurrenceOperation {
        self.operation
    }
    #[must_use]
    pub fn targets(&self) -> &[OccurrenceTarget] {
        &self.targets
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceOutcome {
    operations: Box<[OccurrenceOperation]>,
    targets: Box<[OccurrenceTarget]>,
    numeric_literals: Box<[AuthoredScalar]>,
    parameter_refs: Box<[Box<str>]>,
    chance_percentages: Box<[ExactParameter]>,
    random_policy: Option<RandomOutcomePolicy>,
}

impl OccurrenceOutcome {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        operations: Box<[OccurrenceOperation]>,
        targets: Box<[OccurrenceTarget]>,
        numeric_literals: Box<[AuthoredScalar]>,
        parameter_refs: Box<[Box<str>]>,
        chance_percentages: Box<[ExactParameter]>,
        random_policy: Option<RandomOutcomePolicy>,
    ) -> Self {
        Self {
            operations,
            targets,
            numeric_literals,
            parameter_refs,
            chance_percentages,
            random_policy,
        }
    }
    #[must_use]
    pub fn operations(&self) -> &[OccurrenceOperation] {
        &self.operations
    }
    #[must_use]
    pub fn targets(&self) -> &[OccurrenceTarget] {
        &self.targets
    }
    #[must_use]
    pub fn numeric_literals(&self) -> &[AuthoredScalar] {
        &self.numeric_literals
    }
    #[must_use]
    pub fn parameter_refs(&self) -> &[Box<str>] {
        &self.parameter_refs
    }
    #[must_use]
    pub fn chance_percentages(&self) -> &[ExactParameter] {
        &self.chance_percentages
    }
    #[must_use]
    pub const fn random_policy(&self) -> Option<RandomOutcomePolicy> {
        self.random_policy
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceChoiceDefinition {
    id: OccurrenceChoiceId,
    stable_key: Box<str>,
    variant: OccurrenceVariantId,
    condition_keys: Box<[Box<str>]>,
    next_node_key: Option<Box<str>>,
    text: LocalizedText,
    label_digests: [[u8; 32]; 2],
    result_digests: [[u8; 32]; 2],
    parameter_vectors: Box<[OccurrenceParameterVector]>,
    costs: Box<[OccurrenceCost]>,
    outcomes: Box<[OccurrenceOutcome]>,
}

impl OccurrenceChoiceDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: OccurrenceChoiceId,
        stable_key: &str,
        variant: OccurrenceVariantId,
        condition_keys: Box<[Box<str>]>,
        next_node_key: Option<Box<str>>,
        text: LocalizedText,
        label_digests: [[u8; 32]; 2],
        result_digests: [[u8; 32]; 2],
        parameter_vectors: Box<[OccurrenceParameterVector]>,
        costs: Box<[OccurrenceCost]>,
        outcomes: Box<[OccurrenceOutcome]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            variant,
            condition_keys,
            next_node_key,
            text,
            label_digests,
            result_digests,
            parameter_vectors,
            costs,
            outcomes,
        }
    }
    #[must_use]
    pub const fn id(&self) -> OccurrenceChoiceId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn variant(&self) -> OccurrenceVariantId {
        self.variant
    }
    #[must_use]
    pub fn condition_keys(&self) -> &[Box<str>] {
        &self.condition_keys
    }
    #[must_use]
    pub fn next_node_key(&self) -> Option<&str> {
        self.next_node_key.as_deref()
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub const fn label_digests(&self) -> [[u8; 32]; 2] {
        self.label_digests
    }
    #[must_use]
    pub const fn result_digests(&self) -> [[u8; 32]; 2] {
        self.result_digests
    }
    #[must_use]
    pub fn parameter_vectors(&self) -> &[OccurrenceParameterVector] {
        &self.parameter_vectors
    }
    #[must_use]
    pub fn costs(&self) -> &[OccurrenceCost] {
        &self.costs
    }
    #[must_use]
    pub fn outcomes(&self) -> &[OccurrenceOutcome] {
        &self.outcomes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceVariantDefinition {
    id: OccurrenceVariantId,
    stable_key: Box<str>,
    occurrence: OccurrenceId,
    entry_node_key: Box<str>,
    condition_keys: Box<[Box<str>]>,
    source_dialogue_type: Box<str>,
    text: LocalizedText,
    choices: Box<[OccurrenceChoiceId]>,
}

impl OccurrenceVariantDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: OccurrenceVariantId,
        stable_key: &str,
        occurrence: OccurrenceId,
        entry_node_key: &str,
        condition_keys: Box<[Box<str>]>,
        source_dialogue_type: &str,
        text: LocalizedText,
        choices: Box<[OccurrenceChoiceId]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            occurrence,
            entry_node_key: entry_node_key.into(),
            condition_keys,
            source_dialogue_type: source_dialogue_type.into(),
            text,
            choices,
        }
    }
    #[must_use]
    pub const fn id(&self) -> OccurrenceVariantId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn occurrence(&self) -> OccurrenceId {
        self.occurrence
    }
    #[must_use]
    pub fn entry_node_key(&self) -> &str {
        &self.entry_node_key
    }
    #[must_use]
    pub fn condition_keys(&self) -> &[Box<str>] {
        &self.condition_keys
    }
    #[must_use]
    pub fn source_dialogue_type(&self) -> &str {
        &self.source_dialogue_type
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn choices(&self) -> &[OccurrenceChoiceId] {
        &self.choices
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceDefinition {
    id: OccurrenceId,
    stable_key: Box<str>,
    choice_graph_key: Box<str>,
    pool_tags: Box<[Box<str>]>,
    index_only: bool,
    text: LocalizedText,
    variants: Box<[OccurrenceVariantId]>,
}

impl OccurrenceDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: OccurrenceId,
        stable_key: &str,
        choice_graph_key: &str,
        pool_tags: Box<[Box<str>]>,
        index_only: bool,
        text: LocalizedText,
        variants: Box<[OccurrenceVariantId]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            choice_graph_key: choice_graph_key.into(),
            pool_tags,
            index_only,
            text,
            variants,
        }
    }
    #[must_use]
    pub const fn id(&self) -> OccurrenceId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub fn choice_graph_key(&self) -> &str {
        &self.choice_graph_key
    }
    #[must_use]
    pub fn pool_tags(&self) -> &[Box<str>] {
        &self.pool_tags
    }
    #[must_use]
    pub const fn index_only(&self) -> bool {
        self.index_only
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn variants(&self) -> &[OccurrenceVariantId] {
        &self.variants
    }
}
