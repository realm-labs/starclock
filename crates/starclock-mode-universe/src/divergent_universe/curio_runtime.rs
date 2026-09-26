//! Atomic accepted Curio lifecycle transitions and immutable contributions.

#[path = "curio_acquisition.rs"]
mod acquisition;
#[path = "curio_acquisition_effects.rs"]
mod acquisition_effects;
#[path = "curio_acquisition_rewards.rs"]
mod acquisition_rewards;
#[path = "curio_blessing_plan.rs"]
mod blessing_plan;
#[path = "curio_domain_entry_program.rs"]
mod domain_entry_program;
#[path = "curio_domain_expiry.rs"]
mod domain_expiry;
#[path = "curio_evolution.rs"]
mod evolution;

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_curio_catalog::{
    DivergentUniverseCurioGroupId, DivergentUniverseCurioId, DivergentUniverseCurioStateId,
};
use starclock_data::divergent_universe_decisions::{
    CurioAcquisitionDefinition, CurioBattleReactionDefinition, CurioBattleStatDefinition,
    CurioDomainExpiryDefinition, CurioDomainGrantDefinition, CurioEvolutionDefinition,
};

use crate::path::ExactParameter;

use super::DivergentUniverseRuntimeFactory;
use super::blessing_runtime::{
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
};
use super::curio_catalog::{
    CompiledCurioCatalog, CurioCatalogCompileError, DivergentUniverseCurioAccuracy,
    DivergentUniverseCurioCatalogMembershipRuntime, DivergentUniverseCurioGroupPolicy,
    DivergentUniverseCurioRuntimeDefinition, DivergentUniverseCurioStateRuntime,
};
use super::economy::{self, DivergentUniverseCurrencyKind, DivergentUniverseCurrencyRuntime};
use super::equation_progress::expansion::ExpansionAllowance;
use super::state::{CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT};

const ACQUIRE_PROGRAM: u32 = 22_511;
const ACTIVATE_PROGRAM: u32 = 22_512;
const CHARGES_PROGRAM: u32 = 22_513;
const DESTROY_PROGRAM: u32 = 22_514;
const REPAIR_PROGRAM: u32 = 22_515;
const REPLACE_PROGRAM: u32 = 22_516;
const EVOLVE_PROGRAM: u32 = 22_517;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurioLifecycleAccuracy {
    VersionedProjectPolicyAcceptedStableIdTransitionsNotObservedParity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurioLifecycleState {
    Active,
    Destroyed,
}

impl DivergentUniverseCurioLifecycleState {
    const fn raw(self) -> i64 {
        match self {
            Self::Active => 1,
            Self::Destroyed => 2,
        }
    }

    fn from_raw(value: i64) -> Result<Self, DivergentUniverseCurioRuntimeError> {
        match value {
            1 => Ok(Self::Active),
            2 => Ok(Self::Destroyed),
            _ => Err(DivergentUniverseCurioRuntimeError::InvalidState),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOwnedCurioState {
    state: DivergentUniverseCurioStateId,
    curio: DivergentUniverseCurioId,
    lifecycle: DivergentUniverseCurioLifecycleState,
    charges: u16,
    activations: u32,
}

impl DivergentUniverseOwnedCurioState {
    #[must_use]
    pub const fn state(&self) -> &DivergentUniverseCurioStateId {
        &self.state
    }
    #[must_use]
    pub const fn curio(&self) -> &DivergentUniverseCurioId {
        &self.curio
    }
    #[must_use]
    pub const fn lifecycle(&self) -> DivergentUniverseCurioLifecycleState {
        self.lifecycle
    }
    #[must_use]
    pub const fn charges(&self) -> u16 {
        self.charges
    }
    #[must_use]
    pub const fn activations(&self) -> u32 {
        self.activations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioCommandResolution {
    owned: Box<[DivergentUniverseOwnedCurioState]>,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseCurioCommandResolution {
    #[must_use]
    pub fn owned(&self) -> &[DivergentUniverseOwnedCurioState] {
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
pub struct DivergentUniverseCurioContribution {
    state: DivergentUniverseCurioStateId,
    curio: DivergentUniverseCurioId,
    effect_ids: Box<[Box<str>]>,
    effect_parameters: Box<[ExactParameter]>,
    trigger_kinds: Box<[Box<str>]>,
    mechanic_visibility: Box<str>,
    charges: u16,
    activations: u32,
}

impl DivergentUniverseCurioContribution {
    #[must_use]
    pub const fn state(&self) -> &DivergentUniverseCurioStateId {
        &self.state
    }
    #[must_use]
    pub const fn curio(&self) -> &DivergentUniverseCurioId {
        &self.curio
    }
    #[must_use]
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }
    #[must_use]
    pub fn effect_parameters(&self) -> &[ExactParameter] {
        &self.effect_parameters
    }
    #[must_use]
    pub fn trigger_kinds(&self) -> &[Box<str>] {
        &self.trigger_kinds
    }
    #[must_use]
    pub fn mechanic_visibility(&self) -> &str {
        &self.mechanic_visibility
    }
    #[must_use]
    pub const fn charges(&self) -> u16 {
        self.charges
    }
    #[must_use]
    pub const fn activations(&self) -> u32 {
        self.activations
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseCurioSnapshotDigest([u8; 32]);

impl DivergentUniverseCurioSnapshotDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioSnapshot {
    source_state_hash: ActivityStateHash,
    contributions: Box<[DivergentUniverseCurioContribution]>,
    digest: DivergentUniverseCurioSnapshotDigest,
}

impl DivergentUniverseCurioSnapshot {
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }
    #[must_use]
    pub fn contributions(&self) -> &[DivergentUniverseCurioContribution] {
        &self.contributions
    }
    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseCurioSnapshotDigest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioRuntime {
    catalog: CompiledCurioCatalog,
    component_digest: [u8; 32],
    acquisition_effects: Arc<[CurioAcquisitionDefinition]>,
    evolutions: Arc<[CurioEvolutionDefinition]>,
    domain_expiries: Arc<[CurioDomainExpiryDefinition]>,
    domain_grants: Arc<[CurioDomainGrantDefinition]>,
    battle_stats: Arc<[CurioBattleStatDefinition]>,
    battle_reactions: Arc<[CurioBattleReactionDefinition]>,
    acquisition_blessings: DivergentUniverseBlessingRuntime,
    fragments: DivergentUniverseCurrencyRuntime,
}

impl DivergentUniverseRuntimeFactory {
    pub fn curio_runtime(
        &self,
    ) -> Result<DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError> {
        Ok(DivergentUniverseCurioRuntime {
            catalog: CompiledCurioCatalog::compile(self.bundle.curio_catalog())?
                .with_evolutions(self.decision_catalog().curio_evolutions())?,
            component_digest: self.bundle.identity().component_digest().bytes(),
            acquisition_effects: self.decision_catalog().curio_acquisitions().into(),
            evolutions: self.decision_catalog().curio_evolutions().into(),
            domain_expiries: self.decision_catalog().curio_domain_expiries().into(),
            domain_grants: self.decision_catalog().curio_domain_grants().into(),
            battle_stats: self.decision_catalog().curio_battle_stats().into(),
            battle_reactions: self.decision_catalog().curio_battle_reactions().into(),
            acquisition_blessings: self
                .blessing_runtime()
                .map_err(DivergentUniverseCurioRuntimeError::Blessing)?,
            fragments: economy::compile(
                self.bundle.service_catalog(),
                self.bundle.progression_catalog(),
                self.bundle.curio_catalog(),
                self.decision_catalog(),
            )
            .map_err(|_| {
                DivergentUniverseCurioRuntimeError::InvalidCatalog("acquisition currency")
            })?
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .clone(),
        })
    }
}

impl DivergentUniverseCurioRuntime {
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseCurioAccuracy {
        DivergentUniverseCurioAccuracy::ExactReleasedIdentitiesStatesEffectsTriggersAndCatalogMembership
    }
    #[must_use]
    pub const fn lifecycle_accuracy(&self) -> DivergentUniverseCurioLifecycleAccuracy {
        DivergentUniverseCurioLifecycleAccuracy::VersionedProjectPolicyAcceptedStableIdTransitionsNotObservedParity
    }
    #[must_use]
    pub fn curios(&self) -> &[DivergentUniverseCurioRuntimeDefinition] {
        &self.catalog.curios
    }
    #[must_use]
    pub fn states(&self) -> &[DivergentUniverseCurioStateRuntime] {
        &self.catalog.states
    }
    #[must_use]
    pub fn groups(&self) -> &[DivergentUniverseCurioGroupPolicy] {
        &self.catalog.groups
    }
    #[must_use]
    pub fn catalog_memberships(&self) -> &[DivergentUniverseCurioCatalogMembershipRuntime] {
        &self.catalog.memberships
    }

    pub fn activate_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        state: &DivergentUniverseCurioStateId,
        trigger: &str,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let definition = self.state(state)?;
        if !definition
            .trigger_kinds()
            .iter()
            .any(|value| value.as_ref() == trigger)
        {
            return Err(DivergentUniverseCurioRuntimeError::UnknownTrigger);
        }
        let mut current = CurioState::read(activity)?;
        self.require_lifecycle(
            &current,
            definition.state_key(),
            DivergentUniverseCurioLifecycleState::Active,
        )?;
        let value = value(&current.activations, definition.state_key())?
            .checked_add(1)
            .ok_or(DivergentUniverseCurioRuntimeError::CounterOverflow)?;
        replace_value(&mut current.activations, definition.state_key(), value)?;
        self.apply(activity, expected_state_hash, ACTIVATE_PROGRAM, current)
    }

    pub fn set_accepted_charges(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        state: &DivergentUniverseCurioStateId,
        charges: u16,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let definition = self.state(state)?;
        if self
            .maximum_charges(definition)
            .is_some_and(|maximum| charges > maximum)
        {
            return Err(DivergentUniverseCurioRuntimeError::ChargeLimitExceeded);
        }
        let mut current = CurioState::read(activity)?;
        self.require_lifecycle(
            &current,
            definition.state_key(),
            DivergentUniverseCurioLifecycleState::Active,
        )?;
        replace_value(
            &mut current.charges,
            definition.state_key(),
            i64::from(charges),
        )?;
        self.apply(activity, expected_state_hash, CHARGES_PROGRAM, current)
    }

    pub fn destroy_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        state: &DivergentUniverseCurioStateId,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        self.change_lifecycle(
            activity,
            expected_state_hash,
            state,
            DivergentUniverseCurioLifecycleState::Active,
            DivergentUniverseCurioLifecycleState::Destroyed,
            DESTROY_PROGRAM,
        )
    }

    pub fn repair_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        state: &DivergentUniverseCurioStateId,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        self.change_lifecycle(
            activity,
            expected_state_hash,
            state,
            DivergentUniverseCurioLifecycleState::Destroyed,
            DivergentUniverseCurioLifecycleState::Active,
            REPAIR_PROGRAM,
        )
    }

    pub fn replace_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        removed: &DivergentUniverseCurioStateId,
        acquired: &DivergentUniverseCurioStateId,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        if removed == acquired {
            return Err(DivergentUniverseCurioRuntimeError::SameState);
        }
        let removed_definition = self.state(removed)?;
        let acquired_definition = self.state(acquired)?;
        if acquired_definition.evolution_owner().is_some() {
            return Err(DivergentUniverseCurioRuntimeError::EvolutionOnly);
        }
        let acquired_curio = require_curio(acquired_definition)?;
        let mut current = CurioState::read(activity)?;
        self.validate_current(&current)?;
        let removed_index = current
            .status
            .binary_search_by_key(&removed_definition.state_key(), |value| value.0)
            .map_err(|_| DivergentUniverseCurioRuntimeError::NotOwned)?;
        if self
            .owned_from_state(&current)?
            .iter()
            .any(|owned| &owned.curio == acquired_curio && &owned.state != removed_definition.id())
        {
            return Err(DivergentUniverseCurioRuntimeError::AlreadyOwned);
        }
        remove_key(&mut current.status, removed_index);
        remove(&mut current.charges, removed_definition.state_key())?;
        remove(&mut current.activations, removed_definition.state_key())?;
        insert(
            &mut current.status,
            acquired_definition.state_key(),
            DivergentUniverseCurioLifecycleState::Active.raw(),
        )?;
        insert(
            &mut current.charges,
            acquired_definition.state_key(),
            i64::from(self.maximum_charges(acquired_definition).unwrap_or(0)),
        )?;
        insert(&mut current.activations, acquired_definition.state_key(), 0)?;
        self.apply(activity, expected_state_hash, REPLACE_PROGRAM, current)
    }

    pub fn reject_unresolved_group_offer(
        &self,
        activity: &GraphActivity,
        expected_state_hash: ActivityStateHash,
        group: &DivergentUniverseCurioGroupId,
    ) -> Result<(), DivergentUniverseCurioRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        if self
            .catalog
            .groups
            .binary_search_by(|candidate| candidate.id().cmp(group))
            .is_err()
        {
            return Err(DivergentUniverseCurioRuntimeError::UnknownGroup);
        }
        Err(DivergentUniverseCurioRuntimeError::NoLegalCandidate)
    }

    pub fn owned(
        &self,
        activity: &GraphActivity,
    ) -> Result<Box<[DivergentUniverseOwnedCurioState]>, DivergentUniverseCurioRuntimeError> {
        self.owned_from_view(&activity.player_view())
    }

    pub(super) fn owned_from_view(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Box<[DivergentUniverseOwnedCurioState]>, DivergentUniverseCurioRuntimeError> {
        self.owned_from_state(&CurioState::read_view(view)?)
    }

    pub fn snapshot(
        &self,
        activity: &GraphActivity,
    ) -> Result<DivergentUniverseCurioSnapshot, DivergentUniverseCurioRuntimeError> {
        let owned = self.owned(activity)?;
        let contributions = owned
            .iter()
            .filter(|owned| owned.lifecycle == DivergentUniverseCurioLifecycleState::Active)
            .map(|owned| {
                let state = self.state(&owned.state)?;
                Ok(DivergentUniverseCurioContribution {
                    state: state.id().clone(),
                    curio: require_curio(state)?.clone(),
                    effect_ids: state.effect_ids().into(),
                    effect_parameters: state.effect_parameters().into(),
                    trigger_kinds: state.trigger_kinds().into(),
                    mechanic_visibility: state.mechanic_visibility().into(),
                    charges: owned.charges,
                    activations: owned.activations,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseCurioRuntimeError>>()?;
        let state_hash = activity.state_hash();
        let digest = snapshot_digest(state_hash, self.component_digest, &contributions)?;
        Ok(DivergentUniverseCurioSnapshot {
            source_state_hash: state_hash,
            contributions: contributions.into_boxed_slice(),
            digest,
        })
    }

    fn change_lifecycle(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        state: &DivergentUniverseCurioStateId,
        expected: DivergentUniverseCurioLifecycleState,
        output: DivergentUniverseCurioLifecycleState,
        program: u32,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let definition = self.state(state)?;
        let mut current = CurioState::read(activity)?;
        self.require_lifecycle(&current, definition.state_key(), expected)?;
        replace_value(&mut current.status, definition.state_key(), output.raw())?;
        self.apply(activity, expected_state_hash, program, current)
    }

    fn require_lifecycle(
        &self,
        current: &CurioState,
        key: u64,
        expected: DivergentUniverseCurioLifecycleState,
    ) -> Result<(), DivergentUniverseCurioRuntimeError> {
        self.validate_current(current)?;
        if DivergentUniverseCurioLifecycleState::from_raw(value(&current.status, key)?)? == expected
        {
            Ok(())
        } else {
            Err(DivergentUniverseCurioRuntimeError::InvalidLifecycle)
        }
    }

    fn validate_current(
        &self,
        current: &CurioState,
    ) -> Result<(), DivergentUniverseCurioRuntimeError> {
        if current.status.len() != current.charges.len()
            || current.status.len() != current.activations.len()
            || current.status.iter().any(|(key, lifecycle)| {
                self.catalog
                    .states
                    .iter()
                    .all(|state| state.state_key() != *key)
                    || DivergentUniverseCurioLifecycleState::from_raw(*lifecycle).is_err()
                    || value(&current.charges, *key).is_err()
                    || value(&current.activations, *key).is_err()
            })
            || current
                .charges
                .iter()
                .any(|(_, value)| *value < 0 || *value > i64::from(u16::MAX))
            || current
                .activations
                .iter()
                .any(|(_, value)| *value < 0 || *value > i64::from(u32::MAX))
        {
            Err(DivergentUniverseCurioRuntimeError::InvalidState)
        } else {
            Ok(())
        }
    }

    fn owned_from_state(
        &self,
        current: &CurioState,
    ) -> Result<Box<[DivergentUniverseOwnedCurioState]>, DivergentUniverseCurioRuntimeError> {
        self.validate_current(current)?;
        current
            .status
            .iter()
            .map(|(key, lifecycle)| {
                let state = self
                    .catalog
                    .states
                    .iter()
                    .find(|state| state.state_key() == *key)
                    .ok_or(DivergentUniverseCurioRuntimeError::InvalidState)?;
                Ok(DivergentUniverseOwnedCurioState {
                    state: state.id().clone(),
                    curio: require_curio(state)?.clone(),
                    lifecycle: DivergentUniverseCurioLifecycleState::from_raw(*lifecycle)?,
                    charges: u16::try_from(value(&current.charges, *key)?)
                        .map_err(|_| DivergentUniverseCurioRuntimeError::InvalidState)?,
                    activations: u32::try_from(value(&current.activations, *key)?)
                        .map_err(|_| DivergentUniverseCurioRuntimeError::InvalidState)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    fn state(
        &self,
        id: &DivergentUniverseCurioStateId,
    ) -> Result<&DivergentUniverseCurioStateRuntime, DivergentUniverseCurioRuntimeError> {
        self.catalog
            .states
            .binary_search_by(|state| state.id().cmp(id))
            .ok()
            .and_then(|index| self.catalog.states.get(index))
            .ok_or(DivergentUniverseCurioRuntimeError::UnknownState)
    }

    fn apply(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        raw_program: u32,
        state: CurioState,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        self.apply_operations(
            activity,
            expected_state_hash,
            raw_program,
            state.into_operations(),
        )
    }

    fn apply_operations(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        raw_program: u32,
        operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseCurioCommandResolution, DivergentUniverseCurioRuntimeError> {
        let program = ActivityProgramDefinition::new(program_id(raw_program), operations)
            .map_err(|_| DivergentUniverseCurioRuntimeError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseCurioRuntimeError::Activity)?;
        Ok(DivergentUniverseCurioCommandResolution {
            owned: self.owned(activity)?,
            events,
            state_hash: activity.state_hash(),
        })
    }
}

#[derive(Clone)]
pub(in crate::divergent_universe) struct CurioState {
    status: Box<[(u64, i64)]>,
    charges: Box<[(u64, i64)]>,
    activations: Box<[(u64, i64)]>,
}

fn require_curio(
    state: &DivergentUniverseCurioStateRuntime,
) -> Result<&DivergentUniverseCurioId, DivergentUniverseCurioRuntimeError> {
    state
        .curio()
        .or_else(|| state.evolution_owner())
        .ok_or(DivergentUniverseCurioRuntimeError::MissingCurioIdentity)
}

impl CurioState {
    pub(in crate::divergent_universe) fn apply_expansion_allowance(
        &mut self,
        change: ExpansionAllowance,
    ) -> Result<(), DivergentUniverseCurioRuntimeError> {
        if self
            .status
            .binary_search_by_key(&change.key, |entry| entry.0)
            .is_err()
        {
            // A simultaneous sacrifice/replacement must not resurrect the old holding.
            return Ok(());
        }
        if change.remaining == 0 {
            remove(&mut self.status, change.key)?;
            remove(&mut self.charges, change.key)?;
            remove(&mut self.activations, change.key)?;
        } else {
            replace_value(&mut self.charges, change.key, i64::from(change.remaining))?;
            replace_value(
                &mut self.activations,
                change.key,
                i64::from(change.activations),
            )?;
        }
        Ok(())
    }
    fn read(activity: &GraphActivity) -> Result<Self, DivergentUniverseCurioRuntimeError> {
        Self::read_view(&activity.player_view())
    }

    pub(in crate::divergent_universe) fn read_view(
        view: &ActivityPlayerView,
    ) -> Result<Self, DivergentUniverseCurioRuntimeError> {
        Ok(Self {
            status: counter(view, CURIO_STATES_SLOT)?,
            charges: counter(view, CURIO_CHARGES_SLOT)?,
            activations: counter(view, CURIO_ACTIVATIONS_SLOT)?,
        })
    }

    pub(in crate::divergent_universe) fn into_operations(self) -> Vec<ActivityOperation> {
        vec![
            ActivityOperation::SetCounterMap {
                slot: CURIO_STATES_SLOT,
                values: self.status,
            },
            ActivityOperation::SetCounterMap {
                slot: CURIO_CHARGES_SLOT,
                values: self.charges,
            },
            ActivityOperation::SetCounterMap {
                slot: CURIO_ACTIVATIONS_SLOT,
                values: self.activations,
            },
        ]
    }
}

fn counter(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Box<[(u64, i64)]>, DivergentUniverseCurioRuntimeError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(values) => Some(values.clone()),
            _ => None,
        })
        .ok_or(DivergentUniverseCurioRuntimeError::InvalidState)
}

fn insert(
    values: &mut Box<[(u64, i64)]>,
    key: u64,
    value: i64,
) -> Result<(), DivergentUniverseCurioRuntimeError> {
    let mut updated = values.to_vec();
    let position = updated
        .binary_search_by_key(&key, |entry| entry.0)
        .map_or_else(|position| position, |_| usize::MAX);
    if position == usize::MAX {
        return Err(DivergentUniverseCurioRuntimeError::AlreadyOwned);
    }
    updated.insert(position, (key, value));
    *values = updated.into_boxed_slice();
    Ok(())
}

fn replace_value(
    values: &mut Box<[(u64, i64)]>,
    key: u64,
    value: i64,
) -> Result<(), DivergentUniverseCurioRuntimeError> {
    let index = values
        .binary_search_by_key(&key, |entry| entry.0)
        .map_err(|_| DivergentUniverseCurioRuntimeError::NotOwned)?;
    values[index].1 = value;
    Ok(())
}

fn remove(
    values: &mut Box<[(u64, i64)]>,
    key: u64,
) -> Result<(), DivergentUniverseCurioRuntimeError> {
    let index = values
        .binary_search_by_key(&key, |entry| entry.0)
        .map_err(|_| DivergentUniverseCurioRuntimeError::NotOwned)?;
    remove_key(values, index);
    Ok(())
}

fn remove_key(values: &mut Box<[(u64, i64)]>, index: usize) {
    let mut updated = values.to_vec();
    updated.remove(index);
    *values = updated.into_boxed_slice();
}

fn value(values: &[(u64, i64)], key: u64) -> Result<i64, DivergentUniverseCurioRuntimeError> {
    values
        .binary_search_by_key(&key, |entry| entry.0)
        .ok()
        .and_then(|index| values.get(index))
        .map(|entry| entry.1)
        .ok_or(DivergentUniverseCurioRuntimeError::NotOwned)
}

fn snapshot_digest(
    state_hash: ActivityStateHash,
    component_digest: [u8; 32],
    contributions: &[DivergentUniverseCurioContribution],
) -> Result<DivergentUniverseCurioSnapshotDigest, DivergentUniverseCurioRuntimeError> {
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.divergent-universe.curio-snapshot.v1");
    hash.update(component_digest);
    hash.update(state_hash.bytes());
    push_len(&mut hash, contributions.len())?;
    for contribution in contributions {
        push_text(&mut hash, contribution.state.as_str())?;
        push_text(&mut hash, contribution.curio.as_str())?;
        push_text(&mut hash, &contribution.mechanic_visibility)?;
        hash.update(contribution.charges.to_le_bytes());
        hash.update(contribution.activations.to_le_bytes());
        push_texts(&mut hash, &contribution.effect_ids)?;
        push_texts(&mut hash, &contribution.trigger_kinds)?;
        push_len(&mut hash, contribution.effect_parameters.len())?;
        for parameter in &contribution.effect_parameters {
            hash.update(parameter.coefficient().to_le_bytes());
            hash.update([parameter.scale()]);
        }
    }
    Ok(DivergentUniverseCurioSnapshotDigest(hash.finalize()))
}

fn push_texts(
    hash: &mut CanonicalDigestBuilder,
    values: &[Box<str>],
) -> Result<(), DivergentUniverseCurioRuntimeError> {
    push_len(hash, values.len())?;
    for value in values {
        push_text(hash, value)?;
    }
    Ok(())
}
fn push_text(
    hash: &mut CanonicalDigestBuilder,
    value: &str,
) -> Result<(), DivergentUniverseCurioRuntimeError> {
    push_len(hash, value.len())?;
    hash.update(value.as_bytes());
    Ok(())
}
fn push_len(
    hash: &mut CanonicalDigestBuilder,
    value: usize,
) -> Result<(), DivergentUniverseCurioRuntimeError> {
    hash.update(
        u64::try_from(value)
            .map_err(|_| DivergentUniverseCurioRuntimeError::InvalidState)?
            .to_le_bytes(),
    );
    Ok(())
}
fn validate_hash(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseCurioRuntimeError> {
    if activity.state_hash() == expected {
        Ok(())
    } else {
        Err(DivergentUniverseCurioRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ))
    }
}
fn program_id(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).expect("static Curio program ID is non-zero")
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurioRuntimeError {
    InvalidCatalog(&'static str),
    InvalidState,
    InvalidProgram,
    UnknownState,
    MissingCurioIdentity,
    EvolutionOnly,
    InvalidEvolution,
    UnknownGroup,
    UnknownTrigger,
    NoLegalCandidate,
    InvalidAcquisitionCount,
    AlreadyOwned,
    NotOwned,
    SameState,
    InvalidLifecycle,
    ChargeLimitExceeded,
    CounterOverflow,
    Blessing(DivergentUniverseBlessingRuntimeError),
    Activity(GraphActivityCommandError),
}

impl From<CurioCatalogCompileError> for DivergentUniverseCurioRuntimeError {
    fn from(value: CurioCatalogCompileError) -> Self {
        Self::InvalidCatalog(value.0)
    }
}
impl core::fmt::Display for DivergentUniverseCurioRuntimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Curio runtime error: {self:?}"
        )
    }
}
impl std::error::Error for DivergentUniverseCurioRuntimeError {}
