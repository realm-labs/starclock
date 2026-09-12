//! Mode-owned normal-battle Blessing rewards at verified settlement.

use crate::divergent_universe::equation_progress::expansion::ExpansionOfferConsumption;
use std::slice::from_ref;

#[path = "battle_blessing_weights.rs"]
mod weights;
use weights::BattleWeights;

use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId,
    ActivityExpression, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityPlayerView, ActivityRngLabel, ActivityRngStreams, ActivityStateHash, ActivityValue,
    GraphActivity, GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::{
        DivergentUniverseBlessingCategory, DivergentUniverseBlessingId,
    },
    divergent_universe_decisions::{BattleBlessingPolicy, BattleBlessingPolicyKind},
    divergent_universe_equation_catalog::DivergentUniversePathType,
};

use super::{
    DivergentUniverseBlessingRuntime, DivergentUniverseEntryFlowError,
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    curio_catalog::CompiledCurioCatalog,
    state::{BATTLE_BLESSING_ACCEPTED_SLOT, BATTLE_BLESSING_CANDIDATES_SLOT, CURIO_STATES_SLOT},
};

#[derive(Clone, Debug)]
pub(super) struct BattleBlessings {
    policy: BattleBlessingPolicy,
    blessings: DivergentUniverseBlessingRuntime,
    pool: Box<[(DivergentUniverseBlessingId, u64, DivergentUniversePathType)]>,
    suppression: Box<[u64]>,
    weights: BattleWeights,
}

impl BattleBlessings {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let [policy] = factory.decision_catalog().battle_blessings() else {
            return Err(DivergentUniverseEntryFlowError::InvalidActivityDefinition);
        };
        match policy.kind {
            BattleBlessingPolicyKind::VersionedProjectPolicyUniformUnownedSingleSelectionAvailableSubset => {}
        }
        let blessings = factory
            .blessing_runtime()
            .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        let curios = CompiledCurioCatalog::compile(factory.bundle.curio_catalog())
            .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        let suppression = policy
            .suppression_states
            .iter()
            .map(|id| {
                curios
                    .states
                    .iter()
                    .find(|state| state.id() == id)
                    .map(|state| state.state_key())
                    .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        let mut pool = blessings
            .blessings()
            .iter()
            .filter(|blessing| {
                let rarity = match blessing.category() {
                    DivergentUniverseBlessingCategory::Common => 1,
                    DivergentUniverseBlessingCategory::Rare => 2,
                    DivergentUniverseBlessingCategory::Legendary => 3,
                };
                (policy.rarity.minimum()..=policy.rarity.maximum()).contains(&rarity)
            })
            .map(|blessing| {
                (
                    blessing.id().clone(),
                    blessing.state_key(),
                    blessing.path().clone(),
                )
            })
            .collect::<Vec<_>>();
        pool.sort_by(|left, right| left.0.cmp(&right.0));
        if pool.is_empty() {
            return Err(DivergentUniverseEntryFlowError::InvalidActivityDefinition);
        }
        Ok(Self {
            policy: policy.clone(),
            blessings,
            pool: pool.into_boxed_slice(),
            suppression,
            weights: BattleWeights::compile(factory, &curios)?,
        })
    }

    pub(super) fn node_program(&self, next: ActivityEdgeId) -> Vec<ActivityOperation> {
        let options = (1..=self.policy.offer_width)
            .map(|ordinal| {
                ActivityOptionDefinition::new(
                    ActivityOptionId::new(u64::from(ordinal)).expect("nonzero reward ordinal"),
                    0,
                    ActivityCondition::LessThan(
                        integer(i64::from(ordinal) - 1),
                        ActivityExpression::OrderedIdSetCount(BATTLE_BLESSING_CANDIDATES_SLOT),
                    ),
                    vec![
                        ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Slot(BATTLE_BLESSING_ACCEPTED_SLOT),
                        )),
                        ActivityOperation::Traverse(next),
                    ],
                )
            })
            .collect::<Vec<_>>();
        vec![ActivityOperation::Conditional {
            condition: ActivityCondition::Equal(
                ActivityExpression::OrderedIdSetCount(BATTLE_BLESSING_CANDIDATES_SLOT),
                integer(0),
            ),
            if_true: vec![ActivityOperation::Traverse(next)].into_boxed_slice(),
            if_false: vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::Reward,
                options: options.into_boxed_slice(),
            }]
            .into_boxed_slice(),
        }]
    }

    pub(super) fn generate(
        &self,
        view: &ActivityPlayerView,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        if !candidates(view)?.is_empty() {
            return Err(invalid());
        }
        let owned = self.blessings.reward_owned(view).map_err(|_| invalid())?;
        let states = view
            .slots()
            .iter()
            .find(|slot| slot.id() == CURIO_STATES_SLOT)
            .ok_or_else(invalid)?;
        let ActivityValue::BoundedCounterMap(states) = states.value() else {
            return Err(invalid());
        };
        let mut suppressed = false;
        for key in &self.suppression {
            let status = states
                .binary_search_by_key(key, |entry| entry.0)
                .ok()
                .map_or(0, |index| states[index].1);
            match status {
                0 | 2 => {}
                1 => suppressed = true,
                _ => return Err(invalid()),
            }
        }
        let available = self
            .pool
            .iter()
            .filter(|(id, _, _)| !owned.iter().any(|item| item.blessing() == id))
            .collect::<Vec<_>>();
        let mut selected = Vec::new();
        if !suppressed && !available.is_empty() {
            let count = usize::from(self.policy.offer_width).min(available.len());
            let bonuses = self.weights.snapshot(view)?;
            let weights = available
                .iter()
                .map(|(_, _, path)| {
                    bonuses
                        .get(path)
                        .copied()
                        .unwrap_or(0)
                        .checked_add(1)
                        .ok_or_else(invalid)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let draws = rng
                .choose_weighted_without_replacement(
                    ActivityRngLabel::Reward,
                    23_701,
                    &weights,
                    u16::try_from(count).map_err(|_| invalid())?,
                )
                .map_err(GraphActivityCommandError::Rng)?;
            selected.extend(draws.iter().map(|index| available[*index as usize].1));
            selected.sort_unstable();
        }
        Ok(vec![ActivityOperation::SetOrderedIdSet {
            slot: BATTLE_BLESSING_CANDIDATES_SLOT,
            values: selected.into_boxed_slice(),
        }])
    }

    fn accept(
        &self,
        view: &ActivityPlayerView,
        option: ActivityOptionId,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let index = option
            .get()
            .checked_sub(1)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(invalid)?;
        let key = candidates(view)?.get(index).ok_or_else(invalid)?;
        let (id, _, _) = self
            .pool
            .iter()
            .find(|(_, value, _)| value == key)
            .ok_or_else(invalid)?;
        let mut operations = self
            .blessings
            .acquisition_plan(view, from_ref(id), ExpansionOfferConsumption::Battle, rng)
            .map_err(|_| invalid())?
            .finish(view)?;
        operations.push(ActivityOperation::SetSlot {
            slot: BATTLE_BLESSING_ACCEPTED_SLOT,
            value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
        });
        Ok(operations)
    }
}

impl DivergentUniverseFlowInstance {
    /// Accepts exactly one publicly offered normal-battle Blessing. Inventory,
    /// Equation progress, offer consumption and advancement commit together;
    /// invalid/hidden/duplicate choices preserve the full state and RNG.
    pub fn choose_battle_blessing(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if activity.definition().identity() != self.definition.identity()
            || activity.definition().graph().digest() != self.definition.graph().digest()
        {
            return Err(invalid());
        }
        let view = activity.player_view();
        if !self.is_battle_reward(view.current_node())
            || view
                .decision()
                .is_none_or(|value| value.kind() != ActivityDecisionKind::Reward)
        {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        let runtime = self.battle_blessings.as_ref().ok_or_else(invalid)?;
        activity.choose_option_with_generated_prefix(expected, decision, option, |view, rng| {
            Ok((runtime.accept(view, option, rng)?, ()))
        })?;
        Ok(())
    }
}

fn candidates(view: &ActivityPlayerView) -> Result<&[u64], GraphActivityCommandError> {
    let slot = view
        .slots()
        .iter()
        .find(|slot| slot.id() == BATTLE_BLESSING_CANDIDATES_SLOT)
        .ok_or_else(invalid)?;
    match slot.value() {
        ActivityValue::OrderedIdSet(values) => Ok(values),
        _ => Err(invalid()),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
