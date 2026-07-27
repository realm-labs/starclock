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
mod s02;
pub(crate) mod support;

use support::{
    Decoder, arithmetic, checked_lcm, exact_integer, fragment_delta, invalid_payload,
    invalid_state, inventory, require_at_least, select_candidates, slot, slot_integer,
};

pub const OCCURRENCE_INTERACTION_HANDLER_ID: u32 = 2;
pub const OCCURRENCE_INTERACTION_RUNTIME_REVISION: &str =
    "standard-universe-occurrence-interaction-runtime-v3";
const PAYLOAD_REVISION: u8 = 3;
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
const MAX_PAYLOAD_OPERATIONS: usize = 128;
const DEFERRED_EFFECT_KEY_BASE: u64 = 1 << 63;

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledOccurrenceProgram {
    choice: OccurrenceChoiceId,
    battle_member: Option<EncounterMemberId>,
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
    pub(crate) fn compile(
        catalog: &UniverseCatalog,
        cosmic_fragments: ActivitySlotId,
        blessing_inventory: ActivityInventoryId,
        curio_records: &[CurioActivityRecord],
        curio_bindings: CurioActivityBindings,
        deferred_effects: ActivitySlotId,
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
                    battle_member,
                )
                .map(|compiled| CompiledOccurrenceProgram {
                    choice: choice.id(),
                    battle_member,
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
    battle_member: Option<EncounterMemberId>,
) -> Result<CompiledOccurrenceInteraction, OccurrenceInteractionError> {
    let outcome = choice
        .outcomes()
        .first()
        .ok_or(OccurrenceInteractionError::InvalidChoice)?;
    let blessing_ids = referenced_blessings(outcome, catalog)?;
    let blessing_groups = s02::referenced_blessing_groups(outcome, catalog)?;
    let curio_ids = referenced_curios(outcome, catalog, curio_records)?;
    let mut operations = Vec::new();
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
    if operations.len() > MAX_PAYLOAD_OPERATIONS {
        return Err(OccurrenceInteractionError::TooManyOperations);
    }
    let external_results =
        if outcome.random_policy() == Some(RandomOutcomePolicy::StableUniformOrderedCandidates) {
            externalize_single_selection(&operations)?
        } else {
            Vec::new()
        };
    let random_candidate_count = if external_results.is_empty()
        && outcome.random_policy() == Some(RandomOutcomePolicy::StableUniformOrderedCandidates)
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
                _ => None,
            })
            .try_fold(1_u32, checked_lcm)
    } else {
        None
    };
    let (payload, immediate_operations, deferred_operations) = encode_operations(operations)?;
    Ok(CompiledOccurrenceInteraction {
        battle_member,
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
            TAG_ENSURE_INVENTORY_GROUP => {
                s02::decode_ensure_inventory_group(input, &mut decoder, &mut operations)?;
            }
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
    EnsureInventoryGroup {
        inventory: ActivityInventoryId,
        groups: Vec<Vec<u64>>,
    },
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
            Self::Transition => output.push(TAG_TRANSITION),
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_pairs(
    output: &mut Vec<PayloadOperation>,
    pairs: impl IntoIterator<
        Item = (
            OccurrenceOperation,
            Option<OccurrenceTarget>,
            Option<AuthoredScalar>,
        ),
    >,
    choice: OccurrenceChoiceId,
    cosmic_fragments: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_bindings: CurioActivityBindings,
    deferred_effects: ActivitySlotId,
    blessing_ids: &[u64],
    blessing_groups: &[Vec<u64>],
    curio_records: &[CurioActivityRecord],
    battle_ready: bool,
) -> Result<(), OccurrenceInteractionError> {
    for (index, (operation, target, scalar)) in pairs.into_iter().enumerate() {
        let sign = operation_sign(operation);
        match target {
            Some(OccurrenceTarget::CosmicFragments) if sign != 0 => {
                let scalar = scalar.unwrap_or_else(default_scalar);
                match scalar.unit() {
                    AuthoredScalarUnit::Scalar => {
                        let value = exact_integer(scalar)?;
                        let delta = value
                            .checked_mul(i64::from(sign))
                            .ok_or(OccurrenceInteractionError::Arithmetic)?;
                        output.push(PayloadOperation::FragmentScalar {
                            slot: cosmic_fragments,
                            gain_inventory: curio_bindings.inventory,
                            delta,
                        });
                    }
                    AuthoredScalarUnit::Percent => {
                        output.push(PayloadOperation::FragmentPercent {
                            slot: cosmic_fragments,
                            gain_inventory: curio_bindings.inventory,
                            coefficient: scalar.value().coefficient(),
                            scale: scalar.value().scale(),
                            sign,
                        });
                    }
                }
            }
            Some(OccurrenceTarget::Blessing) if sign != 0 => {
                if sign > 0 && !blessing_groups.is_empty() {
                    output.push(PayloadOperation::EnsureInventoryGroup {
                        inventory: blessing_inventory,
                        groups: blessing_groups.to_vec(),
                    });
                } else {
                    let count = scalar
                        .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
                        .map(exact_integer)
                        .transpose()?
                        .unwrap_or(1)
                        .max(1);
                    output.push(PayloadOperation::Inventory {
                        inventory: blessing_inventory,
                        delta: sign,
                        quantity: u16::try_from(count)
                            .map_err(|_| OccurrenceInteractionError::Arithmetic)?,
                        owned_only: sign < 0 || operation == OccurrenceOperation::Enhance,
                        candidates: blessing_ids.to_vec(),
                    });
                }
            }
            Some(OccurrenceTarget::Curio)
                if sign != 0 && operation != OccurrenceOperation::Enhance =>
            {
                let count = scalar
                    .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
                    .map(exact_integer)
                    .transpose()?
                    .unwrap_or(1)
                    .max(1);
                output.push(PayloadOperation::CurioInventory {
                    bindings: curio_bindings,
                    delta: sign,
                    quantity: u16::try_from(count)
                        .map_err(|_| OccurrenceInteractionError::Arithmetic)?,
                    owned_only: sign < 0,
                    candidates: curio_records.to_vec(),
                });
            }
            Some(OccurrenceTarget::Hp)
                if operation == OccurrenceOperation::Lose
                    && scalar.is_some_and(|value| value.unit() == AuthoredScalarUnit::Percent) =>
            {
                output.push(PayloadOperation::ParticipantHpLoss {
                    scaled_ratio: s02::percent_ratio_scaled(
                        scalar.expect("guarded percentage scalar is present"),
                    )?,
                });
            }
            None if operation == OccurrenceOperation::Battle && battle_ready => {
                output.push(PayloadOperation::Transition);
            }
            None if operation == OccurrenceOperation::Special => {
                output.push(PayloadOperation::Transition);
            }
            _ => output.push(PayloadOperation::DeferredEffect {
                slot: deferred_effects,
                key: deferred_effect_key(choice, index, operation, target)?,
            }),
        }
    }
    Ok(())
}

fn referenced_blessings(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
) -> Result<Vec<u64>, OccurrenceInteractionError> {
    referenced_ids(
        outcome,
        "universe.blessing.",
        catalog
            .blessings()
            .iter()
            .map(|value| (value.stable_key(), u64::from(value.id().get()))),
    )
}

fn referenced_curios(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    records: &[CurioActivityRecord],
) -> Result<Vec<CurioActivityRecord>, OccurrenceInteractionError> {
    let references = outcome
        .parameter_refs()
        .iter()
        .filter(|value| value.starts_with("universe.curio."))
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();
    if references.is_empty() {
        return Ok(records.to_vec());
    }
    let mut selected = Vec::with_capacity(references.len());
    for reference in references {
        let id = catalog
            .curios()
            .iter()
            .find(|value| value.stable_key() == reference)
            .map(|value| value.id())
            .ok_or(OccurrenceInteractionError::InvalidChoice)?;
        let record = records
            .iter()
            .copied()
            .find(|value| value.id() == id)
            .ok_or(OccurrenceInteractionError::InvalidChoice)?;
        selected.push(record);
    }
    selected.sort_unstable_by_key(|value| value.id());
    selected.dedup_by_key(|value| value.id());
    Ok(selected)
}

fn referenced_ids<'a>(
    outcome: &OccurrenceOutcome,
    prefix: &str,
    available: impl IntoIterator<Item = (&'a str, u64)>,
) -> Result<Vec<u64>, OccurrenceInteractionError> {
    let references = outcome
        .parameter_refs()
        .iter()
        .filter(|value| value.starts_with(prefix))
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();
    let available = available.into_iter().collect::<Vec<_>>();
    if references.is_empty() {
        return Ok(available.into_iter().map(|(_, id)| id).collect());
    }
    let mut selected = Vec::with_capacity(references.len());
    for reference in references {
        selected.push(
            available
                .iter()
                .find(|(stable_key, _)| *stable_key == reference)
                .map(|(_, id)| *id)
                .ok_or(OccurrenceInteractionError::InvalidChoice)?,
        );
    }
    selected.sort_unstable();
    selected.dedup();
    Ok(selected)
}

fn externalize_single_selection(
    operations: &[PayloadOperation],
) -> Result<Vec<OccurrenceExternalResult>, OccurrenceInteractionError> {
    let selection = operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| match operation {
            PayloadOperation::Inventory {
                quantity: 1,
                candidates,
                ..
            } => Some((index, candidates.clone())),
            PayloadOperation::CurioInventory {
                quantity: 1,
                candidates,
                ..
            } => Some((
                index,
                candidates
                    .iter()
                    .map(|value| u64::from(value.id().get()))
                    .collect(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    if selection.len() != 1 {
        return Ok(Vec::new());
    }
    let (selection_index, candidates) = &selection[0];
    candidates
        .iter()
        .map(|candidate| {
            let mut concrete = operations.to_vec();
            match &mut concrete[*selection_index] {
                PayloadOperation::Inventory { candidates, .. } => {
                    candidates.clear();
                    candidates.push(*candidate);
                }
                PayloadOperation::CurioInventory { candidates, .. } => {
                    candidates.retain(|value| u64::from(value.id().get()) == *candidate);
                }
                _ => return Err(OccurrenceInteractionError::InvalidChoice),
            }
            let (payload, immediate_operations, deferred_operations) = encode_operations(concrete)?;
            Ok(OccurrenceExternalResult {
                content: *candidate,
                payload: payload.into_boxed_slice(),
                immediate_operations,
                deferred_operations,
            })
        })
        .collect()
}

fn encode_operations(
    operations: Vec<PayloadOperation>,
) -> Result<(Vec<u8>, u16, u16), OccurrenceInteractionError> {
    let deferred_operations = u16::try_from(
        operations
            .iter()
            .filter(|operation| operation.is_deferred())
            .count(),
    )
    .map_err(|_| OccurrenceInteractionError::TooManyOperations)?;
    let immediate_operations = u16::try_from(operations.len())
        .map_err(|_| OccurrenceInteractionError::TooManyOperations)?
        .saturating_sub(deferred_operations);
    let mut payload = Vec::new();
    payload.push(PAYLOAD_REVISION);
    payload.extend_from_slice(
        &u16::try_from(operations.len())
            .map_err(|_| OccurrenceInteractionError::TooManyOperations)?
            .to_le_bytes(),
    );
    for operation in operations {
        operation.encode(&mut payload)?;
    }
    Ok((payload, immediate_operations, deferred_operations))
}

fn lower_costs(
    output: &mut Vec<PayloadOperation>,
    choice: &OccurrenceChoiceDefinition,
    cosmic_fragments: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_inventory: ActivityInventoryId,
    blessing_ids: &[u64],
    curio_ids: &[u64],
) -> Result<(), OccurrenceInteractionError> {
    for cost in choice.costs() {
        for target in cost.targets() {
            match target {
                OccurrenceTarget::CosmicFragments => {
                    output.push(PayloadOperation::RequireFragment {
                        slot: cosmic_fragments,
                        amount: 1,
                    });
                }
                OccurrenceTarget::Blessing => {
                    output.push(PayloadOperation::RequireInventory {
                        inventory: blessing_inventory,
                        candidates: blessing_ids.to_vec(),
                    });
                }
                OccurrenceTarget::Curio => {
                    output.push(PayloadOperation::RequireInventory {
                        inventory: curio_inventory,
                        candidates: curio_ids.to_vec(),
                    });
                }
                OccurrenceTarget::Character | OccurrenceTarget::Hp => {}
            }
        }
    }
    Ok(())
}

fn outcome_pairs(
    outcome: &OccurrenceOutcome,
) -> Vec<(
    OccurrenceOperation,
    Option<OccurrenceTarget>,
    Option<AuthoredScalar>,
)> {
    if outcome.operations().len() == 1 && outcome.targets().len() > 1 {
        return outcome
            .targets()
            .iter()
            .enumerate()
            .map(|(index, target)| {
                (
                    outcome.operations()[0],
                    Some(*target),
                    outcome
                        .numeric_literals()
                        .get(index)
                        .or_else(|| outcome.numeric_literals().first())
                        .copied(),
                )
            })
            .collect();
    }
    outcome
        .operations()
        .iter()
        .enumerate()
        .map(|(index, operation)| {
            (
                *operation,
                outcome
                    .targets()
                    .get(index)
                    .or_else(|| outcome.targets().first())
                    .copied(),
                outcome
                    .numeric_literals()
                    .get(index)
                    .or_else(|| outcome.numeric_literals().first())
                    .copied(),
            )
        })
        .collect()
}

fn deferred_effect_key(
    choice: OccurrenceChoiceId,
    index: usize,
    operation: OccurrenceOperation,
    target: Option<OccurrenceTarget>,
) -> Result<u64, OccurrenceInteractionError> {
    let index = u64::try_from(index).map_err(|_| OccurrenceInteractionError::Arithmetic)?;
    Ok(DEFERRED_EFFECT_KEY_BASE
        | (u64::from(choice.get()) << 24)
        | (index << 8)
        | (u64::from(operation as u8) << 4)
        | target.map_or(15, |value| u64::from(value as u8)))
}

fn default_scalar() -> AuthoredScalar {
    AuthoredScalar::new(
        crate::path::ExactParameter::new(1, 0),
        AuthoredScalarUnit::Scalar,
    )
}

const fn operation_sign(operation: OccurrenceOperation) -> i8 {
    match operation {
        OccurrenceOperation::Obtain | OccurrenceOperation::Enhance => 1,
        OccurrenceOperation::Consume | OccurrenceOperation::Discard | OccurrenceOperation::Lose => {
            -1
        }
        _ => 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccurrenceInteractionError {
    InvalidChoice,
    TooManyOperations,
    TooManyCandidates,
    NonIntegerScalar,
    Arithmetic,
}
