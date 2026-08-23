//! Closed resource identities and mutation policies used by Rule IR.

/// Closed personal-resource mutation semantics used by evaluated proposals.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceUpdateKind {
    Spend,
    Reserve,
    Gain,
    Set,
}

/// Closed mutation semantics for an authored resource-cap change.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceMaximumUpdateKind {
    Add,
    Subtract,
    Set,
}

/// Closed resource address emitted by Rule IR.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleResourceKind {
    Energy,
    SkillPoints,
    Character(Box<str>),
    Team(Box<str>),
}

/// Cause-relative attribution retained by a queued Rule IR action.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleActionOwner {
    Actor,
    CauseOwner,
    CauseApplier,
}

/// Explicit payer for a queued action's authored Skill Point cost.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleActionPaymentPolicy {
    TeamSkillPoints,
    Suppressed,
    TeamResource(Box<str>),
}
