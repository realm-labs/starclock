//! Cognition lifecycle and Secret-frontier program compilation.

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    gold_gears_structural::AreaDefinition,
    gold_gears_unique::{GoldAndGearsUniqueCatalog, Secret},
};

use super::{
    GoldAndGearsEntryError,
    state_layout::{COGNITION_SLOT, PLANE_STATE_SLOT, SECRETS_SLOT},
};

/// Versioned executable resolution of the Goal 08 Cognition policy.
pub const GOLD_AND_GEARS_COGNITION_REVISION: &str = "gold-and-gears-cognition-policy-v1";

const COGNITION_ADJUSTMENT_PROGRAM: u32 = 0x4750_0001;
const COGNITION_CARRY_PROGRAM: u32 = 0x4750_0002;
const PLANE_BOSS_EVALUATION_BASE: u32 = 0x4750_0010;

const PLANE_COGNITION_EVALUATED_LAYER_KEY: u64 = 5;
const PLANE_SECRET_UNLOCKED_KEY: u64 = 6;

const GLOBAL_MINIMUM_CONSTANT: &str = "RogueNous_NousValueLimit_Min";
const GLOBAL_MAXIMUM_CONSTANT: &str = "RogueNous_NousValueLimit_Max";
const LIFECYCLE_POLICY: &str = "cognition-lifecycle-v1";

#[derive(Debug)]
pub(super) struct CognitionRuntimeCatalog {
    ranges: Box<[RuntimeCognitionRange]>,
    secrets: Box<[RuntimeSecret]>,
    global_minimum: i64,
    global_maximum: i64,
    initial: i64,
}

#[derive(Debug)]
struct RuntimeCognitionRange {
    area_key: Box<str>,
    area_source: u32,
    minimum: i64,
    maximum: i64,
}

#[derive(Debug)]
struct RuntimeSecret {
    id: u64,
    order_id: u32,
    key: Box<str>,
    required_area: u32,
    plane_layer: u8,
    minimum: i64,
    maximum: i64,
    predecessors: Box<[u64]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LifecyclePolicy {
    policy_id: String,
    evidence_quality: String,
    initial_value: String,
    adjustment_order: Vec<String>,
    plane_end_evaluation: String,
    eligibility_order: Vec<String>,
    multiple_match_order: Vec<String>,
    no_match_result: String,
    next_plane_carry: String,
    new_run_reset: String,
    replacement_condition: String,
}

impl CognitionRuntimeCatalog {
    pub(super) fn compile(
        unique: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let global_minimum = constant_integer(unique, GLOBAL_MINIMUM_CONSTANT)?;
        let global_maximum = constant_integer(unique, GLOBAL_MAXIMUM_CONSTANT)?;
        if global_minimum != -40 || global_maximum != 40 {
            return Err(GoldAndGearsEntryError::InvalidCognitionRuntime);
        }

        let mut ranges = Vec::with_capacity(unique.cognition_ranges.len());
        let mut initial = None;
        for range in &unique.cognition_ranges {
            let policy: LifecyclePolicy = serde_json::from_str(&range.lifecycle_json)
                .map_err(|_| GoldAndGearsEntryError::InvalidCognitionRuntime)?;
            validate_lifecycle(&policy)?;
            let range_initial = parse(&policy.initial_value)?;
            if initial
                .replace(range_initial)
                .is_some_and(|value| value != range_initial)
            {
                return Err(GoldAndGearsEntryError::InvalidCognitionRuntime);
            }
            let minimum = parse(&range.minimum.0)?;
            let maximum = parse(&range.maximum.0)?;
            if !range.inclusive
                || parse(&range.global_minimum.0)? != global_minimum
                || parse(&range.global_maximum.0)? != global_maximum
                || minimum < global_minimum
                || maximum > global_maximum
            {
                return Err(GoldAndGearsEntryError::InvalidCognitionRuntime);
            }
            ranges.push(RuntimeCognitionRange {
                area_key: range.area_key.clone(),
                area_source: parse_u32(&range.identity.source_id)?,
                minimum,
                maximum,
            });
        }
        ranges.sort_by_key(|range| range.area_source);
        if ranges.len() != 13
            || ranges
                .windows(2)
                .any(|pair| pair[0].area_source == pair[1].area_source)
        {
            return Err(GoldAndGearsEntryError::InvalidCognitionRuntime);
        }

        let mut secrets = unique
            .secrets
            .iter()
            .map(|secret| runtime_secret(secret, unique, global_minimum, global_maximum))
            .collect::<Result<Vec<_>, _>>()?;
        secrets.sort_by_key(|secret| (secret.minimum, secret.maximum, secret.order_id));
        if secrets.len() != 20 {
            return Err(GoldAndGearsEntryError::InvalidCognitionRuntime);
        }

        Ok(Self {
            ranges: ranges.into_boxed_slice(),
            secrets: secrets.into_boxed_slice(),
            global_minimum,
            global_maximum,
            initial: initial.ok_or(GoldAndGearsEntryError::InvalidCognitionRuntime)?,
        })
    }

    pub(super) fn bounds(
        &self,
        area: &AreaDefinition,
    ) -> Result<(i64, i64), GoldAndGearsEntryError> {
        let range = self.range(&area.stable_key)?;
        Ok((range.minimum, range.maximum))
    }

    pub(super) fn compile_adjustment(
        &self,
        area: &str,
        delta: i64,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let range = self.range(area)?;
        if range.minimum.checked_add(delta).is_none() || range.maximum.checked_add(delta).is_none()
        {
            return Err(GoldAndGearsEntryError::InvalidCognitionDelta);
        }
        let adjusted = ActivityExpression::Add(Box::new(cognition()), Box::new(integer(delta)));
        self.clamp_program(COGNITION_ADJUSTMENT_PROGRAM, range, adjusted)
    }

    pub(super) fn compile_carry(
        &self,
        area: &str,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let range = self.range(area)?;
        self.clamp_program(COGNITION_CARRY_PROGRAM, range, cognition())
    }

    pub(super) fn compile_plane_boss_evaluation(
        &self,
        area: &str,
        plane_layer: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        program(
            PLANE_BOSS_EVALUATION_BASE + u32::from(plane_layer),
            self.evaluation_operations(area, plane_layer, Vec::new())?,
        )
    }

    pub(super) fn evaluation_operations(
        &self,
        area: &str,
        plane_layer: u8,
        suffix: Vec<ActivityOperation>,
    ) -> Result<Vec<ActivityOperation>, GoldAndGearsEntryError> {
        if !(1..=3).contains(&plane_layer) {
            return Err(GoldAndGearsEntryError::InvalidPlaneLayer);
        }
        let range = self.range(area)?;
        let mut branch = suffix.clone();
        for secret in self
            .secrets
            .iter()
            .filter(|secret| {
                secret.required_area <= range.area_source && secret.plane_layer == plane_layer
            })
            .rev()
        {
            let mut unlock = vec![
                ActivityOperation::InsertOrderedId {
                    slot: slot(SECRETS_SLOT),
                    id: secret.id,
                },
                set_counter(
                    PLANE_STATE_SLOT,
                    PLANE_SECRET_UNLOCKED_KEY,
                    i64::try_from(secret.id)
                        .map_err(|_| GoldAndGearsEntryError::InvalidCognitionRuntime)?,
                ),
            ];
            unlock.extend(suffix.clone());
            branch = vec![ActivityOperation::Conditional {
                condition: eligibility(secret),
                if_true: unlock.into_boxed_slice(),
                if_false: branch.into_boxed_slice(),
            }];
        }
        let mut operations = vec![set_counter(
            PLANE_STATE_SLOT,
            PLANE_COGNITION_EVALUATED_LAYER_KEY,
            i64::from(plane_layer),
        )];
        operations.extend(branch);
        Ok(operations)
    }

    pub(super) fn frontier(
        &self,
        area: &str,
        state: &ActivityTransactionState,
        plane_layer: u8,
    ) -> Result<Box<[Box<str>]>, GoldAndGearsEntryError> {
        if !(1..=3).contains(&plane_layer) {
            return Err(GoldAndGearsEntryError::InvalidPlaneLayer);
        }
        let range = self.range(area)?;
        let cognition = match state.slot(slot(COGNITION_SLOT)) {
            Some(ActivityValue::BoundedInteger(value)) => *value,
            _ => return Err(GoldAndGearsEntryError::InvalidCognitionState),
        };
        let unlocked = match state.slot(slot(SECRETS_SLOT)) {
            Some(ActivityValue::OrderedIdSet(values)) => values,
            _ => return Err(GoldAndGearsEntryError::InvalidCognitionState),
        };
        Ok(self
            .secrets
            .iter()
            .filter(|secret| {
                secret.required_area <= range.area_source
                    && secret.plane_layer == plane_layer
                    && cognition >= secret.minimum
                    && cognition <= secret.maximum
                    && unlocked.binary_search(&secret.id).is_err()
                    && secret
                        .predecessors
                        .iter()
                        .all(|predecessor| unlocked.binary_search(predecessor).is_ok())
            })
            .map(|secret| secret.key.clone())
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (
            self.ranges.len(),
            self.secrets.len(),
            self.secrets
                .iter()
                .filter(|secret| secret.plane_layer == 3)
                .count(),
        )
    }

    pub(super) const fn initial(&self) -> i64 {
        self.initial
    }

    fn clamp_program(
        &self,
        id: u32,
        range: &RuntimeCognitionRange,
        value: ActivityExpression,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let global = ActivityExpression::Minimum(
            Box::new(ActivityExpression::Maximum(
                Box::new(value),
                Box::new(integer(self.global_minimum)),
            )),
            Box::new(integer(self.global_maximum)),
        );
        let area = ActivityExpression::Minimum(
            Box::new(ActivityExpression::Maximum(
                Box::new(global),
                Box::new(integer(range.minimum)),
            )),
            Box::new(integer(range.maximum)),
        );
        program(
            id,
            vec![ActivityOperation::SetSlot {
                slot: slot(COGNITION_SLOT),
                value: area,
            }],
        )
    }

    fn range(&self, area: &str) -> Result<&RuntimeCognitionRange, GoldAndGearsEntryError> {
        self.ranges
            .iter()
            .find(|range| range.area_key.as_ref() == area)
            .ok_or(GoldAndGearsEntryError::MissingCognitionRange)
    }
}

fn runtime_secret(
    secret: &Secret,
    unique: &GoldAndGearsUniqueCatalog,
    global_minimum: i64,
    global_maximum: i64,
) -> Result<RuntimeSecret, GoldAndGearsEntryError> {
    let minimum = parse(&secret.cognition_minimum.0)?;
    let maximum = parse(&secret.cognition_maximum.0)?;
    if !secret.inclusive
        || secret.evaluation_boundary.as_ref() != "AfterCurrentPlaneBossDefeat"
        || secret.lifecycle_policy.as_ref() != LIFECYCLE_POLICY
        || minimum < global_minimum
        || maximum > global_maximum
        || secret.terminal != (secret.plane_layer == 3)
    {
        return Err(GoldAndGearsEntryError::InvalidCognitionRuntime);
    }
    let predecessors = secret
        .predecessors
        .iter()
        .map(|key| {
            unique
                .secrets
                .iter()
                .find(|candidate| candidate.identity.stable_key == *key)
                .filter(|candidate| candidate.plane_layer + 1 == secret.plane_layer)
                .map(|candidate| u64::from(candidate.identity.id.0))
                .ok_or(GoldAndGearsEntryError::InvalidCognitionRuntime)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    Ok(RuntimeSecret {
        id: u64::from(secret.identity.id.0),
        order_id: parse_u32(&secret.identity.source_id)?,
        key: secret.identity.stable_key.clone(),
        required_area: parse_u32(&secret.area_source)?,
        plane_layer: secret.plane_layer,
        minimum,
        maximum,
        predecessors,
    })
}

fn eligibility(secret: &RuntimeSecret) -> ActivityCondition {
    let mut conditions = vec![
        ActivityCondition::Not(Box::new(ActivityCondition::OrderedIdSetContains {
            slot: slot(SECRETS_SLOT),
            id: secret.id,
        })),
        ActivityCondition::Not(Box::new(ActivityCondition::LessThan(
            cognition(),
            integer(secret.minimum),
        ))),
        ActivityCondition::Not(Box::new(ActivityCondition::LessThan(
            integer(secret.maximum),
            cognition(),
        ))),
    ];
    conditions.extend(secret.predecessors.iter().map(|predecessor| {
        ActivityCondition::OrderedIdSetContains {
            slot: slot(SECRETS_SLOT),
            id: *predecessor,
        }
    }));
    ActivityCondition::All(conditions.into_boxed_slice())
}

fn validate_lifecycle(policy: &LifecyclePolicy) -> Result<(), GoldAndGearsEntryError> {
    let expected = policy.policy_id == LIFECYCLE_POLICY
        && policy.evidence_quality == "ProjectPolicy"
        && policy.adjustment_order.iter().map(String::as_str).eq([
            "apply-cognition-delta",
            "clamp-to-global-range",
            "clamp-to-selected-area-range",
        ])
        && policy.plane_end_evaluation == "after-current-plane-boss-defeat"
        && policy.eligibility_order.iter().map(String::as_str).eq([
            "required-area-at-or-below-selected-area",
            "current-plane-layer",
            "predecessor-secret-frontier",
            "inclusive-cognition-range",
        ])
        && policy.multiple_match_order.iter().map(String::as_str).eq([
            "minimum-cognition",
            "maximum-cognition",
            "secret-id",
        ])
        && policy.no_match_result == "no-secret-unlocked"
        && policy.next_plane_carry == "carry-post-evaluation-value-then-clamp-to-next-area-range"
        && policy.new_run_reset == "reset-to-initial-value"
        && !policy.replacement_condition.is_empty();
    expected
        .then_some(())
        .ok_or(GoldAndGearsEntryError::InvalidCognitionRuntime)
}

fn constant_integer(
    unique: &GoldAndGearsUniqueCatalog,
    source: &str,
) -> Result<i64, GoldAndGearsEntryError> {
    let constant = unique
        .constants
        .iter()
        .find(|constant| constant.identity.source_id.as_ref() == source)
        .filter(|constant| {
            constant.mechanical_role.as_ref() == "Mechanic"
                && constant.value_kind.as_ref() == "Integer"
                && constant.values.len() == 1
        })
        .ok_or(GoldAndGearsEntryError::InvalidCognitionRuntime)?;
    parse(&constant.values[0])
}

fn set_counter(slot_id: u32, key: u64, desired: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: ActivityExpression::Subtract(
            Box::new(integer(desired)),
            Box::new(ActivityExpression::CounterValue {
                slot: slot(slot_id),
                key,
            }),
        ),
    }
}

fn cognition() -> ActivityExpression {
    ActivityExpression::Slot(slot(COGNITION_SLOT))
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Gold and Gears slot is non-zero")
}

fn program(
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(raw).expect("static Gold and Gears program ID is non-zero"),
        operations,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidCognitionRuntime)
}

fn parse(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    value
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidCognitionRuntime)
}

fn parse_u32(value: &str) -> Result<u32, GoldAndGearsEntryError> {
    value
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidCognitionRuntime)
}
