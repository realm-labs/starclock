//! Canonical lowering of Occurrence choices into Activity handler payloads.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityHandlerFault, ActivityHandlerFaultKind,
    ActivityHandlerInput, ActivityHandlerOutput, ActivityInventoryId, ActivityOperation,
    ActivitySlotId, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    curio_activity::{
        CurioActivityBindings, CurioActivityRecord, acquisition_operations, teardown_operations,
    },
    id::EncounterMemberId,
    id::{CurioId, CurioStateId, OccurrenceChoiceId},
    occurrence::{
        AuthoredScalar, AuthoredScalarUnit, OccurrenceChoiceDefinition, OccurrenceOperation,
        OccurrenceOutcome, OccurrenceTarget, RandomOutcomePolicy,
    },
};

mod digest;
mod external;
mod s02;
mod s03;
mod s05;
mod s06;
mod s07;
mod s08;
mod s09;
mod s10;
mod s11;
mod s12;
pub(crate) mod support;

use support::{
    Decoder, arithmetic, checked_lcm, exact_integer, fragment_delta, invalid_payload,
    invalid_state, inventory, lower_costs, lower_pairs, outcome_pairs, referenced_curios,
    require_at_least, select_candidates, slot, slot_integer,
};

pub const OCCURRENCE_INTERACTION_HANDLER_ID: u32 = 2;
pub const OCCURRENCE_INTERACTION_RUNTIME_REVISION: &str =
    "standard-universe-occurrence-interaction-runtime-v14";
const PAYLOAD_REVISION: u8 = 6;
const TAG_FRAGMENT_SCALAR: u8 = 1;
const TAG_FRAGMENT_PERCENT: u8 = 2;
const TAG_INVENTORY: u8 = 3;
const TAG_REQUIRE_INVENTORY: u8 = 4;
const TAG_DEFERRED_EFFECT: u8 = 5;
const TAG_REQUIRE_FRAGMENT: u8 = 6;
const TAG_CURIO_INVENTORY: u8 = 7;
const TAG_TRANSITION: u8 = 8;
const TAG_PARTICIPANT_HP_LOSS: u8 = 9;
const TAG_ENSURE_INVENTORY_GROUP: u8 = 10;
const TAG_PARTICIPANT_HP_RESTORE: u8 = 47;
const MAX_PAYLOAD_OPERATIONS: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledOccurrenceProgram {
    choice: OccurrenceChoiceId,
    battle_member: Option<EncounterMemberId>,
    repeat_key: Option<u64>,
    payload: Box<[u8]>,
    random_candidate_count: Option<u32>,
    immediate_operations: u16,
    deferred_operations: u16,
    external_results: Box<[OccurrenceExternalResult]>,
}

/// Immutable executable payload catalog for the complete authored Occurrence
/// choice partition. Payload bytes remain private to the mode handler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceInteractionRuntimeCatalog {
    programs: Box<[CompiledOccurrenceProgram]>,
    digest: [u8; 32],
}

impl OccurrenceInteractionRuntimeCatalog {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile(
        catalog: &UniverseCatalog,
        cosmic_fragments: ActivitySlotId,
        blessing_inventory: ActivityInventoryId,
        curio_records: &[CurioActivityRecord],
        curio_bindings: CurioActivityBindings,
        deferred_effects: ActivitySlotId,
        interaction_state: ActivitySlotId,
        selected_path: ActivitySlotId,
        formation_inventory: ActivityInventoryId,
    ) -> Result<Self, OccurrenceInteractionError> {
        let occurrence_battles = crate::occurrence_battle::compile(catalog)
            .map_err(|_| OccurrenceInteractionError::InvalidChoice)?;
        let mut programs = catalog
            .occurrence_choices()
            .iter()
            .map(|choice| {
                let battle_member = occurrence_battles
                    .iter()
                    .find(|battle| battle.choice() == choice.id())
                    .map(|battle| battle.member().id());
                compile(
                    choice,
                    catalog,
                    cosmic_fragments,
                    blessing_inventory,
                    curio_records,
                    curio_bindings,
                    deferred_effects,
                    interaction_state,
                    selected_path,
                    formation_inventory,
                    battle_member,
                )
                .map(|compiled| CompiledOccurrenceProgram {
                    choice: choice.id(),
                    battle_member,
                    repeat_key: compiled.repeat_key,
                    payload: compiled.payload.into_boxed_slice(),
                    random_candidate_count: compiled.random_candidate_count,
                    immediate_operations: compiled.immediate_operations,
                    deferred_operations: compiled.deferred_operations,
                    external_results: compiled.external_results,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        programs.sort_unstable_by_key(|program| program.choice);
        if programs.len() != 321
            || programs
                .windows(2)
                .any(|pair| pair[0].choice == pair[1].choice)
            || programs
                .iter()
                .any(|program| program.immediate_operations + program.deferred_operations == 0)
        {
            return Err(OccurrenceInteractionError::InvalidChoice);
        }
        let digest = digest::runtime_catalog(&programs);
        Ok(Self {
            programs: programs.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn choice_count(&self) -> usize {
        self.programs.len()
    }

    #[must_use]
    pub fn immediate_operation_count(&self) -> usize {
        self.programs
            .iter()
            .map(|program| usize::from(program.immediate_operations))
            .sum()
    }

    #[must_use]
    pub fn deferred_operation_count(&self) -> usize {
        self.programs
            .iter()
            .map(|program| usize::from(program.deferred_operations))
            .sum()
    }

    #[must_use]
    pub fn external_result_count(&self) -> usize {
        self.programs
            .iter()
            .map(|program| program.external_results.len())
            .sum()
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub fn compile_choice(
        &self,
        choice: OccurrenceChoiceId,
    ) -> Option<CompiledOccurrenceInteraction> {
        self.programs
            .binary_search_by_key(&choice, |program| program.choice)
            .ok()
            .map(|index| &self.programs[index])
            .map(|program| CompiledOccurrenceInteraction {
                battle_member: program.battle_member,
                repeat_key: program.repeat_key,
                payload: program.payload.to_vec(),
                random_candidate_count: program.random_candidate_count,
                immediate_operations: program.immediate_operations,
                deferred_operations: program.deferred_operations,
                external_results: program
                    .external_results
                    .iter()
                    .map(|result| OccurrenceExternalResult {
                        content: result.content,
                        payload: result.payload.clone(),
                        random_candidate_count: result.random_candidate_count,
                        immediate_operations: result.immediate_operations,
                        deferred_operations: result.deferred_operations,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceExternalResult {
    content: u64,
    payload: Box<[u8]>,
    random_candidate_count: Option<u32>,
    immediate_operations: u16,
    deferred_operations: u16,
}

impl OccurrenceExternalResult {
    #[must_use]
    pub const fn content(&self) -> u64 {
        self.content
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub const fn random_candidate_count(&self) -> Option<u32> {
        self.random_candidate_count
    }

    #[must_use]
    pub const fn immediate_operations(&self) -> u16 {
        self.immediate_operations
    }

    #[must_use]
    pub const fn deferred_operations(&self) -> u16 {
        self.deferred_operations
    }
}

pub struct CompiledOccurrenceInteraction {
    battle_member: Option<EncounterMemberId>,
    repeat_key: Option<u64>,
    payload: Vec<u8>,
    random_candidate_count: Option<u32>,
    immediate_operations: u16,
    deferred_operations: u16,
    external_results: Box<[OccurrenceExternalResult]>,
}

impl CompiledOccurrenceInteraction {
    #[must_use]
    pub const fn battle_member(&self) -> Option<EncounterMemberId> {
        self.battle_member
    }

    #[must_use]
    pub const fn repeat_key(&self) -> Option<u64> {
        self.repeat_key
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub const fn random_candidate_count(&self) -> Option<u32> {
        self.random_candidate_count
    }

    #[must_use]
    pub const fn immediate_operations(&self) -> u16 {
        self.immediate_operations
    }

    #[must_use]
    pub const fn deferred_operations(&self) -> u16 {
        self.deferred_operations
    }

    #[must_use]
    pub fn external_results(&self) -> &[OccurrenceExternalResult] {
        &self.external_results
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile(
    choice: &OccurrenceChoiceDefinition,
    catalog: &UniverseCatalog,
    cosmic_fragments: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_records: &[CurioActivityRecord],
    curio_bindings: CurioActivityBindings,
    deferred_effects: ActivitySlotId,
    interaction_state: ActivitySlotId,
    selected_path: ActivitySlotId,
    formation_inventory: ActivityInventoryId,
    battle_member: Option<EncounterMemberId>,
) -> Result<CompiledOccurrenceInteraction, OccurrenceInteractionError> {
    let outcome = choice
        .outcomes()
        .first()
        .ok_or(OccurrenceInteractionError::InvalidChoice)?;
    let blessing_ids = referenced_blessings(outcome, catalog)?;
    let blessing_groups = s05::referenced_blessing_groups(outcome, catalog)?;
    let curio_ids = referenced_curios(outcome, catalog, curio_records)?;
    let mut operations = Vec::new();
    let external_s08 = s08::externalize(
        outcome,
        catalog,
        blessing_inventory,
        cosmic_fragments,
        curio_bindings,
        curio_records,
        interaction_state,
    )?;
    let external = s11::externalize(
        outcome,
        catalog,
        blessing_inventory,
        cosmic_fragments,
        curio_bindings,
        curio_records,
        interaction_state,
    )?
    .or(s10::externalize(
        outcome,
        catalog,
        curio_bindings,
        curio_records,
        interaction_state,
    )?)
    .or(s09::externalize(
        outcome,
        catalog,
        curio_bindings,
        curio_records,
    )?)
    .or(external_s08);
    let specialized_s08 = s08::lower(
        outcome,
        catalog,
        curio_bindings,
        curio_records,
        interaction_state,
    )?;
    let specialized_s09 = s09::lower(
        outcome,
        catalog,
        blessing_inventory,
        selected_path,
        curio_bindings,
        curio_records,
        interaction_state,
    )?;
    let specialized_s10 = s10::lower(
        outcome,
        cosmic_fragments,
        curio_bindings.inventory,
        interaction_state,
    )?;
    let specialized_s11 = s11::lower(
        outcome,
        catalog,
        blessing_inventory,
        cosmic_fragments,
        curio_bindings,
        curio_records,
        interaction_state,
    )?;
    let specialized_s12 = s12::lower(
        outcome,
        catalog,
        blessing_inventory,
        curio_bindings,
        curio_records,
        interaction_state,
    )?;
    let specialized_s07 = s07::lower(
        outcome,
        catalog,
        blessing_inventory,
        curio_bindings,
        curio_records,
        selected_path,
        formation_inventory,
    )?;
    let progressive_s06 = s06::lower_progressive(
        outcome,
        cosmic_fragments,
        curio_bindings,
        interaction_state,
        &curio_ids,
    )?;
    let progressive_s05 = if progressive_s06.is_none() {
        s05::lower_progressive(
            outcome,
            blessing_inventory,
            interaction_state,
            &blessing_ids,
        )?
    } else {
        None
    };
    let repeat_key = external
        .as_ref()
        .and_then(|value| value.repeat_key)
        .or_else(|| specialized_s09.as_ref().and_then(|value| value.repeat_key))
        .or_else(|| specialized_s11.as_ref().and_then(|value| value.repeat_key))
        .or_else(|| {
            specialized_s08
                .as_ref()
                .and_then(|(_, repeat_key, _)| *repeat_key)
        })
        .or_else(|| progressive_s06.as_ref().map(|value| value.1))
        .or_else(|| progressive_s05.as_ref().map(|value| value.1));
    if let Some((operation, _)) = progressive_s06 {
        operations.push(PayloadOperation::S06(operation));
    } else if let Some((operation, _)) = progressive_s05 {
        operations.push(PayloadOperation::S05(operation));
    } else if let Some(operation) = s05::reset_progressive(outcome, interaction_state)? {
        operations.push(PayloadOperation::S05(operation));
    }
    if let Some(operation) = s06::prepare_reward_paths(outcome, catalog, interaction_state)? {
        operations.push(PayloadOperation::S06(operation));
    }
    let skip_generic_s08 = specialized_s08
        .as_ref()
        .is_some_and(|(_, _, skip_generic)| *skip_generic);
    let skip_generic_s09 = specialized_s09.is_some();
    let skip_generic_s10 = specialized_s10.is_some();
    let skip_generic_s11 = specialized_s11.is_some();
    let skip_generic_s12 = specialized_s12.is_some();
    if let Some(operation) = specialized_s12 {
        operations.push(PayloadOperation::S12(operation));
    }
    if let Some(lowering) = specialized_s11 {
        operations.extend(lowering.operations);
    }
    if let Some(operation) = specialized_s10 {
        operations.push(PayloadOperation::S10(operation));
    }
    if let Some(lowering) = specialized_s09 {
        operations.extend(lowering.operations);
    }
    if let Some((operation, _, _)) = specialized_s08 {
        operations.push(PayloadOperation::S08(operation));
    }
    if external.is_some() {
        operations.push(PayloadOperation::Transition);
    }
    let has_specialized_s07 = specialized_s07.is_some();
    if let Some(operation) = specialized_s07 {
        operations.push(PayloadOperation::S07(operation));
    } else if !skip_generic_s09
        && !skip_generic_s10
        && !skip_generic_s11
        && !skip_generic_s12
        && external.is_none()
    {
        lower_costs(
            &mut operations,
            choice,
            cosmic_fragments,
            blessing_inventory,
            curio_bindings.inventory,
            &blessing_ids,
            &curio_ids
                .iter()
                .map(|value| u64::from(value.id().get()))
                .collect::<Vec<_>>(),
        )?;
    }
    if repeat_key.is_none()
        && !has_specialized_s07
        && !skip_generic_s08
        && !skip_generic_s09
        && !skip_generic_s10
        && !skip_generic_s11
        && !skip_generic_s12
        && external.is_none()
    {
        lower_pairs(
            &mut operations,
            outcome_pairs(outcome),
            choice.id(),
            cosmic_fragments,
            blessing_inventory,
            curio_bindings,
            deferred_effects,
            &blessing_ids,
            &blessing_groups,
            &curio_ids,
            battle_member.is_some(),
        )?;
    }
    if operations.len() > MAX_PAYLOAD_OPERATIONS {
        return Err(OccurrenceInteractionError::TooManyOperations);
    }
    let external_results = if let Some(lowering) = external {
        lowering
            .choices
            .into_iter()
            .map(|choice| {
                let (payload, immediate_operations, deferred_operations) =
                    external::encode_operations(choice.operations)?;
                Ok(OccurrenceExternalResult {
                    content: choice.content,
                    payload: payload.into_boxed_slice(),
                    random_candidate_count: choice.random_candidate_count,
                    immediate_operations,
                    deferred_operations,
                })
            })
            .collect::<Result<Vec<_>, OccurrenceInteractionError>>()?
    } else if outcome.random_policy() == Some(RandomOutcomePolicy::StableUniformOrderedCandidates) {
        external::single_selection(&operations)?
    } else {
        Vec::new()
    };
    let has_s08_random = operations.iter().any(|operation| {
        matches!(operation, PayloadOperation::S08(value) if value.random_candidate_count().is_some())
    });
    let random_candidate_count = if external_results.is_empty()
        && (has_s08_random
            || outcome.random_policy() == Some(RandomOutcomePolicy::StableUniformOrderedCandidates))
    {
        operations
            .iter()
            .filter_map(|operation| match operation {
                PayloadOperation::Inventory { candidates, .. } => {
                    u32::try_from(candidates.len()).ok()
                }
                PayloadOperation::CurioInventory { candidates, .. } => {
                    u32::try_from(candidates.len()).ok()
                }
                PayloadOperation::EnsureInventoryGroup { groups, .. } => {
                    u32::try_from(groups.len()).ok()
                }
                PayloadOperation::S05(operation) => operation.random_candidate_count(),
                PayloadOperation::S06(operation) => operation.random_candidate_count(),
                PayloadOperation::S07(operation) => operation.random_candidate_count(),
                PayloadOperation::S08(operation) => operation.random_candidate_count(),
                PayloadOperation::S09(operation) => operation.random_candidate_count(),
                PayloadOperation::S10(operation) => operation.random_candidate_count(),
                PayloadOperation::S11(operation) => operation.random_candidate_count(),
                PayloadOperation::S12(operation) => operation.random_candidate_count(),
                _ => None,
            })
            .try_fold(1_u32, checked_lcm)
    } else {
        None
    };
    let (payload, immediate_operations, deferred_operations) =
        external::encode_operations(operations)?;
    Ok(CompiledOccurrenceInteraction {
        battle_member,
        repeat_key,
        payload,
        random_candidate_count,
        immediate_operations,
        deferred_operations,
        external_results: external_results.into_boxed_slice(),
    })
}

pub(crate) fn execute(
    input: ActivityHandlerInput<'_>,
) -> Result<ActivityHandlerOutput, ActivityHandlerFault> {
    let mut decoder = Decoder::new(input.payload());
    if decoder.u8()? != PAYLOAD_REVISION {
        return Err(invalid_payload());
    }
    let count = usize::from(decoder.u16()?);
    if count > MAX_PAYLOAD_OPERATIONS {
        return Err(invalid_payload());
    }
    let mut operations = Vec::new();
    for _ in 0..count {
        match decoder.u8()? {
            TAG_FRAGMENT_SCALAR => decode_fragment_scalar(&mut decoder, &mut operations)?,
            TAG_FRAGMENT_PERCENT => {
                decode_fragment_percent(input, &mut decoder, &mut operations)?;
            }
            TAG_INVENTORY => decode_inventory(input, &mut decoder, &mut operations)?,
            TAG_REQUIRE_INVENTORY => {
                decode_inventory_requirement(input, &mut decoder, &mut operations)?;
            }
            TAG_DEFERRED_EFFECT => decode_deferred_effect(&mut decoder, &mut operations)?,
            TAG_REQUIRE_FRAGMENT => {
                let slot = slot(decoder.u32()?)?;
                let amount = decoder.u64()?;
                operations.push(require_at_least(slot, amount)?);
            }
            TAG_CURIO_INVENTORY => {
                decode_curio_inventory(input, &mut decoder, &mut operations)?;
            }
            TAG_TRANSITION => {}
            TAG_PARTICIPANT_HP_LOSS => {
                s02::decode_participant_hp_loss(input, &mut decoder, &mut operations)?;
            }
            TAG_PARTICIPANT_HP_RESTORE => {
                s02::decode_participant_hp_restore(input, &mut decoder, &mut operations)?;
            }
            TAG_ENSURE_INVENTORY_GROUP => {
                s02::decode_ensure_inventory_group(input, &mut decoder, &mut operations)?;
            }
            tag if s05::decode(tag, input, &mut decoder, &mut operations)? => {}
            tag if s06::decode(tag, input, &mut decoder, &mut operations)? => {}
            tag if s07::decode(tag, input, &mut decoder, &mut operations)? => {}
            tag if s08::decode(tag, input, &mut decoder, &mut operations)? => {}
            tag if s09::decode(tag, input, &mut decoder, &mut operations)? => {}
            tag if s11::decode(tag, input, &mut decoder, &mut operations)? => {}
            tag if s10::decode(tag, input, &mut decoder, &mut operations)? => {}
            tag if s12::decode(tag, input, &mut decoder, &mut operations)? => {}
            _ => return Err(invalid_payload()),
        }
    }
    decoder.finish()?;
    Ok(ActivityHandlerOutput::new(operations))
}

fn decode_fragment_scalar(
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let slot = slot(decoder.u32()?)?;
    let gain_inventory = inventory(decoder.u32()?)?;
    let delta = decoder.i64()?;
    if delta < 0 {
        operations.push(require_at_least(slot, delta.unsigned_abs())?);
    }
    operations.push(fragment_delta(slot, gain_inventory, delta));
    Ok(())
}

fn decode_fragment_percent(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let slot = slot(decoder.u32()?)?;
    let gain_inventory = inventory(decoder.u32()?)?;
    let coefficient = decoder.i64()?;
    let scale = decoder.u8()?;
    let sign = decoder.i8()?;
    let current = slot_integer(input, slot)?;
    let divisor = 100_i128
        .checked_mul(10_i128.pow(u32::from(scale)))
        .ok_or_else(arithmetic)?;
    let magnitude = i128::from(current)
        .checked_mul(i128::from(coefficient))
        .ok_or_else(arithmetic)?
        / divisor;
    let magnitude = i64::try_from(magnitude).map_err(|_| arithmetic())?;
    let delta = magnitude
        .checked_mul(i64::from(sign))
        .ok_or_else(arithmetic)?;
    if delta < 0 {
        operations.push(require_at_least(slot, delta.unsigned_abs())?);
    }
    operations.push(fragment_delta(slot, gain_inventory, delta));
    Ok(())
}

fn decode_inventory(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let delta = decoder.i8()?;
    let quantity = usize::from(decoder.u16()?);
    let owned_only = decoder.u8()? != 0;
    let count = usize::from(decoder.u16()?);
    if delta == 0 || quantity == 0 || count == 0 {
        return Err(invalid_payload());
    }
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        candidates.push(decoder.u64()?);
    }
    let selected = select_candidates(
        input,
        inventory,
        &candidates,
        owned_only,
        input.random_index(),
        quantity,
    )?;
    for content in selected {
        let count = ActivityExpression::Literal(ActivityValue::BoundedInteger(1));
        operations.push(if delta > 0 {
            ActivityOperation::AddInventory {
                inventory,
                content,
                count,
            }
        } else {
            ActivityOperation::RemoveInventory {
                inventory,
                content,
                count,
            }
        });
    }
    Ok(())
}

fn decode_inventory_requirement(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let count = usize::from(decoder.u16()?);
    if count == 0 {
        return Err(invalid_payload());
    }
    let mut conditions = Vec::with_capacity(count);
    for _ in 0..count {
        let content = decoder.u64()?;
        conditions.push(ActivityCondition::Not(Box::new(
            ActivityCondition::LessThan(
                ActivityExpression::InventoryCount { inventory, content },
                ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
            ),
        )));
    }
    let inventory_exists = input
        .view()
        .inventories()
        .iter()
        .any(|value| value.id() == inventory);
    if !inventory_exists {
        return Err(invalid_state());
    }
    operations.push(ActivityOperation::Require(ActivityCondition::Any(
        conditions.into_boxed_slice(),
    )));
    Ok(())
}

fn decode_curio_inventory(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = CurioActivityBindings {
        inventory: inventory(decoder.u32()?)?,
        state_slot: slot(decoder.u32()?)?,
        charge_slot: slot(decoder.u32()?)?,
        event_slot: slot(decoder.u32()?)?,
        fragments_slot: slot(decoder.u32()?)?,
    };
    let delta = decoder.i8()?;
    let quantity = usize::from(decoder.u16()?);
    let owned_only = decoder.u8()? != 0;
    let count = usize::from(decoder.u16()?);
    if delta == 0 || quantity == 0 || count == 0 {
        return Err(invalid_payload());
    }
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let record = CurioActivityRecord::new(
            CurioId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
            CurioStateId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
            decoder.u8()?,
            match decoder.i64()? {
                0 => None,
                value => Some(value),
            },
        );
        records.push(match decoder.i64()? {
            0 => record,
            value => record.with_fragment_stack_capture(value),
        });
    }
    if records.windows(2).any(|pair| pair[0].id() >= pair[1].id()) {
        return Err(invalid_payload());
    }
    let candidates = records
        .iter()
        .map(|record| u64::from(record.id().get()))
        .collect::<Vec<_>>();
    let selected = select_candidates(
        input,
        bindings.inventory,
        &candidates,
        owned_only,
        input.random_index(),
        quantity,
    )?;
    for content in selected {
        let id = u32::try_from(content)
            .ok()
            .and_then(CurioId::new)
            .ok_or_else(invalid_payload)?;
        if delta > 0 {
            let record = records
                .binary_search_by_key(&id, |record| record.id())
                .ok()
                .map(|index| records[index])
                .ok_or_else(invalid_payload)?;
            operations.extend(acquisition_operations(record, bindings));
        } else {
            operations.extend(teardown_operations(id, bindings));
        }
    }
    Ok(())
}

fn decode_deferred_effect(
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let slot = slot(decoder.u32()?)?;
    let key = decoder.u64()?;
    operations.push(ActivityOperation::AddCounter {
        slot,
        key,
        delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
    });
    Ok(())
}

#[derive(Clone)]
enum PayloadOperation {
    FragmentScalar {
        slot: ActivitySlotId,
        gain_inventory: ActivityInventoryId,
        delta: i64,
    },
    FragmentPercent {
        slot: ActivitySlotId,
        gain_inventory: ActivityInventoryId,
        coefficient: i64,
        scale: u8,
        sign: i8,
    },
    Inventory {
        inventory: ActivityInventoryId,
        delta: i8,
        quantity: u16,
        owned_only: bool,
        candidates: Vec<u64>,
    },
    CurioInventory {
        bindings: CurioActivityBindings,
        delta: i8,
        quantity: u16,
        owned_only: bool,
        candidates: Vec<CurioActivityRecord>,
    },
    RequireInventory {
        inventory: ActivityInventoryId,
        candidates: Vec<u64>,
    },
    DeferredEffect {
        slot: ActivitySlotId,
        key: u64,
    },
    RequireFragment {
        slot: ActivitySlotId,
        amount: u64,
    },
    ParticipantHpLoss {
        scaled_ratio: i64,
    },
    ParticipantHpRestore {
        scaled_ratio: i64,
    },
    EnsureInventoryGroup {
        inventory: ActivityInventoryId,
        groups: Vec<Vec<u64>>,
    },
    S05(s05::Operation),
    S06(s06::Operation),
    S07(s07::Operation),
    S08(s08::Operation),
    S09(s09::Operation),
    S10(s10::Operation),
    S11(s11::Operation),
    S12(s12::Operation),
    Transition,
}

impl PayloadOperation {
    const fn is_deferred(&self) -> bool {
        matches!(self, Self::DeferredEffect { .. })
    }

    fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::FragmentScalar {
                slot,
                gain_inventory,
                delta,
            } => {
                output.push(TAG_FRAGMENT_SCALAR);
                output.extend_from_slice(&slot.get().to_le_bytes());
                output.extend_from_slice(&gain_inventory.get().to_le_bytes());
                output.extend_from_slice(&delta.to_le_bytes());
            }
            Self::FragmentPercent {
                slot,
                gain_inventory,
                coefficient,
                scale,
                sign,
            } => {
                output.push(TAG_FRAGMENT_PERCENT);
                output.extend_from_slice(&slot.get().to_le_bytes());
                output.extend_from_slice(&gain_inventory.get().to_le_bytes());
                output.extend_from_slice(&coefficient.to_le_bytes());
                output.push(scale);
                output.push(sign as u8);
            }
            Self::Inventory {
                inventory,
                delta,
                quantity,
                owned_only,
                candidates,
            } => {
                output.push(TAG_INVENTORY);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.push(delta as u8);
                output.extend_from_slice(&quantity.to_le_bytes());
                output.push(u8::from(owned_only));
                output.extend_from_slice(
                    &u16::try_from(candidates.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                        .to_le_bytes(),
                );
                for candidate in candidates {
                    output.extend_from_slice(&candidate.to_le_bytes());
                }
            }
            Self::RequireInventory {
                inventory,
                candidates,
            } => {
                output.push(TAG_REQUIRE_INVENTORY);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(
                    &u16::try_from(candidates.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                        .to_le_bytes(),
                );
                for candidate in candidates {
                    output.extend_from_slice(&candidate.to_le_bytes());
                }
            }
            Self::CurioInventory {
                bindings,
                delta,
                quantity,
                owned_only,
                candidates,
            } => {
                output.push(TAG_CURIO_INVENTORY);
                output.extend_from_slice(&bindings.inventory.get().to_le_bytes());
                output.extend_from_slice(&bindings.state_slot.get().to_le_bytes());
                output.extend_from_slice(&bindings.charge_slot.get().to_le_bytes());
                output.extend_from_slice(&bindings.event_slot.get().to_le_bytes());
                output.extend_from_slice(&bindings.fragments_slot.get().to_le_bytes());
                output.push(delta as u8);
                output.extend_from_slice(&quantity.to_le_bytes());
                output.push(u8::from(owned_only));
                output.extend_from_slice(
                    &u16::try_from(candidates.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                        .to_le_bytes(),
                );
                for candidate in candidates {
                    output.extend_from_slice(&candidate.id().get().to_le_bytes());
                    output.extend_from_slice(&candidate.initial_state().get().to_le_bytes());
                    output.push(candidate.initial_charges());
                    output.extend_from_slice(
                        &candidate
                            .acquisition_fragment_divisor()
                            .unwrap_or(0)
                            .to_le_bytes(),
                    );
                    output.extend_from_slice(
                        &candidate
                            .acquisition_fragment_stack_divisor()
                            .unwrap_or(0)
                            .to_le_bytes(),
                    );
                }
            }
            Self::DeferredEffect { slot, key } => {
                output.push(TAG_DEFERRED_EFFECT);
                output.extend_from_slice(&slot.get().to_le_bytes());
                output.extend_from_slice(&key.to_le_bytes());
            }
            Self::RequireFragment { slot, amount } => {
                output.push(TAG_REQUIRE_FRAGMENT);
                output.extend_from_slice(&slot.get().to_le_bytes());
                output.extend_from_slice(&amount.to_le_bytes());
            }
            Self::ParticipantHpLoss { scaled_ratio } => {
                output.push(TAG_PARTICIPANT_HP_LOSS);
                output.extend_from_slice(&scaled_ratio.to_le_bytes());
            }
            Self::ParticipantHpRestore { scaled_ratio } => {
                output.push(TAG_PARTICIPANT_HP_RESTORE);
                output.extend_from_slice(&scaled_ratio.to_le_bytes());
            }
            Self::EnsureInventoryGroup { inventory, groups } => {
                output.push(TAG_ENSURE_INVENTORY_GROUP);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(
                    &u16::try_from(groups.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                        .to_le_bytes(),
                );
                for group in groups {
                    output.extend_from_slice(
                        &u16::try_from(group.len())
                            .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                            .to_le_bytes(),
                    );
                    for content in group {
                        output.extend_from_slice(&content.to_le_bytes());
                    }
                }
            }
            Self::S05(operation) => operation.encode(output)?,
            Self::S06(operation) => operation.encode(output)?,
            Self::S07(operation) => operation.encode(output)?,
            Self::S08(operation) => operation.encode(output)?,
            Self::S09(operation) => operation.encode(output)?,
            Self::S10(operation) => operation.encode(output),
            Self::S11(operation) => operation.encode(output)?,
            Self::S12(operation) => operation.encode(output)?,
            Self::Transition => output.push(TAG_TRANSITION),
        }
        Ok(())
    }
}

fn referenced_blessings(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
) -> Result<Vec<u64>, OccurrenceInteractionError> {
    s03::referenced_blessings(outcome, catalog)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccurrenceInteractionError {
    InvalidChoice,
    TooManyOperations,
    TooManyCandidates,
    NonIntegerScalar,
    Arithmetic,
}
