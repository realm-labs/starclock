//! Exact Equation recipe progress derived from owned Blessing identities.

#[path = "equation_expansion.rs"]
pub(super) mod expansion;
#[path = "equation_progress_transition.rs"]
mod transition;

use transition::EquationProgressTransition;

use std::sync::Arc;

use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::{
        DivergentUniverseBlessingCatalog, DivergentUniverseBlessingId,
    },
    divergent_universe_equation_catalog::{
        DivergentUniverseEquationCatalog, DivergentUniverseEquationId,
        DivergentUniverseEquationRecipeId, DivergentUniverseEquationState,
        DivergentUniversePathType,
    },
};

use super::{
    DivergentUniverseRuntimeFactory,
    state::{
        BLESSINGS_SLOT, EQUATION_BLESSING_SNAPSHOT_SLOT, EQUATION_PROGRESS_DIRTY_SLOT,
        EQUATION_PROGRESS_SLOT, EQUATIONS_SLOT, EXPANDED_EQUATIONS_SLOT,
    },
};

const REFRESH_PROGRAM: u32 = 22_406;

/// Exact recipe and contribution evidence with a mode-owned Activity projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationProgressAccuracy {
    ExactReleasedRecipeAndOwnedBlessingIdentityContribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationExpansionState {
    Unexpanded,
    Expanded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationRecipeRuntime {
    equation: DivergentUniverseEquationId,
    recipe: DivergentUniverseEquationRecipeId,
    equation_key: u64,
    main_path: DivergentUniversePathType,
    main_required: u16,
    sub_path: Option<DivergentUniversePathType>,
    sub_required: u16,
}

impl DivergentUniverseEquationRecipeRuntime {
    #[must_use]
    pub const fn equation(&self) -> &DivergentUniverseEquationId {
        &self.equation
    }

    #[must_use]
    pub const fn recipe(&self) -> &DivergentUniverseEquationRecipeId {
        &self.recipe
    }

    #[must_use]
    pub const fn main_path(&self) -> &DivergentUniversePathType {
        &self.main_path
    }

    #[must_use]
    pub const fn main_required(&self) -> u16 {
        self.main_required
    }

    #[must_use]
    pub const fn sub_path(&self) -> Option<&DivergentUniversePathType> {
        self.sub_path.as_ref()
    }

    #[must_use]
    pub const fn sub_required(&self) -> u16 {
        self.sub_required
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BlessingContributionRuntime {
    blessing: DivergentUniverseBlessingId,
    blessing_key: u64,
    path: DivergentUniversePathType,
    equations: Box<[u64]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationProgressRuntime {
    recipes: Arc<[DivergentUniverseEquationRecipeRuntime]>,
    blessings: Arc<[BlessingContributionRuntime]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationProgressObservation {
    equation: DivergentUniverseEquationId,
    recipe: DivergentUniverseEquationRecipeId,
    main_count: u16,
    main_required: u16,
    sub_count: u16,
    sub_required: u16,
    state: DivergentUniverseEquationExpansionState,
}

impl DivergentUniverseEquationProgressObservation {
    #[must_use]
    pub const fn equation(&self) -> &DivergentUniverseEquationId {
        &self.equation
    }

    #[must_use]
    pub const fn recipe(&self) -> &DivergentUniverseEquationRecipeId {
        &self.recipe
    }

    #[must_use]
    pub const fn main_count(&self) -> u16 {
        self.main_count
    }

    #[must_use]
    pub const fn main_required(&self) -> u16 {
        self.main_required
    }

    #[must_use]
    pub const fn sub_count(&self) -> u16 {
        self.sub_count
    }

    #[must_use]
    pub const fn sub_required(&self) -> u16 {
        self.sub_required
    }

    #[must_use]
    pub const fn state(&self) -> DivergentUniverseEquationExpansionState {
        self.state
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationProgressResolution {
    progress: Box<[DivergentUniverseEquationProgressObservation]>,
    newly_expanded: Box<[DivergentUniverseEquationId]>,
    no_longer_expanded: Box<[DivergentUniverseEquationId]>,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseEquationProgressResolution {
    /// Exact unexpanded/absent-to-expanded edges of this accepted refresh, in
    /// stable identity order. This reports a transition, not a Curio reward.
    #[must_use]
    pub fn newly_expanded(&self) -> &[DivergentUniverseEquationId] {
        &self.newly_expanded
    }

    /// Previously expanded identities absent or unexpanded after this refresh.
    #[must_use]
    pub fn no_longer_expanded(&self) -> &[DivergentUniverseEquationId] {
        &self.no_longer_expanded
    }

    #[must_use]
    pub fn progress(&self) -> &[DivergentUniverseEquationProgressObservation] {
        &self.progress
    }

    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }

    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

impl DivergentUniverseRuntimeFactory {
    pub fn equation_progress_runtime(
        &self,
    ) -> Result<DivergentUniverseEquationProgressRuntime, DivergentUniverseEquationProgressError>
    {
        DivergentUniverseEquationProgressRuntime::compile(
            self.bundle.equation_catalog(),
            self.bundle.blessing_catalog(),
        )
    }
}

impl DivergentUniverseEquationProgressRuntime {
    pub(super) fn compile(
        equations: &DivergentUniverseEquationCatalog,
        blessings: &DivergentUniverseBlessingCatalog,
    ) -> Result<Self, DivergentUniverseEquationProgressError> {
        let recipes = equations
            .equations()
            .iter()
            .enumerate()
            .map(|(index, equation)| {
                let recipe = equations
                    .recipes()
                    .iter()
                    .find(|recipe| recipe.id == equation.recipe && recipe.equation == equation.id)
                    .ok_or(DivergentUniverseEquationProgressError::InvalidCatalog)?;
                let progress = equations
                    .progress()
                    .iter()
                    .find(|progress| {
                        progress.equation == equation.id && progress.recipe == recipe.id
                    })
                    .ok_or(DivergentUniverseEquationProgressError::InvalidCatalog)?;
                let states = equations
                    .states()
                    .iter()
                    .filter(|state| state.equation == equation.id)
                    .collect::<Vec<_>>();
                if progress.main_required != recipe.main_count
                    || progress.sub_required != recipe.sub_count
                    || progress.storage.as_ref() != "DerivedCounts"
                    || progress.refresh_trigger.as_ref() != "OwnedBlessingSetChanged"
                    || states.len() != 2
                    || !states.iter().any(|state| {
                        state.state == DivergentUniverseEquationState::Expanded
                            && state.effect_active
                    })
                    || !states.iter().any(|state| {
                        state.state == DivergentUniverseEquationState::Unexpanded
                            && !state.effect_active
                    })
                {
                    return Err(DivergentUniverseEquationProgressError::InvalidCatalog);
                }
                Ok(DivergentUniverseEquationRecipeRuntime {
                    equation: equation.id.clone(),
                    recipe: recipe.id.clone(),
                    equation_key: ordinal(index)?,
                    main_path: recipe.main_path.clone(),
                    main_required: recipe.main_count,
                    sub_path: recipe.sub_path.clone(),
                    sub_required: recipe.sub_count,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let contributions = blessings
            .contributions()
            .iter()
            .map(|contribution| {
                let blessing_index = blessings
                    .blessings()
                    .binary_search_by(|blessing| blessing.id.cmp(&contribution.blessing))
                    .map_err(|_| DivergentUniverseEquationProgressError::InvalidCatalog)?;
                let equation_keys = contribution
                    .equations
                    .iter()
                    .map(|equation| {
                        recipes
                            .binary_search_by(|recipe| recipe.equation.cmp(equation))
                            .ok()
                            .map(|index| recipes[index].equation_key)
                            .ok_or(DivergentUniverseEquationProgressError::InvalidCatalog)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if contribution.contribution != 1
                    || contribution.contribution_unit.as_ref() != "OwnedBlessingIdentity"
                    || !contribution.base_and_enhanced_count_equally
                    || contribution.refresh_timing.as_ref() != "OwnedBlessingIdentitySetChanged"
                    || contribution.replacement_behavior.as_ref()
                        != "RemoveInputIdentityThenAddAcceptedOutputIdentity"
                    || contribution.runtime_lowered
                    || equation_keys.is_empty()
                    || equation_keys.windows(2).any(|pair| pair[0] >= pair[1])
                {
                    return Err(DivergentUniverseEquationProgressError::InvalidCatalog);
                }
                Ok(BlessingContributionRuntime {
                    blessing: contribution.blessing.clone(),
                    blessing_key: ordinal(blessing_index)?,
                    path: contribution.path.clone(),
                    equations: equation_keys.into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if recipes.len() != 80
            || contributions.len() != 414
            || recipes
                .windows(2)
                .any(|pair| pair[0].equation >= pair[1].equation)
            || contributions
                .windows(2)
                .any(|pair| pair[0].blessing >= pair[1].blessing)
        {
            return Err(DivergentUniverseEquationProgressError::InvalidCatalog);
        }
        Ok(Self {
            recipes: recipes.into(),
            blessings: contributions.into(),
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseEquationProgressAccuracy {
        DivergentUniverseEquationProgressAccuracy::ExactReleasedRecipeAndOwnedBlessingIdentityContribution
    }

    #[must_use]
    pub fn recipes(&self) -> &[DivergentUniverseEquationRecipeRuntime] {
        &self.recipes
    }

    /// Recomputes every owned Equation in stable-ID order after the owned
    /// Blessing identity set or Equation set changes. An unchanged clean input
    /// is rejected without advancing the command sequence or RNG.
    pub fn refresh(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
    ) -> Result<DivergentUniverseEquationProgressResolution, DivergentUniverseEquationProgressError>
    {
        validate_hash(activity, expected_state_hash)?;
        let state = ProgressState::read(activity)?;
        let blessing_ids = self.owned_blessing_keys(&state.blessings)?;
        if !state.dirty && blessing_ids.as_slice() == state.blessing_snapshot.as_ref() {
            return Err(DivergentUniverseEquationProgressError::InputsUnchanged);
        }
        let transition = self.transition_for_inputs(
            &activity.player_view(),
            &state.equations,
            &state.blessings,
        )?;
        let newly_expanded = transition.newly_expanded().to_vec().into_boxed_slice();
        let no_longer_expanded = transition.no_longer_expanded().to_vec().into_boxed_slice();
        let operations = transition.into_operations();
        let program = ActivityProgramDefinition::new(program_id(), operations)
            .map_err(|_| DivergentUniverseEquationProgressError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseEquationProgressError::Activity)?;
        Ok(DivergentUniverseEquationProgressResolution {
            progress: self.observations(activity)?,
            newly_expanded,
            no_longer_expanded,
            events,
            state_hash: activity.state_hash(),
        })
    }

    pub fn observations(
        &self,
        activity: &GraphActivity,
    ) -> Result<
        Box<[DivergentUniverseEquationProgressObservation]>,
        DivergentUniverseEquationProgressError,
    > {
        self.observations_from_view(&activity.player_view())
    }

    pub(super) fn observations_from_view(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<
        Box<[DivergentUniverseEquationProgressObservation]>,
        DivergentUniverseEquationProgressError,
    > {
        let state = ProgressState::read_view(view)?;
        let blessing_ids = self.owned_blessing_keys(&state.blessings)?;
        if state.dirty || blessing_ids.as_slice() != state.blessing_snapshot.as_ref() {
            return Err(DivergentUniverseEquationProgressError::ProgressDirty);
        }
        let expected = self.compute(&state.equations, &blessing_ids)?;
        if expected.progress.as_ref() != state.progress.as_ref()
            || expected.expanded.as_ref() != state.expanded.as_ref()
        {
            return Err(DivergentUniverseEquationProgressError::InvalidState);
        }
        state
            .equations
            .iter()
            .map(|equation_key| {
                let recipe = self.recipe_by_key(*equation_key)?;
                let main_count = progress_value(&state.progress, main_progress_key(*equation_key))?;
                let sub_count = progress_value(&state.progress, sub_progress_key(*equation_key))?;
                Ok(DivergentUniverseEquationProgressObservation {
                    equation: recipe.equation.clone(),
                    recipe: recipe.recipe.clone(),
                    main_count,
                    main_required: recipe.main_required,
                    sub_count,
                    sub_required: recipe.sub_required,
                    state: if state.expanded.binary_search(equation_key).is_ok() {
                        DivergentUniverseEquationExpansionState::Expanded
                    } else {
                        DivergentUniverseEquationExpansionState::Unexpanded
                    },
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub(super) fn refresh_operations_for_activity(
        &self,
        activity: &GraphActivity,
        equations: &[u64],
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseEquationProgressError> {
        let state = ProgressState::read(activity)?;
        self.refresh_operations_for_inputs(&activity.player_view(), equations, &state.blessings)
    }

    pub(super) fn refresh_operations_for_inputs(
        &self,
        view: &ActivityPlayerView,
        equations: &[u64],
        blessings: &[(u64, i64)],
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseEquationProgressError> {
        Ok(self
            .transition_for_inputs(view, equations, blessings)?
            .into_operations())
    }

    /// Plans one final-input transition against the caller's immutable pre-state.
    /// Repeated planning is observational; only the owning transaction commits.
    pub(super) fn transition_for_inputs(
        &self,
        view: &ActivityPlayerView,
        equations: &[u64],
        blessings: &[(u64, i64)],
    ) -> Result<EquationProgressTransition, DivergentUniverseEquationProgressError> {
        let before = ProgressState::read_view(view)?;
        let blessing_ids = self.owned_blessing_keys(blessings)?;
        let computed = self.compute(equations, &blessing_ids)?;
        EquationProgressTransition::new(self, &before.expanded, computed, &blessing_ids)
    }

    fn compute(
        &self,
        equations: &[u64],
        blessing_ids: &[u64],
    ) -> Result<ComputedProgress, DivergentUniverseEquationProgressError> {
        if equations.windows(2).any(|pair| pair[0] >= pair[1])
            || blessing_ids.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(DivergentUniverseEquationProgressError::InvalidState);
        }
        let mut progress = Vec::with_capacity(equations.len().saturating_mul(2));
        let mut expanded = Vec::new();
        for equation_key in equations {
            let recipe = self.recipe_by_key(*equation_key)?;
            let mut main = 0_u16;
            let mut sub = 0_u16;
            for blessing_key in blessing_ids {
                let contribution = self.blessing_by_key(*blessing_key)?;
                if contribution.equations.binary_search(equation_key).is_err() {
                    continue;
                }
                if contribution.path == recipe.main_path {
                    main = main
                        .checked_add(1)
                        .ok_or(DivergentUniverseEquationProgressError::InvalidState)?;
                }
                if recipe
                    .sub_path
                    .as_ref()
                    .is_some_and(|path| contribution.path == *path)
                {
                    sub = sub
                        .checked_add(1)
                        .ok_or(DivergentUniverseEquationProgressError::InvalidState)?;
                }
            }
            progress.push((main_progress_key(*equation_key), i64::from(main)));
            progress.push((sub_progress_key(*equation_key), i64::from(sub)));
            if main >= recipe.main_required && sub >= recipe.sub_required {
                expanded.push(*equation_key);
            }
        }
        Ok(ComputedProgress {
            progress: progress.into_boxed_slice(),
            expanded: expanded.into_boxed_slice(),
        })
    }

    /// Missing-recipe candidates use the same exact contribution joins as
    /// expansion. Enhanced holdings still count once and are never rewarded twice.
    pub(super) fn missing_blessing_keys(
        &self,
        equation: u64,
        blessings: &[(u64, i64)],
    ) -> Result<Vec<u64>, DivergentUniverseEquationProgressError> {
        let owned = self.owned_blessing_keys(blessings)?;
        let computed = self.compute(&[equation], &owned)?;
        let recipe = self.recipe_by_key(equation)?;
        let main_missing =
            progress_value(&computed.progress, main_progress_key(equation))? < recipe.main_required;
        let sub_missing =
            progress_value(&computed.progress, sub_progress_key(equation))? < recipe.sub_required;
        Ok(self
            .blessings
            .iter()
            .filter(|blessing| {
                owned.binary_search(&blessing.blessing_key).is_err()
                    && blessing.equations.binary_search(&equation).is_ok()
                    && ((main_missing && blessing.path == recipe.main_path)
                        || (sub_missing && recipe.sub_path.as_ref() == Some(&blessing.path)))
            })
            .map(|blessing| blessing.blessing_key)
            .collect())
    }

    fn owned_blessing_keys(
        &self,
        blessings: &[(u64, i64)],
    ) -> Result<Vec<u64>, DivergentUniverseEquationProgressError> {
        let mut owned = Vec::new();
        for (key, level) in blessings {
            self.blessing_by_key(*key)?;
            if *level < 0 || *level > 2 {
                return Err(DivergentUniverseEquationProgressError::InvalidState);
            }
            if *level > 0 {
                owned.push(*key);
            }
        }
        Ok(owned)
    }

    fn recipe_by_key(
        &self,
        key: u64,
    ) -> Result<&DivergentUniverseEquationRecipeRuntime, DivergentUniverseEquationProgressError>
    {
        key.checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| self.recipes.get(index))
            .filter(|recipe| recipe.equation_key == key)
            .ok_or(DivergentUniverseEquationProgressError::UnknownEquation)
    }

    fn blessing_by_key(
        &self,
        key: u64,
    ) -> Result<&BlessingContributionRuntime, DivergentUniverseEquationProgressError> {
        key.checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| self.blessings.get(index))
            .filter(|blessing| blessing.blessing_key == key)
            .ok_or(DivergentUniverseEquationProgressError::UnknownBlessing)
    }
}

struct ComputedProgress {
    progress: Box<[(u64, i64)]>,
    expanded: Box<[u64]>,
}

struct ProgressState {
    equations: Box<[u64]>,
    blessings: Box<[(u64, i64)]>,
    progress: Box<[(u64, i64)]>,
    expanded: Box<[u64]>,
    blessing_snapshot: Box<[u64]>,
    dirty: bool,
}

impl ProgressState {
    fn read(activity: &GraphActivity) -> Result<Self, DivergentUniverseEquationProgressError> {
        Self::read_view(&activity.player_view())
    }

    fn read_view(
        view: &ActivityPlayerView,
    ) -> Result<Self, DivergentUniverseEquationProgressError> {
        Ok(Self {
            equations: set(view, EQUATIONS_SLOT)?,
            blessings: counter(view, BLESSINGS_SLOT)?,
            progress: counter(view, EQUATION_PROGRESS_SLOT)?,
            expanded: set(view, EXPANDED_EQUATIONS_SLOT)?,
            blessing_snapshot: set(view, EQUATION_BLESSING_SNAPSHOT_SLOT)?,
            dirty: boolean(view, EQUATION_PROGRESS_DIRTY_SLOT)?,
        })
    }
}

fn set(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Box<[u64]>, DivergentUniverseEquationProgressError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::OrderedIdSet(values) => Some(values.clone()),
            _ => None,
        })
        .ok_or(DivergentUniverseEquationProgressError::InvalidState)
}

fn counter(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Box<[(u64, i64)]>, DivergentUniverseEquationProgressError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(values) => Some(values.clone()),
            _ => None,
        })
        .ok_or(DivergentUniverseEquationProgressError::InvalidState)
}

fn boolean(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<bool, DivergentUniverseEquationProgressError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::Boolean(value) => Some(*value),
            _ => None,
        })
        .ok_or(DivergentUniverseEquationProgressError::InvalidState)
}

fn validate_hash(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseEquationProgressError> {
    if activity.state_hash() == expected {
        Ok(())
    } else {
        Err(DivergentUniverseEquationProgressError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ))
    }
}

fn progress_value(
    values: &[(u64, i64)],
    key: u64,
) -> Result<u16, DivergentUniverseEquationProgressError> {
    values
        .binary_search_by_key(&key, |value| value.0)
        .ok()
        .and_then(|index| u16::try_from(values[index].1).ok())
        .ok_or(DivergentUniverseEquationProgressError::InvalidState)
}

pub(super) fn main_progress_key(equation_key: u64) -> u64 {
    equation_key
        .checked_mul(2)
        .and_then(|value| value.checked_sub(1))
        .expect("bounded Equation key produces non-zero progress key")
}

pub(super) fn sub_progress_key(equation_key: u64) -> u64 {
    equation_key
        .checked_mul(2)
        .expect("bounded Equation key produces progress key")
}

fn ordinal(index: usize) -> Result<u64, DivergentUniverseEquationProgressError> {
    u64::try_from(index + 1)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(DivergentUniverseEquationProgressError::InvalidCatalog)
}

fn program_id() -> ActivityProgramId {
    ActivityProgramId::new(REFRESH_PROGRAM).expect("static Equation refresh program ID")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationProgressError {
    InvalidCatalog,
    UnknownEquation,
    UnknownBlessing,
    InputsUnchanged,
    ProgressDirty,
    InvalidState,
    InvalidProgram,
    Activity(GraphActivityCommandError),
}

impl core::fmt::Display for DivergentUniverseEquationProgressError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Equation progress error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseEquationProgressError {}
