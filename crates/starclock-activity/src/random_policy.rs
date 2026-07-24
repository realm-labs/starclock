//! Authored deterministic bootstrap, checkpoint and visible-offer policies.

use crate::{
    ActivityOptionId, ActivityRngLabel, ActivitySlotId, GraphActivityDefinitionError, NodeId,
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
