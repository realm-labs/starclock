//! Public service offers and externally supplied Adventure outcomes.

use crate::id::BlessingId;

use super::curio_types::GoldAndGearsCurioCandidate;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsServiceAdventureRuleKind {
    AdventureOutcome,
    ServiceBridge,
    ReleasedService,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsServiceAdventureRuleAccuracy {
    ExactPublic,
    ProjectPolicy,
}

/// One frozen Service or Adventure rule bound to the released shared executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsServiceAdventureRuleBinding {
    pub(super) rule_id: Box<str>,
    pub(super) owner_id: Box<str>,
    pub(super) kind: GoldAndGearsServiceAdventureRuleKind,
    pub(super) accuracy: GoldAndGearsServiceAdventureRuleAccuracy,
}

impl GoldAndGearsServiceAdventureRuleBinding {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub const fn kind(&self) -> GoldAndGearsServiceAdventureRuleKind {
        self.kind
    }

    #[must_use]
    pub const fn accuracy(&self) -> GoldAndGearsServiceAdventureRuleAccuracy {
        self.accuracy
    }

    #[must_use]
    pub const fn accuracy_name(&self) -> &'static str {
        match self.accuracy {
            GoldAndGearsServiceAdventureRuleAccuracy::ExactPublic => "ExactPublic",
            GoldAndGearsServiceAdventureRuleAccuracy::ProjectPolicy => "ProjectPolicy",
        }
    }

    #[must_use]
    pub const fn executor(&self) -> &'static str {
        "ReleasedSharedExecutor"
    }

    #[must_use]
    pub const fn operation(&self) -> &'static str {
        match self.kind {
            GoldAndGearsServiceAdventureRuleKind::AdventureOutcome => "ResolveAdventureOutcome",
            GoldAndGearsServiceAdventureRuleKind::ServiceBridge => "ExecuteServiceBridge",
            GoldAndGearsServiceAdventureRuleKind::ReleasedService => "ExecuteReleasedService",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsServiceKind {
    BlessingShop,
    CurioShop,
    Currency,
    Downloader,
    EnhanceBlessing,
    ResetBlessing,
    RespiteOffers,
    Reviver,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsServiceOfferSelector {
    BlessingRarity(u8),
    CurioSlot(u8),
    UseIndex(u8),
    RespitePosition(u8),
    Reviver,
    Downloader,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsServiceStock {
    pub(super) selector: GoldAndGearsServiceOfferSelector,
    pub(super) unit_cost: u32,
    pub(super) maximum_uses: u8,
}

impl GoldAndGearsServiceStock {
    #[must_use]
    pub const fn selector(self) -> GoldAndGearsServiceOfferSelector {
        self.selector
    }

    #[must_use]
    pub const fn unit_cost(self) -> u32 {
        self.unit_cost
    }

    #[must_use]
    pub const fn maximum_uses(self) -> u8 {
        self.maximum_uses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsServiceDefinition {
    pub(super) id: u32,
    pub(super) stable_key: Box<str>,
    pub(super) kind: GoldAndGearsServiceKind,
    pub(super) currency: Option<Box<str>>,
    pub(super) price_formula: Option<Box<str>>,
    pub(super) offer_pool: Option<Box<str>>,
    pub(super) stock: Box<[GoldAndGearsServiceStock]>,
    pub(super) bridge_rule: Box<str>,
    pub(super) released_rule: Box<str>,
}

impl GoldAndGearsServiceDefinition {
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub const fn kind(&self) -> GoldAndGearsServiceKind {
        self.kind
    }

    #[must_use]
    pub fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }

    #[must_use]
    pub fn price_formula(&self) -> Option<&str> {
        self.price_formula.as_deref()
    }

    #[must_use]
    pub fn offer_pool(&self) -> Option<&str> {
        self.offer_pool.as_deref()
    }

    #[must_use]
    pub fn stock(&self) -> &[GoldAndGearsServiceStock] {
        &self.stock
    }

    #[must_use]
    pub fn bridge_rule(&self) -> &str {
        &self.bridge_rule
    }

    #[must_use]
    pub fn released_rule(&self) -> &str {
        &self.released_rule
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsAdventureType {
    CaptureMonster,
    DestroyProp,
    EscapeLaser,
    Turntable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsAdventureMetric {
    Points,
    DestroyedObjects,
    EvadedCycles,
    AlignedHands,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsTechniqueRule {
    Allowed,
    Disabled,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsAdventureThreshold {
    pub(super) objective: u8,
    pub(super) minimum_value: u32,
}

impl GoldAndGearsAdventureThreshold {
    #[must_use]
    pub const fn objective(self) -> u8 {
        self.objective
    }

    #[must_use]
    pub const fn minimum_value(self) -> u32 {
        self.minimum_value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsAdventureDefinition {
    pub(super) id: u32,
    pub(super) stable_key: Box<str>,
    pub(super) room: Box<str>,
    pub(super) adventure_type: GoldAndGearsAdventureType,
    pub(super) metric: GoldAndGearsAdventureMetric,
    pub(super) thresholds: Box<[GoldAndGearsAdventureThreshold]>,
    pub(super) maximum_value: u32,
    pub(super) time_limit_seconds: Option<u16>,
    pub(super) technique_rule: GoldAndGearsTechniqueRule,
    pub(super) rule: Box<str>,
}

impl GoldAndGearsAdventureDefinition {
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }

    #[must_use]
    pub fn room(&self) -> &str {
        &self.room
    }

    #[must_use]
    pub const fn adventure_type(&self) -> GoldAndGearsAdventureType {
        self.adventure_type
    }

    #[must_use]
    pub const fn metric(&self) -> GoldAndGearsAdventureMetric {
        self.metric
    }

    #[must_use]
    pub fn thresholds(&self) -> &[GoldAndGearsAdventureThreshold] {
        &self.thresholds
    }

    #[must_use]
    pub const fn maximum_value(&self) -> u32 {
        self.maximum_value
    }

    #[must_use]
    pub const fn time_limit_seconds(&self) -> Option<u16> {
        self.time_limit_seconds
    }

    #[must_use]
    pub const fn technique_rule(&self) -> GoldAndGearsTechniqueRule {
        self.technique_rule
    }

    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsAdventureExternalOutcome {
    adventure: u32,
    achieved_value: u32,
}

impl GoldAndGearsAdventureExternalOutcome {
    #[must_use]
    pub const fn new(adventure: u32, achieved_value: u32) -> Option<Self> {
        if adventure == 0 {
            None
        } else {
            Some(Self {
                adventure,
                achieved_value,
            })
        }
    }

    #[must_use]
    pub const fn adventure(self) -> u32 {
        self.adventure
    }

    #[must_use]
    pub const fn achieved_value(self) -> u32 {
        self.achieved_value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsAdventureRewardPlan {
    pub(super) adventure: u32,
    pub(super) completed_objectives: u8,
    pub(super) cosmic_fragments: u32,
    pub(super) blessing_rarity: Option<u8>,
    pub(super) curio_choice: bool,
    pub(super) blessing_offer: Option<BlessingId>,
    pub(super) curio_offer: Option<GoldAndGearsCurioCandidate>,
}

impl GoldAndGearsAdventureRewardPlan {
    #[must_use]
    pub const fn adventure(&self) -> u32 {
        self.adventure
    }

    #[must_use]
    pub const fn completed_objectives(&self) -> u8 {
        self.completed_objectives
    }

    #[must_use]
    pub const fn cosmic_fragments(&self) -> u32 {
        self.cosmic_fragments
    }

    #[must_use]
    pub const fn blessing_rarity(&self) -> Option<u8> {
        self.blessing_rarity
    }

    #[must_use]
    pub const fn offers_curio(&self) -> bool {
        self.curio_choice
    }

    #[must_use]
    pub const fn blessing_offer(&self) -> Option<BlessingId> {
        self.blessing_offer
    }

    #[must_use]
    pub const fn curio_offer(&self) -> Option<&GoldAndGearsCurioCandidate> {
        self.curio_offer.as_ref()
    }
}
