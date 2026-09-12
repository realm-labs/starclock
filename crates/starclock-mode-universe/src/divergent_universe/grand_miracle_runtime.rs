//! Exact Grand Miracle eligibility and policy-bound accepted lifecycle state.

use starclock_activity::{
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityStateHash,
    ActivityTransactionEvent, ActivityValue, GraphActivity, GraphActivityCommandError,
};
use starclock_data::divergent_universe_curio_catalog::{
    DivergentUniverseGrandMiracleEligibilityDefinition, DivergentUniverseGrandMiracleId,
    DivergentUniverseGrandMiracleStateId,
};

use super::DivergentUniverseRuntimeFactory;
use super::state::GRAND_MIRACLES_SLOT;

const INSTALL_PROGRAM: u32 = 22_521;
const ACTIVATE_PROGRAM: u32 = 22_522;
const TEARDOWN_PROGRAM: u32 = 22_523;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseGrandMiracleAccuracy {
    ExactReleasedEligibility,
    VersionedProjectPolicyAcceptedLifecycleMissingReleasedMazeBuff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseGrandMiracleLifecycleState {
    Inactive,
    Active,
}

impl DivergentUniverseGrandMiracleLifecycleState {
    const fn raw(self) -> i64 {
        match self {
            Self::Inactive => 1,
            Self::Active => 2,
        }
    }

    fn from_raw(value: i64) -> Result<Self, DivergentUniverseGrandMiracleRuntimeError> {
        match value {
            1 => Ok(Self::Inactive),
            2 => Ok(Self::Active),
            _ => Err(DivergentUniverseGrandMiracleRuntimeError::InvalidState),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGrandMiracleRuntimeDefinition {
    id: DivergentUniverseGrandMiracleId,
    key: u64,
    inactive_state: DivergentUniverseGrandMiracleStateId,
    active_state: DivergentUniverseGrandMiracleStateId,
    character_paths: Box<[Box<str>]>,
    elements: Box<[Box<str>]>,
    effect_ids: Box<[Box<str>]>,
    maze_buff_id: Box<str>,
    maze_buff_resolution: Box<str>,
}

impl DivergentUniverseGrandMiracleRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseGrandMiracleId {
        &self.id
    }

    #[must_use]
    pub const fn inactive_state(&self) -> &DivergentUniverseGrandMiracleStateId {
        &self.inactive_state
    }

    #[must_use]
    pub const fn active_state(&self) -> &DivergentUniverseGrandMiracleStateId {
        &self.active_state
    }

    #[must_use]
    pub fn character_paths(&self) -> &[Box<str>] {
        &self.character_paths
    }

    #[must_use]
    pub fn elements(&self) -> &[Box<str>] {
        &self.elements
    }

    #[must_use]
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }

    #[must_use]
    pub fn maze_buff_id(&self) -> &str {
        &self.maze_buff_id
    }

    #[must_use]
    pub fn maze_buff_resolution(&self) -> &str {
        &self.maze_buff_resolution
    }

    #[must_use]
    pub fn is_eligible(&self, character_paths: &[&str], elements: &[&str]) -> bool {
        self.character_paths
            .iter()
            .any(|candidate| character_paths.contains(&candidate.as_ref()))
            || self
                .elements
                .iter()
                .any(|candidate| elements.contains(&candidate.as_ref()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOwnedGrandMiracle {
    id: DivergentUniverseGrandMiracleId,
    state: DivergentUniverseGrandMiracleStateId,
    lifecycle: DivergentUniverseGrandMiracleLifecycleState,
}

impl DivergentUniverseOwnedGrandMiracle {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseGrandMiracleId {
        &self.id
    }

    #[must_use]
    pub const fn state(&self) -> &DivergentUniverseGrandMiracleStateId {
        &self.state
    }

    #[must_use]
    pub const fn lifecycle(&self) -> DivergentUniverseGrandMiracleLifecycleState {
        self.lifecycle
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGrandMiracleResolution {
    owned: Box<[DivergentUniverseOwnedGrandMiracle]>,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseGrandMiracleResolution {
    #[must_use]
    pub fn owned(&self) -> &[DivergentUniverseOwnedGrandMiracle] {
        &self.owned
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGrandMiracleRuntime {
    definitions: Box<[DivergentUniverseGrandMiracleRuntimeDefinition]>,
    historical_eligibility_exclusions: usize,
}

impl DivergentUniverseRuntimeFactory {
    pub fn grand_miracle_runtime(
        &self,
    ) -> Result<DivergentUniverseGrandMiracleRuntime, DivergentUniverseGrandMiracleRuntimeError>
    {
        compile(self.bundle.curio_catalog())
    }
}

impl DivergentUniverseGrandMiracleRuntime {
    #[must_use]
    pub const fn accuracy(&self) -> [DivergentUniverseGrandMiracleAccuracy; 2] {
        [
            DivergentUniverseGrandMiracleAccuracy::ExactReleasedEligibility,
            DivergentUniverseGrandMiracleAccuracy::VersionedProjectPolicyAcceptedLifecycleMissingReleasedMazeBuff,
        ]
    }

    #[must_use]
    pub fn definitions(&self) -> &[DivergentUniverseGrandMiracleRuntimeDefinition] {
        &self.definitions
    }

    #[must_use]
    pub const fn historical_eligibility_exclusions(&self) -> usize {
        self.historical_eligibility_exclusions
    }

    #[must_use]
    pub fn eligible<'a>(
        &'a self,
        character_paths: &[&str],
        elements: &[&str],
    ) -> Box<[&'a DivergentUniverseGrandMiracleRuntimeDefinition]> {
        self.definitions
            .iter()
            .filter(|definition| definition.is_eligible(character_paths, elements))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub fn install_inactive_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        id: &DivergentUniverseGrandMiracleId,
    ) -> Result<DivergentUniverseGrandMiracleResolution, DivergentUniverseGrandMiracleRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        let definition = self.definition(id)?;
        let mut state = read_state(activity)?;
        if state
            .binary_search_by_key(&definition.key, |entry| entry.0)
            .is_ok()
        {
            return Err(DivergentUniverseGrandMiracleRuntimeError::AlreadyOwned);
        }
        state.insert(
            state.partition_point(|entry| entry.0 < definition.key),
            (
                definition.key,
                DivergentUniverseGrandMiracleLifecycleState::Inactive.raw(),
            ),
        );
        self.apply(activity, expected_state_hash, INSTALL_PROGRAM, state)
    }

    pub fn activate_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        id: &DivergentUniverseGrandMiracleId,
    ) -> Result<DivergentUniverseGrandMiracleResolution, DivergentUniverseGrandMiracleRuntimeError>
    {
        self.transition(
            activity,
            expected_state_hash,
            id,
            DivergentUniverseGrandMiracleLifecycleState::Inactive,
            DivergentUniverseGrandMiracleLifecycleState::Active,
            ACTIVATE_PROGRAM,
        )
    }

    pub fn teardown_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        id: &DivergentUniverseGrandMiracleId,
    ) -> Result<DivergentUniverseGrandMiracleResolution, DivergentUniverseGrandMiracleRuntimeError>
    {
        self.transition(
            activity,
            expected_state_hash,
            id,
            DivergentUniverseGrandMiracleLifecycleState::Active,
            DivergentUniverseGrandMiracleLifecycleState::Inactive,
            TEARDOWN_PROGRAM,
        )
    }

    pub fn owned(
        &self,
        activity: &GraphActivity,
    ) -> Result<Box<[DivergentUniverseOwnedGrandMiracle]>, DivergentUniverseGrandMiracleRuntimeError>
    {
        read_state(activity)?
            .into_iter()
            .map(|(key, raw)| {
                let definition = self
                    .definitions
                    .iter()
                    .find(|definition| definition.key == key)
                    .ok_or(DivergentUniverseGrandMiracleRuntimeError::InvalidState)?;
                let lifecycle = DivergentUniverseGrandMiracleLifecycleState::from_raw(raw)?;
                let state = match lifecycle {
                    DivergentUniverseGrandMiracleLifecycleState::Inactive => {
                        definition.inactive_state.clone()
                    }
                    DivergentUniverseGrandMiracleLifecycleState::Active => {
                        definition.active_state.clone()
                    }
                };
                Ok(DivergentUniverseOwnedGrandMiracle {
                    id: definition.id.clone(),
                    state,
                    lifecycle,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    fn transition(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        id: &DivergentUniverseGrandMiracleId,
        expected: DivergentUniverseGrandMiracleLifecycleState,
        output: DivergentUniverseGrandMiracleLifecycleState,
        program: u32,
    ) -> Result<DivergentUniverseGrandMiracleResolution, DivergentUniverseGrandMiracleRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        let definition = self.definition(id)?;
        let mut state = read_state(activity)?;
        let index = state
            .binary_search_by_key(&definition.key, |entry| entry.0)
            .map_err(|_| DivergentUniverseGrandMiracleRuntimeError::NotOwned)?;
        if DivergentUniverseGrandMiracleLifecycleState::from_raw(state[index].1)? != expected {
            return Err(DivergentUniverseGrandMiracleRuntimeError::InvalidLifecycle);
        }
        state[index].1 = output.raw();
        self.apply(activity, expected_state_hash, program, state)
    }

    fn definition(
        &self,
        id: &DivergentUniverseGrandMiracleId,
    ) -> Result<
        &DivergentUniverseGrandMiracleRuntimeDefinition,
        DivergentUniverseGrandMiracleRuntimeError,
    > {
        self.definitions
            .binary_search_by(|candidate| candidate.id.cmp(id))
            .ok()
            .and_then(|index| self.definitions.get(index))
            .ok_or(DivergentUniverseGrandMiracleRuntimeError::UnknownMiracle)
    }

    fn apply(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        raw_program: u32,
        state: Vec<(u64, i64)>,
    ) -> Result<DivergentUniverseGrandMiracleResolution, DivergentUniverseGrandMiracleRuntimeError>
    {
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(raw_program)
                .expect("static Grand Miracle program ID is non-zero"),
            vec![ActivityOperation::SetCounterMap {
                slot: GRAND_MIRACLES_SLOT,
                values: state.into_boxed_slice(),
            }],
        )
        .map_err(|_| DivergentUniverseGrandMiracleRuntimeError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseGrandMiracleRuntimeError::Activity)?;
        Ok(DivergentUniverseGrandMiracleResolution {
            owned: self.owned(activity)?,
            events,
            state_hash: activity.state_hash(),
        })
    }
}

fn compile(
    catalog: &starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioCatalog,
) -> Result<DivergentUniverseGrandMiracleRuntime, DivergentUniverseGrandMiracleRuntimeError> {
    let historical_eligibility_exclusions = catalog
        .miracle_eligibility()
        .iter()
        .filter(|rule| rule.selector_scope.as_ref() != "Tourn3")
        .count();
    let definitions = catalog
        .miracles()
        .iter()
        .enumerate()
        .map(|(index, miracle)| {
            if miracle.states.len() != 2
                || miracle.eligibility_rules.len() != 1
                || miracle.effect_ids.len() > 1
                || miracle.effect_ids.iter().any(|value| value.is_empty())
                || miracle.maze_buff_resolution.as_ref() != "MissingReleasedRogueMazeBuffRow"
                || miracle.runtime_lowered
            {
                return Err(DivergentUniverseGrandMiracleRuntimeError::InvalidCatalog);
            }
            let eligibility = catalog
                .miracle_eligibility()
                .binary_search_by(|rule| rule.id.cmp(&miracle.eligibility_rules[0]))
                .ok()
                .and_then(|rule_index| catalog.miracle_eligibility().get(rule_index))
                .ok_or(DivergentUniverseGrandMiracleRuntimeError::InvalidCatalog)?;
            validate_current_eligibility(eligibility, &miracle.id)?;
            let inactive = lifecycle_state(catalog, miracle, "Inactive")?;
            let active = lifecycle_state(catalog, miracle, "Active")?;
            Ok(DivergentUniverseGrandMiracleRuntimeDefinition {
                id: miracle.id.clone(),
                key: u64::try_from(index + 1)
                    .map_err(|_| DivergentUniverseGrandMiracleRuntimeError::InvalidCatalog)?,
                inactive_state: inactive.id.clone(),
                active_state: active.id.clone(),
                character_paths: eligibility.character_paths.clone(),
                elements: eligibility.elements.clone(),
                effect_ids: miracle.effect_ids.clone(),
                maze_buff_id: miracle.maze_buff_id.clone(),
                maze_buff_resolution: miracle.maze_buff_resolution.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if definitions.len() != 17 || historical_eligibility_exclusions != 57 {
        return Err(DivergentUniverseGrandMiracleRuntimeError::InvalidCatalog);
    }
    Ok(DivergentUniverseGrandMiracleRuntime {
        definitions: definitions.into_boxed_slice(),
        historical_eligibility_exclusions,
    })
}

fn validate_current_eligibility(
    rule: &DivergentUniverseGrandMiracleEligibilityDefinition,
    miracle: &DivergentUniverseGrandMiracleId,
) -> Result<(), DivergentUniverseGrandMiracleRuntimeError> {
    if rule.miracle.as_ref() != Some(miracle)
        || rule.selector_scope.as_ref() != "Tourn3"
        || rule.eligibility.as_ref() != "AnyListedPathOrElement"
        || (rule.character_paths.is_empty() && rule.elements.is_empty())
        || rule.runtime_lowered
    {
        Err(DivergentUniverseGrandMiracleRuntimeError::InvalidCatalog)
    } else {
        Ok(())
    }
}

fn lifecycle_state<'a>(
    catalog: &'a starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioCatalog,
    miracle: &starclock_data::divergent_universe_curio_catalog::DivergentUniverseGrandMiracleDefinition,
    expected: &str,
) -> Result<
    &'a starclock_data::divergent_universe_curio_catalog::DivergentUniverseGrandMiracleStateDefinition,
    DivergentUniverseGrandMiracleRuntimeError,
>{
    let state = miracle
        .states
        .iter()
        .find_map(|id| {
            catalog
                .miracle_states()
                .binary_search_by(|candidate| candidate.id.cmp(id))
                .ok()
                .and_then(|index| catalog.miracle_states().get(index))
                .filter(|candidate| candidate.state.as_ref() == expected)
        })
        .ok_or(DivergentUniverseGrandMiracleRuntimeError::InvalidCatalog)?;
    if state.miracle != miracle.id
        || state.activation.as_ref() != "Unspecified"
        || state.duration.as_ref() != "Unspecified"
        || state.teardown.as_ref() != "Unspecified"
        || state.simultaneous_trigger_order.as_ref() != "Unspecified"
        || state.fallback.as_ref() != "RejectWithoutMutation"
        || state.runtime_lowered
    {
        Err(DivergentUniverseGrandMiracleRuntimeError::InvalidCatalog)
    } else {
        Ok(state)
    }
}

fn read_state(
    activity: &GraphActivity,
) -> Result<Vec<(u64, i64)>, DivergentUniverseGrandMiracleRuntimeError> {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == GRAND_MIRACLES_SLOT)
        .ok_or(DivergentUniverseGrandMiracleRuntimeError::InvalidState)?
        .value();
    let ActivityValue::BoundedCounterMap(values) = value else {
        return Err(DivergentUniverseGrandMiracleRuntimeError::InvalidState);
    };
    if values.len() > 17
        || values.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        || values.iter().any(|(_, value)| {
            DivergentUniverseGrandMiracleLifecycleState::from_raw(*value).is_err()
        })
    {
        return Err(DivergentUniverseGrandMiracleRuntimeError::InvalidState);
    }
    Ok(values.to_vec())
}

fn validate_hash(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseGrandMiracleRuntimeError> {
    if activity.state_hash() == expected {
        Ok(())
    } else {
        Err(DivergentUniverseGrandMiracleRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseGrandMiracleRuntimeError {
    InvalidCatalog,
    InvalidState,
    InvalidProgram,
    UnknownMiracle,
    AlreadyOwned,
    NotOwned,
    InvalidLifecycle,
    Activity(GraphActivityCommandError),
}

impl core::fmt::Display for DivergentUniverseGrandMiracleRuntimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Grand Miracle runtime error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseGrandMiracleRuntimeError {}
