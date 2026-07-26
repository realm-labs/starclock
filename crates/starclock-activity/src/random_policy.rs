//! Authored deterministic bootstrap, checkpoint and visible-offer policies.

use crate::{
    ActivityCondition, ActivityOperation, ActivityOptionId, ActivityRngLabel, ActivitySlotId,
    GraphActivityDefinitionError, NodeId,
};

/// Deterministic weighted settlement policy for one internal checkpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityRandomCheckpoint {
    pub(crate) node: NodeId,
    pub(crate) label: ActivityRngLabel,
    pub(crate) purpose: u16,
    pub(crate) weights: Box<[(ActivityOptionId, u64)]>,
}

/// Deterministic weighted, without-replacement candidate policy for one
/// player-visible decision. The complete authored option set remains in the
/// node program; this policy only narrows the currently offered subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityRandomOffer {
    pub(crate) node: NodeId,
    pub(crate) label: ActivityRngLabel,
    pub(crate) purpose: u16,
    pub(crate) maximum_options: u16,
    pub(crate) weights: Box<[(ActivityOptionId, u64)]>,
    pub(crate) reroll_counter: Option<(ActivitySlotId, u32)>,
    pub(crate) maximum_options_reduction: Option<(ActivityCondition, u16)>,
    pub(crate) inactive_condition: Option<ActivityCondition>,
    pub(crate) conditional_weight_multipliers:
        Vec<(ActivityCondition, Box<[ActivityOptionId]>, u64)>,
    pub(crate) conditional_candidate_filters: Vec<(ActivityCondition, Box<[ActivityOptionId]>)>,
    pub(crate) selected_option_marker: Option<ActivitySelectedOptionMarker>,
    pub(crate) selection_prefix: Box<[ActivityOperation]>,
}

/// Records a deterministic subset of the visible offer in a private counter
/// map so option settlement can consume authored per-offer state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActivitySelectedOptionMarker {
    pub(crate) condition: ActivityCondition,
    pub(crate) slot: ActivitySlotId,
    pub(crate) label: ActivityRngLabel,
    pub(crate) purpose: u16,
    pub(crate) count: u16,
}

impl ActivityRandomOffer {
    pub fn new(
        node: NodeId,
        label: ActivityRngLabel,
        purpose: u16,
        maximum_options: u16,
        mut weights: Vec<(ActivityOptionId, u64)>,
        reroll_counter: Option<(ActivitySlotId, u32)>,
    ) -> Result<Self, GraphActivityDefinitionError> {
        weights.sort_by_key(|item| item.0);
        if purpose == 0
            || maximum_options == 0
            || usize::from(maximum_options) > weights.len()
            || weights.is_empty()
            || weights.len() > 256
            || weights.iter().any(|item| item.1 == 0)
            || weights.windows(2).any(|pair| pair[0].0 == pair[1].0)
            || reroll_counter.is_some_and(|(_, maximum)| maximum == 0)
        {
            return Err(GraphActivityDefinitionError::InvalidRandomOffer);
        }
        Ok(Self {
            node,
            label,
            purpose,
            maximum_options,
            weights: weights.into_boxed_slice(),
            reroll_counter,
            maximum_options_reduction: None,
            inactive_condition: None,
            conditional_weight_multipliers: Vec::new(),
            conditional_candidate_filters: Vec::new(),
            selected_option_marker: None,
            selection_prefix: Box::new([]),
        })
    }

    /// Reduces the visible offer width when an authored runtime condition is true.
    #[must_use]
    pub fn with_maximum_options_reduction(
        mut self,
        condition: ActivityCondition,
        reduction: u16,
    ) -> Option<Self> {
        if reduction == 0 || reduction >= self.maximum_options {
            None
        } else {
            self.maximum_options_reduction = Some((condition, reduction));
            Some(self)
        }
    }

    /// Allows the owning node program to bypass the offer under one condition.
    #[must_use]
    pub fn with_inactive_condition(mut self, condition: ActivityCondition) -> Self {
        self.inactive_condition = Some(condition);
        self
    }

    /// Multiplies the authored weights of one stable option subset while a
    /// runtime condition is true. Multiple modifiers compose in insertion
    /// order through checked integer multiplication.
    #[must_use]
    pub fn with_conditional_weight_multiplier(
        mut self,
        condition: ActivityCondition,
        mut options: Vec<ActivityOptionId>,
        multiplier: u64,
    ) -> Option<Self> {
        options.sort_unstable();
        if multiplier == 0
            || options.is_empty()
            || options.windows(2).any(|pair| pair[0] == pair[1])
            || options.iter().any(|option| {
                self.weights
                    .binary_search_by_key(option, |item| item.0)
                    .is_err()
            })
        {
            return None;
        }
        self.conditional_weight_multipliers.push((
            condition,
            options.into_boxed_slice(),
            multiplier,
        ));
        Some(self)
    }

    /// Intersects the candidate pool with one authored option subset when the
    /// condition is true.
    #[must_use]
    pub fn with_conditional_candidate_filter(
        mut self,
        condition: ActivityCondition,
        mut options: Vec<ActivityOptionId>,
    ) -> Option<Self> {
        options.sort_unstable();
        options.dedup();
        if options.is_empty()
            || options.iter().any(|option| {
                self.weights
                    .binary_search_by_key(option, |item| item.0)
                    .is_err()
            })
        {
            return None;
        }
        self.conditional_candidate_filters
            .push((condition, options.into_boxed_slice()));
        Some(self)
    }

    /// Marks a random subset of the final visible offer when the condition is
    /// true. A new offer replaces the complete marker map, including on reroll.
    #[must_use]
    pub fn with_selected_option_marker(
        mut self,
        condition: ActivityCondition,
        slot: ActivitySlotId,
        label: ActivityRngLabel,
        purpose: u16,
        count: u16,
    ) -> Option<Self> {
        if purpose == 0 || count == 0 || count > self.maximum_options {
            return None;
        }
        self.selected_option_marker = Some(ActivitySelectedOptionMarker {
            condition,
            slot,
            label,
            purpose,
            count,
        });
        Some(self)
    }

    /// Applies checked state operations atomically before the selected option.
    /// Boundary operations are rejected when the graph definition is built.
    #[must_use]
    pub fn with_selection_prefix(mut self, operations: Vec<ActivityOperation>) -> Option<Self> {
        if operations.is_empty() {
            return None;
        }
        self.selection_prefix = operations.into_boxed_slice();
        Some(self)
    }

    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
    #[must_use]
    pub const fn label(&self) -> ActivityRngLabel {
        self.label
    }
    #[must_use]
    pub const fn purpose(&self) -> u16 {
        self.purpose
    }
    #[must_use]
    pub const fn maximum_options(&self) -> u16 {
        self.maximum_options
    }
    #[must_use]
    pub fn weights(&self) -> &[(ActivityOptionId, u64)] {
        &self.weights
    }
    #[must_use]
    pub const fn reroll_counter(&self) -> Option<(ActivitySlotId, u32)> {
        self.reroll_counter
    }

    #[must_use]
    pub const fn maximum_options_reduction(&self) -> Option<&(ActivityCondition, u16)> {
        self.maximum_options_reduction.as_ref()
    }

    #[must_use]
    pub const fn inactive_condition(&self) -> Option<&ActivityCondition> {
        self.inactive_condition.as_ref()
    }

    #[must_use]
    pub fn conditional_weight_multipliers(
        &self,
    ) -> &[(ActivityCondition, Box<[ActivityOptionId]>, u64)] {
        &self.conditional_weight_multipliers
    }

    #[must_use]
    pub fn conditional_candidate_filters(&self) -> &[(ActivityCondition, Box<[ActivityOptionId]>)] {
        &self.conditional_candidate_filters
    }

    /// Returns stable metadata for the optional per-offer marker policy.
    #[must_use]
    pub fn selected_option_marker(&self) -> Option<(ActivitySlotId, ActivityRngLabel, u16, u16)> {
        self.selected_option_marker
            .as_ref()
            .map(|marker| (marker.slot, marker.label, marker.purpose, marker.count))
    }

    #[must_use]
    pub fn selection_prefix(&self) -> &[ActivityOperation] {
        &self.selection_prefix
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActivityRandomPolicies {
    pub(crate) checkpoints: Vec<ActivityRandomCheckpoint>,
    pub(crate) offers: Vec<ActivityRandomOffer>,
}

impl ActivityRandomPolicies {
    #[must_use]
    pub fn new(
        checkpoints: Vec<ActivityRandomCheckpoint>,
        offers: Vec<ActivityRandomOffer>,
    ) -> Self {
        Self {
            checkpoints,
            offers,
        }
    }
}

impl ActivityRandomCheckpoint {
    pub fn new(
        node: NodeId,
        label: ActivityRngLabel,
        purpose: u16,
        mut weights: Vec<(ActivityOptionId, u64)>,
    ) -> Result<Self, GraphActivityDefinitionError> {
        weights.sort_by_key(|item| item.0);
        if purpose == 0
            || weights.is_empty()
            || weights.len() > 256
            || weights.iter().any(|item| item.1 == 0)
            || weights.windows(2).any(|pair| pair[0].0 == pair[1].0)
        {
            return Err(GraphActivityDefinitionError::InvalidRandomCheckpoint);
        }
        Ok(Self {
            node,
            label,
            purpose,
            weights: weights.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
    #[must_use]
    pub const fn label(&self) -> ActivityRngLabel {
        self.label
    }
    #[must_use]
    pub const fn purpose(&self) -> u16 {
        self.purpose
    }
    #[must_use]
    pub fn weights(&self) -> &[(ActivityOptionId, u64)] {
        &self.weights
    }
}

/// One deterministic bootstrap draw applied before the entry-node program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBootstrapSelection {
    pub(crate) slot: ActivitySlotId,
    pub(crate) label: ActivityRngLabel,
    pub(crate) purpose: u16,
    pub(crate) candidates: Box<[u64]>,
}

impl ActivityBootstrapSelection {
    pub fn new(
        slot: ActivitySlotId,
        label: ActivityRngLabel,
        purpose: u16,
        candidates: Vec<u64>,
    ) -> Result<Self, GraphActivityDefinitionError> {
        if purpose == 0
            || candidates.is_empty()
            || candidates.len() > 256
            || candidates.contains(&0)
            || candidates.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(GraphActivityDefinitionError::InvalidBootstrapSelection);
        }
        Ok(Self {
            slot,
            label,
            purpose,
            candidates: candidates.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn slot(&self) -> ActivitySlotId {
        self.slot
    }
    #[must_use]
    pub const fn label(&self) -> ActivityRngLabel {
        self.label
    }
    #[must_use]
    pub const fn purpose(&self) -> u16 {
        self.purpose
    }
    #[must_use]
    pub fn candidates(&self) -> &[u64] {
        &self.candidates
    }
}
