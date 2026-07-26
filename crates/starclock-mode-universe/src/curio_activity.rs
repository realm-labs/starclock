//! Activity-owned Curio lifecycle records and checked mutation operations.

pub(crate) mod domain;

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
pub(crate) const DESTROYED_CURIO_COUNT_KEY: u64 = u64::MAX - 1;
pub(crate) const DIMENSION_REWARD_PENDING_KEY: u64 = u64::MAX - 2;
pub(crate) const CAVITY_CRITICAL_STACK_KEY: u64 = u64::MAX - 3;
pub(crate) const ROBE_FRAGMENT_SNAPSHOT_KEY: u64 = u64::MAX - 4;
pub(crate) const GOSSIP_CURIO_CONTENT: u64 = 8;
pub(crate) const SOCIETY_TICKET_CURIO_CONTENT: u64 = 14;

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
    pub(crate) fragments_slot: ActivitySlotId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CurioActivityRecord {
    id: CurioId,
    initial_state: CurioStateId,
    initial_charges: u8,
    acquisition_fragment_divisor: Option<i64>,
    acquisition_fragment_stack_divisor: Option<i64>,
}

impl CurioActivityRecord {
    pub(crate) const fn new(
        id: CurioId,
        initial_state: CurioStateId,
        initial_charges: u8,
        acquisition_fragment_divisor: Option<i64>,
    ) -> Self {
        Self {
            id,
            initial_state,
            initial_charges,
            acquisition_fragment_divisor,
            acquisition_fragment_stack_divisor: None,
        }
    }

    pub(crate) const fn with_fragment_stack_capture(mut self, divisor: i64) -> Self {
        self.acquisition_fragment_stack_divisor = Some(divisor);
        self
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

    pub(crate) const fn acquisition_fragment_divisor(self) -> Option<i64> {
        self.acquisition_fragment_divisor
    }

    pub(crate) const fn acquisition_fragment_stack_divisor(self) -> Option<i64> {
        self.acquisition_fragment_stack_divisor
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
            acquisition_fragment_divisor: match state.source_effect_id() {
                "74" => Some(2),
                _ => None,
            },
            acquisition_fragment_stack_divisor: match state.source_effect_id() {
                "85" => Some(100),
                _ => None,
            },
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
    if let Some(divisor) = record.acquisition_fragment_divisor {
        operations.push(fragment_gain(
            bindings,
            ActivityExpression::Divide(
                Box::new(ActivityExpression::Slot(bindings.fragments_slot)),
                Box::new(integer(divisor)),
            ),
        ));
    }
    if let Some(divisor) = record.acquisition_fragment_stack_divisor {
        operations.push(ActivityOperation::AddCounter {
            slot: bindings.event_slot,
            key: CAVITY_CRITICAL_STACK_KEY,
            delta: ActivityExpression::Divide(
                Box::new(ActivityExpression::Slot(bindings.fragments_slot)),
                Box::new(integer(divisor)),
            ),
        });
        operations.push(ActivityOperation::SetSlot {
            slot: bindings.fragments_slot,
            value: integer(0),
        });
    }
    operations.push(ActivityOperation::AddCounter {
        slot: bindings.event_slot,
        key: event_key(record.id, CurioEvent::Acquired),
        delta: integer(1),
    });
    operations
}

pub(crate) fn destroyed_curio_count(value: &ActivityValue) -> Option<u32> {
    let ActivityValue::BoundedCounterMap(entries) = value else {
        return None;
    };
    entries
        .iter()
        .find(|(key, _)| *key == DESTROYED_CURIO_COUNT_KEY)
        .and_then(|(_, value)| u32::try_from(*value).ok())
        .or(Some(0))
}

pub(crate) fn cavity_critical_stacks(value: &ActivityValue) -> Option<i64> {
    let ActivityValue::BoundedCounterMap(entries) = value else {
        return None;
    };
    entries
        .iter()
        .find(|(key, _)| *key == CAVITY_CRITICAL_STACK_KEY)
        .map_or(Some(0), |(_, value)| (*value >= 0).then_some(*value))
}

pub(crate) fn fragment_gain(
    bindings: CurioActivityBindings,
    amount: ActivityExpression,
) -> ActivityOperation {
    let multiplier = ActivityExpression::Add(
        Box::new(integer(1)),
        Box::new(ActivityExpression::InventoryCount {
            inventory: bindings.inventory,
            content: GOSSIP_CURIO_CONTENT,
        }),
    );
    ActivityOperation::AddToSlot {
        slot: bindings.fragments_slot,
        delta: ActivityExpression::Multiply(Box::new(amount), Box::new(multiplier)),
    }
}

/// Applies modifiers that are explicitly limited to post-battle fragment
/// rewards. Gossip first doubles every fragment gain; Society Ticket then
/// scales only this reward category by 175%, with fixed-point-free integer
/// arithmetic and the Activity transaction's checked overflow behavior.
pub(crate) fn battle_fragment_gain(
    bindings: CurioActivityBindings,
    amount: ActivityExpression,
) -> ActivityOperation {
    let gossip = ActivityExpression::Add(
        Box::new(integer(1)),
        Box::new(ActivityExpression::InventoryCount {
            inventory: bindings.inventory,
            content: GOSSIP_CURIO_CONTENT,
        }),
    );
    let society_quarters = ActivityExpression::Add(
        Box::new(integer(4)),
        Box::new(ActivityExpression::Multiply(
            Box::new(ActivityExpression::InventoryCount {
                inventory: bindings.inventory,
                content: SOCIETY_TICKET_CURIO_CONTENT,
            }),
            Box::new(integer(3)),
        )),
    );
    ActivityOperation::AddToSlot {
        slot: bindings.fragments_slot,
        delta: ActivityExpression::Divide(
            Box::new(ActivityExpression::Multiply(
                Box::new(ActivityExpression::Multiply(
                    Box::new(amount),
                    Box::new(gossip),
                )),
                Box::new(society_quarters),
            )),
            Box::new(integer(4)),
        ),
    }
}

pub(crate) fn dimension_reward_condition(bindings: CurioActivityBindings) -> ActivityCondition {
    ActivityCondition::All(
        vec![
            owned(bindings.inventory, 1),
            ActivityCondition::LessThan(
                integer(0),
                ActivityExpression::CounterValue {
                    slot: bindings.charge_slot,
                    key: 1,
                },
            ),
        ]
        .into_boxed_slice(),
    )
}

pub(crate) fn dimension_reward_settlement(
    bindings: CurioActivityBindings,
    ordinary_finish: Vec<ActivityOperation>,
) -> Vec<ActivityOperation> {
    let pending = ActivityCondition::Equal(
        ActivityExpression::CounterValue {
            slot: bindings.event_slot,
            key: DIMENSION_REWARD_PENDING_KEY,
        },
        integer(1),
    );
    let more_than_one_charge = ActivityCondition::LessThan(
        integer(1),
        ActivityExpression::CounterValue {
            slot: bindings.charge_slot,
            key: 1,
        },
    );
    let mut consume = vec![ActivityOperation::AddCounter {
        slot: bindings.event_slot,
        key: DIMENSION_REWARD_PENDING_KEY,
        delta: integer(-1),
    }];
    consume.push(ActivityOperation::Conditional {
        condition: more_than_one_charge,
        if_true: with_finish(
            vec![ActivityOperation::AddCounter {
                slot: bindings.charge_slot,
                key: 1,
                delta: integer(-1),
            }],
            &ordinary_finish,
        ),
        if_false: with_finish(destroy_dimension(bindings), &ordinary_finish),
    });
    vec![ActivityOperation::Conditional {
        condition: pending,
        if_true: consume.into_boxed_slice(),
        if_false: vec![ActivityOperation::Conditional {
            condition: dimension_reward_condition(bindings),
            if_true: vec![ActivityOperation::AddCounter {
                slot: bindings.event_slot,
                key: DIMENSION_REWARD_PENDING_KEY,
                delta: integer(1),
            }]
            .into_boxed_slice(),
            if_false: ordinary_finish.into_boxed_slice(),
        }]
        .into_boxed_slice(),
    }]
}

fn destroy_dimension(bindings: CurioActivityBindings) -> Vec<ActivityOperation> {
    let id = CurioId::new(1).expect("Dimension Reduction Dice ID is non-zero");
    let mut operations = teardown_operations(id, bindings);
    operations.push(ActivityOperation::AddCounter {
        slot: bindings.event_slot,
        key: DESTROYED_CURIO_COUNT_KEY,
        delta: integer(1),
    });
    operations
}

fn with_finish(
    mut prefix: Vec<ActivityOperation>,
    finish: &[ActivityOperation],
) -> Box<[ActivityOperation]> {
    prefix.extend_from_slice(finish);
    prefix.into_boxed_slice()
}

fn owned(inventory: ActivityInventoryId, content: u64) -> ActivityCondition {
    ActivityCondition::Equal(
        ActivityExpression::InventoryCount { inventory, content },
        integer(1),
    )
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
    bindings: CurioActivityBindings,
) -> Result<CurioActivityProjection, CurioActivityProjectionError> {
    let source_event = event_key(id, event);
    let mut operations = vec![
        ActivityOperation::Require(ActivityCondition::Not(Box::new(
            ActivityCondition::LessThan(counter(bindings.event_slot, source_event), integer(1)),
        ))),
        ActivityOperation::AddCounter {
            slot: bindings.event_slot,
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
                operations.push(fragment_gain(bindings, integer(i64::from(*amount))));
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::GrantFragmentsPerFullHpAlly {
                amount_per_ally,
                allies,
            } => {
                let amount = amount_per_ally
                    .checked_mul(u32::from(*allies))
                    .ok_or(CurioActivityProjectionError::Arithmetic)?;
                operations.push(if event == CurioEvent::BattleWon {
                    battle_fragment_gain(bindings, integer(i64::from(amount)))
                } else {
                    fragment_gain(bindings, integer(i64::from(amount)))
                });
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::GrantFragmentsFromCurrent { ratio } => {
                let amount = ratio_amount(cosmic_fragments, ratio.raw_six_decimal())?;
                operations.push(fragment_gain(bindings, integer(i64::from(amount))));
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::LoseCosmicFragmentsRatio { ratio } => {
                let amount = ratio_amount(cosmic_fragments, ratio.raw_six_decimal())?;
                debit_fragments(&mut operations, bindings.fragments_slot, amount);
                immediate_effects = immediate_effects.saturating_add(1);
            }
            CurioEffect::LoseFragmentsAndAddCriticalDamage { fragments, .. } => {
                debit_fragments(&mut operations, bindings.fragments_slot, *fragments);
                immediate_effects = immediate_effects.saturating_add(1);
                deferred = true;
            }
            _ => deferred = true,
        }
        if deferred {
            let index =
                u64::try_from(index).map_err(|_| CurioActivityProjectionError::Arithmetic)?;
            operations.push(ActivityOperation::AddCounter {
                slot: bindings.event_slot,
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
        ActivityGraphDefinition, ActivityInventoryDefinition, ActivityInventoryId,
        ActivityNodeDefinition, ActivityNodeKind, ActivityProgramDefinition, ActivityProgramId,
        ActivityScope, ActivitySlotDefinition, ActivityStateDefinition, ActivityStateSource,
        ActivityStateVisibility, ActivityTerminalOutcome, ActivityTransactionOutcome,
        ActivityTransactionState, NodeId, SectionId, SlotCarryPolicy, SlotResetPoint,
    };

    use super::*;

    #[test]
    fn silver_coin_and_gossip_use_the_same_checked_fragment_gain_pipeline() {
        assert_eq!(acquire_silver(false), 150);
        assert_eq!(acquire_silver(true), 200);
    }

    #[test]
    fn black_hole_trap_counts_full_hp_allies_and_uses_the_gossip_multiplier() {
        assert_eq!(settle_black_hole(false, false), 140);
        assert_eq!(settle_black_hole(true, false), 180);
        assert_eq!(settle_black_hole(false, true), 170);
        assert_eq!(settle_black_hole(true, true), 240);
    }

    #[test]
    fn cavity_acquisition_spends_all_fragments_and_captures_complete_hundreds() {
        let (definition, graph, bindings) = activity_fixture(250);
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(15).unwrap(),
            acquisition_operations(
                CurioActivityRecord::new(
                    CurioId::new(11).unwrap(),
                    CurioStateId::new(1).unwrap(),
                    0,
                    None,
                )
                .with_fragment_stack_capture(100),
                bindings,
            ),
        )
        .unwrap();
        program.validate_against(&definition, &graph).unwrap();
        let mut state = ActivityTransactionState::new(definition, graph.entry());
        apply(&mut state, &program, &graph);
        assert_eq!(
            state.slot(bindings.fragments_slot),
            Some(&ActivityValue::BoundedInteger(0))
        );
        assert!(matches!(
            state.slot(bindings.event_slot),
            Some(ActivityValue::BoundedCounterMap(values))
                if values.iter().any(|(key, count)| {
                    *key == CAVITY_CRITICAL_STACK_KEY && *count == 2
                })
        ));
    }

    #[test]
    fn cogwheel_domain_entry_is_atomic_on_both_threshold_sides() {
        assert_eq!(settle_cogwheel_domain(400), (450, true, 0));
        assert_eq!(settle_cogwheel_domain(460), (0, false, 1));
    }

    #[test]
    fn gold_coin_domain_entry_credits_six_percent_through_fragment_pipeline() {
        assert_eq!(settle_gold_coin_domain(250, false), 265);
        assert_eq!(settle_gold_coin_domain(250, true), 280);
    }

    #[test]
    fn dimension_dice_grants_two_extra_choices_then_destroys_itself() {
        let fragments = ActivitySlotId::new(1).unwrap();
        let states = ActivitySlotId::new(2).unwrap();
        let charges = ActivitySlotId::new(3).unwrap();
        let events = ActivitySlotId::new(4).unwrap();
        let inventory = ActivityInventoryId::new(1).unwrap();
        let definition = test_state_definition(fragments, states, charges, events, inventory);
        let graph = graph();
        let bindings = CurioActivityBindings {
            inventory,
            state_slot: states,
            charge_slot: charges,
            event_slot: events,
            fragments_slot: fragments,
        };
        let setup = ActivityProgramDefinition::new(
            ActivityProgramId::new(10).unwrap(),
            acquisition_operations(
                CurioActivityRecord::new(
                    CurioId::new(1).unwrap(),
                    CurioStateId::new(1).unwrap(),
                    2,
                    None,
                ),
                bindings,
            ),
        )
        .unwrap();
        let ordinary_finish = vec![ActivityOperation::AddCounter {
            slot: events,
            key: 42,
            delta: integer(1),
        }];
        let reward = ActivityProgramDefinition::new(
            ActivityProgramId::new(11).unwrap(),
            dimension_reward_settlement(bindings, ordinary_finish),
        )
        .unwrap();
        setup.validate_against(&definition, &graph).unwrap();
        reward.validate_against(&definition, &graph).unwrap();
        let mut state = ActivityTransactionState::new(definition.clone(), graph.entry());
        apply(&mut state, &setup, &graph);
        for _ in 0..4 {
            apply(&mut state, &reward, &graph);
        }
        let destroyed = ActivityProgramDefinition::new(
            ActivityProgramId::new(13).unwrap(),
            vec![ActivityOperation::Require(ActivityCondition::LessThan(
                ActivityExpression::InventoryCount {
                    inventory,
                    content: 1,
                },
                integer(1),
            ))],
        )
        .unwrap();
        destroyed.validate_against(&definition, &graph).unwrap();
        apply(&mut state, &destroyed, &graph);
        assert!(matches!(
            state.slot(events),
            Some(ActivityValue::BoundedCounterMap(values))
                if values.iter().any(|(key, count)| *key == 42 && *count == 2)
                    && values.iter().any(|(key, count)| {
                        *key == DESTROYED_CURIO_COUNT_KEY && *count == 1
                    })
        ));
    }

    #[test]
    fn curio_projection_commits_immediate_effects_and_records_deferred_effects_atomically() {
        let fragments = ActivitySlotId::new(1).unwrap();
        let events = ActivitySlotId::new(2).unwrap();
        let curio = CurioId::new(42).unwrap();
        let inventory = ActivityInventoryId::new(1).unwrap();
        let event = CurioEvent::Acquired;
        let definition = ActivityStateDefinition::new(
            vec![
                integer_slot(fragments, 100, 0, i64::from(u32::MAX), 1),
                counter_slot(events, event_key(curio, event), 2),
            ],
            vec![
                ActivityInventoryDefinition::new(
                    inventory,
                    ActivityScope::Activity,
                    64,
                    1,
                    SlotCarryPolicy::CarryExact,
                    ActivityStateVisibility::Private,
                    ActivityStateSource::new(3).unwrap(),
                )
                .unwrap(),
            ],
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
        let projection = lower_effects(
            curio,
            event,
            &effects,
            100,
            CurioActivityBindings {
                inventory,
                state_slot: events,
                charge_slot: events,
                event_slot: events,
                fragments_slot: fragments,
            },
        )
        .unwrap();
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

    fn acquire_silver(with_gossip: bool) -> i64 {
        let fragments = ActivitySlotId::new(1).unwrap();
        let states = ActivitySlotId::new(2).unwrap();
        let charges = ActivitySlotId::new(3).unwrap();
        let events = ActivitySlotId::new(4).unwrap();
        let inventory = ActivityInventoryId::new(1).unwrap();
        let definition = test_state_definition(fragments, states, charges, events, inventory);
        let graph = graph();
        let bindings = CurioActivityBindings {
            inventory,
            state_slot: states,
            charge_slot: charges,
            event_slot: events,
            fragments_slot: fragments,
        };
        let mut operations = Vec::new();
        if with_gossip {
            operations.push(ActivityOperation::AddInventory {
                inventory,
                content: GOSSIP_CURIO_CONTENT,
                count: integer(1),
            });
        }
        operations.extend(acquisition_operations(
            CurioActivityRecord::new(
                CurioId::new(2).unwrap(),
                CurioStateId::new(1).unwrap(),
                0,
                Some(2),
            ),
            bindings,
        ));
        let program =
            ActivityProgramDefinition::new(ActivityProgramId::new(12).unwrap(), operations)
                .unwrap();
        program.validate_against(&definition, &graph).unwrap();
        let mut state = ActivityTransactionState::new(definition, graph.entry());
        apply(&mut state, &program, &graph);
        match state.slot(fragments) {
            Some(ActivityValue::BoundedInteger(value)) => *value,
            _ => panic!("fragment slot"),
        }
    }

    fn settle_black_hole(with_gossip: bool, with_society: bool) -> i64 {
        let fragments = ActivitySlotId::new(1).unwrap();
        let events = ActivitySlotId::new(2).unwrap();
        let inventory = ActivityInventoryId::new(1).unwrap();
        let curio = CurioId::new(4).unwrap();
        let event = CurioEvent::BattleWon;
        let definition = ActivityStateDefinition::new(
            vec![
                integer_slot(fragments, 100, 0, i64::from(u32::MAX), 1),
                counter_slot(events, event_key(curio, event), 2),
            ],
            vec![
                ActivityInventoryDefinition::new(
                    inventory,
                    ActivityScope::Activity,
                    64,
                    1,
                    SlotCarryPolicy::CarryExact,
                    ActivityStateVisibility::Private,
                    ActivityStateSource::new(3).unwrap(),
                )
                .unwrap(),
            ],
            vec![],
        )
        .unwrap();
        let graph = graph();
        let bindings = CurioActivityBindings {
            inventory,
            state_slot: events,
            charge_slot: events,
            event_slot: events,
            fragments_slot: fragments,
        };
        let effects = [AppliedCurioEffect::new(
            "black-hole",
            CurioEffect::GrantFragmentsPerFullHpAlly {
                amount_per_ally: 10,
                allies: 4,
            },
        )];
        let projection = lower_effects(curio, event, &effects, 100, bindings).unwrap();
        let mut operations = Vec::new();
        if with_gossip {
            operations.push(ActivityOperation::AddInventory {
                inventory,
                content: GOSSIP_CURIO_CONTENT,
                count: integer(1),
            });
        }
        if with_society {
            operations.push(ActivityOperation::AddInventory {
                inventory,
                content: SOCIETY_TICKET_CURIO_CONTENT,
                count: integer(1),
            });
        }
        operations.extend_from_slice(projection.operations());
        let program =
            ActivityProgramDefinition::new(ActivityProgramId::new(14).unwrap(), operations)
                .unwrap();
        program.validate_against(&definition, &graph).unwrap();
        let mut state = ActivityTransactionState::new(definition, graph.entry());
        apply(&mut state, &program, &graph);
        match state.slot(fragments) {
            Some(ActivityValue::BoundedInteger(value)) => *value,
            _ => panic!("fragment slot"),
        }
    }

    fn settle_cogwheel_domain(initial_fragments: i64) -> (i64, bool, i64) {
        let (definition, graph, bindings) = activity_fixture(initial_fragments);
        let mut operations = acquisition_operations(
            CurioActivityRecord::new(
                CurioId::new(10).unwrap(),
                CurioStateId::new(1).unwrap(),
                0,
                None,
            ),
            bindings,
        );
        operations.extend(domain::cogwheel_domain_entry_settlement(bindings, &[]));
        let program =
            ActivityProgramDefinition::new(ActivityProgramId::new(16).unwrap(), operations)
                .unwrap();
        program.validate_against(&definition, &graph).unwrap();
        let ownership_check = ActivityProgramDefinition::new(
            ActivityProgramId::new(17).unwrap(),
            vec![ActivityOperation::Require(domain::cogwheel_condition(
                bindings,
            ))],
        )
        .unwrap();
        ownership_check
            .validate_against(&definition, &graph)
            .unwrap();
        let mut state = ActivityTransactionState::new(definition, graph.entry());
        apply(&mut state, &program, &graph);
        let fragments = match state.slot(bindings.fragments_slot) {
            Some(ActivityValue::BoundedInteger(value)) => *value,
            _ => panic!("fragment slot"),
        };
        let owned = matches!(
            state.apply_program(
                &ownership_check,
                ActivityCause::new(
                    state.command_sequence().saturating_add(1),
                    ownership_check.id(),
                    graph.entry(),
                )
                .unwrap(),
                &graph,
            ),
            ActivityTransactionOutcome::Committed(_)
        );
        let destroyed = state
            .slot(bindings.event_slot)
            .and_then(destroyed_curio_count)
            .map(i64::from)
            .unwrap_or(0);
        (fragments, owned, destroyed)
    }

    fn settle_gold_coin_domain(initial_fragments: i64, with_gossip: bool) -> i64 {
        let (definition, graph, bindings) = activity_fixture(initial_fragments);
        let mut operations = acquisition_operations(
            CurioActivityRecord::new(
                CurioId::new(21).unwrap(),
                CurioStateId::new(1).unwrap(),
                0,
                None,
            ),
            bindings,
        );
        if with_gossip {
            operations.push(ActivityOperation::AddInventory {
                inventory: bindings.inventory,
                content: GOSSIP_CURIO_CONTENT,
                count: integer(1),
            });
        }
        operations.extend(domain::gold_coin_domain_entry_settlement(bindings, &[]));
        let program =
            ActivityProgramDefinition::new(ActivityProgramId::new(18).unwrap(), operations)
                .unwrap();
        program.validate_against(&definition, &graph).unwrap();
        let mut state = ActivityTransactionState::new(definition, graph.entry());
        apply(&mut state, &program, &graph);
        match state.slot(bindings.fragments_slot) {
            Some(ActivityValue::BoundedInteger(value)) => *value,
            _ => panic!("fragment slot"),
        }
    }

    fn activity_fixture(
        initial_fragments: i64,
    ) -> (
        ActivityStateDefinition,
        ActivityGraphDefinition,
        CurioActivityBindings,
    ) {
        let fragments = ActivitySlotId::new(1).unwrap();
        let states = ActivitySlotId::new(2).unwrap();
        let charges = ActivitySlotId::new(3).unwrap();
        let events = ActivitySlotId::new(4).unwrap();
        let inventory = ActivityInventoryId::new(1).unwrap();
        let definition = ActivityStateDefinition::new(
            vec![
                integer_slot(fragments, initial_fragments, 0, i64::from(u32::MAX), 1),
                empty_counter_slot(states, 2),
                empty_counter_slot(charges, 3),
                empty_counter_slot(events, 4),
            ],
            vec![
                ActivityInventoryDefinition::new(
                    inventory,
                    ActivityScope::Activity,
                    64,
                    1,
                    SlotCarryPolicy::CarryExact,
                    ActivityStateVisibility::Private,
                    ActivityStateSource::new(5).unwrap(),
                )
                .unwrap(),
            ],
            vec![],
        )
        .unwrap();
        (
            definition,
            graph(),
            CurioActivityBindings {
                inventory,
                state_slot: states,
                charge_slot: charges,
                event_slot: events,
                fragments_slot: fragments,
            },
        )
    }

    fn apply(
        state: &mut ActivityTransactionState,
        program: &ActivityProgramDefinition,
        graph: &ActivityGraphDefinition,
    ) {
        let outcome = state.apply_program(
            program,
            ActivityCause::new(
                state.command_sequence().saturating_add(1),
                program.id(),
                graph.entry(),
            )
            .unwrap(),
            graph,
        );
        assert!(
            matches!(outcome, ActivityTransactionOutcome::Committed(_)),
            "{outcome:?}"
        );
    }

    fn test_state_definition(
        fragments: ActivitySlotId,
        states: ActivitySlotId,
        charges: ActivitySlotId,
        events: ActivitySlotId,
        inventory: ActivityInventoryId,
    ) -> ActivityStateDefinition {
        ActivityStateDefinition::new(
            vec![
                integer_slot(fragments, 100, 0, i64::from(u32::MAX), 1),
                empty_counter_slot(states, 2),
                empty_counter_slot(charges, 3),
                empty_counter_slot(events, 4),
            ],
            vec![
                ActivityInventoryDefinition::new(
                    inventory,
                    ActivityScope::Activity,
                    64,
                    1,
                    SlotCarryPolicy::CarryExact,
                    ActivityStateVisibility::Private,
                    ActivityStateSource::new(5).unwrap(),
                )
                .unwrap(),
            ],
            vec![],
        )
        .unwrap()
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

    fn empty_counter_slot(id: ActivitySlotId, source: u64) -> ActivitySlotDefinition {
        ActivitySlotDefinition::new_with_policy(
            id,
            ActivityScope::Activity,
            ActivityValue::BoundedCounterMap(Box::new([])),
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
