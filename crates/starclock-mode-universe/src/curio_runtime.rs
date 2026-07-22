//! Generic Curio ownership, lifecycle and scoped-contribution runtime.

use std::collections::BTreeMap;

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityInventoryView,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivitySlotId,
    ActivitySlotView, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    curio::{CurioDefinition, CurioStateDefinition, CurioStateKind},
    digest::Encoder,
    id::{CurioId, CurioStateId},
    path::ExactParameter,
};

pub const CURIO_RUNTIME_REVISION: &str = "standard-universe-curio-runtime-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurioRuntimeBindings {
    pub inventory: ActivityInventoryId,
    pub state_slot: ActivitySlotId,
    pub charge_slot: ActivitySlotId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurioChargeState {
    remaining: u8,
    maximum: u8,
}

impl CurioChargeState {
    #[must_use]
    pub const fn remaining(self) -> u8 {
        self.remaining
    }
    #[must_use]
    pub const fn maximum(self) -> u8 {
        self.maximum
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioStateContribution {
    id: CurioStateId,
    kind: CurioStateKind,
    source_effect_id: Box<str>,
    rule_key: Box<str>,
    parameters: Box<[ExactParameter]>,
    charge: Option<CurioChargeState>,
}

impl CurioStateContribution {
    #[must_use]
    pub const fn id(&self) -> CurioStateId {
        self.id
    }
    #[must_use]
    pub const fn kind(&self) -> CurioStateKind {
        self.kind
    }
    #[must_use]
    pub fn source_effect_id(&self) -> &str {
        &self.source_effect_id
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
    #[must_use]
    pub const fn charge(&self) -> Option<CurioChargeState> {
        self.charge
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioContribution {
    curio: CurioId,
    stable_key: Box<str>,
    tags: Box<[Box<str>]>,
    pool_tags: Box<[Box<str>]>,
    rule_key: Box<str>,
    state: CurioStateContribution,
}

impl CurioContribution {
    #[must_use]
    pub const fn curio(&self) -> CurioId {
        self.curio
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        &self.tags
    }
    #[must_use]
    pub fn pool_tags(&self) -> &[Box<str>] {
        &self.pool_tags
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub const fn state(&self) -> &CurioStateContribution {
        &self.state
    }
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.pool_tags
            .iter()
            .any(|tag| tag.as_ref() == "polarity:negative")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioContributionSet {
    entries: Box<[CurioContribution]>,
    digest: [u8; 32],
}

impl CurioContributionSet {
    #[must_use]
    pub fn entries(&self) -> &[CurioContribution] {
        &self.entries
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioRuntimeDefinition {
    curio: CurioId,
    stable_key: Box<str>,
    initial_state: CurioStateId,
    tags: Box<[Box<str>]>,
    pool_tags: Box<[Box<str>]>,
    rule_key: Box<str>,
    states: Box<[CurioRuntimeState]>,
}

impl CurioRuntimeDefinition {
    #[must_use]
    pub const fn curio(&self) -> CurioId {
        self.curio
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn initial_state(&self) -> CurioStateId {
        self.initial_state
    }
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        &self.tags
    }
    #[must_use]
    pub fn pool_tags(&self) -> &[Box<str>] {
        &self.pool_tags
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn states(&self) -> &[CurioRuntimeState] {
        &self.states
    }

    fn state(&self, id: CurioStateId) -> Option<&CurioRuntimeState> {
        self.states
            .binary_search_by_key(&id, CurioRuntimeState::id)
            .ok()
            .map(|index| &self.states[index])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioRuntimeState {
    id: CurioStateId,
    kind: CurioStateKind,
    maximum_charges: Option<u8>,
    charge_parameter_index: Option<u8>,
    next_state: Option<CurioStateId>,
    repair_state: Option<CurioStateId>,
    replacement_curio: Option<CurioId>,
    source_effect_id: Box<str>,
    rule_key: Box<str>,
    parameters: Box<[ExactParameter]>,
}

impl CurioRuntimeState {
    #[must_use]
    pub const fn id(&self) -> CurioStateId {
        self.id
    }
    #[must_use]
    pub const fn kind(&self) -> CurioStateKind {
        self.kind
    }
    #[must_use]
    pub const fn maximum_charges(&self) -> Option<u8> {
        self.maximum_charges
    }
    #[must_use]
    pub const fn charge_parameter_index(&self) -> Option<u8> {
        self.charge_parameter_index
    }
    #[must_use]
    pub const fn next_state(&self) -> Option<CurioStateId> {
        self.next_state
    }
    #[must_use]
    pub const fn repair_state(&self) -> Option<CurioStateId> {
        self.repair_state
    }
    #[must_use]
    pub const fn replacement_curio(&self) -> Option<CurioId> {
        self.replacement_curio
    }
    #[must_use]
    pub fn source_effect_id(&self) -> &str {
        &self.source_effect_id
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioRuntimeCatalog {
    definitions: Box<[CurioRuntimeDefinition]>,
    digest: [u8; 32],
}

impl CurioRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, CurioRuntimeError> {
        let mut definitions = Vec::with_capacity(catalog.curios().len());
        for curio in catalog.curios() {
            definitions.push(compile_definition(catalog, curio)?);
        }
        definitions.sort_by_key(CurioRuntimeDefinition::curio);
        if definitions.len() != 61
            || definitions
                .windows(2)
                .any(|pair| pair[0].curio == pair[1].curio)
            || definitions
                .iter()
                .map(|value| value.states.len())
                .sum::<usize>()
                != 67
        {
            return Err(CurioRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&definitions);
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub fn definitions(&self) -> &[CurioRuntimeDefinition] {
        &self.definitions
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn definition(&self, id: CurioId) -> Option<&CurioRuntimeDefinition> {
        self.definitions
            .binary_search_by_key(&id, CurioRuntimeDefinition::curio)
            .ok()
            .map(|index| &self.definitions[index])
    }

    #[must_use]
    pub fn acquisition_option(
        &self,
        id: CurioId,
        option: ActivityOptionId,
        priority: i32,
        bindings: CurioRuntimeBindings,
        mut settlement: Vec<ActivityOperation>,
    ) -> Option<ActivityOptionDefinition> {
        let definition = self.definition(id)?;
        let mut operations = acquisition_operations(definition, bindings);
        operations.append(&mut settlement);
        Some(ActivityOptionDefinition::new(
            option,
            priority,
            available(id, bindings),
            operations,
        ))
    }

    pub fn consume_charge_operations(
        &self,
        id: CurioId,
        expected_remaining: u8,
        bindings: CurioRuntimeBindings,
    ) -> Result<Box<[ActivityOperation]>, CurioRuntimeError> {
        let definition = self
            .definition(id)
            .ok_or(CurioRuntimeError::UnknownCurio(id))?;
        let state = definition
            .states
            .iter()
            .find(|state| state.maximum_charges.is_some())
            .ok_or(CurioRuntimeError::CurioHasNoCharges(id))?;
        let maximum = state.maximum_charges.expect("charged state");
        if expected_remaining == 0 || expected_remaining > maximum {
            return Err(CurioRuntimeError::InvalidExpectedCharge(id));
        }
        let mut operations = vec![
            require_owned(id, bindings),
            require_state(id, state.id, bindings),
            ActivityOperation::Require(equals(
                counter(bindings.charge_slot, id),
                i64::from(expected_remaining),
            )),
            ActivityOperation::AddCounter {
                slot: bindings.charge_slot,
                key: u64::from(id.get()),
                delta: integer(-1),
            },
        ];
        if expected_remaining == 1
            && let Some(next) = state.next_state
        {
            operations.push(transition_operation(id, state.id, next, bindings));
        }
        Ok(operations.into_boxed_slice())
    }

    pub fn repair_operations(
        &self,
        id: CurioId,
        bindings: CurioRuntimeBindings,
    ) -> Result<Box<[ActivityOperation]>, CurioRuntimeError> {
        let definition = self
            .definition(id)
            .ok_or(CurioRuntimeError::UnknownCurio(id))?;
        let repairing = definition
            .states
            .iter()
            .find(|state| state.kind == CurioStateKind::Repairing)
            .ok_or(CurioRuntimeError::CurioCannotBeRepaired(id))?;
        let target = repairing
            .repair_state
            .ok_or(CurioRuntimeError::InvalidState(repairing.id))?;
        Ok(vec![
            require_owned(id, bindings),
            require_state(id, repairing.id, bindings),
            ActivityOperation::AddCounter {
                slot: bindings.charge_slot,
                key: u64::from(id.get()),
                delta: ActivityExpression::Negate(Box::new(counter(bindings.charge_slot, id))),
            },
            transition_operation(id, repairing.id, target, bindings),
        ]
        .into_boxed_slice())
    }

    pub fn replacement_operations(
        &self,
        removed: CurioId,
        acquired: CurioId,
        bindings: CurioRuntimeBindings,
    ) -> Result<Box<[ActivityOperation]>, CurioRuntimeError> {
        if removed == acquired {
            return Err(CurioRuntimeError::ReplacementCycle(removed));
        }
        let acquired_definition = self
            .definition(acquired)
            .ok_or(CurioRuntimeError::UnknownCurio(acquired))?;
        self.definition(removed)
            .ok_or(CurioRuntimeError::UnknownCurio(removed))?;
        let mut operations = vec![
            require_owned(removed, bindings),
            ActivityOperation::Require(available(acquired, bindings)),
        ];
        operations.extend(teardown_operations(removed, bindings));
        operations.extend(acquisition_operations(acquired_definition, bindings));
        Ok(operations.into_boxed_slice())
    }

    pub fn teardown_operations(
        &self,
        id: CurioId,
        bindings: CurioRuntimeBindings,
    ) -> Result<Box<[ActivityOperation]>, CurioRuntimeError> {
        self.definition(id)
            .ok_or(CurioRuntimeError::UnknownCurio(id))?;
        let mut operations = vec![require_owned(id, bindings)];
        operations.extend(teardown_operations(id, bindings));
        Ok(operations.into_boxed_slice())
    }

    pub fn contributions(
        &self,
        inventory: &ActivityInventoryView,
        state: &ActivitySlotView,
        charges: &ActivitySlotView,
    ) -> Result<CurioContributionSet, CurioRuntimeError> {
        let state = counter_entries(state).ok_or(CurioRuntimeError::InvalidStateSlot)?;
        let charges = counter_entries(charges).ok_or(CurioRuntimeError::InvalidChargeSlot)?;
        self.contributions_from_raw(inventory.entries(), state, charges)
    }

    pub fn contributions_from_owned(
        &self,
        inventory: &[(CurioId, u32)],
        states: &[(CurioId, CurioStateId)],
        charges: &[(CurioId, u8)],
    ) -> Result<CurioContributionSet, CurioRuntimeError> {
        let inventory = inventory
            .iter()
            .map(|(id, count)| (u64::from(id.get()), *count))
            .collect::<Vec<_>>();
        let states = states
            .iter()
            .map(|(id, state)| (u64::from(id.get()), i64::from(state.get())))
            .collect::<Vec<_>>();
        let charges = charges
            .iter()
            .map(|(id, count)| (u64::from(id.get()), i64::from(*count)))
            .collect::<Vec<_>>();
        self.contributions_from_raw(&inventory, &states, &charges)
    }

    fn contributions_from_raw(
        &self,
        inventory: &[(u64, u32)],
        states: &[(u64, i64)],
        charges: &[(u64, i64)],
    ) -> Result<CurioContributionSet, CurioRuntimeError> {
        let state_map = canonical_map(states)?;
        let charge_map = canonical_map(charges)?;
        let mut entries = Vec::with_capacity(inventory.len());
        for (raw, stacks) in inventory {
            let id = parse_curio(*raw)?;
            if *stacks != 1 {
                return Err(CurioRuntimeError::InvalidInventoryStack(id));
            }
            let definition = self
                .definition(id)
                .ok_or(CurioRuntimeError::UnknownInventoryEntry(*raw))?;
            let raw_state = *state_map.get(raw).unwrap_or(&0);
            let raw_state = u32::try_from(raw_state)
                .ok()
                .and_then(CurioStateId::new)
                .ok_or(CurioRuntimeError::MissingCurrentState(id))?;
            let state = definition
                .state(raw_state)
                .ok_or(CurioRuntimeError::StateOwnershipMismatch(raw_state))?;
            let raw_charge = *charge_map.get(raw).unwrap_or(&0);
            let charge = match state.maximum_charges {
                Some(maximum) => {
                    let remaining = u8::try_from(raw_charge)
                        .ok()
                        .filter(|value| *value <= maximum)
                        .ok_or(CurioRuntimeError::InvalidChargeValue(id))?;
                    Some(CurioChargeState { remaining, maximum })
                }
                None if raw_charge == 0 => None,
                None => return Err(CurioRuntimeError::UnexpectedChargeValue(id)),
            };
            entries.push(CurioContribution {
                curio: id,
                stable_key: definition.stable_key.clone(),
                tags: definition.tags.clone(),
                pool_tags: definition.pool_tags.clone(),
                rule_key: definition.rule_key.clone(),
                state: CurioStateContribution {
                    id: state.id,
                    kind: state.kind,
                    source_effect_id: state.source_effect_id.clone(),
                    rule_key: state.rule_key.clone(),
                    parameters: state.parameters.clone(),
                    charge,
                },
            });
        }
        entries.sort_by_key(CurioContribution::curio);
        if entries
            .windows(2)
            .any(|pair| pair[0].curio == pair[1].curio)
        {
            return Err(CurioRuntimeError::DuplicateInventoryEntry);
        }
        self.validate_no_orphans(&entries, &state_map, &charge_map)?;
        let digest = contribution_digest(&entries);
        Ok(CurioContributionSet {
            entries: entries.into_boxed_slice(),
            digest,
        })
    }

    fn validate_no_orphans(
        &self,
        entries: &[CurioContribution],
        states: &BTreeMap<u64, i64>,
        charges: &BTreeMap<u64, i64>,
    ) -> Result<(), CurioRuntimeError> {
        for map in [states, charges] {
            for (raw, value) in map {
                let id = parse_curio(*raw)?;
                if self.definition(id).is_none()
                    || (*value != 0 && !entries.iter().any(|entry| entry.curio == id))
                {
                    return Err(CurioRuntimeError::OrphanedRuntimeState(*raw));
                }
            }
        }
        Ok(())
    }
}

fn compile_definition(
    catalog: &UniverseCatalog,
    curio: &CurioDefinition,
) -> Result<CurioRuntimeDefinition, CurioRuntimeError> {
    let mut states = Vec::with_capacity(curio.states().len());
    for id in curio.states() {
        let state = catalog
            .curio_state(*id)
            .ok_or(CurioRuntimeError::MissingState(*id))?;
        if state.curio() != curio.id() {
            return Err(CurioRuntimeError::StateOwnershipMismatch(*id));
        }
        states.push(runtime_state(state)?);
    }
    states.sort_by_key(CurioRuntimeState::id);
    if !states.iter().any(|state| state.id == curio.initial_state()) {
        return Err(CurioRuntimeError::MissingState(curio.initial_state()));
    }
    Ok(CurioRuntimeDefinition {
        curio: curio.id(),
        stable_key: curio.stable_key().into(),
        initial_state: curio.initial_state(),
        tags: curio.tags().to_vec().into_boxed_slice(),
        pool_tags: curio.pool_tags().to_vec().into_boxed_slice(),
        rule_key: curio.rule_key().into(),
        states: states.into_boxed_slice(),
    })
}

fn runtime_state(state: &CurioStateDefinition) -> Result<CurioRuntimeState, CurioRuntimeError> {
    let maximum_charges = state
        .charges()
        .map(|value| {
            if value.scale() != 0 {
                return Err(CurioRuntimeError::NonIntegralCharges(state.id()));
            }
            u8::try_from(value.coefficient())
                .ok()
                .filter(|value| *value != 0)
                .ok_or(CurioRuntimeError::InvalidState(state.id()))
        })
        .transpose()?;
    Ok(CurioRuntimeState {
        id: state.id(),
        kind: state.kind(),
        maximum_charges,
        charge_parameter_index: state.charge_parameter_index(),
        next_state: state.next_state(),
        repair_state: state.repair_state(),
        replacement_curio: state.replacement_curio(),
        source_effect_id: state.source_effect_id().into(),
        rule_key: state.rule_key().into(),
        parameters: state.parameters().to_vec().into_boxed_slice(),
    })
}

fn acquisition_operations(
    definition: &CurioRuntimeDefinition,
    bindings: CurioRuntimeBindings,
) -> Vec<ActivityOperation> {
    let state = definition
        .state(definition.initial_state)
        .expect("compiled initial state exists");
    let mut operations = vec![
        ActivityOperation::AddInventory {
            inventory: bindings.inventory,
            content: u64::from(definition.curio.get()),
            count: integer(1),
        },
        ActivityOperation::AddCounter {
            slot: bindings.state_slot,
            key: u64::from(definition.curio.get()),
            delta: integer(i64::from(state.id.get())),
        },
    ];
    if let Some(charges) = state.maximum_charges {
        operations.push(ActivityOperation::AddCounter {
            slot: bindings.charge_slot,
            key: u64::from(definition.curio.get()),
            delta: integer(i64::from(charges)),
        });
    }
    operations
}

fn teardown_operations(id: CurioId, bindings: CurioRuntimeBindings) -> Vec<ActivityOperation> {
    vec![
        ActivityOperation::RemoveInventory {
            inventory: bindings.inventory,
            content: u64::from(id.get()),
            count: integer(1),
        },
        ActivityOperation::AddCounter {
            slot: bindings.state_slot,
            key: u64::from(id.get()),
            delta: ActivityExpression::Negate(Box::new(counter(bindings.state_slot, id))),
        },
        ActivityOperation::AddCounter {
            slot: bindings.charge_slot,
            key: u64::from(id.get()),
            delta: ActivityExpression::Negate(Box::new(counter(bindings.charge_slot, id))),
        },
    ]
}

fn transition_operation(
    id: CurioId,
    from: CurioStateId,
    to: CurioStateId,
    bindings: CurioRuntimeBindings,
) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: bindings.state_slot,
        key: u64::from(id.get()),
        delta: integer(i64::from(to.get()) - i64::from(from.get())),
    }
}

fn require_owned(id: CurioId, bindings: CurioRuntimeBindings) -> ActivityOperation {
    ActivityOperation::Require(equals(inventory_count(bindings.inventory, id), 1))
}

fn require_state(
    id: CurioId,
    state: CurioStateId,
    bindings: CurioRuntimeBindings,
) -> ActivityOperation {
    ActivityOperation::Require(equals(
        counter(bindings.state_slot, id),
        i64::from(state.get()),
    ))
}

fn available(id: CurioId, bindings: CurioRuntimeBindings) -> ActivityCondition {
    ActivityCondition::All(
        vec![
            equals(inventory_count(bindings.inventory, id), 0),
            equals(counter(bindings.state_slot, id), 0),
            equals(counter(bindings.charge_slot, id), 0),
        ]
        .into_boxed_slice(),
    )
}

fn inventory_count(inventory: ActivityInventoryId, id: CurioId) -> ActivityExpression {
    ActivityExpression::InventoryCount {
        inventory,
        content: u64::from(id.get()),
    }
}

fn counter(slot: ActivitySlotId, id: CurioId) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot,
        key: u64::from(id.get()),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn equals(expression: ActivityExpression, value: i64) -> ActivityCondition {
    ActivityCondition::Equal(expression, integer(value))
}

fn counter_entries(slot: &ActivitySlotView) -> Option<&[(u64, i64)]> {
    match slot.value() {
        ActivityValue::BoundedCounterMap(entries) => Some(entries),
        _ => None,
    }
}

fn canonical_map(entries: &[(u64, i64)]) -> Result<BTreeMap<u64, i64>, CurioRuntimeError> {
    let mut result = BTreeMap::new();
    for (key, value) in entries {
        if *key == 0 || result.insert(*key, *value).is_some() {
            return Err(CurioRuntimeError::InvalidCounterMap);
        }
    }
    Ok(result)
}

fn parse_curio(raw: u64) -> Result<CurioId, CurioRuntimeError> {
    u32::try_from(raw)
        .ok()
        .and_then(CurioId::new)
        .ok_or(CurioRuntimeError::UnknownInventoryEntry(raw))
}

fn catalog_digest(definitions: &[CurioRuntimeDefinition]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-curio-runtime-catalog-v1");
    encoder.text(CURIO_RUNTIME_REVISION);
    encoder.u32(definitions.len() as u32);
    for definition in definitions {
        encoder.u32(definition.curio.get());
        encoder.text(&definition.stable_key);
        encoder.u32(definition.initial_state.get());
        encode_texts(&mut encoder, &definition.tags);
        encode_texts(&mut encoder, &definition.pool_tags);
        encoder.text(&definition.rule_key);
        encoder.u32(definition.states.len() as u32);
        for state in &definition.states {
            encoder.u32(state.id.get());
            encoder.u8(state.kind as u8);
            encoder.u8(state.maximum_charges.unwrap_or(0));
            encoder.u8(state.charge_parameter_index.unwrap_or(0));
            encoder.u32(state.next_state.map_or(0, CurioStateId::get));
            encoder.u32(state.repair_state.map_or(0, CurioStateId::get));
            encoder.u32(state.replacement_curio.map_or(0, CurioId::get));
            encoder.text(&state.source_effect_id);
            encoder.text(&state.rule_key);
            encode_parameters(&mut encoder, &state.parameters);
        }
    }
    encoder.finish()
}

fn contribution_digest(entries: &[CurioContribution]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-curio-contribution-set-v1");
    encoder.u32(entries.len() as u32);
    for entry in entries {
        encoder.u32(entry.curio.get());
        encoder.text(&entry.stable_key);
        encode_texts(&mut encoder, &entry.tags);
        encode_texts(&mut encoder, &entry.pool_tags);
        encoder.text(&entry.rule_key);
        encoder.u32(entry.state.id.get());
        encoder.u8(entry.state.kind as u8);
        encoder.text(&entry.state.source_effect_id);
        encoder.text(&entry.state.rule_key);
        encode_parameters(&mut encoder, &entry.state.parameters);
        encoder.u8(entry.state.charge.map_or(0, |charge| charge.remaining));
        encoder.u8(entry.state.charge.map_or(0, |charge| charge.maximum));
    }
    encoder.finish()
}

fn encode_texts(encoder: &mut Encoder, values: &[Box<str>]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.text(value);
    }
}

fn encode_parameters(encoder: &mut Encoder, values: &[ExactParameter]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.i64(value.coefficient());
        encoder.u8(value.scale());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioRuntimeError {
    InvalidDenominator,
    UnknownCurio(CurioId),
    MissingState(CurioStateId),
    InvalidState(CurioStateId),
    StateOwnershipMismatch(CurioStateId),
    NonIntegralCharges(CurioStateId),
    CurioHasNoCharges(CurioId),
    InvalidExpectedCharge(CurioId),
    CurioCannotBeRepaired(CurioId),
    ReplacementCycle(CurioId),
    UnknownInventoryEntry(u64),
    InvalidInventoryStack(CurioId),
    DuplicateInventoryEntry,
    MissingCurrentState(CurioId),
    InvalidChargeValue(CurioId),
    UnexpectedChargeValue(CurioId),
    InvalidStateSlot,
    InvalidChargeSlot,
    MissingInventory,
    InvalidCounterMap,
    OrphanedRuntimeState(u64),
}
