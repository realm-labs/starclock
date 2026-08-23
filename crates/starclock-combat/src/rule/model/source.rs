//! Stable source attribution and closed values carried by Rule IR.

use crate::{Scalar, SourceDefinitionId};

/// Stable generic semantic class for rule attribution and filtering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceClass {
    Unit,
    Ability,
    Effect,
    Equipment,
    Progression,
    Enemy,
    Encounter,
    Activity,
    Mode,
    Synthetic,
}

/// Immutable generic source identity retained by a rule definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleSource {
    pub(super) definition: SourceDefinitionId,
    pub(super) class: SourceClass,
    pub(super) tags: Box<[SourceDefinitionId]>,
    pub(super) digest: [u8; 32],
}

/// Runtime value kind declared by a state slot or expression.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleValueKind {
    Integer,
    Scalar,
    Boolean,
    StableId,
    OptionalStableId,
    OrderedStableIdSet,
}

/// Closed value carried by typed expressions and state-slot emissions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleValue {
    Integer(i64),
    Scalar(Scalar),
    Boolean(bool),
    StableId(u64),
    OptionalStableId(Option<u64>),
    OrderedStableIdSet(Box<[u64]>),
}
