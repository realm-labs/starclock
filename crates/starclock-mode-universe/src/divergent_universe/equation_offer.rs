//! Deterministic Equation offer and ownership command boundaries.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityPlayerView,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel, ActivityRngStreams,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCatalog, DivergentUniverseEquationId, DivergentUniverseEquationOfferId,
};

use super::{
    DivergentUniverseRuntimeFactory,
    equation_grants::EquationGrants,
    equation_progress::{
        DivergentUniverseEquationProgressError, DivergentUniverseEquationProgressRuntime,
    },
    equation_transition::{
        DivergentUniverseEquationTransitionAccuracy, DivergentUniverseEquationTransitionError,
        DivergentUniverseEquationTransitionPolicy, DivergentUniverseEquationTransitionRuntime,
    },
    state::{
        BLESSINGS_SLOT, EQUATION_OFFER_SOURCE_SLOT, EQUATION_OFFERS_SLOT,
        EQUATION_REROLL_COUNT_SLOT, EQUATIONS_SLOT,
    },
};

const OFFER_PROGRAM: u32 = 22_401;
const REROLL_PROGRAM: u32 = 22_402;
const ACQUIRE_PROGRAM: u32 = 22_403;
const REPLACE_PROGRAM: u32 = 22_404;
const DISCARD_PROGRAM: u32 = 22_405;
const OFFER_PURPOSE_BASE: u16 = 0x5100;
const OFFER_WIDTH: u16 = 3;
const REROLL_LIMIT: u8 = 1;

/// Accuracy of the executable policy used for released RandomID rows whose
/// candidate membership and weights are not published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationOfferAccuracy {
    /// All exact Version 4.4 Equations that are not owned are eligible, ordered
    /// by stable ID, uniformly weighted, offered three at a time, and may be
    /// rerolled once. This is replaceable project policy, not observed parity.
    VersionedProjectPolicyUniformUnownedThreeOneReroll,
}

/// One executable projection of an exact released Equation RandomID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationOfferPolicy {
    id: DivergentUniverseEquationOfferId,
    state_key: u64,
    purpose: u16,
}

impl DivergentUniverseEquationOfferPolicy {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseEquationOfferId {
        &self.id
    }

    #[must_use]
    pub const fn purpose(&self) -> u16 {
        self.purpose
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EquationProjection {
    id: DivergentUniverseEquationId,
    state_key: u64,
}

/// Immutable executable projection for all 136 released Equation RandomIDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationOfferRuntime {
    offers: Arc<[DivergentUniverseEquationOfferPolicy]>,
    equations: Arc<[EquationProjection]>,
    progress: Arc<DivergentUniverseEquationProgressRuntime>,
    transitions: Arc<DivergentUniverseEquationTransitionRuntime>,
    grants: Arc<EquationGrants>,
}

/// Player-visible current Equation offer. Only these candidate IDs may be
/// passed to acquire or replacement commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationOfferObservation {
    offer: DivergentUniverseEquationOfferId,
    candidates: Box<[DivergentUniverseEquationId]>,
    rerolls_used: u8,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseEquationOfferObservation {
    #[must_use]
    pub const fn offer(&self) -> &DivergentUniverseEquationOfferId {
        &self.offer
    }

    #[must_use]
    pub fn candidates(&self) -> &[DivergentUniverseEquationId] {
        &self.candidates
    }

    #[must_use]
    pub const fn rerolls_used(&self) -> u8 {
        self.rerolls_used
    }

    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationCommandResolution {
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseEquationCommandResolution {
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
    /// Compiles every released Equation RandomID into the bounded executable
    /// project policy without promoting its unknown source membership to exact.
    pub fn equation_offer_runtime(
        &self,
    ) -> Result<DivergentUniverseEquationOfferRuntime, DivergentUniverseEquationRuntimeError> {
        let progress = DivergentUniverseEquationProgressRuntime::compile(
            self.bundle.equation_catalog(),
            self.bundle.blessing_catalog(),
        )
        .map_err(DivergentUniverseEquationRuntimeError::Progress)?;
        let transitions =
            DivergentUniverseEquationTransitionRuntime::compile(self.bundle.equation_catalog())
                .map_err(DivergentUniverseEquationRuntimeError::Transition)?;
        DivergentUniverseEquationOfferRuntime::compile(
            self.bundle.equation_catalog(),
            Arc::new(progress),
            Arc::new(transitions),
            Arc::new(
                EquationGrants::compile(self)
                    .map_err(DivergentUniverseEquationRuntimeError::Activity)?,
            ),
        )
    }
}

impl DivergentUniverseEquationOfferRuntime {
    fn compile(
        catalog: &DivergentUniverseEquationCatalog,
        progress: Arc<DivergentUniverseEquationProgressRuntime>,
        transitions: Arc<DivergentUniverseEquationTransitionRuntime>,
        grants: Arc<EquationGrants>,
    ) -> Result<Self, DivergentUniverseEquationRuntimeError> {
        let equations = catalog
            .equations()
            .iter()
            .enumerate()
            .map(|(index, equation)| EquationProjection {
                id: equation.id.clone(),
                state_key: u64::try_from(index + 1).expect("80 Equations fit u64"),
            })
            .collect::<Vec<_>>();
        let offers = catalog
            .offers()
            .iter()
            .enumerate()
            .map(|(index, offer)| {
                let ordinal = u16::try_from(index)
                    .map_err(|_| DivergentUniverseEquationRuntimeError::InvalidCatalog)?;
                Ok(DivergentUniverseEquationOfferPolicy {
                    id: offer.id.clone(),
                    state_key: stable_key(offer.id.as_str()),
                    purpose: OFFER_PURPOSE_BASE
                        .checked_add(ordinal)
                        .ok_or(DivergentUniverseEquationRuntimeError::InvalidCatalog)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if equations.len() != 80
            || offers.len() != 136
            || duplicate_key(equations.iter().map(|value| value.state_key))
            || duplicate_key(offers.iter().map(|value| value.state_key))
        {
            return Err(DivergentUniverseEquationRuntimeError::InvalidCatalog);
        }
        Ok(Self {
            offers: offers.into(),
            equations: equations.into(),
            progress,
            transitions,
            grants,
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseEquationOfferAccuracy {
        DivergentUniverseEquationOfferAccuracy::VersionedProjectPolicyUniformUnownedThreeOneReroll
    }

    #[must_use]
    pub fn offers(&self) -> &[DivergentUniverseEquationOfferPolicy] {
        &self.offers
    }

    #[must_use]
    pub fn candidate_identity_count(&self) -> u16 {
        u16::try_from(self.equations.len()).expect("compiled Equation denominator fits u16")
    }

    #[must_use]
    pub const fn selection_count(&self) -> u16 {
        OFFER_WIDTH
    }

    #[must_use]
    pub const fn reroll_limit(&self) -> u8 {
        REROLL_LIMIT
    }

    #[must_use]
    pub fn transition_accuracy(&self) -> DivergentUniverseEquationTransitionAccuracy {
        self.transitions.accuracy()
    }

    #[must_use]
    pub fn transition_policies(&self) -> &[DivergentUniverseEquationTransitionPolicy] {
        self.transitions.policies()
    }

    /// Opens one offer from the Reward RNG stream. No eligible Equation returns
    /// a typed rejection before a draw or mutation is attempted.
    pub fn begin_offer(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        offer: &DivergentUniverseEquationOfferId,
    ) -> Result<DivergentUniverseEquationOfferObservation, DivergentUniverseEquationRuntimeError>
    {
        self.validate_hash(activity, expected_state_hash)?;
        if self.observation(activity)?.is_some() {
            return Err(DivergentUniverseEquationRuntimeError::OfferAlreadyActive);
        }
        let policy = self.offer(offer)?;
        let candidates = self.eligible(activity)?;
        if candidates.is_empty() {
            return Err(DivergentUniverseEquationRuntimeError::NoLegalCandidate);
        }
        let selected = self.sample(activity, expected_state_hash, policy, &candidates, false)?;
        self.observation(activity)?
            .filter(|observation| observation.candidates == selected)
            .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)
    }

    /// Replaces the visible subset with another deterministic draw from the
    /// same full eligible pool. Repetition across rerolls is permitted by the
    /// explicit project policy; sampling within one offer is without replacement.
    pub fn reroll_offer(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
    ) -> Result<DivergentUniverseEquationOfferObservation, DivergentUniverseEquationRuntimeError>
    {
        self.validate_hash(activity, expected_state_hash)?;
        let current = self
            .observation(activity)?
            .ok_or(DivergentUniverseEquationRuntimeError::NoActiveOffer)?;
        if current.rerolls_used >= REROLL_LIMIT {
            return Err(DivergentUniverseEquationRuntimeError::RerollLimitReached);
        }
        let policy = self.offer(&current.offer)?;
        let candidates = self.eligible(activity)?;
        if candidates.is_empty() {
            return Err(DivergentUniverseEquationRuntimeError::NoLegalCandidate);
        }
        let selected = self.sample(activity, expected_state_hash, policy, &candidates, true)?;
        self.observation(activity)?
            .filter(|observation| observation.candidates == selected)
            .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)
    }

    pub fn acquire(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        equation: &DivergentUniverseEquationId,
    ) -> Result<DivergentUniverseEquationCommandResolution, DivergentUniverseEquationRuntimeError>
    {
        self.validate_hash(activity, expected_state_hash)?;
        let key = self.equation(equation)?.state_key;
        let state = StateSnapshot::read(activity)?;
        if !state.offered.contains(&key) {
            return Err(DivergentUniverseEquationRuntimeError::NotOffered);
        }
        let (mut operations, owned) = self.acquisition_operations(&activity.player_view(), key)?;
        operations.insert(
            0,
            ActivityOperation::Require(contains(EQUATION_OFFERS_SLOT, key)),
        );
        self.apply_acquisition(
            activity,
            expected_state_hash,
            ACQUIRE_PROGRAM,
            key,
            &owned,
            operations,
        )
    }

    /// Lowers a reward already authenticated by its owning decision boundary.
    /// The caller must execute this inside a generated transaction so that RNG,
    /// ownership, progress and acquisition grants roll back together. This does
    /// not prove pool membership or consume an unrelated internal offer.
    pub(super) fn accepted_acquisition_operations(
        &self,
        view: &ActivityPlayerView,
        equation: &DivergentUniverseEquationId,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseEquationRuntimeError> {
        let state = StateSnapshot::read_view(view)?;
        if view.terminal().is_some() {
            return Err(DivergentUniverseEquationRuntimeError::InvalidState);
        }
        if state.source.is_some() || !state.offered.is_empty() || state.rerolls != 0 {
            return Err(DivergentUniverseEquationRuntimeError::OfferAlreadyActive);
        }
        let key = self.equation(equation)?.state_key;
        let (mut operations, owned) = self.acquisition_operations(view, key)?;
        operations.extend(
            self.grants
                .generate(view, key, &owned, rng)
                .map_err(DivergentUniverseEquationRuntimeError::Activity)?,
        );
        Ok(operations)
    }

    fn acquisition_operations(
        &self,
        view: &ActivityPlayerView,
        key: u64,
    ) -> Result<(Vec<ActivityOperation>, Vec<u64>), DivergentUniverseEquationRuntimeError> {
        let mut owned = set(view, EQUATIONS_SLOT)?.to_vec();
        let position = owned
            .binary_search(&key)
            .err()
            .ok_or(DivergentUniverseEquationRuntimeError::AlreadyOwned)?;
        owned.insert(position, key);
        let blessings = view
            .slots()
            .iter()
            .find(|slot| slot.id() == BLESSINGS_SLOT)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedCounterMap(values) => Some(values.as_ref()),
                _ => None,
            })
            .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)?;
        let mut operations = vec![
            ActivityOperation::Require(not(contains(EQUATIONS_SLOT, key))),
            ActivityOperation::InsertOrderedId {
                slot: EQUATIONS_SLOT,
                id: key,
            },
        ];
        operations.extend(
            self.progress
                .refresh_operations_for_inputs(view, &owned, blessings)
                .map_err(DivergentUniverseEquationRuntimeError::Progress)?,
        );
        Ok((operations, owned))
    }

    pub fn replace(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        removed: &DivergentUniverseEquationId,
        acquired: &DivergentUniverseEquationId,
    ) -> Result<DivergentUniverseEquationCommandResolution, DivergentUniverseEquationRuntimeError>
    {
        self.validate_hash(activity, expected_state_hash)?;
        let removed = self.equation(removed)?.state_key;
        let acquired = self.equation(acquired)?.state_key;
        if removed == acquired {
            return Err(DivergentUniverseEquationRuntimeError::SameEquation);
        }
        let state = StateSnapshot::read(activity)?;
        if !state.owned.contains(&removed) {
            return Err(DivergentUniverseEquationRuntimeError::NotOwned);
        }
        if !state.offered.contains(&acquired) {
            return Err(DivergentUniverseEquationRuntimeError::NotOffered);
        }
        if state.owned.contains(&acquired) {
            return Err(DivergentUniverseEquationRuntimeError::AlreadyOwned);
        }
        let mut owned = state.owned.to_vec();
        let removed_position = owned
            .binary_search(&removed)
            .expect("validated owned Equation is present");
        owned.remove(removed_position);
        let acquired_position = owned
            .binary_search(&acquired)
            .expect_err("validated unowned Equation has an insertion position");
        owned.insert(acquired_position, acquired);
        let mut operations = vec![
            ActivityOperation::Require(contains(EQUATIONS_SLOT, removed)),
            ActivityOperation::Require(contains(EQUATION_OFFERS_SLOT, acquired)),
            ActivityOperation::Require(not(contains(EQUATIONS_SLOT, acquired))),
            ActivityOperation::RemoveOrderedId {
                slot: EQUATIONS_SLOT,
                id: removed,
            },
            ActivityOperation::InsertOrderedId {
                slot: EQUATIONS_SLOT,
                id: acquired,
            },
        ];
        operations.extend(
            self.progress
                .refresh_operations_for_activity(activity, &owned)
                .map_err(DivergentUniverseEquationRuntimeError::Progress)?,
        );
        self.apply_acquisition(
            activity,
            expected_state_hash,
            REPLACE_PROGRAM,
            acquired,
            &owned,
            operations,
        )
    }

    fn apply_acquisition(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        program: u32,
        acquired: u64,
        owned: &[u64],
        mut operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseEquationCommandResolution, DivergentUniverseEquationRuntimeError>
    {
        let view = activity.player_view();
        let resolution = activity
            .apply_generated_boundary(expected, program_id(program), |rng| {
                operations.extend(self.grants.generate(&view, acquired, owned, rng)?);
                operations.extend([
                    clear_offer_source(),
                    clear_offer_candidates(),
                    reset_reroll(),
                ]);
                Ok((operations, ()))
            })
            .map_err(DivergentUniverseEquationRuntimeError::Activity)?;
        Ok(DivergentUniverseEquationCommandResolution {
            events: resolution.events().to_vec().into_boxed_slice(),
            state_hash: resolution.state_hash(),
        })
    }

    pub fn discard(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        equation: &DivergentUniverseEquationId,
    ) -> Result<DivergentUniverseEquationCommandResolution, DivergentUniverseEquationRuntimeError>
    {
        self.validate_hash(activity, expected_state_hash)?;
        let key = self.equation(equation)?.state_key;
        let state = StateSnapshot::read(activity)?;
        if !state.owned.contains(&key) {
            return Err(DivergentUniverseEquationRuntimeError::NotOwned);
        }
        let mut owned = state.owned.to_vec();
        let position = owned
            .binary_search(&key)
            .expect("validated owned Equation is present");
        owned.remove(position);
        let mut operations = vec![
            ActivityOperation::Require(contains(EQUATIONS_SLOT, key)),
            ActivityOperation::RemoveOrderedId {
                slot: EQUATIONS_SLOT,
                id: key,
            },
        ];
        operations.extend(
            self.progress
                .refresh_operations_for_activity(activity, &owned)
                .map_err(DivergentUniverseEquationRuntimeError::Progress)?,
        );
        self.apply(activity, expected_state_hash, DISCARD_PROGRAM, operations)
    }

    pub fn observation(
        &self,
        activity: &GraphActivity,
    ) -> Result<
        Option<DivergentUniverseEquationOfferObservation>,
        DivergentUniverseEquationRuntimeError,
    > {
        let state = StateSnapshot::read(activity)?;
        let Some(source) = state.source else {
            return if state.offered.is_empty() && state.rerolls == 0 {
                Ok(None)
            } else {
                Err(DivergentUniverseEquationRuntimeError::InvalidState)
            };
        };
        let offer = self
            .offers
            .iter()
            .find(|offer| offer.state_key == source)
            .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)?;
        let mut candidates = state
            .offered
            .iter()
            .map(|key| {
                self.equations
                    .iter()
                    .find(|equation| equation.state_key == *key)
                    .map(|equation| equation.id.clone())
                    .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)
            })
            .collect::<Result<Vec<_>, _>>()?;
        candidates.sort_unstable();
        Ok(Some(DivergentUniverseEquationOfferObservation {
            offer: offer.id.clone(),
            candidates: candidates.into_boxed_slice(),
            rerolls_used: state.rerolls,
            state_hash: activity.state_hash(),
        }))
    }

    fn sample(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        policy: &DivergentUniverseEquationOfferPolicy,
        candidates: &[u64],
        reroll: bool,
    ) -> Result<Box<[DivergentUniverseEquationId]>, DivergentUniverseEquationRuntimeError> {
        let candidate_keys = candidates.to_vec();
        let equations = Arc::clone(&self.equations);
        let source = policy.state_key;
        let purpose = policy.purpose;
        let program = if reroll {
            REROLL_PROGRAM
        } else {
            OFFER_PROGRAM
        };
        let resolution = activity
            .apply_generated_boundary(expected_state_hash, program_id(program), move |rng| {
                let weights = vec![1_u64; candidate_keys.len()];
                let selected = rng
                    .choose_weighted_without_replacement(
                        ActivityRngLabel::Reward,
                        purpose,
                        &weights,
                        OFFER_WIDTH,
                    )
                    .map_err(GraphActivityCommandError::Rng)?;
                let mut keys = selected
                    .iter()
                    .map(|index| candidate_keys[*index as usize])
                    .collect::<Vec<_>>();
                keys.sort_unstable();
                let mut ids = keys
                    .iter()
                    .map(|key| {
                        equations
                            .iter()
                            .find(|equation| equation.state_key == *key)
                            .map(|equation| equation.id.clone())
                            .ok_or(GraphActivityCommandError::Runtime(
                                starclock_activity::GraphActivityRuntimeError::InvalidBoundaryProgram,
                            ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                ids.sort_unstable();
                let mut operations = vec![
                    ActivityOperation::SetSlot {
                        slot: EQUATION_OFFER_SOURCE_SLOT,
                        value: literal(ActivityValue::OptionalId(Some(source))),
                    },
                    ActivityOperation::SetOrderedIdSet {
                        slot: EQUATION_OFFERS_SLOT,
                        values: keys.into_boxed_slice(),
                    },
                ];
                operations.push(ActivityOperation::SetSlot {
                    slot: EQUATION_REROLL_COUNT_SLOT,
                    value: literal(ActivityValue::BoundedInteger(i64::from(reroll))),
                });
                Ok((operations, ids.into_boxed_slice()))
            })
            .map_err(DivergentUniverseEquationRuntimeError::Activity)?;
        Ok(resolution.into_value())
    }

    fn eligible(
        &self,
        activity: &GraphActivity,
    ) -> Result<Vec<u64>, DivergentUniverseEquationRuntimeError> {
        let owned = StateSnapshot::read(activity)?.owned;
        Ok(self
            .equations
            .iter()
            .filter(|equation| !owned.contains(&equation.state_key))
            .map(|equation| equation.state_key)
            .collect())
    }

    fn offer(
        &self,
        id: &DivergentUniverseEquationOfferId,
    ) -> Result<&DivergentUniverseEquationOfferPolicy, DivergentUniverseEquationRuntimeError> {
        self.offers
            .binary_search_by(|offer| offer.id.cmp(id))
            .ok()
            .map(|index| &self.offers[index])
            .ok_or(DivergentUniverseEquationRuntimeError::UnknownOffer)
    }

    fn equation(
        &self,
        id: &DivergentUniverseEquationId,
    ) -> Result<&EquationProjection, DivergentUniverseEquationRuntimeError> {
        self.equations
            .binary_search_by(|equation| equation.id.cmp(id))
            .ok()
            .map(|index| &self.equations[index])
            .ok_or(DivergentUniverseEquationRuntimeError::UnknownEquation)
    }

    fn apply(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        program: u32,
        operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseEquationCommandResolution, DivergentUniverseEquationRuntimeError>
    {
        let program = ActivityProgramDefinition::new(program_id(program), operations)
            .map_err(|_| DivergentUniverseEquationRuntimeError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseEquationRuntimeError::Activity)?;
        Ok(DivergentUniverseEquationCommandResolution {
            events,
            state_hash: activity.state_hash(),
        })
    }

    fn validate_hash(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
    ) -> Result<(), DivergentUniverseEquationRuntimeError> {
        if activity.state_hash() == expected {
            Ok(())
        } else {
            Err(DivergentUniverseEquationRuntimeError::Activity(
                GraphActivityCommandError::StaleStateHash,
            ))
        }
    }
}

struct StateSnapshot {
    owned: Box<[u64]>,
    offered: Box<[u64]>,
    source: Option<u64>,
    rerolls: u8,
}

impl StateSnapshot {
    fn read(activity: &GraphActivity) -> Result<Self, DivergentUniverseEquationRuntimeError> {
        Self::read_view(&activity.player_view())
    }

    fn read_view(view: &ActivityPlayerView) -> Result<Self, DivergentUniverseEquationRuntimeError> {
        Ok(Self {
            owned: set(view, EQUATIONS_SLOT)?,
            offered: set(view, EQUATION_OFFERS_SLOT)?,
            source: optional(view, EQUATION_OFFER_SOURCE_SLOT)?,
            rerolls: integer(view, EQUATION_REROLL_COUNT_SLOT)?
                .try_into()
                .map_err(|_| DivergentUniverseEquationRuntimeError::InvalidState)?,
        })
    }
}

fn set(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Box<[u64]>, DivergentUniverseEquationRuntimeError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::OrderedIdSet(values) => Some(values.clone()),
            _ => None,
        })
        .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)
}

fn optional(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Option<u64>, DivergentUniverseEquationRuntimeError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::OptionalId(value) => Some(*value),
            _ => None,
        })
        .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)
}

fn integer(
    view: &starclock_activity::ActivityPlayerView,
    slot: starclock_activity::ActivitySlotId,
) -> Result<i64, DivergentUniverseEquationRuntimeError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedInteger(value) => Some(*value),
            _ => None,
        })
        .ok_or(DivergentUniverseEquationRuntimeError::InvalidState)
}

fn contains(slot: starclock_activity::ActivitySlotId, id: u64) -> ActivityCondition {
    ActivityCondition::OrderedIdSetContains { slot, id }
}

fn not(condition: ActivityCondition) -> ActivityCondition {
    ActivityCondition::Not(Box::new(condition))
}

fn clear_offer_source() -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot: EQUATION_OFFER_SOURCE_SLOT,
        value: literal(ActivityValue::OptionalId(None)),
    }
}

fn clear_offer_candidates() -> ActivityOperation {
    ActivityOperation::SetOrderedIdSet {
        slot: EQUATION_OFFERS_SLOT,
        values: Box::new([]),
    }
}

fn reset_reroll() -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot: EQUATION_REROLL_COUNT_SLOT,
        value: literal(ActivityValue::BoundedInteger(0)),
    }
}

fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}

fn program_id(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).expect("static Equation program ID is non-zero")
}

fn stable_key(value: &str) -> u64 {
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.divergent-universe.equation-runtime-key.v1");
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

fn duplicate_key(values: impl Iterator<Item = u64>) -> bool {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    values.windows(2).any(|pair| pair[0] == pair[1])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationRuntimeError {
    InvalidCatalog,
    UnknownOffer,
    UnknownEquation,
    OfferAlreadyActive,
    NoActiveOffer,
    NoLegalCandidate,
    RerollLimitReached,
    NotOffered,
    AlreadyOwned,
    NotOwned,
    SameEquation,
    InvalidState,
    InvalidProgram,
    Progress(DivergentUniverseEquationProgressError),
    Transition(DivergentUniverseEquationTransitionError),
    Activity(GraphActivityCommandError),
}

impl core::fmt::Display for DivergentUniverseEquationRuntimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Equation runtime error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseEquationRuntimeError {}
