//! Gold-owned Curio copies, lifecycle programs and bounded offer pools.

use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel, ActivityRngStreams,
    ActivitySlotId, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    digest::Encoder,
    gold_gears_content::{GoldAndGearsContentCatalog, types::Curio},
    id::CurioId,
};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    curio_types::{
        GoldAndGearsCurioCandidate, GoldAndGearsCurioCategory, GoldAndGearsCurioContribution,
        GoldAndGearsCurioContributionSet, GoldAndGearsCurioDefinition, GoldAndGearsCurioId,
        GoldAndGearsCurioOfferContext, GoldAndGearsCurioOfferSource, GoldAndGearsCurioParameter,
        GoldAndGearsCurioState,
    },
    state_layout::{
        CONTENT_CURIO_CHARGE_BASE, CONTENT_CURIO_STATE_BASE, CONTENT_LIFECYCLE_SLOT,
        CURIO_INVENTORY,
    },
};

pub const GOLD_AND_GEARS_CURIO_RUNTIME_REVISION: &str = "gold-and-gears-curio-runtime-v1";
pub const GOLD_AND_GEARS_CURIO_OFFER_POLICY_REVISION: &str = "gold-and-gears-curio-offer-policy-v1";
pub const GOLD_AND_GEARS_CURIO_OFFER_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

const TRAILBLAZE_PURPOSE: u16 = 0x4772;
const CONUNDRUM_PURPOSE: u16 = 0x4773;
const OCCURRENCE_PURPOSE: u16 = 0x4774;
const SERVICE_PURPOSE: u16 = 0x4775;
const REPLACEMENT_PURPOSE: u16 = 0x4776;

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsCurioRuntimeCatalog {
    definitions: Box<[GoldAndGearsCurioDefinition]>,
    digest: [u8; 32],
}

impl GoldAndGearsCurioRuntimeCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
        standard: &UniverseCatalog,
        shared_runtime: &CurioRuntimeCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        if content.curios.len() != 80
            || content.curio_states.len() != 80
            || shared_runtime.definitions().len() != 61
        {
            return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
        }
        let mut definitions = content
            .curios
            .iter()
            .map(|curio| compile_definition(content, standard, shared_runtime, curio))
            .collect::<Result<Vec<_>, _>>()?;
        definitions.sort_by_key(|definition| (definition.handbook_order, definition.source_id));
        if definitions.windows(2).any(|pair| {
            pair[0].id == pair[1].id || pair[0].handbook_order == pair[1].handbook_order
        }) || definitions
            .iter()
            .filter(|definition| definition.shared_curio.is_some())
            .count()
            != 61
            || definitions
                .iter()
                .filter(|definition| definition.shared_curio.is_none())
                .count()
                != 19
        {
            return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
        }
        let digest = catalog_digest(&definitions);
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            digest,
        })
    }

    pub(super) fn definitions(&self) -> &[GoldAndGearsCurioDefinition] {
        &self.definitions
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    fn definition(&self, id: GoldAndGearsCurioId) -> Option<&GoldAndGearsCurioDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.id == id)
    }

    fn candidates(
        &self,
        context: &GoldAndGearsCurioOfferContext,
        owned: &[GoldAndGearsCurioId],
    ) -> Result<Box<[GoldAndGearsCurioCandidate]>, GoldAndGearsEntryError> {
        let mut owned = owned.to_vec();
        owned.sort_unstable();
        if owned.windows(2).any(|pair| pair[0] == pair[1])
            || owned.iter().any(|id| self.definition(*id).is_none())
        {
            return Err(GoldAndGearsEntryError::InvalidCurioInventory);
        }
        if let Some(keys) = &context.eligible_keys {
            for key in keys {
                let Some(definition) = self
                    .definitions
                    .iter()
                    .find(|definition| definition.stable_key.as_ref() == key.as_ref())
                else {
                    return Err(GoldAndGearsEntryError::InvalidCurioOffer);
                };
                if definition.category != context.category {
                    return Err(GoldAndGearsEntryError::InvalidCurioOffer);
                }
            }
        }
        Ok(self
            .definitions
            .iter()
            .filter(|definition| definition.category == context.category)
            .filter(|definition| owned.binary_search(&definition.id).is_err())
            .filter(|definition| {
                context.eligible_keys.as_ref().is_none_or(|keys| {
                    keys.binary_search_by(|key| key.as_ref().cmp(&definition.stable_key))
                        .is_ok()
                })
            })
            .map(|definition| GoldAndGearsCurioCandidate {
                id: definition.id,
                stable_key: definition.stable_key.clone(),
                category: definition.category,
                shared: definition.shared_curio.is_some(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }

    fn contributions(
        &self,
        owned: &[(GoldAndGearsCurioId, u32)],
        states: &[(GoldAndGearsCurioId, GoldAndGearsCurioState)],
        remaining_or_progress: &[(GoldAndGearsCurioId, u8)],
    ) -> Result<GoldAndGearsCurioContributionSet, GoldAndGearsEntryError> {
        let owned = canonical_map(owned, |(_, count)| *count == 1)?;
        let states = canonical_map(states, |_| true)?;
        let counters = canonical_map(remaining_or_progress, |_| true)?;
        if owned.keys().any(|id| self.definition(*id).is_none())
            || states.keys().any(|id| !owned.contains_key(id))
            || counters.keys().any(|id| !owned.contains_key(id))
            || owned.keys().any(|id| !states.contains_key(id))
        {
            return Err(GoldAndGearsEntryError::InvalidCurioInventory);
        }
        let mut entries = Vec::with_capacity(owned.len());
        for id in owned.keys() {
            let definition = self
                .definition(*id)
                .ok_or(GoldAndGearsEntryError::InvalidCurioInventory)?;
            let state = states[id];
            let counter = counters.get(id).copied().unwrap_or(0);
            validate_contribution_state(definition, state, counter)?;
            let (source_effect_id, parameters) = contribution_payload(definition, state)?;
            entries.push(GoldAndGearsCurioContribution {
                id: *id,
                shared_curio: definition.shared_curio,
                state,
                remaining_or_progress: counter,
                source_effect_id,
                parameters,
            });
        }
        let digest = contribution_digest(&entries);
        Ok(GoldAndGearsCurioContributionSet {
            entries: entries.into_boxed_slice(),
            digest,
        })
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn curio_definitions(&self) -> &[GoldAndGearsCurioDefinition] {
        self.content_runtime.curios.definitions()
    }

    #[must_use]
    pub fn curio_runtime_digest(&self) -> [u8; 32] {
        self.content_runtime.curios.digest()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn curio_definitions(&self) -> &[GoldAndGearsCurioDefinition] {
        self.content_runtime.curios.definitions()
    }

    #[must_use]
    pub fn curio_runtime_digest(&self) -> [u8; 32] {
        self.content_runtime.curios.digest()
    }

    pub fn curio_candidates(
        &self,
        context: &GoldAndGearsCurioOfferContext,
        owned: &[GoldAndGearsCurioId],
    ) -> Result<Box<[GoldAndGearsCurioCandidate]>, GoldAndGearsEntryError> {
        self.content_runtime.curios.candidates(context, owned)
    }

    pub fn select_curios(
        &self,
        context: &GoldAndGearsCurioOfferContext,
        owned: &[GoldAndGearsCurioId],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[GoldAndGearsCurioCandidate]>, GoldAndGearsEntryError> {
        let candidates = self.content_runtime.curios.candidates(context, owned)?;
        if maximum == 0 || candidates.is_empty() {
            return Ok(Box::new([]));
        }
        let (label, purpose) = offer_rng(context.source);
        let selected = rng.transact(|working| {
            working
                .choose_weighted_without_replacement(
                    label,
                    purpose,
                    &vec![1; candidates.len()],
                    maximum,
                )
                .map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)
        })?;
        let mut selected = selected.into_vec();
        selected.sort_unstable();
        selected
            .into_iter()
            .map(|index| {
                candidates
                    .get(index as usize)
                    .cloned()
                    .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn compile_curio_acquisition(
        &self,
        id: GoldAndGearsCurioId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let definition = self.curio_definition(id)?;
        program(
            0x4A10_0000_u32
                .checked_add(id.get())
                .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?,
            acquisition_operations(definition),
        )
    }

    pub fn compile_curio_charge_use(
        &self,
        id: GoldAndGearsCurioId,
        expected_remaining: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let definition = self.curio_definition(id)?;
        let maximum = definition
            .maximum_charges
            .ok_or(GoldAndGearsEntryError::CurioHasNoCharges(id))?;
        if expected_remaining == 0 || expected_remaining > maximum {
            return Err(GoldAndGearsEntryError::InvalidCurioCharge(id));
        }
        let mut operations = require_owned_state(id, GoldAndGearsCurioState::Active);
        operations.push(require_counter(
            charge_key(id),
            i64::from(expected_remaining),
        ));
        operations.push(add_counter(charge_key(id), -1));
        if expected_remaining == 1 {
            operations.push(transition(
                id,
                GoldAndGearsCurioState::Active,
                GoldAndGearsCurioState::Destroyed,
            ));
        }
        program(0x4A20_0000 + id.get(), operations)
    }

    pub fn compile_curio_source_destruction(
        &self,
        id: GoldAndGearsCurioId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let definition = self.curio_definition(id)?;
        if definition.maximum_charges.is_some()
            || definition.terminal_state != GoldAndGearsCurioState::Destroyed
            || definition.decrement_event.as_ref() != "SourceConditionWithoutNumericCharges"
        {
            return Err(GoldAndGearsEntryError::InvalidCurioLifecycle(id));
        }
        let mut operations = require_owned_state(id, GoldAndGearsCurioState::Active);
        operations.push(transition(
            id,
            GoldAndGearsCurioState::Active,
            GoldAndGearsCurioState::Destroyed,
        ));
        program(0x4A30_0000 + id.get(), operations)
    }

    pub fn compile_curio_repair_progress(
        &self,
        id: GoldAndGearsCurioId,
        expected_progress: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let definition = self.curio_definition(id)?;
        let required = definition
            .repair_after_completed_battles
            .ok_or(GoldAndGearsEntryError::CurioCannotBeRepaired(id))?;
        if expected_progress >= required {
            return Err(GoldAndGearsEntryError::InvalidCurioRepairProgress(id));
        }
        let mut operations = require_owned_state(id, GoldAndGearsCurioState::Repairing);
        operations.push(require_counter(
            charge_key(id),
            i64::from(expected_progress),
        ));
        if expected_progress + 1 == required {
            operations.push(add_counter(charge_key(id), -i64::from(expected_progress)));
            operations.push(transition(
                id,
                GoldAndGearsCurioState::Repairing,
                GoldAndGearsCurioState::Fixed,
            ));
        } else {
            operations.push(add_counter(charge_key(id), 1));
        }
        program(
            0x4A40_0000 + id.get() * 4 + u32::from(expected_progress),
            operations,
        )
    }

    pub fn compile_curio_repair(
        &self,
        id: GoldAndGearsCurioId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let definition = self.curio_definition(id)?;
        if definition.repair_after_completed_battles.is_none() {
            return Err(GoldAndGearsEntryError::CurioCannotBeRepaired(id));
        }
        let mut operations = require_owned_state(id, GoldAndGearsCurioState::Repairing);
        operations.push(add_counter(
            charge_key(id),
            ActivityExpression::Negate(Box::new(counter(charge_key(id)))),
        ));
        operations.push(transition(
            id,
            GoldAndGearsCurioState::Repairing,
            GoldAndGearsCurioState::Fixed,
        ));
        program(0x4A50_0000 + id.get(), operations)
    }

    pub fn compile_curio_replacement(
        &self,
        removed: GoldAndGearsCurioId,
        acquired: GoldAndGearsCurioId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        if removed == acquired {
            return Err(GoldAndGearsEntryError::InvalidCurioReplacement);
        }
        self.curio_definition(removed)?;
        let acquired_definition = self.curio_definition(acquired)?;
        let mut operations = teardown_operations(removed);
        operations.extend(acquisition_operations(acquired_definition));
        let id = 0x4B00_0000_u32
            .checked_add(
                removed
                    .get()
                    .checked_mul(4_096)
                    .and_then(|value| value.checked_add(acquired.get()))
                    .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?;
        program(id, operations)
    }

    pub fn compile_curio_teardown(
        &self,
        id: GoldAndGearsCurioId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.curio_definition(id)?;
        program(0x4A60_0000 + id.get(), teardown_operations(id))
    }

    pub fn curio_contributions(
        &self,
        owned: &[(GoldAndGearsCurioId, u32)],
        states: &[(GoldAndGearsCurioId, GoldAndGearsCurioState)],
        remaining_or_progress: &[(GoldAndGearsCurioId, u8)],
    ) -> Result<GoldAndGearsCurioContributionSet, GoldAndGearsEntryError> {
        self.content_runtime
            .curios
            .contributions(owned, states, remaining_or_progress)
    }

    fn curio_definition(
        &self,
        id: GoldAndGearsCurioId,
    ) -> Result<&GoldAndGearsCurioDefinition, GoldAndGearsEntryError> {
        self.content_runtime
            .curios
            .definition(id)
            .ok_or(GoldAndGearsEntryError::UnknownCurio(id))
    }
}

fn compile_definition(
    content: &GoldAndGearsContentCatalog,
    standard: &UniverseCatalog,
    shared_runtime: &CurioRuntimeCatalog,
    curio: &Curio,
) -> Result<GoldAndGearsCurioDefinition, GoldAndGearsEntryError> {
    let source_id = parse_u32(&curio.source_id)?;
    let mode_copy_id = parse_u32(&curio.mode_copy_id)?;
    let shared_curio = if curio.shared {
        let definition = standard
            .curios()
            .iter()
            .find(|definition| definition.stable_key() == curio.key.as_str())
            .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?;
        if shared_runtime.definition(definition.id()).is_none() {
            return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
        }
        Some(definition.id())
    } else {
        None
    };
    let id = GoldAndGearsCurioId::new(if curio.shared {
        source_id
    } else {
        mode_copy_id
    })
    .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?;
    let [state_key] = curio.states.as_ref() else {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    };
    let state = content
        .curio_states
        .iter()
        .find(|state| state.id == curio.initial_state_id && state.key == *state_key)
        .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?;
    if state.curio_id != curio.id
        || state.pool_category.as_ref() != curio.pool_category.as_ref()
        || curio.random_offer_eligibility.as_ref() != "OfferRuleRequired"
    {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    }
    let category = category(&curio.pool_category)?;
    if curio.selection_pool.as_str() != pool_key(category) {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    }
    let lifecycle: Lifecycle = serde_json::from_str(state.lifecycle.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)?;
    let selection: SelectionPolicy = serde_json::from_str(state.selection_policy.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)?;
    if selection.policy_id.as_ref() != "curio-random-selection-v1"
        || selection.evidence_quality.as_ref() != "ProjectPolicy"
        || selection.candidate_order.as_ref() != "stable-handbook-order-then-source-id"
        || selection.offer_eligibility.as_ref() != "BoundByOccurrenceOrServiceRule"
        || selection.unresolved_offer_behavior.as_ref() != "FailClosed"
    {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    }
    let initial_state = lifecycle_state(&lifecycle.initial_state)?;
    if initial_state != lifecycle_state(&state.state_kind)? {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    }
    let terminal_state = lifecycle_state(&lifecycle.terminal_state)?;
    let maximum_charges = optional_u8(&lifecycle.charges)?;
    let repair_after_completed_battles = optional_u8(&lifecycle.repair_after_completed_battles)?;
    let repair: RepairTarget = serde_json::from_str(state.repair_target.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)?;
    let parameters = decode_parameters(state.parameters.as_str())?;
    let fixed_parameters = decode_parameters_from_values(repair.parameter_values.as_deref())?;
    let fixed_source_effect_id = repair.source_effect_id;
    if initial_state == GoldAndGearsCurioState::Repairing {
        if terminal_state != GoldAndGearsCurioState::Fixed
            || repair_after_completed_battles != Some(3)
            || repair.state_kind.as_deref() != Some("Fixed")
            || fixed_source_effect_id.is_none()
        {
            return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
        }
    } else if repair_after_completed_battles.is_some()
        || fixed_source_effect_id.is_some()
        || !fixed_parameters.is_empty()
    {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    }
    if maximum_charges.is_some() != !lifecycle.decrement_event.is_empty()
        && lifecycle.decrement_event.as_ref() != "SourceConditionWithoutNumericCharges"
    {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    }
    if lifecycle.charge_parameter_index > 0
        && usize::from(lifecycle.charge_parameter_index) > parameters.len()
    {
        return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
    }
    Ok(GoldAndGearsCurioDefinition {
        id,
        stable_key: curio.key.as_str().into(),
        source_id,
        handbook_order: u16::try_from(curio.handbook_order)
            .ok()
            .filter(|value| *value > 0)
            .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?,
        category,
        shared_curio,
        initial_state,
        terminal_state,
        maximum_charges,
        decrement_event: lifecycle.decrement_event,
        repair_after_completed_battles,
        source_effect_id: state.source_effect_id.clone(),
        parameters,
        fixed_source_effect_id,
        fixed_parameters,
        replaces_all_possessed: lifecycle.replacement_operation.as_ref()
            == "ReplaceAllPossessedCuriosIncludingSelfWithRandomCurios",
        post_destruction_effect: nonempty(lifecycle.post_destruction_effect),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lifecycle {
    initial_state: Box<str>,
    terminal_state: Box<str>,
    charges: Box<str>,
    charge_parameter_index: u8,
    decrement_event: Box<str>,
    repair_after_completed_battles: Box<str>,
    #[serde(rename = "repair_operation")]
    _repair_operation: Box<str>,
    replacement_operation: Box<str>,
    post_destruction_effect: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionPolicy {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    candidate_order: Box<str>,
    offer_eligibility: Box<str>,
    unresolved_offer_behavior: Box<str>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RepairTarget {
    #[serde(default)]
    state_kind: Option<Box<str>>,
    #[serde(default)]
    source_effect_id: Option<Box<str>>,
    #[serde(default)]
    parameter_values: Option<Box<[IndexedDecimal]>>,
    #[serde(default)]
    #[serde(rename = "display_parameter_values")]
    _display_parameter_values: Option<Box<[IndexedDecimal]>>,
    #[serde(default)]
    #[serde(rename = "inherited_rule_ids")]
    _inherited_rule_ids: Option<Box<[Box<str>]>>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedDecimal {
    index: usize,
    value: Box<str>,
}

fn decode_parameters(
    json: &str,
) -> Result<Box<[GoldAndGearsCurioParameter]>, GoldAndGearsEntryError> {
    let values: Vec<IndexedDecimal> =
        serde_json::from_str(json).map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)?;
    decode_parameters_from_values(Some(&values))
}

fn decode_parameters_from_values(
    values: Option<&[IndexedDecimal]>,
) -> Result<Box<[GoldAndGearsCurioParameter]>, GoldAndGearsEntryError> {
    values
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            if parameter.index != index + 1 {
                return Err(GoldAndGearsEntryError::InvalidCurioRuntime);
            }
            let (coefficient, scale) = exact_decimal(&parameter.value)
                .ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?;
            Ok(GoldAndGearsCurioParameter::new(coefficient, scale))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn validate_contribution_state(
    definition: &GoldAndGearsCurioDefinition,
    state: GoldAndGearsCurioState,
    counter: u8,
) -> Result<(), GoldAndGearsEntryError> {
    let valid = match state {
        GoldAndGearsCurioState::Active => {
            definition.initial_state == GoldAndGearsCurioState::Active
                && match definition.maximum_charges {
                    Some(maximum) => counter > 0 && counter <= maximum,
                    None => counter == 0,
                }
        }
        GoldAndGearsCurioState::Repairing => {
            definition.initial_state == GoldAndGearsCurioState::Repairing
                && definition
                    .repair_after_completed_battles
                    .is_some_and(|required| counter < required)
        }
        GoldAndGearsCurioState::Fixed => {
            definition.initial_state == GoldAndGearsCurioState::Repairing && counter == 0
        }
        GoldAndGearsCurioState::Destroyed => {
            definition.terminal_state == GoldAndGearsCurioState::Destroyed && counter == 0
        }
        GoldAndGearsCurioState::Replaced => {
            definition.terminal_state == GoldAndGearsCurioState::Replaced && counter == 0
        }
    };
    if valid {
        Ok(())
    } else {
        Err(GoldAndGearsEntryError::InvalidCurioInventory)
    }
}

fn contribution_payload(
    definition: &GoldAndGearsCurioDefinition,
    state: GoldAndGearsCurioState,
) -> Result<(Box<str>, Box<[GoldAndGearsCurioParameter]>), GoldAndGearsEntryError> {
    match state {
        GoldAndGearsCurioState::Active | GoldAndGearsCurioState::Repairing => Ok((
            definition.source_effect_id.clone(),
            definition.parameters.clone(),
        )),
        GoldAndGearsCurioState::Fixed => Ok((
            definition
                .fixed_source_effect_id
                .clone()
                .ok_or(GoldAndGearsEntryError::InvalidCurioInventory)?,
            definition.fixed_parameters.clone(),
        )),
        GoldAndGearsCurioState::Destroyed | GoldAndGearsCurioState::Replaced => {
            Ok((definition.source_effect_id.clone(), Box::new([])))
        }
    }
}

fn acquisition_operations(definition: &GoldAndGearsCurioDefinition) -> Vec<ActivityOperation> {
    let id = definition.id;
    let mut operations = vec![
        require_inventory(id, 0),
        require_counter(state_key(id), 0),
        require_counter(charge_key(id), 0),
        ActivityOperation::AddInventory {
            inventory: curio_inventory(),
            content: u64::from(id.get()),
            count: integer(1),
        },
        add_counter(state_key(id), definition.initial_state.value()),
    ];
    if let Some(charges) = definition.maximum_charges {
        operations.push(add_counter(charge_key(id), i64::from(charges)));
    }
    operations
}

fn teardown_operations(id: GoldAndGearsCurioId) -> Vec<ActivityOperation> {
    vec![
        require_inventory(id, 1),
        ActivityOperation::RemoveInventory {
            inventory: curio_inventory(),
            content: u64::from(id.get()),
            count: integer(1),
        },
        add_counter(
            state_key(id),
            ActivityExpression::Negate(Box::new(counter(state_key(id)))),
        ),
        add_counter(
            charge_key(id),
            ActivityExpression::Negate(Box::new(counter(charge_key(id)))),
        ),
    ]
}

fn require_owned_state(
    id: GoldAndGearsCurioId,
    state: GoldAndGearsCurioState,
) -> Vec<ActivityOperation> {
    vec![
        require_inventory(id, 1),
        require_counter(state_key(id), state.value()),
    ]
}

fn require_inventory(id: GoldAndGearsCurioId, count: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::InventoryCount {
            inventory: curio_inventory(),
            content: u64::from(id.get()),
        },
        integer(count),
    ))
}

fn require_counter(key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(counter(key), integer(value)))
}

fn transition(
    id: GoldAndGearsCurioId,
    from: GoldAndGearsCurioState,
    to: GoldAndGearsCurioState,
) -> ActivityOperation {
    add_counter(state_key(id), to.value() - from.value())
}

fn add_counter(key: u64, delta: impl IntoActivityExpression) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: lifecycle_slot(),
        key,
        delta: delta.into_expression(),
    }
}

fn counter(key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: lifecycle_slot(),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

trait IntoActivityExpression {
    fn into_expression(self) -> ActivityExpression;
}

impl IntoActivityExpression for i64 {
    fn into_expression(self) -> ActivityExpression {
        integer(self)
    }
}

impl IntoActivityExpression for ActivityExpression {
    fn into_expression(self) -> ActivityExpression {
        self
    }
}

fn program(
    id: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(id).ok_or(GoldAndGearsEntryError::InvalidCurioRuntime)?,
        operations,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)
}

fn offer_rng(source: GoldAndGearsCurioOfferSource) -> (ActivityRngLabel, u16) {
    match source {
        GoldAndGearsCurioOfferSource::TrailblazeBonus => {
            (ActivityRngLabel::Reward, TRAILBLAZE_PURPOSE)
        }
        GoldAndGearsCurioOfferSource::AuxiliaryConundrum => {
            (ActivityRngLabel::Reward, CONUNDRUM_PURPOSE)
        }
        GoldAndGearsCurioOfferSource::Occurrence => {
            (ActivityRngLabel::Occurrence, OCCURRENCE_PURPOSE)
        }
        GoldAndGearsCurioOfferSource::Service => (ActivityRngLabel::Shop, SERVICE_PURPOSE),
        GoldAndGearsCurioOfferSource::Replacement => {
            (ActivityRngLabel::Reward, REPLACEMENT_PURPOSE)
        }
    }
}

fn category(value: &str) -> Result<GoldAndGearsCurioCategory, GoldAndGearsEntryError> {
    match value {
        "Normal" => Ok(GoldAndGearsCurioCategory::Normal),
        "Negative" => Ok(GoldAndGearsCurioCategory::Negative),
        "ErrorCode" => Ok(GoldAndGearsCurioCategory::ErrorCode),
        _ => Err(GoldAndGearsEntryError::InvalidCurioRuntime),
    }
}

const fn pool_key(category: GoldAndGearsCurioCategory) -> &'static str {
    match category {
        GoldAndGearsCurioCategory::Normal => "gold-gears.curio-pool.normal",
        GoldAndGearsCurioCategory::Negative => "gold-gears.curio-pool.negative",
        GoldAndGearsCurioCategory::ErrorCode => "gold-gears.curio-pool.errorcode",
    }
}

fn lifecycle_state(value: &str) -> Result<GoldAndGearsCurioState, GoldAndGearsEntryError> {
    match value {
        "Active" => Ok(GoldAndGearsCurioState::Active),
        "Repairing" => Ok(GoldAndGearsCurioState::Repairing),
        "Fixed" => Ok(GoldAndGearsCurioState::Fixed),
        "Destroyed" => Ok(GoldAndGearsCurioState::Destroyed),
        "Replaced" => Ok(GoldAndGearsCurioState::Replaced),
        _ => Err(GoldAndGearsEntryError::InvalidCurioRuntime),
    }
}

fn optional_u8(value: &str) -> Result<Option<u8>, GoldAndGearsEntryError> {
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)
    }
}

fn parse_u32(value: &str) -> Result<u32, GoldAndGearsEntryError> {
    value
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidCurioRuntime)
}

fn nonempty(value: Box<str>) -> Option<Box<str>> {
    (!value.is_empty()).then_some(value)
}

fn exact_decimal(value: &str) -> Option<(i64, u8)> {
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, ""), |parts| parts);
    let scale = u8::try_from(fraction.len()).ok()?;
    let coefficient = format!("{whole}{fraction}").parse::<i64>().ok()?;
    Some((if negative { -coefficient } else { coefficient }, scale))
}

fn canonical_map<K: Copy + Ord, V: Copy>(
    values: &[(K, V)],
    valid: impl Fn(&(K, V)) -> bool,
) -> Result<BTreeMap<K, V>, GoldAndGearsEntryError> {
    let mut output = BTreeMap::new();
    for value in values {
        if !valid(value) || output.insert(value.0, value.1).is_some() {
            return Err(GoldAndGearsEntryError::InvalidCurioInventory);
        }
    }
    Ok(output)
}

fn catalog_digest(definitions: &[GoldAndGearsCurioDefinition]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-gears.curio-runtime.v1");
    encoder.text(GOLD_AND_GEARS_CURIO_RUNTIME_REVISION);
    encoder.text(GOLD_AND_GEARS_CURIO_OFFER_POLICY_REVISION);
    for definition in definitions {
        encoder.u32(definition.id.get());
        encoder.text(&definition.stable_key);
        encoder.u32(definition.source_id);
        encoder.u32(u32::from(definition.handbook_order));
        encoder.u8(definition.category as u8);
        encoder.u32(definition.shared_curio.map_or(0, CurioId::get));
        encoder.u8(definition.initial_state as u8);
        encoder.u8(definition.terminal_state as u8);
        encoder.u8(definition.maximum_charges.unwrap_or(0));
        encoder.text(&definition.decrement_event);
        encoder.u8(definition.repair_after_completed_battles.unwrap_or(0));
        encoder.text(&definition.source_effect_id);
        encode_parameters(&mut encoder, &definition.parameters);
        encoder.text(definition.fixed_source_effect_id.as_deref().unwrap_or(""));
        encode_parameters(&mut encoder, &definition.fixed_parameters);
        encoder.bool(definition.replaces_all_possessed);
        encoder.text(definition.post_destruction_effect.as_deref().unwrap_or(""));
    }
    encoder.finish()
}

fn contribution_digest(entries: &[GoldAndGearsCurioContribution]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-gears.curio-contribution.v1");
    for entry in entries {
        encoder.u32(entry.id.get());
        encoder.u32(entry.shared_curio.map_or(0, CurioId::get));
        encoder.u8(entry.state as u8);
        encoder.u8(entry.remaining_or_progress);
        encoder.text(&entry.source_effect_id);
        encode_parameters(&mut encoder, &entry.parameters);
    }
    encoder.finish()
}

fn encode_parameters(encoder: &mut Encoder, parameters: &[GoldAndGearsCurioParameter]) {
    encoder.u32(u32::try_from(parameters.len()).expect("Curio parameter count fits u32"));
    for parameter in parameters {
        encoder.i64(parameter.coefficient());
        encoder.u8(parameter.scale());
    }
}

fn state_key(id: GoldAndGearsCurioId) -> u64 {
    CONTENT_CURIO_STATE_BASE + u64::from(id.get())
}

fn charge_key(id: GoldAndGearsCurioId) -> u64 {
    CONTENT_CURIO_CHARGE_BASE + u64::from(id.get())
}

fn curio_inventory() -> ActivityInventoryId {
    ActivityInventoryId::new(CURIO_INVENTORY).expect("static inventory is non-zero")
}

fn lifecycle_slot() -> ActivitySlotId {
    ActivitySlotId::new(CONTENT_LIFECYCLE_SLOT).expect("static lifecycle slot is non-zero")
}
