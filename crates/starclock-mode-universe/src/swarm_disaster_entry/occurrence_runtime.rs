//! Swarm Occurrence pools, variant graphs and bounded random outcomes.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{ActivityRngLabel, ActivityRngStreams};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::interaction_access::{
        ChoiceInput, InteractionRuntimeInput, OccurrenceInput, VariantInput,
    },
};

use super::SwarmDisasterRuntimeInstance;

pub const SWARM_DISASTER_OCCURRENCE_RUNTIME_REVISION: &str = "swarm-disaster-occurrence-runtime-v1";
pub const SWARM_DISASTER_OCCURRENCE_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

const OCCURRENCE_POOL_PURPOSE: u16 = 0x5350;
const RANDOM_OUTCOME_PURPOSE_BASE: u16 = 0x5400;

#[derive(Clone, Debug)]
pub(super) struct OccurrenceRuntimeCatalog {
    occurrences: Box<[RuntimeOccurrence]>,
    variants: Box<[RuntimeVariant]>,
    choices: Box<[RuntimeChoice]>,
    digest: [u8; 32],
}

#[derive(Clone, Debug)]
struct RuntimeOccurrence {
    id: u32,
    key: Box<str>,
    order: u16,
    event_type: Box<str>,
    pool: Box<str>,
    variants: Box<[u32]>,
}

#[derive(Clone, Debug)]
struct RuntimeVariant {
    id: u32,
    key: Box<str>,
    occurrences: Box<[u32]>,
    choices: Box<[u32]>,
    graph: Box<str>,
}

#[derive(Clone, Debug)]
struct RuntimeChoice {
    id: u32,
    key: Box<str>,
    variant: u32,
    ordinal: u16,
    node_ordinal: u16,
    option_ordinal: u16,
    conditions: Box<str>,
    costs: Box<str>,
    outcomes: Box<str>,
    display: Box<str>,
    seeded_uniform: bool,
}

impl OccurrenceRuntimeCatalog {
    pub(super) fn compile(
        input: &InteractionRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        if input.occurrences.len() != 75 || input.variants.len() != 57 || input.choices.len() != 308
        {
            return Err(invalid("Swarm Occurrence denominator drift"));
        }
        let choice_keys = unique_keys(&input.choices, |row| (row.id, row.key.as_ref()))?;
        let variant_keys = unique_keys(&input.variants, |row| (row.id, row.key.as_ref()))?;
        let occurrence_keys = unique_keys(&input.occurrences, |row| (row.id, row.key.as_ref()))?;
        let mut choices = input
            .choices
            .iter()
            .map(|row| compile_choice(row, &variant_keys))
            .collect::<Result<Vec<_>, _>>()?;
        choices.sort_unstable_by_key(|row| row.id);
        let mut variants = input
            .variants
            .iter()
            .map(|row| compile_variant(row, &occurrence_keys, &choice_keys, &choices))
            .collect::<Result<Vec<_>, _>>()?;
        variants.sort_unstable_by_key(|row| row.id);
        let mut occurrences = input
            .occurrences
            .iter()
            .map(|row| compile_occurrence(row, &variant_keys, &variants))
            .collect::<Result<Vec<_>, _>>()?;
        occurrences.sort_unstable_by_key(|row| (row.order, row.id));
        if choices.windows(2).any(|pair| pair[0].id == pair[1].id)
            || variants.windows(2).any(|pair| pair[0].id == pair[1].id)
            || occurrences
                .iter()
                .map(|row| row.id)
                .collect::<BTreeSet<_>>()
                .len()
                != 75
            || choices.iter().filter(|row| row.seeded_uniform).count() != 60
            || occurrences
                .iter()
                .filter(|row| row.pool.as_ref() == "occurrence")
                .count()
                != 55
            || occurrences
                .iter()
                .filter(|row| row.pool.as_ref() == "the-swarm")
                .count()
                != 14
            || occurrences
                .iter()
                .filter(|row| row.pool.as_ref() == "encounter")
                .count()
                != 3
            || occurrences
                .iter()
                .filter(|row| row.pool.as_ref() == "deal")
                .count()
                != 3
        {
            return Err(reference("Swarm Occurrence exact-once closure drift"));
        }
        let digest = catalog_digest(&occurrences, &variants, &choices);
        Ok(Self {
            occurrences: occurrences.into_boxed_slice(),
            variants: variants.into_boxed_slice(),
            choices: choices.into_boxed_slice(),
            digest,
        })
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize) {
        (
            self.occurrences.len(),
            self.variants.len(),
            self.choices.len(),
            self.choices.iter().filter(|row| row.seeded_uniform).count(),
        )
    }

    fn occurrence(&self, key: &str) -> Option<&RuntimeOccurrence> {
        self.occurrences.iter().find(|row| row.key.as_ref() == key)
    }

    fn choice(&self, key: &str) -> Option<&RuntimeChoice> {
        self.choices.iter().find(|row| row.key.as_ref() == key)
    }

    fn weighted_candidates(
        &self,
        pool: &str,
        weighted: &[(String, u64)],
    ) -> Result<Vec<(&RuntimeOccurrence, u64)>, UniverseCatalogLoadError> {
        if !matches!(pool, "occurrence" | "the-swarm" | "encounter" | "deal") {
            return Err(reference("unknown Swarm Occurrence pool"));
        }
        let mut weights = BTreeMap::new();
        for (key, weight) in weighted {
            if *weight == 0 || weights.insert(key.as_str(), *weight).is_some() {
                return Err(reference("invalid Swarm Occurrence weight binding"));
            }
        }
        let mut candidates = Vec::new();
        for (key, weight) in weights {
            let row = self
                .occurrence(key)
                .filter(|row| row.pool.as_ref() == pool)
                .ok_or_else(|| reference("Occurrence weight escapes its pool"))?;
            candidates.push((row, weight));
        }
        candidates.sort_unstable_by_key(|(row, _)| (row.order, row.id));
        Ok(candidates)
    }
}

impl SwarmDisasterRuntimeInstance {
    #[must_use]
    pub fn occurrence_runtime_digest(&self) -> [u8; 32] {
        self.occurrence_rules
            .occurrences(&self.occurrences)
            .digest()
    }

    #[must_use]
    pub fn occurrence_count(&self) -> usize {
        self.occurrence_rules
            .occurrences(&self.occurrences)
            .occurrences
            .len()
    }

    pub fn select_occurrence(
        &self,
        pool: &str,
        weighted_candidates: &[(String, u64)],
        rng: &mut ActivityRngStreams,
    ) -> Result<Option<Box<str>>, UniverseCatalogLoadError> {
        let occurrences = self.occurrence_rules.occurrences(&self.occurrences);
        let candidates = occurrences.weighted_candidates(pool, weighted_candidates)?;
        let weights = candidates
            .iter()
            .map(|(_, weight)| *weight)
            .collect::<Vec<_>>();
        let draw = rng.transact(|working| {
            working
                .choose_weighted(
                    ActivityRngLabel::Occurrence,
                    OCCURRENCE_POOL_PURPOSE,
                    &weights,
                )
                .map_err(|_| invalid("Swarm Occurrence pool RNG failure"))
        })?;
        draw.map(|(index, _)| {
            candidates
                .get(index as usize)
                .map(|(row, _)| row.key.clone())
                .ok_or_else(|| invalid("Swarm Occurrence pool mapping failure"))
        })
        .transpose()
    }

    pub fn occurrence_variant_keys(
        &self,
        occurrence: &str,
    ) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
        let occurrences = self.occurrence_rules.occurrences(&self.occurrences);
        let occurrence = occurrences
            .occurrence(occurrence)
            .ok_or_else(|| reference("unknown Swarm Occurrence"))?;
        occurrence
            .variants
            .iter()
            .map(|id| {
                occurrences
                    .variants
                    .binary_search_by_key(id, |row| row.id)
                    .ok()
                    .map(|index| occurrences.variants[index].key.clone())
                    .ok_or_else(|| invalid("Occurrence variant mapping failure"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn occurrence_choice_keys(
        &self,
        variant: &str,
    ) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
        let occurrences = self.occurrence_rules.occurrences(&self.occurrences);
        let variant = occurrences
            .variants
            .iter()
            .find(|row| row.key.as_ref() == variant)
            .ok_or_else(|| reference("unknown Swarm Occurrence variant"))?;
        variant
            .choices
            .iter()
            .map(|id| {
                occurrences
                    .choices
                    .binary_search_by_key(id, |row| row.id)
                    .ok()
                    .map(|index| occurrences.choices[index].key.clone())
                    .ok_or_else(|| invalid("Occurrence choice mapping failure"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn select_occurrence_outcome_candidates(
        &self,
        choice: &str,
        candidates: &[u64],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[u64]>, UniverseCatalogLoadError> {
        let occurrences = self.occurrence_rules.occurrences(&self.occurrences);
        let choice = occurrences
            .choice(choice)
            .filter(|choice| choice.seeded_uniform)
            .ok_or_else(|| reference("Occurrence choice has no random outcome policy"))?;
        let mut candidates = candidates.to_vec();
        candidates.sort_unstable();
        if candidates.contains(&0) || candidates.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(reference("invalid Occurrence outcome candidates"));
        }
        if maximum == 0 || candidates.is_empty() {
            return Ok(Box::new([]));
        }
        let purpose = RANDOM_OUTCOME_PURPOSE_BASE
            .checked_add(
                u16::try_from(choice.id).map_err(|_| invalid("Occurrence choice ID overflow"))?,
            )
            .ok_or_else(|| invalid("Occurrence RNG purpose overflow"))?;
        let selected = rng.transact(|working| {
            working
                .choose_weighted_without_replacement(
                    ActivityRngLabel::Occurrence,
                    purpose,
                    &vec![1; candidates.len()],
                    maximum,
                )
                .map_err(|_| invalid("Occurrence outcome RNG failure"))
        })?;
        let mut output = selected
            .iter()
            .map(|index| {
                candidates
                    .get(*index as usize)
                    .copied()
                    .ok_or_else(|| invalid("Occurrence outcome mapping failure"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        output.sort_unstable();
        Ok(output.into_boxed_slice())
    }
}

fn compile_occurrence(
    row: &OccurrenceInput,
    variant_keys: &BTreeMap<&str, u32>,
    available_variants: &[RuntimeVariant],
) -> Result<RuntimeOccurrence, UniverseCatalogLoadError> {
    let pool: PoolPolicy = serde_json::from_str(&row.pool_rules)
        .map_err(|_| reference("invalid Swarm Occurrence pool policy"))?;
    let pool_id = pool
        .pool_id
        .strip_prefix("swarm-disaster.occurrence-pool.")
        .ok_or_else(|| reference("invalid Swarm Occurrence pool ID"))?;
    if pool.eligibility.as_ref() != "OwningDomainOrServiceBindingRequired"
        || pool.unresolved_offer_behavior.as_ref() != "FailClosed"
        || pool.weight_policy.as_ref() != "OwningBindingMustProvideWeight"
        || !matches!(
            row.source_event_type.as_ref(),
            "Occurrence" | "The Swarm" | "Encounter" | "Deal"
        )
    {
        return Err(reference("Swarm Occurrence pool contract drift"));
    }
    let variants = resolve_keys(&row.variant_keys, variant_keys)?;
    if variants.is_empty()
        || variants.iter().any(|id| {
            variants_for_id(available_variants, *id).is_none()
                || variants.iter().filter(|candidate| *candidate == id).count() != 1
        })
    {
        return Err(reference("Swarm Occurrence variant closure drift"));
    }
    Ok(RuntimeOccurrence {
        id: row.id,
        key: row.key.clone(),
        order: row.order,
        event_type: row.source_event_type.clone(),
        pool: pool_id.into(),
        variants,
    })
}

fn variants_for_id(variants: &[RuntimeVariant], id: u32) -> Option<&RuntimeVariant> {
    variants.iter().find(|row| row.id == id)
}

fn compile_variant(
    row: &VariantInput,
    occurrence_keys: &BTreeMap<&str, u32>,
    choice_keys: &BTreeMap<&str, u32>,
    choices: &[RuntimeChoice],
) -> Result<RuntimeVariant, UniverseCatalogLoadError> {
    let occurrences = resolve_keys(&row.occurrence_keys, occurrence_keys)?;
    let choice_ids = resolve_keys(&row.choice_keys, choice_keys)?;
    let graph: Vec<GraphRef> = serde_json::from_str(&row.graph)
        .map_err(|_| reference("invalid Occurrence graph reference"))?;
    if occurrences.is_empty()
        || choice_ids.is_empty()
        || graph.is_empty()
        || graph.iter().any(|entry| {
            entry.locator.is_empty() || entry.path.is_empty() || !sha256(&entry.sha256)
        })
        || choice_ids.iter().any(|id| {
            choices
                .iter()
                .all(|choice| choice.id != *id || choice.variant != row.id)
        })
    {
        return Err(reference("Swarm Occurrence variant graph drift"));
    }
    Ok(RuntimeVariant {
        id: row.id,
        key: row.key.clone(),
        occurrences,
        choices: choice_ids,
        graph: row.graph.clone(),
    })
}

fn compile_choice(
    row: &ChoiceInput,
    variant_keys: &BTreeMap<&str, u32>,
) -> Result<RuntimeChoice, UniverseCatalogLoadError> {
    if !variant_keys.values().any(|id| *id == row.variant)
        || row.ordinal == 0
        || [
            row.conditions.as_ref(),
            row.costs.as_ref(),
            row.outcomes.as_ref(),
            row.display.as_ref(),
        ]
        .iter()
        .any(|value| serde_json::from_str::<serde_json::Value>(value).is_err())
    {
        return Err(reference("Swarm Occurrence choice drift"));
    }
    let outcomes: Vec<Outcome> = serde_json::from_str(&row.outcomes)
        .map_err(|_| reference("invalid Occurrence outcomes"))?;
    let seeded_uniform = outcomes.iter().any(|outcome| {
        outcome.probability_policy.as_deref() == Some("SeededUniformStableSourceOrder")
    });
    if outcomes.is_empty()
        || outcomes.iter().any(|outcome| {
            outcome.operations.is_empty()
                || (outcome.probability_policy.as_deref() == Some("SeededUniformStableSourceOrder")
                    && outcome.unresolved_candidate_pool.as_deref() != Some("FailClosed"))
        })
    {
        return Err(reference("Occurrence outcome policy drift"));
    }
    Ok(RuntimeChoice {
        id: row.id,
        key: row.key.clone(),
        variant: row.variant,
        ordinal: row.ordinal,
        node_ordinal: row.node_ordinal,
        option_ordinal: row.option_ordinal,
        conditions: row.conditions.clone(),
        costs: row.costs.clone(),
        outcomes: row.outcomes.clone(),
        display: row.display.clone(),
        seeded_uniform,
    })
}

fn unique_keys<'a, T>(
    rows: &'a [T],
    fields: impl Fn(&'a T) -> (u32, &'a str),
) -> Result<BTreeMap<&'a str, u32>, UniverseCatalogLoadError> {
    let mut ids = BTreeSet::new();
    let mut keys = BTreeMap::new();
    for row in rows {
        let (id, key) = fields(row);
        if id == 0 || key.is_empty() || !ids.insert(id) || keys.insert(key, id).is_some() {
            return Err(reference("duplicate Occurrence runtime identity"));
        }
    }
    Ok(keys)
}

fn resolve_keys(
    keys: &[Box<str>],
    known: &BTreeMap<&str, u32>,
) -> Result<Box<[u32]>, UniverseCatalogLoadError> {
    let values = keys
        .iter()
        .map(|key| {
            known
                .get(key.as_ref())
                .copied()
                .ok_or_else(|| reference("unknown Occurrence reference"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(reference("duplicate Occurrence reference"))
    } else {
        Ok(values.into_boxed_slice())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PoolPolicy {
    eligibility: Box<str>,
    pool_id: Box<str>,
    unresolved_offer_behavior: Box<str>,
    weight_policy: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphRef {
    locator: Box<str>,
    path: Box<str>,
    sha256: Box<str>,
}

#[derive(Deserialize)]
struct Outcome {
    operations: Box<[Box<str>]>,
    #[serde(rename = "targets")]
    _targets: Box<[Box<str>]>,
    #[serde(default)]
    probability_policy: Option<Box<str>>,
    #[serde(default)]
    unresolved_candidate_pool: Option<Box<str>>,
}

fn catalog_digest(
    occurrences: &[RuntimeOccurrence],
    variants: &[RuntimeVariant],
    choices: &[RuntimeChoice],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.occurrence-runtime.v1");
    encoder.text(SWARM_DISASTER_OCCURRENCE_RUNTIME_REVISION);
    encoder.text(SWARM_DISASTER_OCCURRENCE_POLICY_ACCURACY);
    for row in occurrences {
        encoder.u32(row.id);
        encoder.text(&row.key);
        encoder.u32(u32::from(row.order));
        encoder.text(&row.event_type);
        encoder.text(&row.pool);
        encode_ids(&mut encoder, &row.variants);
    }
    for row in variants {
        encoder.u32(row.id);
        encoder.text(&row.key);
        encode_ids(&mut encoder, &row.occurrences);
        encode_ids(&mut encoder, &row.choices);
        encoder.text(&row.graph);
    }
    for row in choices {
        encoder.u32(row.id);
        encoder.text(&row.key);
        encoder.u32(row.variant);
        encoder.u32(u32::from(row.ordinal));
        encoder.u32(u32::from(row.node_ordinal));
        encoder.u32(u32::from(row.option_ordinal));
        encoder.text(&row.conditions);
        encoder.text(&row.costs);
        encoder.text(&row.outcomes);
        encoder.text(&row.display);
        encoder.bool(row.seeded_uniform);
    }
    encoder.finish()
}

fn encode_ids(encoder: &mut Encoder, ids: &[u32]) {
    encoder.u32(ids.len() as u32);
    for id in ids {
        encoder.u32(*id);
    }
}

fn sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "occurrence_runtime_tests.rs"]
mod tests;
