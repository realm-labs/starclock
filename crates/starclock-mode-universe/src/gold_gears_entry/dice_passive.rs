//! Typed Custom Dice passive triggers and selected-Path accumulation.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use super::{
    GoldAndGearsEntryError,
    dice_resolution::{CompiledDiceRuntime, DiceKind},
    state_layout::{
        DEFERRED_DICE_PASSIVE_BASE, DEFERRED_EFFECTS_SLOT, PROGRESSION_DICE_PATH_BOOST_STACKS_KEY,
        PROGRESSION_DICE_PATH_TRIGGER_PROGRESS_KEY, PROGRESSION_SLOT,
        RESOURCE_COSMIC_FRAGMENTS_KEY, RESOURCE_DICE_REROLLS_KEY, RUN_RESOURCES_SLOT,
    },
};

const PASSIVE_PROGRAM_BASE: u32 = 0x4770_0000;

/// Domain class needed by Custom Dice passive dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsDiceDomain {
    Other,
    Occurrence,
    AbnormalOccurrence,
    Reward,
    Elite,
    Boss,
    Transaction,
}

/// One already-validated Activity fact presented to the selected Custom Dice.
///
/// Counts are exact event multiplicities. Snapshot fields are current totals,
/// not deltas, which prevents repeated observation from duplicating Path
/// stacks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsDicePassiveEvent {
    TrottersDefeated {
        count: u32,
    },
    KnowledgeApplied {
        count: u32,
    },
    DomainEntered {
        plane_layer: u8,
        domain: GoldAndGearsDiceDomain,
        beacon_id: Option<u32>,
        has_knowledge: bool,
        non_adjacent: bool,
        knowledge_domain_count: u32,
    },
    OccurrenceInteractionsCompleted {
        count: u32,
    },
    BattleVictory {
        elite: bool,
    },
    CountdownSnapshot {
        remaining: u32,
    },
    KnowledgeDomainsCollapsed {
        count: u32,
        premium_domain: bool,
        had_beacon: bool,
    },
    StorePurchase {
        cosmic_fragments_spent: u32,
    },
    CuriosAcquired {
        count: u32,
        total_owned: u32,
    },
    MovementCompleted {
        count: u32,
    },
    GeneralBuffBattleVictory {
        faces_used: u32,
    },
}

pub(super) fn compile_passive(
    dice: &CompiledDiceRuntime,
    state: &ActivityTransactionState,
    event: GoldAndGearsDicePassiveEvent,
) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
    validate_event(event)?;
    let mut effect = PassiveEffect::default();
    match (dice.kind, event) {
        (DiceKind::Trotter, GoldAndGearsDicePassiveEvent::TrottersDefeated { count }) => {
            effect.fragments = scaled_count(count, 80)?;
            effect.path = PathChange::Accumulate(i64::from(count));
        }
        (DiceKind::Knowledge, GoldAndGearsDicePassiveEvent::KnowledgeApplied { count }) => {
            effect.defer(dice.dice_id, event_code(event), i64::from(count))?;
        }
        (
            DiceKind::Knowledge,
            GoldAndGearsDicePassiveEvent::DomainEntered {
                has_knowledge: true,
                ..
            },
        ) => effect.path = PathChange::Accumulate(1),
        (
            DiceKind::Beacon,
            GoldAndGearsDicePassiveEvent::DomainEntered {
                plane_layer,
                beacon_id: Some(beacon_id),
                ..
            },
        ) => {
            effect.path = PathChange::Accumulate(1);
            let marker = beacon_once_marker(dice.dice_id, plane_layer, beacon_id)?;
            if counter_value(state, DEFERRED_EFFECTS_SLOT, marker).unwrap_or(0) == 0 {
                effect.deferred_key = Some(marker);
                effect.deferred_delta = 1;
            }
        }
        (
            DiceKind::Occurrence,
            GoldAndGearsDicePassiveEvent::DomainEntered {
                domain:
                    GoldAndGearsDiceDomain::Occurrence
                    | GoldAndGearsDiceDomain::AbnormalOccurrence
                    | GoldAndGearsDiceDomain::Reward,
                ..
            },
        ) => {
            effect.defer(dice.dice_id, event_code(event), 1)?;
        }
        (
            DiceKind::Occurrence,
            GoldAndGearsDicePassiveEvent::OccurrenceInteractionsCompleted { count },
        ) => effect.path = PathChange::Accumulate(i64::from(count)),
        (DiceKind::Elite, GoldAndGearsDicePassiveEvent::BattleVictory { elite }) => {
            effect.path = PathChange::Accumulate(1);
            if elite {
                effect.defer(dice.dice_id, event_code(event), 1)?;
            }
        }
        (
            DiceKind::Domain,
            GoldAndGearsDicePassiveEvent::DomainEntered {
                non_adjacent: true, ..
            },
        ) => effect.path = PathChange::Accumulate(1),
        (DiceKind::Countdown, GoldAndGearsDicePassiveEvent::CountdownSnapshot { remaining }) => {
            effect.path = PathChange::Snapshot(i64::from(remaining))
        }
        (
            DiceKind::Countdown,
            GoldAndGearsDicePassiveEvent::DomainEntered {
                domain,
                has_knowledge,
                ..
            },
        ) if has_knowledge || domain == GoldAndGearsDiceDomain::Boss => {
            effect.defer(dice.dice_id, event_code(event), 1)?;
        }
        (
            DiceKind::KnowledgeProtection,
            GoldAndGearsDicePassiveEvent::DomainEntered {
                domain:
                    GoldAndGearsDiceDomain::Boss
                    | GoldAndGearsDiceDomain::AbnormalOccurrence
                    | GoldAndGearsDiceDomain::Transaction,
                knowledge_domain_count,
                ..
            },
        ) => {
            effect.fragments = scaled_count(knowledge_domain_count, 15)?;
            effect.path = PathChange::Accumulate(1);
        }
        (
            DiceKind::KnowledgeCollapse,
            GoldAndGearsDicePassiveEvent::KnowledgeDomainsCollapsed {
                count,
                premium_domain,
                had_beacon,
            },
        ) => {
            let multiplier = 1_i64
                .checked_shl(u32::from(premium_domain) + u32::from(had_beacon))
                .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?;
            effect.fragments = scaled_count(count, 50)?
                .checked_mul(multiplier)
                .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?;
            effect.path = PathChange::Accumulate(i64::from(count));
        }
        (
            DiceKind::Transaction,
            GoldAndGearsDicePassiveEvent::StorePurchase {
                cosmic_fragments_spent,
            },
        ) => {
            let spent = i64::from(cosmic_fragments_spent);
            effect.fragments = spent
                .checked_mul(300_000)
                .and_then(|value| value.checked_div(1_000_000))
                .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?;
            effect.path = PathChange::Accumulate(spent);
        }
        (DiceKind::Curio, GoldAndGearsDicePassiveEvent::CuriosAcquired { count, total_owned }) => {
            effect.fragments = scaled_count(count, 40)?;
            effect.path = PathChange::Snapshot(i64::from(total_owned));
        }
        (DiceKind::GeneralBuff, GoldAndGearsDicePassiveEvent::MovementCompleted { count }) => {
            effect.rerolls = i64::from(count)
        }
        (
            DiceKind::GeneralBuff,
            GoldAndGearsDicePassiveEvent::GeneralBuffBattleVictory { faces_used },
        ) => effect.path = PathChange::Accumulate(i64::from(faces_used)),
        _ => return Ok(None),
    }

    let operations = effect.operations(dice, state)?;
    let id = PASSIVE_PROGRAM_BASE
        .checked_add(
            dice.dice_id
                .checked_mul(32)
                .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?,
        )
        .and_then(|value| value.checked_add(event_code(event)))
        .and_then(ActivityProgramId::new)
        .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?;
    ActivityProgramDefinition::new(id, operations)
        .map(Some)
        .map_err(|_| GoldAndGearsEntryError::InvalidDicePassiveEvent)
}

pub(super) const fn allows_same_domain_movement(dice: &CompiledDiceRuntime) -> bool {
    matches!(dice.kind, DiceKind::Domain)
}

pub(super) const fn preserves_knowledge_domains(dice: &CompiledDiceRuntime) -> bool {
    matches!(dice.kind, DiceKind::KnowledgeProtection)
}

pub(super) const fn persists_general_buff_faces(dice: &CompiledDiceRuntime) -> bool {
    matches!(dice.kind, DiceKind::GeneralBuff)
}

pub(super) fn path_boost_stacks(state: &ActivityTransactionState) -> Option<u32> {
    counter_value(
        state,
        PROGRESSION_SLOT,
        PROGRESSION_DICE_PATH_BOOST_STACKS_KEY,
    )
    .and_then(|value| u32::try_from(value).ok())
}

#[derive(Clone, Copy, Debug, Default)]
enum PathChange {
    #[default]
    None,
    Accumulate(i64),
    Snapshot(i64),
}

#[derive(Clone, Copy, Debug, Default)]
struct PassiveEffect {
    fragments: i64,
    rerolls: i64,
    deferred_key: Option<u64>,
    deferred_delta: i64,
    path: PathChange,
}

impl PassiveEffect {
    fn defer(&mut self, dice_id: u32, code: u32, delta: i64) -> Result<(), GoldAndGearsEntryError> {
        self.deferred_key = DEFERRED_DICE_PASSIVE_BASE
            .checked_add(u64::from(dice_id) * 32)
            .and_then(|value| value.checked_add(u64::from(code)));
        self.deferred_delta = delta;
        self.deferred_key
            .map(|_| ())
            .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)
    }

    fn operations(
        self,
        dice: &CompiledDiceRuntime,
        state: &ActivityTransactionState,
    ) -> Result<Vec<ActivityOperation>, GoldAndGearsEntryError> {
        let mut operations = Vec::new();
        if self.fragments != 0 {
            operations.push(add_counter(
                RUN_RESOURCES_SLOT,
                RESOURCE_COSMIC_FRAGMENTS_KEY,
                self.fragments,
            ));
        }
        if self.rerolls != 0 {
            operations.push(add_counter(
                RUN_RESOURCES_SLOT,
                RESOURCE_DICE_REROLLS_KEY,
                self.rerolls,
            ));
        }
        if let Some(raw_key) = self.deferred_key {
            let current = counter_value(state, DEFERRED_EFFECTS_SLOT, raw_key).unwrap_or(0);
            operations.push(require_counter(DEFERRED_EFFECTS_SLOT, raw_key, current));
            operations.push(add_counter(
                DEFERRED_EFFECTS_SLOT,
                raw_key,
                self.deferred_delta,
            ));
        }
        append_path_operations(&mut operations, dice, state, self.path)?;
        Ok(operations)
    }
}

fn append_path_operations(
    operations: &mut Vec<ActivityOperation>,
    dice: &CompiledDiceRuntime,
    state: &ActivityTransactionState,
    change: PathChange,
) -> Result<(), GoldAndGearsEntryError> {
    let progress = counter_value(
        state,
        PROGRESSION_SLOT,
        PROGRESSION_DICE_PATH_TRIGGER_PROGRESS_KEY,
    )
    .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?;
    let stacks = counter_value(
        state,
        PROGRESSION_SLOT,
        PROGRESSION_DICE_PATH_BOOST_STACKS_KEY,
    )
    .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?;
    let (next_progress, next_stacks) = match change {
        PathChange::None => return Ok(()),
        PathChange::Accumulate(units) => {
            let total = progress
                .checked_add(units)
                .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?;
            (
                total % dice.path_trigger_interval,
                stacks
                    .checked_add(total / dice.path_trigger_interval)
                    .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)?,
            )
        }
        PathChange::Snapshot(units) => (0, units / dice.path_trigger_interval),
    };
    operations.extend([
        require_counter(
            PROGRESSION_SLOT,
            PROGRESSION_DICE_PATH_TRIGGER_PROGRESS_KEY,
            progress,
        ),
        require_counter(
            PROGRESSION_SLOT,
            PROGRESSION_DICE_PATH_BOOST_STACKS_KEY,
            stacks,
        ),
        add_counter(
            PROGRESSION_SLOT,
            PROGRESSION_DICE_PATH_TRIGGER_PROGRESS_KEY,
            next_progress - progress,
        ),
        add_counter(
            PROGRESSION_SLOT,
            PROGRESSION_DICE_PATH_BOOST_STACKS_KEY,
            next_stacks - stacks,
        ),
    ]);
    Ok(())
}

fn validate_event(event: GoldAndGearsDicePassiveEvent) -> Result<(), GoldAndGearsEntryError> {
    let valid = match event {
        GoldAndGearsDicePassiveEvent::TrottersDefeated { count }
        | GoldAndGearsDicePassiveEvent::KnowledgeApplied { count }
        | GoldAndGearsDicePassiveEvent::OccurrenceInteractionsCompleted { count }
        | GoldAndGearsDicePassiveEvent::MovementCompleted { count } => count > 0,
        GoldAndGearsDicePassiveEvent::DomainEntered {
            plane_layer,
            beacon_id,
            ..
        } => (1..=3).contains(&plane_layer) && beacon_id.is_none_or(|id| id != 0),
        GoldAndGearsDicePassiveEvent::BattleVictory { .. }
        | GoldAndGearsDicePassiveEvent::CountdownSnapshot { .. } => true,
        GoldAndGearsDicePassiveEvent::KnowledgeDomainsCollapsed { count, .. }
        | GoldAndGearsDicePassiveEvent::CuriosAcquired { count, .. } => count > 0,
        GoldAndGearsDicePassiveEvent::StorePurchase {
            cosmic_fragments_spent,
        } => cosmic_fragments_spent > 0,
        GoldAndGearsDicePassiveEvent::GeneralBuffBattleVictory { faces_used } => faces_used > 0,
    };
    if valid {
        Ok(())
    } else {
        Err(GoldAndGearsEntryError::InvalidDicePassiveEvent)
    }
}

const fn event_code(event: GoldAndGearsDicePassiveEvent) -> u32 {
    match event {
        GoldAndGearsDicePassiveEvent::TrottersDefeated { .. } => 1,
        GoldAndGearsDicePassiveEvent::KnowledgeApplied { .. } => 2,
        GoldAndGearsDicePassiveEvent::DomainEntered { .. } => 3,
        GoldAndGearsDicePassiveEvent::OccurrenceInteractionsCompleted { .. } => 4,
        GoldAndGearsDicePassiveEvent::BattleVictory { .. } => 5,
        GoldAndGearsDicePassiveEvent::CountdownSnapshot { .. } => 6,
        GoldAndGearsDicePassiveEvent::KnowledgeDomainsCollapsed { .. } => 7,
        GoldAndGearsDicePassiveEvent::StorePurchase { .. } => 8,
        GoldAndGearsDicePassiveEvent::CuriosAcquired { .. } => 9,
        GoldAndGearsDicePassiveEvent::MovementCompleted { .. } => 10,
        GoldAndGearsDicePassiveEvent::GeneralBuffBattleVictory { .. } => 11,
    }
}

fn beacon_once_marker(
    dice_id: u32,
    plane_layer: u8,
    beacon_id: u32,
) -> Result<u64, GoldAndGearsEntryError> {
    DEFERRED_DICE_PASSIVE_BASE
        .checked_add(0x0100_0000)
        .and_then(|value| value.checked_add(u64::from(dice_id) * 0x1000))
        .and_then(|value| value.checked_add(u64::from(plane_layer) * 0x1_0000_0000))
        .and_then(|value| value.checked_add(u64::from(beacon_id)))
        .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)
}

fn scaled_count(count: u32, value: i64) -> Result<i64, GoldAndGearsEntryError> {
    i64::from(count)
        .checked_mul(value)
        .ok_or(GoldAndGearsEntryError::InvalidDicePassiveEvent)
}

fn require_counter(slot_id: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        counter(slot_id, key),
        integer(value),
    ))
}

fn add_counter(slot_id: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: integer(delta),
    }
}

fn counter_value(state: &ActivityTransactionState, slot_id: u32, key: u64) -> Option<i64> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map(|index| values[index].1),
        _ => None,
    }
}

fn counter(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Gold and Gears slot is non-zero")
}
