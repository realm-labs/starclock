//! Atomic Divergent Blessing offers and ownership transitions.

#[path = "blessing_acquisition.rs"]
mod acquisition;

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityPlayerView, ActivityProgramDefinition,
    ActivityProgramId, ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_blessing_catalog::{
    DivergentUniverseBlessingGroupId, DivergentUniverseBlessingId,
};

use super::DivergentUniverseRuntimeFactory;
use super::blessing_catalog::{
    CompiledBlessingCatalog, DivergentUniverseBlessingAccuracy,
    DivergentUniverseBlessingEnhancementRuntime, DivergentUniverseBlessingGroupRuntime,
    DivergentUniverseBlessingOfferCandidate, DivergentUniverseBlessingRuntimeDefinition,
};
use super::equation_progress::expansion::{EquationExpansionRewards, ExpansionOfferConsumption};
use super::equation_progress::{
    DivergentUniverseEquationProgressError, DivergentUniverseEquationProgressRuntime,
};
use super::state::{
    BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT, BLESSINGS_SLOT, EQUATIONS_SLOT,
};

const BEGIN_OFFER_PROGRAM: u32 = 22_441;
const ACQUIRE_PROGRAM: u32 = 22_442;
const ENHANCE_PROGRAM: u32 = 22_443;
const REPLACE_PROGRAM: u32 = 22_444;
const REWRITE_PATH_PROGRAM: u32 = 22_445;
const ACCEPTED_REPLACE_MANY_PROGRAM: u32 = 22_446;
const ACCEPTED_REWRITE_PATH_MANY_PROGRAM: u32 = 22_447;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBlessingServiceRewriteKind {
    Replace,
    RewritePath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAcceptedBlessingRewrite {
    removed: DivergentUniverseBlessingId,
    acquired: DivergentUniverseBlessingId,
}

impl DivergentUniverseAcceptedBlessingRewrite {
    pub fn new(
        removed: DivergentUniverseBlessingId,
        acquired: DivergentUniverseBlessingId,
    ) -> Result<Self, DivergentUniverseBlessingRuntimeError> {
        if removed == acquired {
            Err(DivergentUniverseBlessingRuntimeError::SameBlessing)
        } else {
            Ok(Self { removed, acquired })
        }
    }
    #[must_use]
    pub const fn removed(&self) -> &DivergentUniverseBlessingId {
        &self.removed
    }
    #[must_use]
    pub const fn acquired(&self) -> &DivergentUniverseBlessingId {
        &self.acquired
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingOfferObservation {
    group: DivergentUniverseBlessingGroupId,
    candidates: Box<[DivergentUniverseBlessingOfferCandidate]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseBlessingOfferObservation {
    #[must_use]
    pub const fn group(&self) -> &DivergentUniverseBlessingGroupId {
        &self.group
    }
    #[must_use]
    pub fn candidates(&self) -> &[DivergentUniverseBlessingOfferCandidate] {
        &self.candidates
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOwnedBlessing {
    blessing: DivergentUniverseBlessingId,
    level: u16,
}

impl DivergentUniverseOwnedBlessing {
    #[must_use]
    pub const fn blessing(&self) -> &DivergentUniverseBlessingId {
        &self.blessing
    }
    #[must_use]
    pub const fn level(&self) -> u16 {
        self.level
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingCommandResolution {
    owned: Box<[DivergentUniverseOwnedBlessing]>,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseBlessingCommandResolution {
    #[must_use]
    pub fn owned(&self) -> &[DivergentUniverseOwnedBlessing] {
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
pub struct DivergentUniverseBlessingRuntime {
    catalog: CompiledBlessingCatalog,
    progress: Arc<DivergentUniverseEquationProgressRuntime>,
    expansion: Arc<EquationExpansionRewards>,
}

impl DivergentUniverseRuntimeFactory {
    pub fn blessing_runtime(
        &self,
    ) -> Result<DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError> {
        let catalog =
            CompiledBlessingCatalog::compile(self.bundle.blessing_catalog(), stable_group_key)
                .map_err(|_| DivergentUniverseBlessingRuntimeError::InvalidCatalog)?;
        let progress = DivergentUniverseEquationProgressRuntime::compile(
            self.bundle.equation_catalog(),
            self.bundle.blessing_catalog(),
        )
        .map_err(DivergentUniverseBlessingRuntimeError::Progress)?;
        Ok(DivergentUniverseBlessingRuntime {
            catalog,
            progress: Arc::new(progress),
            expansion: Arc::new(
                EquationExpansionRewards::compile(self)
                    .map_err(DivergentUniverseBlessingRuntimeError::Activity)?,
            ),
        })
    }
}

impl DivergentUniverseBlessingRuntime {
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseBlessingAccuracy {
        DivergentUniverseBlessingAccuracy::ExactReleasedPathsIdentitiesLevelsEnhancementsAndClosedGroups
    }
    #[must_use]
    pub const fn path_count(&self) -> u16 {
        self.catalog.path_count
    }
    #[must_use]
    pub fn blessings(&self) -> &[DivergentUniverseBlessingRuntimeDefinition] {
        &self.catalog.blessings
    }
    #[must_use]
    pub fn enhancements(&self) -> &[DivergentUniverseBlessingEnhancementRuntime] {
        &self.catalog.enhancements
    }
    #[must_use]
    pub fn groups(&self) -> &[DivergentUniverseBlessingGroupRuntime] {
        &self.catalog.groups
    }

    /// Publishes every currently legal member of an exact closed group in the
    /// group's released source order. No selection draw is invented because
    /// the released group rows explicitly leave their weight program unknown.
    pub fn begin_group_offer(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        group: &DivergentUniverseBlessingGroupId,
    ) -> Result<DivergentUniverseBlessingOfferObservation, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        if self.offer_observation(activity)?.is_some() {
            return Err(DivergentUniverseBlessingRuntimeError::OfferAlreadyActive);
        }
        self.validate_clean_progress(activity)?;
        let group = self.group(group)?;
        let state = BlessingState::read(activity)?;
        self.validate_owned(&state.owned)?;
        let candidates = legal_candidates(group, &state.owned);
        if candidates.is_empty() {
            return Err(DivergentUniverseBlessingRuntimeError::NoLegalCandidate);
        }
        let mut stored = candidates
            .iter()
            .map(|candidate| (candidate.state_key(), i64::from(candidate.level())))
            .collect::<Vec<_>>();
        stored.sort_unstable_by_key(|candidate| candidate.0);
        let program = ActivityProgramDefinition::new(
            program_id(BEGIN_OFFER_PROGRAM),
            vec![
                ActivityOperation::SetSlot {
                    slot: BLESSING_OFFER_SOURCE_SLOT,
                    value: literal(ActivityValue::OptionalId(Some(group.state_key()))),
                },
                ActivityOperation::SetCounterMap {
                    slot: BLESSING_OFFERS_SLOT,
                    values: stored.into_boxed_slice(),
                },
            ],
        )
        .map_err(|_| DivergentUniverseBlessingRuntimeError::InvalidProgram)?;
        activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseBlessingRuntimeError::Activity)?;
        self.offer_observation(activity)?
            .filter(|observation| observation.candidates.as_ref() == candidates.as_slice())
            .ok_or(DivergentUniverseBlessingRuntimeError::InvalidState)
    }

    pub fn acquire(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        blessing: &DivergentUniverseBlessingId,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        self.accept_level(
            activity,
            expected_state_hash,
            blessing,
            1,
            OwnershipTransition::Acquire,
            true,
        )
    }

    pub fn enhance(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        blessing: &DivergentUniverseBlessingId,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        self.accept_level(
            activity,
            expected_state_hash,
            blessing,
            2,
            OwnershipTransition::Enhance,
            true,
        )
    }

    /// Executes the catalog's exact base-to-enhanced rewrite for an explicitly
    /// accepted owned identity without inventing a service selector or cost.
    pub fn enhance_accepted_identity(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        blessing: &DivergentUniverseBlessingId,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        self.accept_level(
            activity,
            expected_state_hash,
            blessing,
            2,
            OwnershipTransition::Enhance,
            false,
        )
    }

    /// Executes an accepted Workbench enhancement and its service-owned
    /// currency/receipt operations in the same generic Activity transaction.
    pub(super) fn enhance_accepted_identity_with_operations(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        blessing: &DivergentUniverseBlessingId,
        mut service_operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        self.validate_clean_progress(activity)?;
        let definition = self.blessing(blessing)?;
        if self.offer_observation(activity)?.is_some() {
            return Err(DivergentUniverseBlessingRuntimeError::OfferAlreadyActive);
        }
        let state = BlessingState::read(activity)?;
        let mut owned = state.owned.to_vec();
        let position = owned
            .binary_search_by_key(&definition.state_key(), |value| value.0)
            .map_err(|_| DivergentUniverseBlessingRuntimeError::NotOwned)?;
        if owned[position].1 != 1 {
            return Err(DivergentUniverseBlessingRuntimeError::NotBaseLevel);
        }
        owned[position].1 = 2;
        let mut operations = vec![ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: owned.into_boxed_slice(),
        }];
        operations.append(&mut service_operations);
        self.apply(activity, expected_state_hash, ENHANCE_PROGRAM, operations)
    }

    /// Replaces one explicitly selected owned identity with one offered base
    /// identity. Hidden candidates, cost and service availability are left to
    /// the owning service batch; this boundary executes only the accepted pair.
    pub fn replace(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        removed: &DivergentUniverseBlessingId,
        acquired: &DivergentUniverseBlessingId,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        self.replace_identity(
            activity,
            expected_state_hash,
            removed,
            acquired,
            REPLACE_PROGRAM,
        )
    }

    /// Executes the accepted identity pair for the path-rewrite shape. The
    /// service selector remains an explicit replaceable policy owned by P4-B5.
    pub fn rewrite_path(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        removed: &DivergentUniverseBlessingId,
        acquired: &DivergentUniverseBlessingId,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        self.replace_identity(
            activity,
            expected_state_hash,
            removed,
            acquired,
            REWRITE_PATH_PROGRAM,
        )
    }

    /// Executes one or more caller-accepted service rewrites in a single
    /// Activity transaction. All removals are validated against the same
    /// pre-state, all accepted outputs begin at base level, and Equation
    /// contribution refresh observes only the final identity set.
    pub fn apply_accepted_rewrites(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        kind: DivergentUniverseBlessingServiceRewriteKind,
        rewrites: &[DivergentUniverseAcceptedBlessingRewrite],
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        self.validate_clean_progress(activity)?;
        if self.offer_observation(activity)?.is_some() {
            return Err(DivergentUniverseBlessingRuntimeError::OfferAlreadyActive);
        }
        if rewrites.is_empty() {
            return Err(DivergentUniverseBlessingRuntimeError::NoAcceptedRewrite);
        }
        let mut compiled = rewrites
            .iter()
            .map(|rewrite| {
                Ok((
                    self.blessing(&rewrite.removed)?.state_key(),
                    self.blessing(&rewrite.acquired)?.state_key(),
                ))
            })
            .collect::<Result<Vec<_>, DivergentUniverseBlessingRuntimeError>>()?;
        compiled.sort_unstable();
        if compiled.windows(2).any(|pair| pair[0].0 == pair[1].0)
            || duplicate(compiled.iter().map(|value| value.1))
            || compiled.iter().any(|(removed, acquired)| {
                removed == acquired
                    || compiled
                        .binary_search_by_key(acquired, |value| value.0)
                        .is_ok()
            })
        {
            return Err(DivergentUniverseBlessingRuntimeError::InvalidRewriteSet);
        }
        let state = BlessingState::read(activity)?;
        let mut owned = state.owned.to_vec();
        for (removed, _) in &compiled {
            let position = owned
                .binary_search_by_key(removed, |value| value.0)
                .map_err(|_| DivergentUniverseBlessingRuntimeError::NotOwned)?;
            owned.remove(position);
        }
        for (_, acquired) in &compiled {
            let position = match owned.binary_search_by_key(acquired, |value| value.0) {
                Ok(_) => return Err(DivergentUniverseBlessingRuntimeError::AlreadyOwned),
                Err(position) => position,
            };
            owned.insert(position, (*acquired, 1));
        }
        let mut operations = vec![ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: owned.clone().into_boxed_slice(),
        }];
        operations.extend(
            self.progress
                .refresh_operations_for_inputs(&activity.player_view(), &state.equations, &owned)
                .map_err(DivergentUniverseBlessingRuntimeError::Progress)?,
        );
        self.apply_with_expansion(
            activity,
            expected_state_hash,
            match kind {
                DivergentUniverseBlessingServiceRewriteKind::Replace => {
                    ACCEPTED_REPLACE_MANY_PROGRAM
                }
                DivergentUniverseBlessingServiceRewriteKind::RewritePath => {
                    ACCEPTED_REWRITE_PATH_MANY_PROGRAM
                }
            },
            operations,
            &owned,
            ExpansionOfferConsumption::None,
        )
    }

    pub fn offer_observation(
        &self,
        activity: &GraphActivity,
    ) -> Result<
        Option<DivergentUniverseBlessingOfferObservation>,
        DivergentUniverseBlessingRuntimeError,
    > {
        self.offer_observation_from_view(&activity.player_view())
    }

    fn offer_observation_from_view(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<
        Option<DivergentUniverseBlessingOfferObservation>,
        DivergentUniverseBlessingRuntimeError,
    > {
        let state = BlessingState::read_view(view)?;
        self.validate_owned(&state.owned)?;
        let Some(source) = state.source else {
            return if state.offered.is_empty() {
                Ok(None)
            } else {
                Err(DivergentUniverseBlessingRuntimeError::InvalidState)
            };
        };
        let group = self
            .catalog
            .groups
            .iter()
            .find(|group| group.state_key() == source)
            .ok_or(DivergentUniverseBlessingRuntimeError::InvalidState)?;
        let candidates = legal_candidates(group, &state.owned);
        let mut expected = candidates
            .iter()
            .map(|candidate| (candidate.state_key(), i64::from(candidate.level())))
            .collect::<Vec<_>>();
        expected.sort_unstable_by_key(|candidate| candidate.0);
        if expected.as_slice() != state.offered.as_ref() {
            return Err(DivergentUniverseBlessingRuntimeError::InvalidState);
        }
        Ok(Some(DivergentUniverseBlessingOfferObservation {
            group: group.id().clone(),
            candidates: candidates.into_boxed_slice(),
            state_hash: view.state_hash(),
        }))
    }

    pub fn owned(
        &self,
        activity: &GraphActivity,
    ) -> Result<Box<[DivergentUniverseOwnedBlessing]>, DivergentUniverseBlessingRuntimeError> {
        let state = BlessingState::read(activity)?;
        self.owned_from_values(&state.owned)
    }

    fn accept_level(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        blessing: &DivergentUniverseBlessingId,
        level: u16,
        transition: OwnershipTransition,
        require_offer: bool,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        self.validate_clean_progress(activity)?;
        let definition = self.blessing(blessing)?;
        let offer = self.offer_observation(activity)?;
        if require_offer {
            let offer = offer.ok_or(DivergentUniverseBlessingRuntimeError::NoActiveOffer)?;
            if !offer.candidates.iter().any(|candidate| {
                candidate.state_key() == definition.state_key() && candidate.level() == level
            }) {
                return Err(DivergentUniverseBlessingRuntimeError::NotOffered);
            }
        } else if offer.is_some() {
            return Err(DivergentUniverseBlessingRuntimeError::OfferAlreadyActive);
        }
        let state = BlessingState::read(activity)?;
        let mut owned = state.owned.to_vec();
        match transition {
            OwnershipTransition::Acquire => {
                if owned
                    .binary_search_by_key(&definition.state_key(), |value| value.0)
                    .is_ok()
                {
                    return Err(DivergentUniverseBlessingRuntimeError::AlreadyOwned);
                }
                let position = owned
                    .binary_search_by_key(&definition.state_key(), |value| value.0)
                    .expect_err("unowned Blessing has insertion position");
                owned.insert(position, (definition.state_key(), 1));
            }
            OwnershipTransition::Enhance => {
                let position = owned
                    .binary_search_by_key(&definition.state_key(), |value| value.0)
                    .map_err(|_| DivergentUniverseBlessingRuntimeError::NotOwned)?;
                if owned[position].1 != 1 {
                    return Err(DivergentUniverseBlessingRuntimeError::NotBaseLevel);
                }
                owned[position].1 = 2;
            }
        }
        let mut operations = vec![ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: owned.clone().into_boxed_slice(),
        }];
        if transition == OwnershipTransition::Acquire {
            operations.extend(
                self.progress
                    .refresh_operations_for_inputs(
                        &activity.player_view(),
                        &state.equations,
                        &owned,
                    )
                    .map_err(DivergentUniverseBlessingRuntimeError::Progress)?,
            );
        }
        if require_offer {
            operations.extend(clear_offer());
        }
        self.apply_with_expansion(
            activity,
            expected_state_hash,
            if transition == OwnershipTransition::Acquire {
                ACQUIRE_PROGRAM
            } else {
                ENHANCE_PROGRAM
            },
            operations,
            &owned,
            if require_offer {
                ExpansionOfferConsumption::Internal
            } else {
                ExpansionOfferConsumption::None
            },
        )
    }

    fn replace_identity(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        removed: &DivergentUniverseBlessingId,
        acquired: &DivergentUniverseBlessingId,
        program: u32,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        validate_hash(activity, expected_state_hash)?;
        self.validate_clean_progress(activity)?;
        let removed = self.blessing(removed)?;
        let acquired = self.blessing(acquired)?;
        if removed.state_key() == acquired.state_key() {
            return Err(DivergentUniverseBlessingRuntimeError::SameBlessing);
        }
        let offer = self
            .offer_observation(activity)?
            .ok_or(DivergentUniverseBlessingRuntimeError::NoActiveOffer)?;
        if !offer.candidates.iter().any(|candidate| {
            candidate.state_key() == acquired.state_key() && candidate.level() == 1
        }) {
            return Err(DivergentUniverseBlessingRuntimeError::NotOffered);
        }
        let state = BlessingState::read(activity)?;
        let mut owned = state.owned.to_vec();
        let removed_position = owned
            .binary_search_by_key(&removed.state_key(), |value| value.0)
            .map_err(|_| DivergentUniverseBlessingRuntimeError::NotOwned)?;
        owned.remove(removed_position);
        let acquired_position =
            match owned.binary_search_by_key(&acquired.state_key(), |value| value.0) {
                Ok(_) => return Err(DivergentUniverseBlessingRuntimeError::AlreadyOwned),
                Err(position) => position,
            };
        owned.insert(acquired_position, (acquired.state_key(), 1));
        let mut operations = vec![ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: owned.clone().into_boxed_slice(),
        }];
        operations.extend(
            self.progress
                .refresh_operations_for_inputs(&activity.player_view(), &state.equations, &owned)
                .map_err(DivergentUniverseBlessingRuntimeError::Progress)?,
        );
        operations.extend(clear_offer());
        self.apply_with_expansion(
            activity,
            expected_state_hash,
            program,
            operations,
            &owned,
            ExpansionOfferConsumption::Internal,
        )
    }

    fn apply(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        raw_program: u32,
        operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseBlessingCommandResolution, DivergentUniverseBlessingRuntimeError>
    {
        let program = ActivityProgramDefinition::new(program_id(raw_program), operations)
            .map_err(|_| DivergentUniverseBlessingRuntimeError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseBlessingRuntimeError::Activity)?;
        Ok(DivergentUniverseBlessingCommandResolution {
            owned: self.owned(activity)?,
            events,
            state_hash: activity.state_hash(),
        })
    }

    fn validate_clean_progress(
        &self,
        activity: &GraphActivity,
    ) -> Result<(), DivergentUniverseBlessingRuntimeError> {
        self.progress
            .observations(activity)
            .map(|_| ())
            .map_err(DivergentUniverseBlessingRuntimeError::Progress)
    }

    fn validate_owned(
        &self,
        values: &[(u64, i64)],
    ) -> Result<(), DivergentUniverseBlessingRuntimeError> {
        if values.iter().any(|(key, level)| {
            (*level != 1 && *level != 2)
                || self
                    .catalog
                    .blessings
                    .iter()
                    .all(|blessing| blessing.state_key() != *key)
        }) {
            Err(DivergentUniverseBlessingRuntimeError::InvalidState)
        } else {
            Ok(())
        }
    }

    fn owned_from_values(
        &self,
        values: &[(u64, i64)],
    ) -> Result<Box<[DivergentUniverseOwnedBlessing]>, DivergentUniverseBlessingRuntimeError> {
        self.validate_owned(values)?;
        values
            .iter()
            .map(|(key, level)| {
                let blessing = self
                    .catalog
                    .blessings
                    .iter()
                    .find(|blessing| blessing.state_key() == *key)
                    .ok_or(DivergentUniverseBlessingRuntimeError::InvalidState)?;
                Ok(DivergentUniverseOwnedBlessing {
                    blessing: blessing.id().clone(),
                    level: u16::try_from(*level)
                        .map_err(|_| DivergentUniverseBlessingRuntimeError::InvalidState)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    fn blessing(
        &self,
        id: &DivergentUniverseBlessingId,
    ) -> Result<&DivergentUniverseBlessingRuntimeDefinition, DivergentUniverseBlessingRuntimeError>
    {
        self.catalog
            .blessings
            .binary_search_by(|blessing| blessing.id().cmp(id))
            .ok()
            .map(|index| &self.catalog.blessings[index])
            .ok_or(DivergentUniverseBlessingRuntimeError::UnknownBlessing)
    }

    fn group(
        &self,
        id: &DivergentUniverseBlessingGroupId,
    ) -> Result<&DivergentUniverseBlessingGroupRuntime, DivergentUniverseBlessingRuntimeError> {
        self.catalog
            .groups
            .binary_search_by(|group| group.id().cmp(id))
            .ok()
            .map(|index| &self.catalog.groups[index])
            .ok_or(DivergentUniverseBlessingRuntimeError::UnknownGroup)
    }
}

fn legal_candidates(
    group: &DivergentUniverseBlessingGroupRuntime,
    owned: &[(u64, i64)],
) -> Vec<DivergentUniverseBlessingOfferCandidate> {
    group
        .candidates()
        .iter()
        .filter(|candidate| {
            let current = owned
                .binary_search_by_key(&candidate.state_key(), |value| value.0)
                .ok()
                .map_or(0, |index| owned[index].1);
            match candidate.level() {
                1 => current == 0,
                2 => current == 1,
                _ => false,
            }
        })
        .cloned()
        .collect()
}

fn duplicate(values: impl Iterator<Item = u64>) -> bool {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    values.windows(2).any(|pair| pair[0] == pair[1])
}

struct BlessingState {
    owned: Box<[(u64, i64)]>,
    offered: Box<[(u64, i64)]>,
    source: Option<u64>,
    equations: Box<[u64]>,
}

impl BlessingState {
    fn read(activity: &GraphActivity) -> Result<Self, DivergentUniverseBlessingRuntimeError> {
        Self::read_view(&activity.player_view())
    }

    fn read_view(view: &ActivityPlayerView) -> Result<Self, DivergentUniverseBlessingRuntimeError> {
        Ok(Self {
            owned: counter(view, BLESSINGS_SLOT)?,
            offered: counter(view, BLESSING_OFFERS_SLOT)?,
            source: optional(view, BLESSING_OFFER_SOURCE_SLOT)?,
            equations: set(view, EQUATIONS_SLOT)?,
        })
    }
}

fn counter(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Box<[(u64, i64)]>, DivergentUniverseBlessingRuntimeError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(values) => Some(values.clone()),
            _ => None,
        })
        .ok_or(DivergentUniverseBlessingRuntimeError::InvalidState)
}

fn set(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Box<[u64]>, DivergentUniverseBlessingRuntimeError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::OrderedIdSet(values) => Some(values.clone()),
            _ => None,
        })
        .ok_or(DivergentUniverseBlessingRuntimeError::InvalidState)
}

fn optional(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Option<u64>, DivergentUniverseBlessingRuntimeError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::OptionalId(value) => Some(*value),
            _ => None,
        })
        .ok_or(DivergentUniverseBlessingRuntimeError::InvalidState)
}

fn clear_offer() -> [ActivityOperation; 2] {
    [
        ActivityOperation::SetCounterMap {
            slot: BLESSING_OFFERS_SLOT,
            values: Box::new([]),
        },
        ActivityOperation::SetSlot {
            slot: BLESSING_OFFER_SOURCE_SLOT,
            value: literal(ActivityValue::OptionalId(None)),
        },
    ]
}

fn validate_hash(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseBlessingRuntimeError> {
    if activity.state_hash() == expected {
        Ok(())
    } else {
        Err(DivergentUniverseBlessingRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ))
    }
}

fn stable_group_key(value: &str) -> u64 {
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.divergent-universe.blessing-group-runtime-key.v1");
    hash.update(
        u64::try_from(value.len())
            .expect("stable ID length fits u64")
            .to_le_bytes(),
    );
    hash.update(value.as_bytes());
    let bytes = hash.finalize();
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(raw).max(1)
}

fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}

fn program_id(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).expect("static Blessing program ID is non-zero")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OwnershipTransition {
    Acquire,
    Enhance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBlessingRuntimeError {
    InvalidCatalog,
    UnknownGroup,
    UnknownBlessing,
    OfferAlreadyActive,
    NoActiveOffer,
    NoLegalCandidate,
    InvalidAcquisitionCount,
    NotOffered,
    AlreadyOwned,
    NotOwned,
    NotBaseLevel,
    SameBlessing,
    NoAcceptedRewrite,
    InvalidRewriteSet,
    InvalidState,
    InvalidProgram,
    Progress(DivergentUniverseEquationProgressError),
    Activity(GraphActivityCommandError),
}

impl core::fmt::Display for DivergentUniverseBlessingRuntimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Blessing runtime error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseBlessingRuntimeError {}
