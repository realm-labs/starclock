//! Countdown, Planar Disarray and selected boss-decay Activity programs.

use std::collections::BTreeSet;

use serde_json::Value;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::{
        SwarmBossDecayRuntimeInput, SwarmCountdownRuntimeInput,
    },
};

use super::state::{COUNTDOWN, DISARRAY};

const MOVE_PROGRAM_ID: u32 = 0x5350_0001;
const ADJUSTMENT_PROGRAM_ID: u32 = 0x5350_0002;
const BOSS_DECAY_PROGRAM_ID: u32 = 0x5350_0003;
const COUNTDOWN_MINIMUM: i64 = -1_000_000;
const COUNTDOWN_MAXIMUM: i64 = 1_000_000;

const DISARRAY_LEVEL_KEY: u64 = 1;
const ENEMY_DAMAGE_DEALT_PERCENT_KEY: u64 = 2;
const ENEMY_DAMAGE_RECEIVED_REDUCTION_PERCENT_KEY: u64 = 3;
const ENEMY_SPEED_PERCENT_KEY: u64 = 4;
const BOSS_DECAY_KEY_BASE: u64 = 0x1000_0000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum BossDecayThreshold {
    PlaneOne,
    PlaneTwo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DisarrayTier {
    minimum: i64,
    maximum: i64,
    damage_dealt: i64,
    damage_received_reduction: i64,
    speed: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BossDecayContribution {
    id: u32,
    key: Box<str>,
    threshold: BossDecayThreshold,
    effect_program: Box<str>,
}

impl BossDecayContribution {
    pub(crate) fn key(&self) -> &str {
        &self.key
    }

    pub(crate) fn effect_program(&self) -> &str {
        &self.effect_program
    }
}

#[derive(Debug)]
pub(super) struct CountdownRuntimeCatalog {
    _initial: i64,
    warning: i64,
    movement_delta: i64,
    tiers: Box<[DisarrayTier]>,
    _source_constant_count: usize,
    boss_decay: Box<[BossDecayContribution]>,
    _disabled_boss_decay_count: usize,
}

impl CountdownRuntimeCatalog {
    pub(super) fn compile(
        input: SwarmCountdownRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let initial = integer(&input.initial)?;
        let warning = integer(&input.warning)?;
        let movement_delta = integer(&input.movement_delta)?;
        let tiers = tiers(&input.tiers)?;
        let source_constant_count = source_constant_count(&input.source_constants)?;
        let mut boss_decay = Vec::new();
        let mut disabled_boss_decay_count = 0;
        for row in input.boss_decay {
            if row.enabled {
                boss_decay.push(boss_decay_contribution(row)?);
            } else {
                disabled_boss_decay_count += 1;
            }
        }
        boss_decay.sort_unstable_by_key(|row| row.id);
        if initial != 20
            || warning != 5
            || movement_delta != -1
            || tiers.len() != 3
            || tiers[0].minimum != 1
            || tiers[0].maximum != 5
            || tiers[1].minimum != 6
            || tiers[1].maximum != 10
            || tiers[2].minimum != 11
            || tiers[2].maximum != 20
            || source_constant_count != 19
            || boss_decay.len() != 15
            || disabled_boss_decay_count != 27
            || boss_decay.windows(2).any(|pair| pair[0].id >= pair[1].id)
            || boss_decay
                .iter()
                .any(|row| row.key().is_empty() || row.effect_program().is_empty())
        {
            return Err(invalid("Swarm Countdown runtime denominator drift"));
        }
        Ok(Self {
            _initial: initial,
            warning,
            movement_delta,
            tiers,
            _source_constant_count: source_constant_count,
            boss_decay: boss_decay.into_boxed_slice(),
            _disabled_boss_decay_count: disabled_boss_decay_count,
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, i64, i64, i64) {
        (
            self._source_constant_count,
            self.boss_decay.len(),
            self._disabled_boss_decay_count,
            self._initial,
            self.warning,
            self.movement_delta,
        )
    }

    pub(super) fn compile_move(
        &self,
        state: &ActivityTransactionState,
        adjustments: &[(u32, i64)],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let current = countdown(state)?;
        let current_level = counter_value(state, DISARRAY_LEVEL_KEY)?;
        if current_level < 0 || (current < 0 && current_level == 0) {
            return Err(invalid("inconsistent Swarm Countdown and Disarray state"));
        }
        let adjustments = ordered_adjustments(adjustments, true)?;
        let mut next_countdown = checked_countdown(current, self.movement_delta)?;
        for (_, delta) in &adjustments {
            next_countdown = checked_countdown(next_countdown, *delta)?;
        }
        let next_level = if current_level > 0 || current == 0 {
            current_level
                .checked_add(1)
                .ok_or_else(|| invalid("Swarm Disarray level overflow"))?
        } else {
            current_level
        };
        let current_modifiers = self.modifiers(current_level)?;
        let next_modifiers = self.modifiers(next_level)?;
        let mut operations = state_requirements(current, current_level, current_modifiers);
        operations.push(add_countdown(self.movement_delta));
        operations.extend(
            adjustments
                .into_iter()
                .map(|(_, delta)| add_countdown(delta)),
        );
        if next_level != current_level {
            operations.extend(disarray_values(next_level, next_modifiers));
        }
        program(MOVE_PROGRAM_ID, operations)
    }

    pub(super) fn compile_adjustments(
        &self,
        state: &ActivityTransactionState,
        adjustments: &[(u32, i64)],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let current = countdown(state)?;
        let adjustments = ordered_adjustments(adjustments, false)?;
        let mut desired = current;
        for (_, delta) in &adjustments {
            desired = checked_countdown(desired, *delta)?;
        }
        let mut operations = vec![require_countdown(current)];
        operations.extend(
            adjustments
                .into_iter()
                .map(|(_, delta)| add_countdown(delta)),
        );
        program(ADJUSTMENT_PROGRAM_ID, operations)
    }

    pub(super) fn compile_boss_decay_selection(
        &self,
        state: &ActivityTransactionState,
        keys: &[&str],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        if keys.is_empty() {
            return Err(invalid("Swarm boss-decay selection is empty"));
        }
        let existing = self.selected_boss_decay(state)?;
        let mut thresholds = existing
            .iter()
            .map(|row| row.threshold)
            .collect::<BTreeSet<_>>();
        let mut selected = keys
            .iter()
            .map(|key| {
                self.boss_decay
                    .iter()
                    .find(|row| row.key.as_ref() == *key)
                    .ok_or_else(|| invalid("unknown or disabled Swarm boss-decay contribution"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        selected.sort_unstable_by_key(|row| row.id);
        if selected.windows(2).any(|pair| pair[0].id == pair[1].id)
            || selected.iter().any(|row| !thresholds.insert(row.threshold))
        {
            return Err(invalid("duplicate Swarm boss-decay threshold"));
        }
        let mut operations = Vec::with_capacity(selected.len() * 2);
        for row in selected {
            let key = boss_decay_key(row.id)?;
            operations.push(ActivityOperation::Require(ActivityCondition::Equal(
                counter(DISARRAY, key),
                literal(0),
            )));
            operations.push(set_counter(DISARRAY, key, 1));
        }
        program(BOSS_DECAY_PROGRAM_ID, operations)
    }

    pub(super) fn selected_boss_decay<'a>(
        &'a self,
        state: &ActivityTransactionState,
    ) -> Result<Box<[&'a BossDecayContribution]>, UniverseCatalogLoadError> {
        let mut selected = Vec::new();
        for row in &self.boss_decay {
            match counter_value(state, boss_decay_key(row.id)?)? {
                0 => {}
                1 => selected.push(row),
                _ => return Err(invalid("invalid Swarm boss-decay selection state")),
            }
        }
        if selected.len() > 2
            || selected
                .iter()
                .map(|row| row.threshold)
                .collect::<BTreeSet<_>>()
                .len()
                != selected.len()
        {
            return Err(invalid("invalid Swarm boss-decay contribution set"));
        }
        Ok(selected.into_boxed_slice())
    }

    pub(super) fn completion_requirements(
        &self,
        state: &ActivityTransactionState,
        plane_layer: u8,
    ) -> Result<Vec<ActivityCondition>, UniverseCatalogLoadError> {
        let required = match plane_layer {
            1 => &[BossDecayThreshold::PlaneOne][..],
            2 => &[BossDecayThreshold::PlaneTwo][..],
            3 => &[BossDecayThreshold::PlaneOne, BossDecayThreshold::PlaneTwo][..],
            _ => return Err(invalid("invalid Swarm plane layer")),
        };
        let selected = self.selected_boss_decay(state)?;
        required
            .iter()
            .map(|threshold| {
                let row = selected
                    .iter()
                    .find(|row| row.threshold == *threshold)
                    .ok_or_else(|| invalid("required Swarm boss-decay threshold is missing"))?;
                Ok(ActivityCondition::Equal(
                    counter(DISARRAY, boss_decay_key(row.id)?),
                    literal(1),
                ))
            })
            .collect()
    }

    pub(super) fn countdown(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        countdown(state)
    }

    pub(super) fn disarray_level(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        counter_value(state, DISARRAY_LEVEL_KEY)
    }

    pub(super) fn disarray_modifiers(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<(i64, i64, i64), UniverseCatalogLoadError> {
        Ok((
            counter_value(state, ENEMY_DAMAGE_DEALT_PERCENT_KEY)?,
            counter_value(state, ENEMY_DAMAGE_RECEIVED_REDUCTION_PERCENT_KEY)?,
            counter_value(state, ENEMY_SPEED_PERCENT_KEY)?,
        ))
    }

    pub(super) fn warning_active(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        Ok((0..=self.warning).contains(&countdown(state)?))
    }

    fn modifiers(&self, level: i64) -> Result<(i64, i64, i64), UniverseCatalogLoadError> {
        let effective = level.clamp(0, 20);
        let mut result = (0_i64, 0_i64, 0_i64);
        for tier in &self.tiers {
            let count = effective
                .min(tier.maximum)
                .checked_sub(tier.minimum)
                .and_then(|value| value.checked_add(1))
                .unwrap_or(0)
                .max(0);
            result.0 = checked_product_sum(result.0, count, tier.damage_dealt)?;
            result.1 = checked_product_sum(result.1, count, tier.damage_received_reduction)?;
            result.2 = checked_product_sum(result.2, count, tier.speed)?;
        }
        Ok(result)
    }
}

fn boss_decay_contribution(
    row: SwarmBossDecayRuntimeInput,
) -> Result<BossDecayContribution, UniverseCatalogLoadError> {
    let threshold = match row.threshold.as_ref() {
        "PlaneOneBossChoice" => BossDecayThreshold::PlaneOne,
        "PlaneTwoBossChoice" => BossDecayThreshold::PlaneTwo,
        _ => return Err(invalid("enabled Swarm boss-decay threshold is invalid")),
    };
    let parameters = serde_json::from_str::<Vec<Value>>(&row.effect_program)
        .map_err(|_| invalid("invalid Swarm boss-decay effect program"))?;
    if row.id == 0
        || row.key.is_empty()
        || parameters
            .iter()
            .any(|value| value.as_str().is_none_or(str::is_empty))
    {
        return Err(invalid("invalid Swarm boss-decay contribution"));
    }
    Ok(BossDecayContribution {
        id: row.id,
        key: row.key,
        threshold,
        effect_program: row.effect_program,
    })
}

fn tiers(value: &str) -> Result<Box<[DisarrayTier]>, UniverseCatalogLoadError> {
    serde_json::from_str::<Vec<Value>>(value)
        .map_err(|_| invalid("invalid Swarm Disarray tier program"))?
        .iter()
        .map(|value| {
            let value = value
                .as_object()
                .ok_or_else(|| invalid("invalid Swarm Disarray tier"))?;
            Ok(DisarrayTier {
                minimum: object_integer(value, "minimum_level")?,
                maximum: object_integer(value, "maximum_level")?,
                damage_dealt: object_integer(value, "enemy_damage_dealt_per_level_percent")?,
                damage_received_reduction: object_integer(
                    value,
                    "enemy_damage_received_reduction_per_level_percent",
                )?,
                speed: object_integer(value, "enemy_speed_per_level_percent")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn source_constant_count(value: &str) -> Result<usize, UniverseCatalogLoadError> {
    let rows = serde_json::from_str::<Vec<Value>>(value)
        .map_err(|_| invalid("invalid Swarm source-constant bindings"))?;
    let mut ids = BTreeSet::new();
    for row in &rows {
        let row = row
            .as_object()
            .ok_or_else(|| invalid("invalid Swarm source-constant binding"))?;
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid("invalid Swarm source-constant ID"))?;
        if !ids.insert(id)
            || !row.contains_key("value")
            || row
                .get("source_locator")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
        {
            return Err(invalid("invalid Swarm source-constant binding"));
        }
    }
    Ok(rows.len())
}

fn object_integer(
    value: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<i64, UniverseCatalogLoadError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("missing Swarm Disarray tier integer"))
        .and_then(integer)
}

fn ordered_adjustments(
    adjustments: &[(u32, i64)],
    allow_empty: bool,
) -> Result<Vec<(u32, i64)>, UniverseCatalogLoadError> {
    let mut result = adjustments.to_vec();
    result.sort_unstable_by_key(|(operation_id, _)| *operation_id);
    if (!allow_empty && result.is_empty())
        || result
            .iter()
            .any(|(operation_id, delta)| *operation_id == 0 || *delta == 0)
        || result.windows(2).any(|pair| pair[0].0 == pair[1].0)
    {
        return Err(invalid("invalid Swarm Countdown adjustment set"));
    }
    Ok(result)
}

fn state_requirements(
    countdown: i64,
    level: i64,
    modifiers: (i64, i64, i64),
) -> Vec<ActivityOperation> {
    vec![
        require_countdown(countdown),
        require_counter(DISARRAY_LEVEL_KEY, level),
        require_counter(ENEMY_DAMAGE_DEALT_PERCENT_KEY, modifiers.0),
        require_counter(ENEMY_DAMAGE_RECEIVED_REDUCTION_PERCENT_KEY, modifiers.1),
        require_counter(ENEMY_SPEED_PERCENT_KEY, modifiers.2),
    ]
}

fn disarray_values(level: i64, modifiers: (i64, i64, i64)) -> Vec<ActivityOperation> {
    vec![
        set_counter(DISARRAY, DISARRAY_LEVEL_KEY, level),
        set_counter(DISARRAY, ENEMY_DAMAGE_DEALT_PERCENT_KEY, modifiers.0),
        set_counter(
            DISARRAY,
            ENEMY_DAMAGE_RECEIVED_REDUCTION_PERCENT_KEY,
            modifiers.1,
        ),
        set_counter(DISARRAY, ENEMY_SPEED_PERCENT_KEY, modifiers.2),
    ]
}

fn require_countdown(value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::Slot(slot(COUNTDOWN)),
        literal(value),
    ))
}

fn require_counter(key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        counter(DISARRAY, key),
        literal(value),
    ))
}

fn add_countdown(delta: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot: slot(COUNTDOWN),
        delta: literal(delta),
    }
}

fn set_counter(slot_id: u32, key: u64, desired: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: ActivityExpression::Subtract(
            Box::new(literal(desired)),
            Box::new(counter(slot_id, key)),
        ),
    }
}

fn counter(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn literal(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn countdown(state: &ActivityTransactionState) -> Result<i64, UniverseCatalogLoadError> {
    match state.slot(slot(COUNTDOWN)) {
        Some(ActivityValue::BoundedInteger(value)) => Ok(*value),
        _ => Err(invalid("invalid Swarm Countdown state")),
    }
}

fn counter_value(
    state: &ActivityTransactionState,
    key: u64,
) -> Result<i64, UniverseCatalogLoadError> {
    match state.slot(slot(DISARRAY)) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1)),
        _ => Err(invalid("invalid Swarm Disarray state")),
    }
}

fn boss_decay_key(id: u32) -> Result<u64, UniverseCatalogLoadError> {
    BOSS_DECAY_KEY_BASE
        .checked_add(u64::from(id))
        .ok_or_else(|| invalid("Swarm boss-decay key overflow"))
}

fn checked_countdown(value: i64, delta: i64) -> Result<i64, UniverseCatalogLoadError> {
    value
        .checked_add(delta)
        .filter(|value| (COUNTDOWN_MINIMUM..=COUNTDOWN_MAXIMUM).contains(value))
        .ok_or_else(|| invalid("Swarm Countdown adjustment exceeds bounds"))
}

fn checked_product_sum(
    current: i64,
    count: i64,
    per_level: i64,
) -> Result<i64, UniverseCatalogLoadError> {
    count
        .checked_mul(per_level)
        .and_then(|value| current.checked_add(value))
        .ok_or_else(|| invalid("Swarm Disarray modifier overflow"))
}

fn integer(value: &str) -> Result<i64, UniverseCatalogLoadError> {
    value
        .parse::<i64>()
        .map_err(|_| invalid("invalid Swarm authoritative integer"))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Swarm slot is non-zero")
}

fn program(
    id: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(id).expect("static Swarm program ID is non-zero"),
        operations,
    )
    .map_err(|_| invalid("invalid Swarm Countdown program"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
