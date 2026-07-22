//! Immutable run service, shop-policy and Ability Tree definitions.

use crate::definition::LocalizedText;
use crate::id::{AbilityTreeNodeId, ServiceId};
use crate::path::ExactParameter;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ServiceKind {
    Currency = 0,
    ResetBlessing = 1,
    Reviver = 2,
    Downloader = 3,
    RespiteOffers = 4,
    EnhanceBlessing = 5,
    BlessingShop = 6,
    CurioShop = 7,
    TrailblazeBonus = 8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceParameter {
    key: Box<str>,
    value: Box<str>,
}

impl ServiceParameter {
    pub(crate) fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceDefinition {
    id: ServiceId,
    stable_key: Box<str>,
    kind: ServiceKind,
    currency_key: Option<Box<str>>,
    price_formula_key: Option<Box<str>>,
    offer_pool_key: Option<Box<str>>,
    rule_key: Box<str>,
    text: LocalizedText,
    parameters: Box<[ServiceParameter]>,
}

impl ServiceDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: ServiceId,
        stable_key: &str,
        kind: ServiceKind,
        currency_key: Option<Box<str>>,
        price_formula_key: Option<Box<str>>,
        offer_pool_key: Option<Box<str>>,
        rule_key: &str,
        text: LocalizedText,
        parameters: Box<[ServiceParameter]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            kind,
            currency_key,
            price_formula_key,
            offer_pool_key,
            rule_key: rule_key.into(),
            text,
            parameters,
        }
    }
    #[must_use]
    pub const fn id(&self) -> ServiceId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn kind(&self) -> ServiceKind {
        self.kind
    }
    #[must_use]
    pub fn currency_key(&self) -> Option<&str> {
        self.currency_key.as_deref()
    }
    #[must_use]
    pub fn price_formula_key(&self) -> Option<&str> {
        self.price_formula_key.as_deref()
    }
    #[must_use]
    pub fn offer_pool_key(&self) -> Option<&str> {
        self.offer_pool_key.as_deref()
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn parameters(&self) -> &[ServiceParameter] {
        &self.parameters
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityEffectClass {
    Run = 0,
    Battle = 1,
    RunAndBattle = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityOperation {
    Unlock = 0,
    AddStat = 1,
    UnlockFormationSlot = 2,
    Set = 3,
    AddLimit = 4,
    Enable = 5,
    AddCurrency = 6,
    AddChoice = 7,
    AddResource = 8,
    SetRatio = 9,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityValueUnit {
    Boolean = 0,
    Flat = 1,
    Count = 2,
    Ratio = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityTreeCost {
    source_item_id: Box<str>,
    amount: ExactParameter,
}

impl AbilityTreeCost {
    pub(crate) fn new(source_item_id: &str, amount: ExactParameter) -> Self {
        Self {
            source_item_id: source_item_id.into(),
            amount,
        }
    }
    #[must_use]
    pub fn source_item_id(&self) -> &str {
        &self.source_item_id
    }
    #[must_use]
    pub const fn amount(&self) -> ExactParameter {
        self.amount
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityTreeEffect {
    operation: AbilityOperation,
    target_key: Box<str>,
    value: ExactParameter,
    unit: AbilityValueUnit,
    condition: Option<Box<str>>,
}

impl AbilityTreeEffect {
    pub(crate) fn new(
        operation: AbilityOperation,
        target_key: &str,
        value: ExactParameter,
        unit: AbilityValueUnit,
        condition: Option<Box<str>>,
    ) -> Self {
        Self {
            operation,
            target_key: target_key.into(),
            value,
            unit,
            condition,
        }
    }
    #[must_use]
    pub const fn operation(&self) -> AbilityOperation {
        self.operation
    }
    #[must_use]
    pub fn target_key(&self) -> &str {
        &self.target_key
    }
    #[must_use]
    pub const fn value(&self) -> ExactParameter {
        self.value
    }
    #[must_use]
    pub const fn unit(&self) -> AbilityValueUnit {
        self.unit
    }
    #[must_use]
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityTreeNodeDefinition {
    id: AbilityTreeNodeId,
    stable_key: Box<str>,
    important: bool,
    effect_class: AbilityEffectClass,
    effect_tag_en: Box<str>,
    effect_tag_zh_cn: Box<str>,
    external_unlock_keys: Box<[Box<str>]>,
    rule_key: Box<str>,
    text: LocalizedText,
    prerequisites: Box<[AbilityTreeNodeId]>,
    costs: Box<[AbilityTreeCost]>,
    effects: Box<[AbilityTreeEffect]>,
    parameters: Box<[ExactParameter]>,
}

impl AbilityTreeNodeDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: AbilityTreeNodeId,
        stable_key: &str,
        important: bool,
        effect_class: AbilityEffectClass,
        effect_tag_en: &str,
        effect_tag_zh_cn: &str,
        external_unlock_keys: Box<[Box<str>]>,
        rule_key: &str,
        text: LocalizedText,
        prerequisites: Box<[AbilityTreeNodeId]>,
        costs: Box<[AbilityTreeCost]>,
        effects: Box<[AbilityTreeEffect]>,
        parameters: Box<[ExactParameter]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            important,
            effect_class,
            effect_tag_en: effect_tag_en.into(),
            effect_tag_zh_cn: effect_tag_zh_cn.into(),
            external_unlock_keys,
            rule_key: rule_key.into(),
            text,
            prerequisites,
            costs,
            effects,
            parameters,
        }
    }
    #[must_use]
    pub const fn id(&self) -> AbilityTreeNodeId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn important(&self) -> bool {
        self.important
    }
    #[must_use]
    pub const fn effect_class(&self) -> AbilityEffectClass {
        self.effect_class
    }
    #[must_use]
    pub fn effect_tag_en(&self) -> &str {
        &self.effect_tag_en
    }
    #[must_use]
    pub fn effect_tag_zh_cn(&self) -> &str {
        &self.effect_tag_zh_cn
    }
    #[must_use]
    pub fn external_unlock_keys(&self) -> &[Box<str>] {
        &self.external_unlock_keys
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn prerequisites(&self) -> &[AbilityTreeNodeId] {
        &self.prerequisites
    }
    #[must_use]
    pub fn costs(&self) -> &[AbilityTreeCost] {
        &self.costs
    }
    #[must_use]
    pub fn effects(&self) -> &[AbilityTreeEffect] {
        &self.effects
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
}
