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
    digest::Encoder,
    id::{CurioId, CurioStateId, OccurrenceChoiceId},
    occurrence::{
        AuthoredScalar, AuthoredScalarUnit, OccurrenceChoiceDefinition, OccurrenceOperation,
        OccurrenceOutcome, OccurrenceTarget, RandomOutcomePolicy,
    },
};

pub const OCCURRENCE_INTERACTION_HANDLER_ID: u32 = 2;
pub const OCCURRENCE_INTERACTION_RUNTIME_REVISION: &str =
    "standard-universe-occurrence-interaction-runtime-v2";
const PAYLOAD_REVISION: u8 = 2;
const TAG_FRAGMENT_SCALAR: u8 = 1;
const TAG_FRAGMENT_PERCENT: u8 = 2;
const TAG_INVENTORY: u8 = 3;
const TAG_REQUIRE_INVENTORY: u8 = 4;
const TAG_DEFERRED_EFFECT: u8 = 5;
const TAG_REQUIRE_FRAGMENT: u8 = 6;
const TAG_CURIO_INVENTORY: u8 = 7;
const MAX_PAYLOAD_OPERATIONS: usize = 128;
const DEFERRED_EFFECT_KEY_BASE: u64 = 1 << 63;

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledOccurrenceProgram {
    choice: OccurrenceChoiceId,
    payload: Box<[u8]>,
    random_candidate_count: Option<u32>,
    immediate_operations: u16,
    deferred_operations: u16,
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
        let mut programs = catalog
            .occurrence_choices()
            .iter()
            .map(|choice| {
                compile(
                    choice,
                    catalog,
                    cosmic_fragments,
                    blessing_inventory,
                    curio_records,
                    curio_bindings,
                    deferred_effects,
                )
                .map(|compiled| CompiledOccurrenceProgram {
                    choice: choice.id(),
                    payload: compiled.payload.into_boxed_slice(),
                    random_candidate_count: compiled.random_candidate_count,
                    immediate_operations: compiled.immediate_operations,
                    deferred_operations: compiled.deferred_operations,
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
        let digest = runtime_catalog_digest(&programs);
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
                payload: program.payload.to_vec(),
                random_candidate_count: program.random_candidate_count,
                immediate_operations: program.immediate_operations,
                deferred_operations: program.deferred_operations,
            })
    }
}

pub struct CompiledOccurrenceInteraction {
    payload: Vec<u8>,
    random_candidate_count: Option<u32>,
    immediate_operations: u16,
    deferred_operations: u16,
}

impl CompiledOccurrenceInteraction {
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

pub(crate) fn compile(
    choice: &OccurrenceChoiceDefinition,
    catalog: &UniverseCatalog,
    cosmic_fragments: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_records: &[CurioActivityRecord],
    curio_bindings: CurioActivityBindings,
    deferred_effects: ActivitySlotId,
) -> Result<CompiledOccurrenceInteraction, OccurrenceInteractionError> {
    let blessing_ids = catalog
        .blessings()
        .iter()
        .map(|value| u64::from(value.id().get()))
        .collect::<Vec<_>>();
    let curio_ids = curio_records
        .iter()
        .map(|value| u64::from(value.id().get()))
        .collect::<Vec<_>>();
    let outcome = choice
        .outcomes()
        .first()
        .ok_or(OccurrenceInteractionError::InvalidChoice)?;
    let mut operations = Vec::new();
    lower_costs(
        &mut operations,
        choice,
        cosmic_fragments,
        blessing_inventory,
        curio_bindings.inventory,
        &blessing_ids,
        &curio_ids,
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
        curio_records,
    )?;
    if operations.len() > MAX_PAYLOAD_OPERATIONS {
        return Err(OccurrenceInteractionError::TooManyOperations);
    }
    let random_candidate_count =
        if outcome.random_policy() == Some(RandomOutcomePolicy::StableUniformOrderedCandidates) {
            operations
                .iter()
                .filter_map(|operation| match operation {
                    PayloadOperation::Inventory { candidates, .. } => {
                        u32::try_from(candidates.len()).ok()
                    }
                    PayloadOperation::CurioInventory { candidates, .. } => {
                        u32::try_from(candidates.len()).ok()
                    }
                    _ => None,
                })
                .try_fold(1_u32, checked_lcm)
        } else {
            None
        };
    let mut payload = Vec::new();
    payload.push(PAYLOAD_REVISION);
    payload.extend_from_slice(
        &u16::try_from(operations.len())
            .map_err(|_| OccurrenceInteractionError::TooManyOperations)?
            .to_le_bytes(),
    );
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
    for operation in operations {
        operation.encode(&mut payload)?;
    }
    Ok(CompiledOccurrenceInteraction {
        payload,
        random_candidate_count,
        immediate_operations,
        deferred_operations,
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
    let delta = decoder.i64()?;
    if delta < 0 {
        operations.push(require_at_least(slot, delta.unsigned_abs())?);
    }
    operations.push(add_slot(slot, delta));
    Ok(())
}

fn decode_fragment_percent(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let slot = slot(decoder.u32()?)?;
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
    operations.push(add_slot(slot, delta));
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
        records.push(CurioActivityRecord::new(
            CurioId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
            CurioStateId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
            decoder.u8()?,
        ));
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

enum PayloadOperation {
    FragmentScalar {
        slot: ActivitySlotId,
        delta: i64,
    },
    FragmentPercent {
        slot: ActivitySlotId,
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
}

impl PayloadOperation {
    const fn is_deferred(&self) -> bool {
        matches!(self, Self::DeferredEffect { .. })
    }

    fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::FragmentScalar { slot, delta } => {
                output.push(TAG_FRAGMENT_SCALAR);
                output.extend_from_slice(&slot.get().to_le_bytes());
                output.extend_from_slice(&delta.to_le_bytes());
            }
            Self::FragmentPercent {
                slot,
                coefficient,
                scale,
                sign,
            } => {
                output.push(TAG_FRAGMENT_PERCENT);
                output.extend_from_slice(&slot.get().to_le_bytes());
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
    curio_records: &[CurioActivityRecord],
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
                            delta,
                        });
                    }
                    AuthoredScalarUnit::Percent => {
                        output.push(PayloadOperation::FragmentPercent {
                            slot: cosmic_fragments,
                            coefficient: scalar.value().coefficient(),
                            scale: scalar.value().scale(),
                            sign,
                        });
                    }
                }
            }
            Some(OccurrenceTarget::Blessing) if sign != 0 => {
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
            _ => output.push(PayloadOperation::DeferredEffect {
                slot: deferred_effects,
                key: deferred_effect_key(choice, index, operation, target)?,
            }),
        }
    }
    Ok(())
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

fn runtime_catalog_digest(programs: &[CompiledOccurrenceProgram]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-occurrence-interaction-runtime-v1");
    encoder.text(OCCURRENCE_INTERACTION_RUNTIME_REVISION);
    encoder.u32(programs.len() as u32);
    for program in programs {
        encoder.u32(program.choice.get());
        encoder.u32(program.payload.len() as u32);
        for byte in &program.payload {
            encoder.u8(*byte);
        }
        encoder.u32(program.random_candidate_count.unwrap_or(0));
        encoder.u32(u32::from(program.immediate_operations));
        encoder.u32(u32::from(program.deferred_operations));
    }
    encoder.finish()
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

fn exact_integer(value: AuthoredScalar) -> Result<i64, OccurrenceInteractionError> {
    let divisor = 10_i64
        .checked_pow(u32::from(value.value().scale()))
        .ok_or(OccurrenceInteractionError::Arithmetic)?;
    if value.value().coefficient() % divisor != 0 {
        return Err(OccurrenceInteractionError::NonIntegerScalar);
    }
    Ok(value.value().coefficient() / divisor)
}

fn checked_lcm(left: u32, right: u32) -> Option<u32> {
    let gcd = gcd(left, right);
    left.checked_div(gcd)?
        .checked_mul(right)
        .filter(|value| *value <= 65_536)
}

const fn gcd(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn select_candidates(
    input: ActivityHandlerInput<'_>,
    inventory: ActivityInventoryId,
    candidates: &[u64],
    owned_only: bool,
    random_index: Option<u32>,
    quantity: usize,
) -> Result<Vec<u64>, ActivityHandlerFault> {
    let eligible = if owned_only {
        let entries = input
            .view()
            .inventories()
            .iter()
            .find(|value| value.id() == inventory)
            .ok_or_else(invalid_state)?
            .entries();
        candidates
            .iter()
            .copied()
            .filter(|candidate| {
                entries
                    .iter()
                    .any(|entry| entry.0 == *candidate && entry.1 > 0)
            })
            .collect::<Vec<_>>()
    } else {
        candidates.to_vec()
    };
    if eligible.len() < quantity {
        return Err(invalid_state());
    }
    let start = random_index.map_or(0, |index| index as usize % eligible.len());
    Ok((0..quantity)
        .map(|offset| eligible[(start + offset) % eligible.len()])
        .collect())
}

fn slot_integer(
    input: ActivityHandlerInput<'_>,
    id: ActivitySlotId,
) -> Result<i64, ActivityHandlerFault> {
    input
        .view()
        .slots()
        .iter()
        .find(|value| value.id() == id)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedInteger(value) => Some(*value),
            _ => None,
        })
        .ok_or_else(invalid_state)
}

fn add_slot(slot: ActivitySlotId, delta: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot,
        delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(delta)),
    }
}

fn require_at_least(
    slot: ActivitySlotId,
    amount: u64,
) -> Result<ActivityOperation, ActivityHandlerFault> {
    let amount = i64::try_from(amount).map_err(|_| arithmetic())?;
    Ok(ActivityOperation::Require(ActivityCondition::Not(
        Box::new(ActivityCondition::LessThan(
            ActivityExpression::Slot(slot),
            ActivityExpression::Literal(ActivityValue::BoundedInteger(amount)),
        )),
    )))
}

fn slot(value: u32) -> Result<ActivitySlotId, ActivityHandlerFault> {
    ActivitySlotId::new(value).ok_or_else(invalid_payload)
}

fn inventory(value: u32) -> Result<ActivityInventoryId, ActivityHandlerFault> {
    ActivityInventoryId::new(value).ok_or_else(invalid_payload)
}

struct Decoder<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Decoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ActivityHandlerFault> {
        let end = self.cursor.checked_add(count).ok_or_else(invalid_payload)?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or_else(invalid_payload)?;
        self.cursor = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, ActivityHandlerFault> {
        Ok(self.take(1)?[0])
    }

    fn i8(&mut self) -> Result<i8, ActivityHandlerFault> {
        Ok(i8::from_le_bytes([self.u8()?]))
    }

    fn u16(&mut self) -> Result<u16, ActivityHandlerFault> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, ActivityHandlerFault> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, ActivityHandlerFault> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn i64(&mut self) -> Result<i64, ActivityHandlerFault> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn finish(self) -> Result<(), ActivityHandlerFault> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid_payload())
        }
    }
}

fn invalid_payload() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidPayload)
}

fn invalid_state() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidState)
}

fn arithmetic() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::Arithmetic)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccurrenceInteractionError {
    InvalidChoice,
    TooManyOperations,
    TooManyCandidates,
    NonIntegerScalar,
    Arithmetic,
}
