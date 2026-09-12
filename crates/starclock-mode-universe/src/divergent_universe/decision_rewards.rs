//! Current typed event reward policy; logical checkpoint binding is mode-owned.

#[path = "decision_reward_conditions.rs"]
mod conditions;

use std::sync::Arc;

use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngError, ActivityRngLabel, ActivityRngStreams,
    ActivityValue,
};
use starclock_data::{
    divergent_universe_blessing_catalog::{
        DivergentUniverseBlessingCategory, DivergentUniverseBlessingId,
    },
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
    divergent_universe_decisions::{
        DecisionCatalog, DecisionChoice, DecisionChoiceId, DecisionExhaustion, DecisionPolicyKind,
        DecisionReward, RewardRarityRange,
    },
};

use super::{
    DivergentUniverseRuntimeFactory,
    blessing_runtime::{DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError},
    curio_runtime::{DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError},
    economy::{
        self, DivergentUniverseCurrencyKind, DivergentUniverseCurrencyRuntime,
        DivergentUniverseEconomyError,
    },
    state::CURRENCIES_SLOT,
};

const CURIO_PURPOSE: u16 = 23_611;
const BLESSING_PURPOSE: u16 = 23_612;

/// Immutable executor for the authored single-outcome choice policy. Candidate
/// identity and random policy remain distinct from acquisition-effect parity.
#[derive(Clone, Debug)]
pub struct DecisionRewardRuntime {
    decisions: Arc<DecisionCatalog>,
    curios: DivergentUniverseCurioRuntime,
    blessings: DivergentUniverseBlessingRuntime,
    fragments: DivergentUniverseCurrencyRuntime,
}

/// Accepted inventory grant; this does not assert acquisition triggers ran.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionRewardGrant {
    Fragments(u64),
    Curios(Box<[DivergentUniverseCurioStateId]>),
    Blessings(Box<[DivergentUniverseBlessingId]>),
}

/// Operations prepared for the shared generated-option transaction. Nothing has
/// been committed when this value is returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRewardProgram {
    operations: Vec<ActivityOperation>,
    grant: DecisionRewardGrant,
}

impl DecisionRewardProgram {
    #[must_use]
    pub fn into_parts(self) -> (Vec<ActivityOperation>, DecisionRewardGrant) {
        (self.operations, self.grant)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionRewardError {
    UnknownChoice,
    UnsupportedOutcomeSequence,
    InvalidCurrencyState,
    InvalidCatalog,
    InsufficientFragments { required: u64, available: u64 },
    InsufficientCandidates { required: u16, available: u16 },
    Curio(DivergentUniverseCurioRuntimeError),
    Blessing(DivergentUniverseBlessingRuntimeError),
    Economy(DivergentUniverseEconomyError),
    Rng(ActivityRngError),
}

impl DivergentUniverseRuntimeFactory {
    /// Builds the current policy executor from private production catalogs.
    /// This does not make occurrences reachable or authorize player rewards.
    pub fn decision_reward_runtime(&self) -> Result<DecisionRewardRuntime, DecisionRewardError> {
        let economy = economy::compile(
            self.bundle.service_catalog(),
            self.bundle.progression_catalog(),
            self.bundle.curio_catalog(),
            self.decision_catalog(),
        )
        .map_err(DecisionRewardError::Economy)?;
        Ok(DecisionRewardRuntime {
            decisions: Arc::new(self.decision_catalog().clone()),
            curios: self.curio_runtime().map_err(DecisionRewardError::Curio)?,
            blessings: self
                .blessing_runtime()
                .map_err(DecisionRewardError::Blessing)?,
            fragments: economy
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .clone(),
        })
    }
}

impl DecisionRewardRuntime {
    /// The owning Activity must bind this exact decision/reference digest.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        self.decisions.digest()
    }

    /// Checks costs and the complete unowned pool without consuming RNG.
    /// The owning offer compiler must disable an InsufficientCandidates choice;
    /// this check itself does not edit an offered decision. It never silently
    /// reduces the count or invents a substitute reward.
    pub fn check_choice(
        &self,
        view: &ActivityPlayerView,
        choice: &DecisionChoiceId,
    ) -> Result<(), DecisionRewardError> {
        self.plan(view, self.choice(choice)?).map(|_| ())
    }

    /// Generates costs and inventory rewards inside an owning shared Activity
    /// generated-option transaction. The caller must not publish the program or
    /// RNG changes without that transaction's successful commit. This function
    /// includes reviewed immediate Curio grant operations but does not itself
    /// commit them, consume an option or finish a room. Other acquisition effects
    /// remain pending. Invalid pools/costs/state reject before a selection draw.
    pub fn generate_choice(
        &self,
        view: &ActivityPlayerView,
        choice: &DecisionChoiceId,
        rng: &mut ActivityRngStreams,
    ) -> Result<DecisionRewardProgram, DecisionRewardError> {
        let choice = self.choice(choice)?;
        let plan = self.plan(view, choice)?;
        let mut operations = Vec::new();
        if choice.fragment_cost != 0 {
            operations.push(
                self.fragments
                    .spend_operation(choice.fragment_cost)
                    .map_err(DecisionRewardError::Economy)?,
            );
        }
        let grant = match plan {
            RewardPlan::Fragments(amount) => {
                operations.extend(
                    self.fragments
                        .credit_operations(amount)
                        .map_err(DecisionRewardError::Economy)?,
                );
                DecisionRewardGrant::Fragments(amount)
            }
            RewardPlan::Curios(count, candidates) => {
                let selected = select(candidates, count, rng, CURIO_PURPOSE)?;
                operations.extend(
                    self.curios
                        .acquisition_operations(view, &selected, rng)
                        .map_err(DecisionRewardError::Curio)?,
                );
                DecisionRewardGrant::Curios(selected.into_boxed_slice())
            }
            RewardPlan::Blessings(count, candidates) => {
                let selected = select(candidates, count, rng, BLESSING_PURPOSE)?;
                operations.extend(
                    self.blessings
                        .acquisition_operations(view, &selected, rng)
                        .map_err(DecisionRewardError::Blessing)?,
                );
                DecisionRewardGrant::Blessings(selected.into_boxed_slice())
            }
        };
        Ok(DecisionRewardProgram { operations, grant })
    }

    fn choice(&self, id: &DecisionChoiceId) -> Result<&DecisionChoice, DecisionRewardError> {
        self.decisions
            .occurrences()
            .iter()
            .flat_map(|event| match (event.policy.kind, event.policy.exhaustion) {
                (
                    DecisionPolicyKind::VersionedProjectPolicyUniformUnownedCurrentCatalog,
                    DecisionExhaustion::DisableChoice,
                ) => event.choices.iter(),
            })
            .find(|choice| &choice.id == id)
            .ok_or(DecisionRewardError::UnknownChoice)
    }

    fn plan(
        &self,
        view: &ActivityPlayerView,
        choice: &DecisionChoice,
    ) -> Result<RewardPlan, DecisionRewardError> {
        let [outcome] = choice.outcomes.as_ref() else {
            return Err(DecisionRewardError::UnsupportedOutcomeSequence);
        };
        self.reward_plan(view, choice.fragment_cost, outcome.reward)
    }

    fn reward_plan(
        &self,
        view: &ActivityPlayerView,
        fragment_cost: u64,
        reward: DecisionReward,
    ) -> Result<RewardPlan, DecisionRewardError> {
        let available = self.fragment_balance(view)?;
        if available < fragment_cost {
            return Err(DecisionRewardError::InsufficientFragments {
                required: fragment_cost,
                available,
            });
        }
        match reward {
            DecisionReward::Fragments(amount) => {
                self.fragments
                    .credit_operations(amount)
                    .map_err(DecisionRewardError::Economy)?;
                Ok(RewardPlan::Fragments(amount))
            }
            DecisionReward::Curios { count, rarity } => {
                let owned = self
                    .curios
                    .owned_from_view(view)
                    .map_err(DecisionRewardError::Curio)?;
                let mut candidates = Vec::new();
                for curio in self.curios.curios() {
                    if !contains(rarity, curio_rarity(curio.category()))
                        || owned.iter().any(|held| held.curio() == curio.id())
                    {
                        continue;
                    }
                    let state = self
                        .curios
                        .states()
                        .iter()
                        .filter(|state| state.curio() == Some(curio.id()))
                        .min_by(|left, right| left.id().as_str().cmp(right.id().as_str()))
                        .ok_or(DecisionRewardError::InvalidCatalog)?;
                    if self
                        .curios
                        .acquisition_rewards_available(view, state.id())
                        .map_err(DecisionRewardError::Curio)?
                    {
                        candidates.push((curio.id().clone(), state.id().clone()));
                    }
                }
                candidates.sort_by(|left, right| left.0.as_str().cmp(right.0.as_str()));
                require_count(count, candidates.len())?;
                Ok(RewardPlan::Curios(
                    count,
                    candidates.into_iter().map(|(_, state)| state).collect(),
                ))
            }
            DecisionReward::Blessings { count, rarity } => {
                let owned = self
                    .blessings
                    .reward_owned(view)
                    .map_err(DecisionRewardError::Blessing)?;
                let mut candidates = self
                    .blessings
                    .blessings()
                    .iter()
                    .filter(|blessing| {
                        contains(rarity, Some(blessing_rarity(blessing.category())))
                            && !owned.iter().any(|held| held.blessing() == blessing.id())
                    })
                    .map(|blessing| blessing.id().clone())
                    .collect::<Vec<_>>();
                candidates.sort_by(|left, right| left.as_str().cmp(right.as_str()));
                require_count(count, candidates.len())?;
                Ok(RewardPlan::Blessings(count, candidates))
            }
        }
    }

    pub(super) fn fragment_balance(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<u64, DecisionRewardError> {
        let values = view
            .slots()
            .iter()
            .find(|slot| slot.id() == CURRENCIES_SLOT)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedCounterMap(values) => Some(values),
                _ => None,
            })
            .ok_or(DecisionRewardError::InvalidCurrencyState)?;
        let amount = values
            .binary_search_by_key(&self.fragments.key(), |entry| entry.0)
            .map_or(0, |index| values[index].1);
        u64::try_from(amount).map_err(|_| DecisionRewardError::InvalidCurrencyState)
    }

    pub(super) fn curio_reward_operations(
        &self,
        view: &ActivityPlayerView,
        reward: DecisionReward,
        sacrificed: Option<&DivergentUniverseCurioStateId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, DecisionRewardError> {
        let RewardPlan::Curios(count, candidates) = self.reward_plan(view, 0, reward)? else {
            return Err(DecisionRewardError::InvalidCatalog);
        };
        let selected = select(candidates, count, rng, CURIO_PURPOSE)?;
        self.curios
            .sacrifice_acquisition_operations(view, sacrificed, &selected, rng)
            .map_err(DecisionRewardError::Curio)
    }
}

enum RewardPlan {
    Fragments(u64),
    Curios(u16, Vec<DivergentUniverseCurioStateId>),
    Blessings(u16, Vec<DivergentUniverseBlessingId>),
}

fn require_count(required: u16, available: usize) -> Result<(), DecisionRewardError> {
    let available = u16::try_from(available).map_err(|_| DecisionRewardError::InvalidCatalog)?;
    if available < required {
        Err(DecisionRewardError::InsufficientCandidates {
            required,
            available,
        })
    } else {
        Ok(())
    }
}

fn select<T>(
    mut candidates: Vec<T>,
    count: u16,
    rng: &mut ActivityRngStreams,
    purpose: u16,
) -> Result<Vec<T>, DecisionRewardError> {
    require_count(count, candidates.len())?;
    let mut selected = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let length =
            u32::try_from(candidates.len()).map_err(|_| DecisionRewardError::InvalidCatalog)?;
        let draw = rng
            .choose_index(ActivityRngLabel::Reward, purpose, length)
            .map_err(DecisionRewardError::Rng)?
            .ok_or(DecisionRewardError::InvalidCatalog)?;
        let index =
            usize::try_from(draw.value()).map_err(|_| DecisionRewardError::InvalidCatalog)?;
        selected.push(candidates.remove(index));
    }
    Ok(selected)
}

fn contains(range: RewardRarityRange, rarity: Option<u8>) -> bool {
    rarity.is_some_and(|value| (range.minimum()..=range.maximum()).contains(&value))
}

const fn curio_rarity(category: DivergentUniverseCurioCategory) -> Option<u8> {
    match category {
        DivergentUniverseCurioCategory::Common => Some(1),
        DivergentUniverseCurioCategory::Rare => Some(2),
        DivergentUniverseCurioCategory::Legendary => Some(3),
        DivergentUniverseCurioCategory::Negative => None,
    }
}

const fn blessing_rarity(category: DivergentUniverseBlessingCategory) -> u8 {
    match category {
        DivergentUniverseBlessingCategory::Common => 1,
        DivergentUniverseBlessingCategory::Rare => 2,
        DivergentUniverseBlessingCategory::Legendary => 3,
    }
}
