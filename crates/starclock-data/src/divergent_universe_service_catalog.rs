//! Immutable economy, service, Occurrence and external-Adventure definitions.

use std::collections::{BTreeMap, BTreeSet};

macro_rules! stable_id {
    ($name:ident,$prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(v: impl Into<Box<str>>) -> Result<Self, DivergentUniverseServiceError> {
                let v = v.into();
                if !v.starts_with($prefix) || v.len() == $prefix.len() {
                    Err(error(concat!(stringify!($name), " namespace mismatch")))
                } else {
                    Ok(Self(v))
                }
            }
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
stable_id!(DivergentUniverseCurrencyId, "divergent-universe.currency.");
stable_id!(
    DivergentUniverseWorkbenchId,
    "divergent-universe.workbench."
);
stable_id!(
    DivergentUniverseWorkbenchFunctionId,
    "divergent-universe.workbench-function."
);
stable_id!(
    DivergentUniverseGambleGroupId,
    "divergent-universe.gamble-group."
);
stable_id!(
    DivergentUniverseGambleUnitId,
    "divergent-universe.gamble-unit."
);
stable_id!(
    DivergentUniverseCurseChestId,
    "divergent-universe.curse-chest."
);
stable_id!(
    DivergentUniverseOccurrenceId,
    "divergent-universe.occurrence."
);
stable_id!(
    DivergentUniverseOccurrenceVariantId,
    "divergent-universe.occurrence-variant."
);
stable_id!(
    DivergentUniverseModeServiceId,
    "divergent-universe.mode-service-npc."
);
stable_id!(
    DivergentUniverseServiceRuleId,
    "divergent-universe.service-rule."
);
stable_id!(
    DivergentUniverseServiceOfferId,
    "divergent-universe.service-offer."
);
stable_id!(
    DivergentUniverseAdventureOutcomeId,
    "divergent-universe.adventure-outcome."
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurrencyDefinition {
    pub id: DivergentUniverseCurrencyId,
    pub scope: Box<str>,
    pub gain_rules: Box<[Box<str>]>,
    pub spend_rules: Box<[Box<str>]>,
    pub reset_rule: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWorkbenchDefinition {
    pub id: DivergentUniverseWorkbenchId,
    pub functions: Box<[DivergentUniverseWorkbenchFunctionId]>,
    pub currencies: Box<[DivergentUniverseCurrencyId]>,
    pub availability: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniversePriceRule {
    pub currency: Box<str>,
    pub formula: Box<str>,
    pub reset: Box<str>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWorkbenchFunctionDefinition {
    pub id: DivergentUniverseWorkbenchFunctionId,
    pub function_type: Box<str>,
    pub input_policy: Box<str>,
    pub output_policy: Box<str>,
    pub price_rule: DivergentUniversePriceRule,
    pub candidates: Box<[Box<str>]>,
    pub weights: Box<[Box<str>]>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGambleGroupDefinition {
    pub id: DivergentUniverseGambleGroupId,
    pub group_type: Box<str>,
    pub group_level: Box<str>,
    pub units: Box<[DivergentUniverseGambleUnitId]>,
    pub weights: Box<[Box<str>]>,
    pub draw_count: Box<str>,
    pub offer_policy: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGambleOutcome {
    pub operation: Box<str>,
    pub category: Option<Box<str>>,
    pub source_group_id: Option<Box<str>>,
    pub currency_id: Option<DivergentUniverseCurrencyId>,
    pub amount: Option<Box<str>>,
    pub resolution: Option<Box<str>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGambleUnitDefinition {
    pub id: DivergentUniverseGambleUnitId,
    pub unit_type: Box<str>,
    pub outcome: DivergentUniverseGambleOutcome,
    pub parameters: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurseChoice {
    pub operation: Box<str>,
    pub count: Option<Box<str>>,
    pub minimum: Option<Box<str>>,
    pub maximum: Option<Box<str>>,
    pub maximum_count: Option<Box<str>>,
    pub pool: Option<Box<str>>,
    pub preferred_path: Option<Box<str>>,
    pub shard: Option<Box<str>>,
    pub weight_policy: Option<Box<str>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurseChestDefinition {
    pub id: DivergentUniverseCurseChestId,
    pub chest_type: Box<str>,
    pub choices: Box<[DivergentUniverseCurseChoice]>,
    pub parameters: Box<[Box<str>]>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceUnlock {
    pub ordinal: u16,
    pub source_field: Box<str>,
    pub variant: DivergentUniverseOccurrenceVariantId,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceDefinition {
    pub id: DivergentUniverseOccurrenceId,
    pub variants: Box<[DivergentUniverseOccurrenceVariantId]>,
    pub choice_ids: Box<[Box<str>]>,
    pub handbook_priority: u16,
    pub handbook_used: bool,
    pub selection_policy: Box<str>,
    pub unlock_rules: Box<[DivergentUniverseOccurrenceUnlock]>,
    pub unresolved_offer_behavior: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceEntryCondition {
    pub kind: Box<str>,
    pub occurrence: DivergentUniverseOccurrenceId,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOccurrenceVariantDefinition {
    pub id: DivergentUniverseOccurrenceVariantId,
    pub occurrence: DivergentUniverseOccurrenceId,
    pub occurrences: Box<[DivergentUniverseOccurrenceId]>,
    pub entry_conditions: Box<[DivergentUniverseOccurrenceEntryCondition]>,
    pub choice_ids: Box<[Box<str>]>,
    pub graph_path: Box<str>,
    pub graph_resolution: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseModeServiceDefinition {
    pub id: DivergentUniverseModeServiceId,
    pub service_kind: Box<str>,
    pub choice_ids: Box<[Box<str>]>,
    pub graph_path: Box<str>,
    pub graph_resolution: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseServiceRuleDefinition {
    pub id: DivergentUniverseServiceRuleId,
    pub service_kind: Box<str>,
    pub currency: Option<DivergentUniverseCurrencyId>,
    pub price: Box<str>,
    pub ordered_operations: Box<[Box<str>]>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseServiceOfferDefinition {
    pub id: DivergentUniverseServiceOfferId,
    pub service_id: Box<str>,
    pub candidates: Box<[Box<str>]>,
    pub weights: Box<[Box<str>]>,
    pub refresh_rule: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergentUniverseServiceValue {
    Scalar(Box<str>),
    Array(Box<[Box<str>]>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAdventureParameter {
    pub fields: Box<[(Box<str>, DivergentUniverseServiceValue)]>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAbstractOutcome {
    pub input: Option<Box<str>>,
    pub kind: Box<str>,
    pub ordered_operations: Box<[Box<str>]>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAdventureOutcomeDefinition {
    pub id: DivergentUniverseAdventureOutcomeId,
    pub adventure_type: Box<str>,
    pub room_id: Box<str>,
    pub parameter_group_id: Box<str>,
    pub parameter_program: Box<[DivergentUniverseAdventureParameter]>,
    pub abstract_outcome: DivergentUniverseAbstractOutcome,
    pub action_gameplay: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseServiceCatalogParts {
    pub currencies: Vec<DivergentUniverseCurrencyDefinition>,
    pub workbenches: Vec<DivergentUniverseWorkbenchDefinition>,
    pub functions: Vec<DivergentUniverseWorkbenchFunctionDefinition>,
    pub gamble_groups: Vec<DivergentUniverseGambleGroupDefinition>,
    pub gamble_units: Vec<DivergentUniverseGambleUnitDefinition>,
    pub curse_chests: Vec<DivergentUniverseCurseChestDefinition>,
    pub occurrences: Vec<DivergentUniverseOccurrenceDefinition>,
    pub variants: Vec<DivergentUniverseOccurrenceVariantDefinition>,
    pub mode_services: Vec<DivergentUniverseModeServiceDefinition>,
    pub service_rules: Vec<DivergentUniverseServiceRuleDefinition>,
    pub offers: Vec<DivergentUniverseServiceOfferDefinition>,
    pub adventures: Vec<DivergentUniverseAdventureOutcomeDefinition>,
    pub source_obligations: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseServiceCatalog {
    parts: DivergentUniverseServiceCatalogParts,
}
impl DivergentUniverseServiceCatalog {
    pub fn new(
        mut p: DivergentUniverseServiceCatalogParts,
    ) -> Result<Self, DivergentUniverseServiceError> {
        let counts = [
            (p.currencies.len(), 2),
            (p.workbenches.len(), 11),
            (p.functions.len(), 6),
            (p.gamble_groups.len(), 126),
            (p.gamble_units.len(), 89),
            (p.curse_chests.len(), 29),
            (p.occurrences.len(), 118),
            (p.variants.len(), 97),
            (p.mode_services.len(), 23),
            (p.service_rules.len(), 6),
            (p.offers.len(), 161),
            (p.adventures.len(), 32),
        ];
        if counts.iter().any(|(a, e)| a != e) {
            return Err(error("service/Occurrence table denominator drift"));
        }
        if p.source_obligations != 531 {
            return Err(error("service/Occurrence source obligation closure drift"));
        }
        p.currencies.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.workbenches.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.functions.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.gamble_groups.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.gamble_units.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.curse_chests.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.occurrences.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.variants.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.mode_services.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.service_rules.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.offers.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.adventures.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        validate(&p)?;
        Ok(Self { parts: p })
    }
    #[must_use]
    pub fn currencies(&self) -> &[DivergentUniverseCurrencyDefinition] {
        &self.parts.currencies
    }
    #[must_use]
    pub fn workbenches(&self) -> &[DivergentUniverseWorkbenchDefinition] {
        &self.parts.workbenches
    }
    #[must_use]
    pub fn functions(&self) -> &[DivergentUniverseWorkbenchFunctionDefinition] {
        &self.parts.functions
    }
    #[must_use]
    pub fn gamble_groups(&self) -> &[DivergentUniverseGambleGroupDefinition] {
        &self.parts.gamble_groups
    }
    #[must_use]
    pub fn gamble_units(&self) -> &[DivergentUniverseGambleUnitDefinition] {
        &self.parts.gamble_units
    }
    #[must_use]
    pub fn curse_chests(&self) -> &[DivergentUniverseCurseChestDefinition] {
        &self.parts.curse_chests
    }
    #[must_use]
    pub fn occurrences(&self) -> &[DivergentUniverseOccurrenceDefinition] {
        &self.parts.occurrences
    }
    #[must_use]
    pub fn variants(&self) -> &[DivergentUniverseOccurrenceVariantDefinition] {
        &self.parts.variants
    }
    #[must_use]
    pub fn mode_services(&self) -> &[DivergentUniverseModeServiceDefinition] {
        &self.parts.mode_services
    }
    #[must_use]
    pub fn service_rules(&self) -> &[DivergentUniverseServiceRuleDefinition] {
        &self.parts.service_rules
    }
    #[must_use]
    pub fn offers(&self) -> &[DivergentUniverseServiceOfferDefinition] {
        &self.parts.offers
    }
    #[must_use]
    pub fn adventures(&self) -> &[DivergentUniverseAdventureOutcomeDefinition] {
        &self.parts.adventures
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.parts.source_obligations
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseServiceCatalogParts {
        self.parts
    }
}
fn validate(p: &DivergentUniverseServiceCatalogParts) -> Result<(), DivergentUniverseServiceError> {
    let currencies = p
        .currencies
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let functions = p
        .functions
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let groups = p
        .gamble_groups
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let chests = p
        .curse_chests
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let occurrences = p
        .occurrences
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let variants = p
        .variants
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    if currencies.len() != 2
        || functions.len() != 6
        || groups.len() != 126
        || chests.len() != 29
        || occurrences.len() != 118
        || variants.len() != 97
    {
        return Err(error("duplicate service/Occurrence identity"));
    }
    if p.workbenches.iter().any(|x| {
        x.runtime_lowered
            || x.functions.is_empty()
            || x.functions.len() > 3
            || x.availability.as_ref() != "Unspecified"
            || x.functions.iter().any(|id| !functions.contains_key(id))
            || x.currencies.iter().any(|id| !currencies.contains_key(id))
    }) {
        return Err(error("Workbench closure drift"));
    }
    if p.functions.iter().any(|x| {
        x.runtime_lowered
            || !x.candidates.is_empty()
            || !x.weights.is_empty()
            || x.fallback.as_ref() != "RejectWithoutMutation"
    }) {
        return Err(error("Workbench candidate boundary drift"));
    }
    if p.gamble_groups.iter().any(|x| {
        x.runtime_lowered
            || !x.units.is_empty()
            || !x.weights.is_empty()
            || x.fallback.as_ref() != "RejectWithoutMutation"
    }) {
        return Err(error("Gamble group must remain fail closed"));
    }
    if p.gamble_units.iter().any(|x| {
        x.runtime_lowered
            || x.outcome
                .currency_id
                .as_ref()
                .is_some_and(|id| !currencies.contains_key(id))
    }) {
        return Err(error("Gamble unit outcome closure drift"));
    }
    if p.curse_chests.iter().any(|x| {
        x.runtime_lowered
            || x.choices.len() != 3
            || x.choices
                .last()
                .is_none_or(|choice| choice.operation.as_ref() != "LeaveWithoutMutation")
            || x.fallback.as_ref() != "LeaveWithoutMutation"
    }) {
        return Err(error("Curse Chest choice closure drift"));
    }
    if p.occurrences.iter().any(|x| {
        x.runtime_lowered
            || x.variants.len() != 1
            || !x.choice_ids.is_empty()
            || x.unlock_rules.is_empty()
            || x.unresolved_offer_behavior.as_ref() != "FailClosed"
            || x.variants.iter().any(|id| !variants.contains_key(id))
    }) {
        return Err(error("Occurrence identity boundary drift"));
    }
    if p.variants.iter().any(|x| {
        x.runtime_lowered
            || !occurrences.contains_key(&x.occurrence)
            || x.occurrences.is_empty()
            || x.occurrences[0] != x.occurrence
            || x.occurrences.iter().any(|id| !occurrences.contains_key(id))
            || !x.choice_ids.is_empty()
            || x.graph_resolution.as_ref() != "MissingAtPinnedRevision"
            || x.fallback.as_ref() != "RejectWithoutMutation"
    }) {
        return Err(error("Occurrence variant fail-closed boundary drift"));
    }
    if p.mode_services.iter().any(|x| {
        x.runtime_lowered
            || !x.choice_ids.is_empty()
            || x.graph_resolution.as_ref() != "MissingAtPinnedRevision"
            || x.service_kind.as_ref() != "UnclassifiedMissingGraph"
            || x.fallback.as_ref() != "RejectWithoutMutation"
    }) {
        return Err(error("mode service missing-graph boundary drift"));
    }
    if p.service_rules.iter().any(|x| {
        x.runtime_lowered
            || x.ordered_operations.len() != 2
            || x.fallback.as_ref() != "RejectWithoutMutation"
            || x.currency
                .as_ref()
                .is_some_and(|id| !currencies.contains_key(id))
    }) {
        return Err(error("service rule closure drift"));
    }
    let service_ids = functions
        .keys()
        .map(|id| id.as_str())
        .chain(groups.keys().map(|id| id.as_str()))
        .chain(chests.keys().map(|id| id.as_str()))
        .collect::<BTreeSet<_>>();
    if p.offers.iter().any(|x| {
        x.runtime_lowered
            || !x.candidates.is_empty()
            || !x.weights.is_empty()
            || !service_ids.contains(x.service_id.as_ref())
    }) {
        return Err(error("service offer fail-closed closure drift"));
    }
    if p.adventures.iter().any(|x| {
        x.runtime_lowered
            || x.action_gameplay.as_ref() != "Excluded"
            || x.fallback.as_ref() != "RejectWithoutMutation"
            || (x.adventure_type.as_ref() == "RogueWolfGun" && !x.parameter_program.is_empty())
            || (x.adventure_type.as_ref() != "RogueWolfGun" && x.parameter_program.is_empty())
    }) {
        return Err(error("Adventure abstract-outcome boundary drift"));
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseServiceError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseServiceError {}
fn error(message: &str) -> DivergentUniverseServiceError {
    DivergentUniverseServiceError {
        message: message.into(),
    }
}
