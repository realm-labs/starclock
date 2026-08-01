//! Explicit boss selection and atomic plane-completion programs.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityGraphDefinition, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivitySlotId, ActivityTerminalOutcome,
    ActivityTransactionState, ActivityValue,
};

use crate::{
    definition::RecommendedElement,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_structural::transition_access::SwarmBossChoiceRuntimeInput,
};

use super::{
    dice_control::REROLL_CHARGE_KEY,
    state::{PLANE, RESOURCES},
    topology::CompiledPlane,
    trail,
};

const BOSS_SELECTION_PROGRAM_BASE: u32 = 0x5360_0000;
const PLANE_COMPLETION_PROGRAM_BASE: u32 = 0x5360_0010;

const PLANE_SELECTED_BOSS_KEY: u64 = 4;
const PLANE_SELECTED_BOSS_LAYER_KEY: u64 = 5;
const PLANE_COMPLETED_LAYER_KEY: u64 = 6;

#[derive(Debug)]
pub(super) struct PlaneTransitionRuntimeCatalog {
    bosses: Box<[RuntimeBossChoice]>,
}

#[derive(Debug)]
struct RuntimeBossChoice {
    id: u32,
    key: Box<str>,
    source_id: u32,
    _display_level: u16,
    _enemy_variant_id: u32,
    _weakness_elements: Box<[RecommendedElement]>,
}

impl PlaneTransitionRuntimeCatalog {
    pub(super) fn compile(
        input: Box<[SwarmBossChoiceRuntimeInput]>,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let mut bosses = input
            .into_vec()
            .into_iter()
            .map(|boss| {
                let source_id = boss
                    .source_id
                    .parse::<u32>()
                    .map_err(|_| invalid("invalid Swarm boss source ID"))?;
                let enemy_variant_id = boss
                    .enemy_variant_id
                    .parse::<u32>()
                    .map_err(|_| invalid("invalid Swarm enemy variant ID"))?;
                if boss.id == 0
                    || boss.key.is_empty()
                    || source_id == 0
                    || enemy_variant_id == 0
                    || boss.display_level == 0
                    || boss.weakness_elements.is_empty()
                {
                    return Err(invalid("invalid Swarm boss choice descriptor"));
                }
                Ok(RuntimeBossChoice {
                    id: boss.id,
                    key: boss.key,
                    source_id,
                    _display_level: boss.display_level,
                    _enemy_variant_id: enemy_variant_id,
                    _weakness_elements: boss.weakness_elements,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        bosses.sort_unstable_by_key(|boss| boss.source_id);
        if bosses.len() != 2
            || bosses
                .windows(2)
                .any(|pair| pair[0].source_id >= pair[1].source_id || pair[0].id == pair[1].id)
        {
            return Err(invalid("Swarm boss-choice denominator drift"));
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
        if plane_counter(state, PLANE_SELECTED_BOSS_LAYER_KEY)? != i64::from(plane_layer) {
            return None;
        }
        let source_id = u32::try_from(plane_counter(state, PLANE_SELECTED_BOSS_KEY)?).ok()?;
        self.bosses
            .iter()
            .find(|boss| boss.source_id == source_id)
            .map(|boss| boss.key.as_ref())
    }

    pub(super) fn compile_selection(
        &self,
        plane_layer: u8,
        boss: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        require_plane_layer(plane_layer)?;
        let boss = self
            .bosses
            .iter()
            .find(|candidate| candidate.key.as_ref() == boss)
            .ok_or_else(|| invalid("unknown Swarm boss choice"))?;
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
        boss_decay_conditions: Vec<ActivityCondition>,
        state: &ActivityTransactionState,
        graph: &ActivityGraphDefinition,
        planes: &[CompiledPlane],
        plane_layer: u8,
        next_plane_rerolls: i64,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        require_plane_layer(plane_layer)?;
        let plane = planes
            .get(usize::from(plane_layer - 1))
            .ok_or_else(|| invalid("invalid Swarm plane layer"))?;
        if state.current_node() != plane.end {
            return Err(invalid("Swarm plane completion is not at the plane end"));
        }
        let edges = graph.outgoing(plane.end).collect::<Vec<_>>();
        let [edge] = edges.as_slice() else {
            return Err(invalid("invalid Swarm plane transition edge"));
        };
        let selected_source = plane_counter(state, PLANE_SELECTED_BOSS_KEY)
            .filter(|source_id| {
                u32::try_from(*source_id).is_ok_and(|source_id| {
                    self.bosses.iter().any(|boss| boss.source_id == source_id)
                })
            })
            .ok_or_else(|| invalid("valid Swarm boss selection is missing"))?;
        if plane_counter(state, PLANE_SELECTED_BOSS_LAYER_KEY) != Some(i64::from(plane_layer)) {
            return Err(invalid("Swarm boss selection is for a different plane"));
        }
        let mut conditions = vec![
            ActivityCondition::Equal(counter(PLANE_SELECTED_BOSS_KEY), integer(selected_source)),
            ActivityCondition::Equal(
                counter(PLANE_SELECTED_BOSS_LAYER_KEY),
                integer(i64::from(plane_layer)),
            ),
        ];
        conditions.extend(boss_decay_conditions);
        let mut operations = vec![ActivityOperation::Require(ActivityCondition::All(
            conditions.into_boxed_slice(),
        ))];
        operations.push(set_counter(
            PLANE_COMPLETED_LAYER_KEY,
            i64::from(plane_layer),
        ));
        if plane_layer < 3 && next_plane_rerolls != 0 {
            operations.push(trail::add_counter(
                RESOURCES,
                REROLL_CHARGE_KEY,
                next_plane_rerolls,
            ));
        }
        operations.push(ActivityOperation::Traverse(edge.id()));
        if plane_layer == 3 {
            operations.push(ActivityOperation::Terminal(
                ActivityTerminalOutcome::Completed,
            ));
        }
        program(
            PLANE_COMPLETION_PROGRAM_BASE + u32::from(plane_layer),
            operations,
        )
    }

    #[cfg(test)]
    pub(super) fn descriptors(
        &self,
    ) -> impl ExactSizeIterator<Item = (u32, &str, u32, u16, u32, &[RecommendedElement])> {
        self.bosses.iter().map(|boss| {
            (
                boss.id,
                boss.key.as_ref(),
                boss.source_id,
                boss._display_level,
                boss._enemy_variant_id,
                boss._weakness_elements.as_ref(),
            )
        })
    }
}

fn plane_counter(state: &ActivityTransactionState, key: u64) -> Option<i64> {
    let ActivityValue::BoundedCounterMap(values) = state.slot(slot(PLANE))? else {
        return None;
    };
    Some(
        values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
    )
}

fn set_counter(key: u64, desired: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(PLANE),
        key,
        delta: ActivityExpression::Subtract(Box::new(integer(desired)), Box::new(counter(key))),
    }
}

fn counter(key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(PLANE),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn require_plane_layer(plane_layer: u8) -> Result<(), UniverseCatalogLoadError> {
    if (1..=3).contains(&plane_layer) {
        Ok(())
    } else {
        Err(invalid("invalid Swarm plane layer"))
    }
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Swarm slot is non-zero")
}

fn program(
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(raw).expect("static Swarm program ID is non-zero"),
        operations,
    )
    .map_err(|_| invalid("invalid Swarm plane-transition program"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
