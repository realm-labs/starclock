//! Explicit boss selection and atomic plane-completion programs.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityGraphDefinition, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivitySlotId, ActivityTerminalOutcome,
    ActivityTransactionState, ActivityValue, NodeId,
};

use crate::gold_gears_structural::GoldAndGearsStructuralCatalog;

use super::state_layout;
use super::{
    GoldAndGearsEntryError, cognition::CognitionRuntimeCatalog, state_layout::PLANE_STATE_SLOT,
};

const BOSS_SELECTION_PROGRAM_BASE: u32 = 0x4760_0000;
const PLANE_COMPLETION_PROGRAM_BASE: u32 = 0x4760_0010;

const PLANE_SELECTED_BOSS_KEY: u64 = 7;
const PLANE_COMPLETED_LAYER_KEY: u64 = 8;
const PLANE_SELECTED_BOSS_LAYER_KEY: u64 = 9;

#[derive(Debug)]
pub(super) struct PlaneTransitionRuntimeCatalog {
    bosses: Box<[RuntimeBossChoice]>,
}

#[derive(Debug)]
struct RuntimeBossChoice {
    key: Box<str>,
    source_id: u32,
}

impl PlaneTransitionRuntimeCatalog {
    pub(super) fn compile(
        structural: &GoldAndGearsStructuralCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut bosses = structural
            .boss_choices
            .iter()
            .map(|boss| {
                Ok(RuntimeBossChoice {
                    key: boss.stable_key.clone(),
                    source_id: boss
                        .source_id
                        .parse()
                        .map_err(|_| GoldAndGearsEntryError::InvalidPlaneTransition)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        bosses.sort_by_key(|boss| boss.source_id);
        if bosses.len() != 6
            || bosses
                .windows(2)
                .any(|pair| pair[0].source_id == pair[1].source_id)
        {
            return Err(GoldAndGearsEntryError::InvalidPlaneTransition);
        }
        Ok(Self {
            bosses: bosses.into_boxed_slice(),
        })
    }

    pub(super) fn choices(&self) -> impl ExactSizeIterator<Item = &str> {
        self.bosses.iter().map(|boss| boss.key.as_ref())
    }

    pub(super) fn selected_boss(
        &self,
        state: &ActivityTransactionState,
        plane_layer: u8,
    ) -> Option<&str> {
        let ActivityValue::BoundedCounterMap(values) =
            state.slot(slot(state_layout::PLANE_STATE_SLOT))?
        else {
            return None;
        };
        let value = |key| {
            values
                .binary_search_by_key(&key, |(candidate, _)| *candidate)
                .ok()
                .map(|index| values[index].1)
        };
        if value(PLANE_SELECTED_BOSS_LAYER_KEY) != Some(i64::from(plane_layer)) {
            return None;
        }
        let source_id = u32::try_from(value(PLANE_SELECTED_BOSS_KEY)?).ok()?;
        self.bosses
            .iter()
            .find(|boss| boss.source_id == source_id)
            .map(|boss| boss.key.as_ref())
    }

    pub(super) fn compile_selection(
        &self,
        plane_layer: u8,
        boss: &str,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        if !(1..=3).contains(&plane_layer) {
            return Err(GoldAndGearsEntryError::InvalidPlaneLayer);
        }
        let boss = self
            .bosses
            .iter()
            .find(|candidate| candidate.key.as_ref() == boss)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownBossChoice(boss.into()))?;
        program(
            BOSS_SELECTION_PROGRAM_BASE + u32::from(plane_layer),
            vec![
                set_counter(PLANE_SELECTED_BOSS_KEY, i64::from(boss.source_id)),
                set_counter(PLANE_SELECTED_BOSS_LAYER_KEY, i64::from(plane_layer)),
            ],
        )
    }

    pub(super) fn compile_completion(
        &self,
        cognition: &CognitionRuntimeCatalog,
        area: &str,
        graph: &ActivityGraphDefinition,
        plane_ends: &[NodeId],
        plane_layer: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let end = plane_ends
            .get(usize::from(plane_layer.saturating_sub(1)))
            .copied()
            .ok_or(GoldAndGearsEntryError::InvalidPlaneLayer)?;
        let edges = graph.outgoing(end).collect::<Vec<_>>();
        let [edge] = edges.as_slice() else {
            return Err(GoldAndGearsEntryError::InvalidPlaneTransition);
        };
        let mut suffix = vec![
            set_counter(PLANE_COMPLETED_LAYER_KEY, i64::from(plane_layer)),
            ActivityOperation::Traverse(edge.id()),
        ];
        if plane_layer == 3 {
            suffix.push(ActivityOperation::Terminal(
                ActivityTerminalOutcome::Completed,
            ));
        }
        let mut operations = vec![ActivityOperation::Require(ActivityCondition::All(
            vec![
                ActivityCondition::Not(Box::new(ActivityCondition::Equal(
                    ActivityExpression::CounterValue {
                        slot: slot(PLANE_STATE_SLOT),
                        key: PLANE_SELECTED_BOSS_KEY,
                    },
                    integer(0),
                ))),
                ActivityCondition::Equal(
                    ActivityExpression::CounterValue {
                        slot: slot(PLANE_STATE_SLOT),
                        key: PLANE_SELECTED_BOSS_LAYER_KEY,
                    },
                    integer(i64::from(plane_layer)),
                ),
            ]
            .into_boxed_slice(),
        ))];
        operations.extend(cognition.evaluation_operations(area, plane_layer, suffix)?);
        program(
            PLANE_COMPLETION_PROGRAM_BASE + u32::from(plane_layer),
            operations,
        )
    }

    #[cfg(test)]
    pub(super) fn denominator(&self) -> usize {
        self.bosses.len()
    }
}

fn set_counter(key: u64, desired: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(PLANE_STATE_SLOT),
        key,
        delta: ActivityExpression::Subtract(
            Box::new(integer(desired)),
            Box::new(ActivityExpression::CounterValue {
                slot: slot(PLANE_STATE_SLOT),
                key,
            }),
        ),
    }
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
    .map_err(|_| GoldAndGearsEntryError::InvalidPlaneTransition)
}
