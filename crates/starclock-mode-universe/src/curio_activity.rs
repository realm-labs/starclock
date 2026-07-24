//! Activity-owned Curio lifecycle records and checked mutation operations.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityOperation, ActivitySlotId,
    ActivityValue,
};

use crate::{
    curio_effect_runtime::{AppliedCurioEffect, CurioEffect, CurioEvent},
    curio_runtime::{CurioRuntimeCatalog, CurioRuntimeError},
    id::{CurioId, CurioStateId},
};

const DEFERRED_EFFECT_KEY_BASE: u64 = 1 << 63;
const SIX_DECIMAL_SCALE: i128 = 1_000_000;

/// Checked Activity projection for one recorded Curio event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioActivityProjection {
    operations: Box<[ActivityOperation]>,
    immediate_effects: u16,
    deferred_effects: u16,
}

impl CurioActivityProjection {
    #[must_use]
    pub fn operations(&self) -> &[ActivityOperation] {
        &self.operations
    }

    #[must_use]
    pub const fn immediate_effects(&self) -> u16 {
        self.immediate_effects
    }

    #[must_use]
    pub const fn deferred_effects(&self) -> u16 {
        self.deferred_effects
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CurioActivityBindings {
    pub(crate) inventory: ActivityInventoryId,
    pub(crate) state_slot: ActivitySlotId,
    pub(crate) charge_slot: ActivitySlotId,
    pub(crate) event_slot: ActivitySlotId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CurioActivityRecord {
    id: CurioId,
    initial_state: CurioStateId,
    initial_charges: u8,
}

impl CurioActivityRecord {
    pub(crate) const fn new(id: CurioId, initial_state: CurioStateId, initial_charges: u8) -> Self {
        Self {
            id,
            initial_state,
            initial_charges,
        }
    }

    pub(crate) const fn id(self) -> CurioId {
        self.id
    }

    pub(crate) const fn initial_state(self) -> CurioStateId {
        self.initial_state
    }

    pub(crate) const fn initial_charges(self) -> u8 {
        self.initial_charges
    }
}

pub(crate) fn compile_records(
    runtime: &CurioRuntimeCatalog,
) -> Result<Box<[CurioActivityRecord]>, CurioRuntimeError> {
    let mut records = Vec::with_capacity(runtime.definitions().len());
    for definition in runtime.definitions() {
        let state = definition
            .states()
            .iter()
            .find(|state| state.id() == definition.initial_state())
            .ok_or(CurioRuntimeError::MissingState(definition.initial_state()))?;
        records.push(CurioActivityRecord {
            id: definition.curio(),
            initial_state: state.id(),
            initial_charges: state.maximum_charges().unwrap_or(0),
        });
    }
    records.sort_unstable_by_key(|record| record.id);
    if records.len() != 61 || records.windows(2).any(|pair| pair[0].id >= pair[1].id) {
        return Err(CurioRuntimeError::InvalidDenominator);
    }
    Ok(records.into_boxed_slice())
}

pub(crate) fn acquisition_operations(
    record: CurioActivityRecord,
    bindings: CurioActivityBindings,
) -> Vec<ActivityOperation> {
    let content = u64::from(record.id.get());
    let mut operations = vec![
        ActivityOperation::Require(ActivityCondition::LessThan(
            ActivityExpression::InventoryCount {
                inventory: bindings.inventory,
                content,
            },
            integer(1),
        )),
        ActivityOperation::Require(ActivityCondition::Equal(
            counter(bindings.state_slot, content),
            integer(0),
        )),
        ActivityOperation::Require(ActivityCondition::Equal(
            counter(bindings.charge_slot, content),
            integer(0),
        )),
        ActivityOperation::AddInventory {
            inventory: bindings.inventory,
            content,
            count: integer(1),
        },
        ActivityOperation::AddCounter {
            slot: bindings.state_slot,
            key: content,
            delta: integer(i64::from(record.initial_state.get())),
        },
    ];
    if record.initial_charges != 0 {
        operations.push(ActivityOperation::AddCounter {
            slot: bindings.charge_slot,
            key: content,
            delta: integer(i64::from(record.initial_charges)),
        });
    }
    operations.push(ActivityOperation::AddCounter {
        slot: bindings.event_slot,
        key: event_key(record.id, CurioEvent::Acquired),
        delta: integer(1),
    });
    operations
}

pub(crate) fn teardown_operations(
    id: CurioId,
    bindings: CurioActivityBindings,
) -> Vec<ActivityOperation> {
    let content = u64::from(id.get());
    vec![
        ActivityOperation::Require(ActivityCondition::Not(Box::new(
            ActivityCondition::LessThan(
                ActivityExpression::InventoryCount {
                    inventory: bindings.inventory,
                    content,
                },
                integer(1),
            ),
        ))),
        ActivityOperation::RemoveInventory {
            inventory: bindings.inventory,
            content,
            count: integer(1),
        },
        ActivityOperation::AddCounter {
            slot: bindings.state_slot,
            key: content,
            delta: ActivityExpression::Negate(Box::new(counter(bindings.state_slot, content))),
        },
        ActivityOperation::AddCounter {
            slot: bindings.charge_slot,
            key: content,
            delta: ActivityExpression::Negate(Box::new(counter(bindings.charge_slot, content))),
        },
    ]
}

pub(crate) const fn event_key(id: CurioId, event: CurioEvent) -> u64 {
    ((event as u64) << 32) | id.get() as u64
}

pub(crate) fn lower_effects(
    id: CurioId,
    event: CurioEvent,
    effects: &[AppliedCurioEffect],
    cosmic_fragments: u32,
    fragments_slot: ActivitySlotId,
    event_slot: ActivitySlotId,
) -> Result<CurioActivityProjection, CurioActivityProjectionError> {
    let source_event = event_key(id, event);
    let mut operations = vec![
        ActivityOperation::Require(ActivityCondition::Not(Box::new(
            ActivityCondition::LessThan(counter(event_slot, source_event), integer(1)),
        ))),
        ActivityOperation::AddCounter {
            slot: event_slot,
            key: source_event,
            delta: integer(-1),
        },
    ];
    let mut immediate_effects = 0_u16;
    let mut deferred_effects = 0_u16;
    for (index, applied) in effects.iter().enumerate() {
        let mut deferred = false;
        match applied.effect() {
            CurioEffect::GrantCosmicFragments { amount } => {
                operations.push(add_fragments(fragments_slot, i64::from(*amount)));
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::GrantFragmentsPerFullHpAlly {
                amount_per_ally,
                allies,
            } => {
                let amount = amount_per_ally
                    .checked_mul(u32::from(*allies))
                    .ok_or(CurioActivityProjectionError::Arithmetic)?;
                operations.push(add_fragments(fragments_slot, i64::from(amount)));
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::GrantFragmentsFromCurrent { ratio } => {
                let amount = ratio_amount(cosmic_fragments, ratio.raw_six_decimal())?;
                operations.push(add_fragments(fragments_slot, i64::from(amount)));
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::LoseCosmicFragmentsRatio { ratio } => {
                let amount = ratio_amount(cosmic_fragments, ratio.raw_six_decimal())?;
                debit_fragments(&mut operations, fragments_slot, amount);
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::LoseFragmentsAndAddCriticalDamage { fragments, .. } => {
                debit_fragments(&mut operations, fragments_slot, *fragments);
                immediate_effects = immediate_effects.saturating_add(1);
                deferred = true;
            }
            _ => deferred = true,
        }
        if deferred {
            let index =
                u64::try_from(index).map_err(|_| CurioActivityProjectionError::Arithmetic)?;
            operations.push(ActivityOperation::AddCounter {
                slot: event_slot,
                key: deferred_effect_key(id, event, index),
                delta: integer(1),
            });
            deferred_effects = deferred_effects.saturating_add(1);
        }
    }
    Ok(CurioActivityProjection {
        operations: operations.into_boxed_slice(),
        immediate_effects,
        deferred_effects,
    })
}

fn add_fragments(slot: ActivitySlotId, amount: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot,
        delta: integer(amount),
    }
}

fn debit_fragments(operations: &mut Vec<ActivityOperation>, slot: ActivitySlotId, amount: u32) {
    let amount = i64::from(amount);
    operations.push(ActivityOperation::Require(ActivityCondition::Not(
        Box::new(ActivityCondition::LessThan(
            ActivityExpression::Slot(slot),
            integer(amount),
        )),
    )));
    operations.push(add_fragments(slot, -amount));
}

fn ratio_amount(
    cosmic_fragments: u32,
    raw_ratio: i64,
) -> Result<u32, CurioActivityProjectionError> {
    if raw_ratio < 0 {
        return Err(CurioActivityProjectionError::Arithmetic);
    }
    i128::from(cosmic_fragments)
        .checked_mul(i128::from(raw_ratio))
        .map(|value| value / SIX_DECIMAL_SCALE)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(CurioActivityProjectionError::Arithmetic)
}

const fn deferred_effect_key(id: CurioId, event: CurioEvent, index: u64) -> u64 {
    DEFERRED_EFFECT_KEY_BASE
        | ((event as u64) << 56)
        | ((id.get() as u64) << 24)
        | (index & 0x00ff_ffff)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioActivityProjectionError {
    Arithmetic,
}

impl core::fmt::Display for CurioActivityProjectionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Curio Activity projection failed: {self:?}")
    }
}

impl std::error::Error for CurioActivityProjectionError {}

fn counter(slot: ActivitySlotId, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue { slot, key }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

#[cfg(test)]
mod tests {
    use starclock_activity::{
        ActivityCause, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
        ActivityGraphDefinition, ActivityNodeDefinition, ActivityNodeKind,
        ActivityProgramDefinition, ActivityProgramId, ActivityScope, ActivitySlotDefinition,
        ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility,
        ActivityTerminalOutcome, ActivityTransactionOutcome, ActivityTransactionState, NodeId,
        SectionId, SlotCarryPolicy, SlotResetPoint,
    };

    use super::*;

    #[test]
    fn curio_projection_commits_immediate_effects_and_records_deferred_effects_atomically() {
        let fragments = ActivitySlotId::new(1).unwrap();
        let events = ActivitySlotId::new(2).unwrap();
        let curio = CurioId::new(42).unwrap();
        let event = CurioEvent::Acquired;
        let definition = ActivityStateDefinition::new(
            vec![
                integer_slot(fragments, 100, 0, i64::from(u32::MAX), 1),
                counter_slot(events, event_key(curio, event), 2),
            ],
            vec![],
            vec![],
        )
        .unwrap();
        let graph = graph();
        let effects = [
            AppliedCurioEffect::new(
                "test.immediate",
                CurioEffect::GrantCosmicFragments { amount: 25 },
            ),
            AppliedCurioEffect::new("test.deferred", CurioEffect::SuppressPostCombatBlessing),
        ];
        let projection = lower_effects(curio, event, &effects, 100, fragments, events).unwrap();
        assert_eq!(projection.immediate_effects(), 1);
        assert_eq!(projection.deferred_effects(), 1);

        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(1).unwrap(),
            projection.operations().to_vec(),
        )
        .unwrap();
        program.validate_against(&definition, &graph).unwrap();
        let mut state = ActivityTransactionState::new(definition, graph.entry());
        assert!(matches!(
            state.apply_program(
                &program,
                ActivityCause::new(1, program.id(), graph.entry()).unwrap(),
                &graph,
            ),
            ActivityTransactionOutcome::Committed(_)
        ));
        assert_eq!(
            state.slot(fragments),
            Some(&ActivityValue::BoundedInteger(125))
        );
        assert!(matches!(
            state.slot(events),
            Some(ActivityValue::BoundedCounterMap(values))
                if values.iter().any(|(key, count)| {
                    *key == event_key(curio, event) && *count == 0
                })
                    && values.iter().any(|(key, count)| {
                        *key == deferred_effect_key(curio, event, 1) && *count == 1
                    })
        ));
    }

    fn integer_slot(
        id: ActivitySlotId,
        initial: i64,
        minimum: i64,
        maximum: i64,
        source: u64,
    ) -> ActivitySlotDefinition {
        ActivitySlotDefinition::new_with_policy(
            id,
            ActivityScope::Activity,
            ActivityValue::BoundedInteger(initial),
            Some((minimum, maximum)),
            None,
            vec![SlotResetPoint::ActivityStart],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Private,
            ActivityStateSource::new(source).unwrap(),
        )
        .unwrap()
    }

    fn counter_slot(id: ActivitySlotId, key: u64, source: u64) -> ActivitySlotDefinition {
        ActivitySlotDefinition::new_with_policy(
            id,
            ActivityScope::Activity,
            ActivityValue::BoundedCounterMap(vec![(key, 1)].into_boxed_slice()),
            Some((0, 32)),
            Some(32),
            vec![SlotResetPoint::ActivityStart],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Private,
            ActivityStateSource::new(source).unwrap(),
        )
        .unwrap()
    }

    fn graph() -> ActivityGraphDefinition {
        let entry = NodeId::new(1).unwrap();
        let terminal = NodeId::new(2).unwrap();
        let section = SectionId::new(1).unwrap();
        ActivityGraphDefinition::new(
            entry,
            vec![
                ActivityNodeDefinition::new(entry, section, ActivityNodeKind::Choice, 1).unwrap(),
                ActivityNodeDefinition::new(
                    terminal,
                    section,
                    ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                    1,
                )
                .unwrap(),
            ],
            vec![
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(1).unwrap(),
                    entry,
                    terminal,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap(),
            ],
            2,
        )
        .unwrap()
    }
}
