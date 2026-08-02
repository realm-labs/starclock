//! Public Gold and Gears Curio lifecycle and offer values.

use crate::id::CurioId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GoldAndGearsCurioId(u32);

impl GoldAndGearsCurioId {
    #[must_use]
    pub const fn new(value: u32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsCurioCategory {
    Normal,
    Negative,
    ErrorCode,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i64)]
pub enum GoldAndGearsCurioState {
    Active = 1,
    Repairing = 2,
    Fixed = 3,
    Destroyed = 4,
    Replaced = 5,
}

impl GoldAndGearsCurioState {
    pub(super) const fn value(self) -> i64 {
        self as i64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsCurioOfferSource {
    TrailblazeBonus,
    AuxiliaryConundrum,
    Occurrence,
    Service,
    Replacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsCurioOfferContext {
    pub(super) source: GoldAndGearsCurioOfferSource,
    pub(super) category: GoldAndGearsCurioCategory,
    pub(super) eligible_keys: Option<Box<[Box<str>]>>,
}

impl GoldAndGearsCurioOfferContext {
    pub fn full_category(
        source: GoldAndGearsCurioOfferSource,
        category: GoldAndGearsCurioCategory,
    ) -> Option<Self> {
        matches!(
            source,
            GoldAndGearsCurioOfferSource::TrailblazeBonus
                | GoldAndGearsCurioOfferSource::AuxiliaryConundrum
        )
        .then_some(Self {
            source,
            category,
            eligible_keys: None,
        })
    }

    pub fn explicit(
        source: GoldAndGearsCurioOfferSource,
        category: GoldAndGearsCurioCategory,
        mut eligible_keys: Vec<Box<str>>,
    ) -> Option<Self> {
        if matches!(
            source,
            GoldAndGearsCurioOfferSource::TrailblazeBonus
                | GoldAndGearsCurioOfferSource::AuxiliaryConundrum
        ) {
            return None;
        }
        eligible_keys.sort_unstable();
        if eligible_keys.iter().any(|key| key.trim().is_empty())
            || eligible_keys.windows(2).any(|pair| pair[0] == pair[1])
        {
            return None;
        }
        Some(Self {
            source,
            category,
            eligible_keys: Some(eligible_keys.into_boxed_slice()),
        })
    }

    #[must_use]
    pub const fn source(&self) -> GoldAndGearsCurioOfferSource {
        self.source
    }

    #[must_use]
    pub const fn category(&self) -> GoldAndGearsCurioCategory {
        self.category
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GoldAndGearsCurioParameter {
    coefficient: i64,
    scale: u8,
}

impl GoldAndGearsCurioParameter {
    pub(super) const fn new(coefficient: i64, scale: u8) -> Self {
        Self { coefficient, scale }
    }

    #[must_use]
    pub const fn coefficient(self) -> i64 {
        self.coefficient
    }

    #[must_use]
    pub const fn scale(self) -> u8 {
        self.scale
    }
}

/// Frozen Curio rule responsibility bound to one production executor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsCurioRuleKind {
    LifecycleState,
    Contribution,
}

/// Truthful owner of a Curio rule's executable semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsCurioRuleOwnership {
    Shared,
    GoldAndGears,
}

/// One of the 160 frozen Curio lifecycle rules with terminal dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsCurioRuleBinding {
    pub(super) rule_id: Box<str>,
    pub(super) owner_id: Box<str>,
    pub(super) kind: GoldAndGearsCurioRuleKind,
    pub(super) ownership: GoldAndGearsCurioRuleOwnership,
}

impl GoldAndGearsCurioRuleBinding {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn kind(&self) -> GoldAndGearsCurioRuleKind {
        self.kind
    }

    #[must_use]
    pub const fn ownership(&self) -> GoldAndGearsCurioRuleOwnership {
        self.ownership
    }

    #[must_use]
    pub const fn accuracy(&self) -> &'static str {
        "ProjectPolicy"
    }

    #[must_use]
    pub const fn executor(&self) -> &'static str {
        match self.ownership {
            GoldAndGearsCurioRuleOwnership::Shared => "ReleasedSharedExecutor",
            GoldAndGearsCurioRuleOwnership::GoldAndGears => "ActivityAndCombatPrograms",
        }
    }

    #[must_use]
    pub const fn operation(&self) -> &'static str {
        match self.kind {
            GoldAndGearsCurioRuleKind::LifecycleState => "ExecuteCurioLifecycle",
            GoldAndGearsCurioRuleKind::Contribution => "ProjectCurioContribution",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsCurioDefinition {
    pub(super) id: GoldAndGearsCurioId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: u32,
    pub(super) lifecycle_rule: Box<str>,
    pub(super) contribution_rule: Box<str>,
    pub(super) handbook_order: u16,
    pub(super) category: GoldAndGearsCurioCategory,
    pub(super) shared_curio: Option<CurioId>,
    pub(super) initial_state: GoldAndGearsCurioState,
    pub(super) terminal_state: GoldAndGearsCurioState,
    pub(super) maximum_charges: Option<u8>,
    pub(super) decrement_event: Box<str>,
    pub(super) repair_after_completed_battles: Option<u8>,
    pub(super) source_effect_id: Box<str>,
    pub(super) parameters: Box<[GoldAndGearsCurioParameter]>,
    pub(super) fixed_source_effect_id: Option<Box<str>>,
    pub(super) fixed_parameters: Box<[GoldAndGearsCurioParameter]>,
    pub(super) replaces_all_possessed: bool,
    pub(super) post_destruction_effect: Option<Box<str>>,
}

impl GoldAndGearsCurioDefinition {
    #[must_use]
    pub const fn id(&self) -> GoldAndGearsCurioId {
        self.id
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub const fn source_id(&self) -> u32 {
        self.source_id
    }

    #[must_use]
    pub const fn handbook_order(&self) -> u16 {
        self.handbook_order
    }

    #[must_use]
    pub const fn category(&self) -> GoldAndGearsCurioCategory {
        self.category
    }

    #[must_use]
    pub const fn shared_curio(&self) -> Option<CurioId> {
        self.shared_curio
    }

    #[must_use]
    pub const fn initial_state(&self) -> GoldAndGearsCurioState {
        self.initial_state
    }

    #[must_use]
    pub const fn terminal_state(&self) -> GoldAndGearsCurioState {
        self.terminal_state
    }

    #[must_use]
    pub const fn maximum_charges(&self) -> Option<u8> {
        self.maximum_charges
    }

    #[must_use]
    pub fn decrement_event(&self) -> &str {
        &self.decrement_event
    }

    #[must_use]
    pub const fn repair_after_completed_battles(&self) -> Option<u8> {
        self.repair_after_completed_battles
    }

    #[must_use]
    pub const fn replaces_all_possessed(&self) -> bool {
        self.replaces_all_possessed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsCurioCandidate {
    pub(super) id: GoldAndGearsCurioId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: u32,
    pub(super) category: GoldAndGearsCurioCategory,
    pub(super) shared: bool,
}

impl GoldAndGearsCurioCandidate {
    #[must_use]
    pub const fn id(&self) -> GoldAndGearsCurioId {
        self.id
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub const fn source_id(&self) -> u32 {
        self.source_id
    }

    #[must_use]
    pub const fn category(&self) -> GoldAndGearsCurioCategory {
        self.category
    }

    #[must_use]
    pub const fn shared(&self) -> bool {
        self.shared
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsCurioContribution {
    pub(super) id: GoldAndGearsCurioId,
    pub(super) shared_curio: Option<CurioId>,
    pub(super) state: GoldAndGearsCurioState,
    pub(super) remaining_or_progress: u8,
    pub(super) source_effect_id: Box<str>,
    pub(super) parameters: Box<[GoldAndGearsCurioParameter]>,
}

impl GoldAndGearsCurioContribution {
    #[must_use]
    pub const fn id(&self) -> GoldAndGearsCurioId {
        self.id
    }

    #[must_use]
    pub const fn shared_curio(&self) -> Option<CurioId> {
        self.shared_curio
    }

    #[must_use]
    pub const fn state(&self) -> GoldAndGearsCurioState {
        self.state
    }

    #[must_use]
    pub const fn remaining_or_progress(&self) -> u8 {
        self.remaining_or_progress
    }

    #[must_use]
    pub fn source_effect_id(&self) -> &str {
        &self.source_effect_id
    }

    #[must_use]
    pub fn parameters(&self) -> &[GoldAndGearsCurioParameter] {
        &self.parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsCurioContributionSet {
    pub(super) entries: Box<[GoldAndGearsCurioContribution]>,
    pub(super) digest: [u8; 32],
}

impl GoldAndGearsCurioContributionSet {
    #[must_use]
    pub fn entries(&self) -> &[GoldAndGearsCurioContribution] {
        &self.entries
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}
