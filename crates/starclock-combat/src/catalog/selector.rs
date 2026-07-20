//! Typed unit-selector plans used by authored Rule IR programs.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorOrigin {
    Source,
    Owner,
    Actor,
    Applier,
    PrimaryTarget,
    CurrentSubject,
    Team,
    Encounter,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorSide {
    Same,
    Opposing,
    Any,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleLifePredicate {
    Any,
    Alive,
    Downed,
    Defeated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RulePresencePredicate {
    Any,
    Present,
    Reserved,
    Departed,
    Untargetable,
    Linked,
    Transformed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorReference {
    CurrentState,
    EventSnapshot,
    ActionSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorOrdering {
    Formation,
    Timeline,
    HpRatioAscending,
    HpRatioDescending,
    StatAscending,
    StatDescending,
    EventOrder,
    StableId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorChoice {
    All,
    First,
    PrimaryPlusAdjacent,
    RngUniform,
    RngWeighted,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleEmptyPoolPolicy {
    NoOp,
    Skip,
    CancelRemaining,
    Fault,
}

/// Complete typed selector plan retained in the immutable combat catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleUnitSelector {
    pub(crate) origin: RuleSelectorOrigin,
    pub(crate) side: RuleSelectorSide,
    pub(crate) life: RuleLifePredicate,
    pub(crate) presence: RulePresencePredicate,
    pub(crate) reference: RuleSelectorReference,
    pub(crate) ordering: RuleSelectorOrdering,
    pub(crate) minimum: u16,
    pub(crate) maximum: u16,
    pub(crate) empty_pool: RuleEmptyPoolPolicy,
    pub(crate) choice: RuleSelectorChoice,
    pub(crate) rng_purpose: Option<Box<str>>,
    pub(crate) repeated: bool,
}

impl RuleUnitSelector {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        origin: RuleSelectorOrigin,
        side: RuleSelectorSide,
        life: RuleLifePredicate,
        presence: RulePresencePredicate,
        reference: RuleSelectorReference,
        ordering: RuleSelectorOrdering,
        minimum: u16,
        maximum: u16,
        empty_pool: RuleEmptyPoolPolicy,
        choice: RuleSelectorChoice,
        rng_purpose: Option<Box<str>>,
        repeated: bool,
    ) -> Option<Self> {
        (maximum > 0 && minimum <= maximum).then_some(Self {
            origin,
            side,
            life,
            presence,
            reference,
            ordering,
            minimum,
            maximum,
            empty_pool,
            choice,
            rng_purpose,
            repeated,
        })
    }

    #[must_use]
    pub const fn origin(&self) -> RuleSelectorOrigin {
        self.origin
    }
    #[must_use]
    pub const fn side(&self) -> RuleSelectorSide {
        self.side
    }
    #[must_use]
    pub const fn life(&self) -> RuleLifePredicate {
        self.life
    }
    #[must_use]
    pub const fn presence(&self) -> RulePresencePredicate {
        self.presence
    }
    #[must_use]
    pub const fn reference(&self) -> RuleSelectorReference {
        self.reference
    }
    #[must_use]
    pub const fn ordering(&self) -> RuleSelectorOrdering {
        self.ordering
    }
    #[must_use]
    pub const fn minimum(&self) -> u16 {
        self.minimum
    }
    #[must_use]
    pub const fn maximum(&self) -> u16 {
        self.maximum
    }
    #[must_use]
    pub const fn empty_pool(&self) -> RuleEmptyPoolPolicy {
        self.empty_pool
    }
    #[must_use]
    pub const fn choice(&self) -> RuleSelectorChoice {
        self.choice
    }
    #[must_use]
    pub fn rng_purpose(&self) -> Option<&str> {
        self.rng_purpose.as_deref()
    }
    #[must_use]
    pub const fn repeated(&self) -> bool {
        self.repeated
    }
}
