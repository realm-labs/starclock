//! Frozen first Ordinary vertical-slice content transitions.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use super::DivergentUniverseBlessingRuntime;
use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityPlayerView,
    ActivityProgramDefinition, ActivityProgramId, ActivityStateHash, ActivityTransactionEvent,
    ActivityValue, GraphActivity, GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::divergent_universe::DivergentUniverseBundleCandidate;
use starclock_data::divergent_universe_blessing_catalog::DivergentUniverseBlessingId;
use starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId;
use std::slice::from_ref;

use super::{
    entry_flow::DivergentUniverseFlowInstance,
    equation_offer::DivergentUniverseEquationOfferRuntime,
    equation_progress::DivergentUniverseEquationProgressRuntime,
    state::{
        BLESSINGS_SLOT, CURRENCIES_SLOT, EQUATIONS_SLOT, SERVICE_RECEIPTS_SLOT, TITAN_BOONS_SLOT,
    },
};

const EQUATION_ID: &str = "divergent-universe.equation.3102001";
const BLESSING_ID: &str = "divergent-universe.blessing.615130";
const TITAN_BOON_ID: &str = "divergent-universe.titan-boon.10101";
const WORKBENCH_ID: &str = "divergent-universe.workbench.101";
const WORKBENCH_FUNCTION_ID: &str = "divergent-universe.workbench-function.1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseVerticalSliceAccuracy {
    ExactReleasedJoin,
    VersionedProjectPolicyNotObservedParity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseVerticalSliceTransition {
    AcquireEquation,
    AcquireBlessing,
    EnhanceBlessingAtWorkbench,
    AcceptTitanBoon,
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseVerticalSliceContent {
    component_digest: [u8; 32],
    equation_key: u64,
    blessing_key: u64,
    titan_boon_key: u64,
    workbench_receipt_key: u64,
    workbench_heat_key: u64,
    base_blessing_level: u16,
    enhanced_blessing_level: u16,
    contribution_digest: [u8; 32],
    progress: Arc<DivergentUniverseEquationProgressRuntime>,
    acquisition: Arc<DivergentUniverseEquationOfferRuntime>,
    blessing_acquisition: Arc<DivergentUniverseBlessingRuntime>,
    blessing_id: DivergentUniverseBlessingId,
    equation_id: DivergentUniverseEquationId,
}

impl DivergentUniverseVerticalSliceContent {
    #[must_use]
    pub const fn equation_key(&self) -> u64 {
        self.equation_key
    }
    #[must_use]
    pub const fn blessing_key(&self) -> u64 {
        self.blessing_key
    }
    #[must_use]
    pub const fn titan_boon_key(&self) -> u64 {
        self.titan_boon_key
    }
    #[must_use]
    pub const fn workbench_receipt_key(&self) -> u64 {
        self.workbench_receipt_key
    }
    #[must_use]
    pub const fn contribution_digest(&self) -> [u8; 32] {
        self.contribution_digest
    }
    #[must_use]
    pub const fn battle_contribution_accuracy(&self) -> DivergentUniverseVerticalSliceAccuracy {
        DivergentUniverseVerticalSliceAccuracy::ExactReleasedJoin
    }
    #[must_use]
    pub const fn workbench_cost_accuracy(&self) -> DivergentUniverseVerticalSliceAccuracy {
        DivergentUniverseVerticalSliceAccuracy::VersionedProjectPolicyNotObservedParity
    }
}

impl DivergentUniverseFlowInstance {
    #[must_use]
    pub fn first_ordinary_vertical_slice_content(
        &self,
    ) -> Option<&DivergentUniverseVerticalSliceContent> {
        self.vertical_slice_content.as_deref()
    }

    pub fn apply_first_ordinary_vertical_slice_transition(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        transition: DivergentUniverseVerticalSliceTransition,
    ) -> Result<Box<[ActivityTransactionEvent]>, DivergentUniverseVerticalSliceError> {
        if !self.is_first_ordinary_vertical_slice()
            || activity.definition().identity() != self.definition.identity()
        {
            return Err(DivergentUniverseVerticalSliceError::DefinitionMismatch);
        }
        if expected_state_hash != activity.state_hash() {
            return Err(DivergentUniverseVerticalSliceError::Activity(
                GraphActivityCommandError::StaleStateHash,
            ));
        }
        if activity.player_view().terminal().is_some() {
            return Err(DivergentUniverseVerticalSliceError::ActivityCompleted);
        }
        let content = self
            .vertical_slice_content
            .as_deref()
            .ok_or(DivergentUniverseVerticalSliceError::DefinitionMismatch)?;
        if content.component_digest != self.component_digest {
            return Err(DivergentUniverseVerticalSliceError::DefinitionMismatch);
        }
        let (raw_program, mut operations) = match transition {
            DivergentUniverseVerticalSliceTransition::AcquireEquation => {
                (30_001, vec![require(set_count(EQUATIONS_SLOT), 0)])
            }
            DivergentUniverseVerticalSliceTransition::AcquireBlessing => (
                30_002,
                vec![
                    ActivityOperation::Require(ActivityCondition::OrderedIdSetContains {
                        slot: EQUATIONS_SLOT,
                        id: content.equation_key,
                    }),
                    require(counter(BLESSINGS_SLOT, content.blessing_key), 0),
                    ActivityOperation::SetCounter {
                        slot: BLESSINGS_SLOT,
                        key: content.blessing_key,
                        value: integer(i64::from(content.base_blessing_level)),
                    },
                ],
            ),
            DivergentUniverseVerticalSliceTransition::EnhanceBlessingAtWorkbench => (
                30_003,
                vec![
                    require(
                        counter(BLESSINGS_SLOT, content.blessing_key),
                        i64::from(content.base_blessing_level),
                    ),
                    ActivityOperation::Require(ActivityCondition::Compare {
                        left: counter(CURRENCIES_SLOT, content.workbench_heat_key),
                        operator: starclock_activity::ActivityComparison::GreaterOrEqual,
                        right: integer(1),
                    }),
                    ActivityOperation::AddCounter {
                        slot: CURRENCIES_SLOT,
                        key: content.workbench_heat_key,
                        delta: integer(-1),
                    },
                    ActivityOperation::SetCounter {
                        slot: BLESSINGS_SLOT,
                        key: content.blessing_key,
                        value: integer(i64::from(content.enhanced_blessing_level)),
                    },
                    ActivityOperation::SetCounter {
                        slot: SERVICE_RECEIPTS_SLOT,
                        key: content.workbench_receipt_key,
                        value: integer(1),
                    },
                ],
            ),
            DivergentUniverseVerticalSliceTransition::AcceptTitanBoon => (
                30_004,
                vec![
                    require(
                        counter(SERVICE_RECEIPTS_SLOT, content.workbench_receipt_key),
                        1,
                    ),
                    ActivityOperation::InsertOrderedId {
                        slot: TITAN_BOONS_SLOT,
                        id: content.titan_boon_key,
                    },
                ],
            ),
        };
        if matches!(
            transition,
            DivergentUniverseVerticalSliceTransition::AcquireEquation
                | DivergentUniverseVerticalSliceTransition::AcquireBlessing
        ) {
            let view = activity.player_view();
            let resolution = activity
                .apply_generated_boundary(
                    expected_state_hash,
                    ActivityProgramId::new(raw_program)
                        .expect("reserved Equation acquisition program"),
                    |rng| {
                        let generated = if transition
                            == DivergentUniverseVerticalSliceTransition::AcquireEquation
                        {
                            content
                                .acquisition
                                .accepted_acquisition_operations(&view, &content.equation_id, rng)
                                .map_err(|_| {
                                    GraphActivityCommandError::Runtime(
                                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                                    )
                                })?
                        } else {
                            content
                                .blessing_acquisition
                                .acquisition_operations(&view, from_ref(&content.blessing_id), rng)
                                .map_err(|_| {
                                    GraphActivityCommandError::Runtime(
                                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                                    )
                                })?
                        };
                        operations.extend(generated);
                        Ok((operations, ()))
                    },
                )
                .map_err(DivergentUniverseVerticalSliceError::Activity)?;
            return Ok(resolution.events().to_vec().into_boxed_slice());
        }
        operations.extend(content.progress_operations(&activity.player_view(), transition)?);
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(raw_program).expect("reserved vertical-slice program ID"),
            operations,
        )
        .map_err(|_| DivergentUniverseVerticalSliceError::InvalidProgram)?;
        activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseVerticalSliceError::Activity)
    }
}

impl DivergentUniverseVerticalSliceContent {
    fn progress_operations(
        &self,
        view: &ActivityPlayerView,
        transition: DivergentUniverseVerticalSliceTransition,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseVerticalSliceError> {
        let equations = view
            .slots()
            .iter()
            .find(|slot| slot.id() == EQUATIONS_SLOT)
            .ok_or(DivergentUniverseVerticalSliceError::InvalidProgram)?;
        let blessings = view
            .slots()
            .iter()
            .find(|slot| slot.id() == BLESSINGS_SLOT)
            .ok_or(DivergentUniverseVerticalSliceError::InvalidProgram)?;
        let (ActivityValue::OrderedIdSet(equations), ActivityValue::BoundedCounterMap(blessings)) =
            (equations.value(), blessings.value())
        else {
            return Err(DivergentUniverseVerticalSliceError::InvalidProgram);
        };
        let mut equations = equations.iter().copied().collect::<BTreeSet<_>>();
        let mut blessings = blessings.iter().copied().collect::<BTreeMap<_, _>>();
        match transition {
            DivergentUniverseVerticalSliceTransition::AcquireEquation => {
                equations.insert(self.equation_key);
            }
            DivergentUniverseVerticalSliceTransition::AcquireBlessing => {
                blessings.insert(self.blessing_key, i64::from(self.base_blessing_level));
            }
            DivergentUniverseVerticalSliceTransition::EnhanceBlessingAtWorkbench => {
                blessings.insert(self.blessing_key, i64::from(self.enhanced_blessing_level));
            }
            DivergentUniverseVerticalSliceTransition::AcceptTitanBoon => {}
        }
        self.progress
            .refresh_operations_for_inputs(
                view,
                &equations.into_iter().collect::<Vec<_>>(),
                &blessings.into_iter().collect::<Vec<_>>(),
            )
            .map_err(|_| DivergentUniverseVerticalSliceError::InvalidProgram)
    }
}

pub(super) fn compile(
    bundle: &DivergentUniverseBundleCandidate,
    workbench_heat_key: u64,
    acquisition: Arc<DivergentUniverseEquationOfferRuntime>,
    blessing_acquisition: Arc<DivergentUniverseBlessingRuntime>,
) -> Result<DivergentUniverseVerticalSliceContent, DivergentUniverseVerticalSliceError> {
    let equations = bundle.equation_catalog();
    let equation_index = equations
        .equations()
        .iter()
        .position(|value| value.id.as_str() == EQUATION_ID)
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;
    let equation = &equations.equations()[equation_index];
    let recipe = equations
        .recipes()
        .iter()
        .find(|value| value.id == equation.recipe && value.equation == equation.id)
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;

    let blessings = bundle.blessing_catalog();
    let blessing_index = blessings
        .blessings()
        .iter()
        .position(|value| value.id.as_str() == BLESSING_ID)
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;
    let blessing = &blessings.blessings()[blessing_index];
    let _contribution = blessings
        .contributions()
        .iter()
        .find(|value| {
            value.blessing == blessing.id
                && value
                    .equations
                    .iter()
                    .any(|candidate| candidate == &equation.id)
                && value.path == recipe.main_path
                && value.base_and_enhanced_count_equally
        })
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;
    let mut levels = blessing
        .levels
        .iter()
        .filter_map(|id| blessings.levels().iter().find(|value| &value.id == id))
        .collect::<Vec<_>>();
    levels.sort_unstable_by_key(|value| value.level);
    if levels.len() != 2 || levels[0].level == 0 || levels[1].level <= levels[0].level {
        return Err(DivergentUniverseVerticalSliceError::CatalogJoin);
    }

    let titans = bundle.titan_catalog();
    let titan_index = titans
        .boons()
        .iter()
        .position(|value| value.id.as_str() == TITAN_BOON_ID)
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;
    let titan = &titans.boons()[titan_index];
    let titan_contribution = titans
        .contributions()
        .iter()
        .find(|value| value.id == titan.contribution)
        .filter(|value| {
            value.scope.as_ref() == "Battle"
                && value.activation.as_ref() == "AcceptedGoldenBloodBoon"
                && value.teardown.as_ref() == "BattleEnd"
                && !value.ordered_effects.is_empty()
        })
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;

    let services = bundle.service_catalog();
    let workbench_index = services
        .workbenches()
        .iter()
        .position(|value| value.id.as_str() == WORKBENCH_ID)
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;
    let workbench = &services.workbenches()[workbench_index];
    let _function = services
        .functions()
        .iter()
        .find(|value| value.id.as_str() == WORKBENCH_FUNCTION_ID)
        .filter(|value| {
            workbench.functions.contains(&value.id)
                && value.function_type.as_ref() == "BuffEnhance"
                && value.input_policy.as_ref() == "OwnedBaseBlessing"
                && value.output_policy.as_ref() == "SameIdentityEnhancedBlessing"
                && value.fallback.as_ref() == "RejectWithoutMutation"
        })
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)?;

    let component_digest = bundle.identity().component_digest().bytes();
    let contribution_digest = digest(&[
        b"starclock.divergent-universe.vertical-slice.titan-contribution.v1",
        &component_digest,
        titan.id.as_str().as_bytes(),
        titan_contribution.id.as_str().as_bytes(),
        titan_contribution.ordered_effects[0].operation.as_bytes(),
    ]);
    Ok(DivergentUniverseVerticalSliceContent {
        acquisition,
        blessing_acquisition,
        blessing_id: blessing.id.clone(),
        equation_id: equation.id.clone(),
        component_digest,
        equation_key: ordinal(equation_index)?,
        blessing_key: ordinal(blessing_index)?,
        titan_boon_key: ordinal(titan_index)?,
        workbench_receipt_key: ordinal(workbench_index)?,
        workbench_heat_key,
        base_blessing_level: levels[0].level,
        enhanced_blessing_level: levels[1].level,
        contribution_digest,
        progress: Arc::new(
            DivergentUniverseEquationProgressRuntime::compile(equations, blessings)
                .map_err(|_| DivergentUniverseVerticalSliceError::CatalogJoin)?,
        ),
    })
}

fn require(left: ActivityExpression, right: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(left, integer(right)))
}

fn counter(slot: starclock_activity::ActivitySlotId, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue { slot, key }
}

fn set_count(slot: starclock_activity::ActivitySlotId) -> ActivityExpression {
    ActivityExpression::OrderedIdSetCount(slot)
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn ordinal(index: usize) -> Result<u64, DivergentUniverseVerticalSliceError> {
    u64::try_from(index + 1)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(DivergentUniverseVerticalSliceError::CatalogJoin)
}

fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    for part in parts {
        hash.update(
            u64::try_from(part.len())
                .expect("slice length fits u64")
                .to_le_bytes(),
        );
        hash.update(part);
    }
    hash.finalize()
}

#[derive(Debug)]
pub enum DivergentUniverseVerticalSliceError {
    CatalogJoin,
    DefinitionMismatch,
    ActivityCompleted,
    InvalidProgram,
    Activity(GraphActivityCommandError),
}

impl std::fmt::Display for DivergentUniverseVerticalSliceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe vertical slice: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseVerticalSliceError {}
