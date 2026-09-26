//! Public domain selection over Route decisions and a per-layer domain label.

use super::{
    DivergentUniverseEntryFlowError, DivergentUniverseFlowInstance,
    DivergentUniverseRuntimeFactory, battle_route::domain_choice_node,
    curio_runtime::DivergentUniverseCurioRuntime, room_lifecycle::exit_condition,
    state::BATTLE_DOMAIN_SLOT,
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId,
    ActivityExpression, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityPlayerView, ActivityStateHash, ActivityValue, GraphActivity, GraphActivityCommandError,
    GraphActivityRuntimeError,
};
use starclock_data::divergent_universe_decisions::{
    BattleRewardDomain, DomainChoiceDefinition, DomainChoicePolicy,
};

#[derive(Clone, Debug)]
pub(super) struct DomainChoices {
    definitions: Box<[DomainChoiceDefinition]>,
    curios: DivergentUniverseCurioRuntime,
}

impl DomainChoices {
    pub(super) fn new(
        factory: &DivergentUniverseRuntimeFactory,
        choices: Box<[DomainChoiceDefinition]>,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        for choice in &choices {
            match choice.policy {
                DomainChoicePolicy::VersionedProjectPolicyExplicitLaterLayerChoices => {}
            }
        }
        Ok(Self {
            definitions: choices,
            curios: factory
                .curio_runtime()
                .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?,
        })
    }

    pub(super) fn program(&self, next: ActivityEdgeId) -> Vec<ActivityOperation> {
        let available = ActivityCondition::All(
            vec![
                exit_condition(),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(BATTLE_DOMAIN_SLOT),
                    ActivityExpression::Literal(ActivityValue::OptionalId(None)),
                ),
            ]
            .into_boxed_slice(),
        );
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Route,
            options: self
                .definitions
                .iter()
                .map(|choice| {
                    ActivityOptionDefinition::new(
                        ActivityOptionId::new(u64::from(choice.ordinal))
                            .expect("validated domain ordinal"),
                        0,
                        available.clone(),
                        vec![
                            ActivityOperation::Require(ActivityCondition::All(
                                vec![
                                    exit_condition(),
                                    ActivityCondition::Equal(
                                        ActivityExpression::Slot(BATTLE_DOMAIN_SLOT),
                                        ActivityExpression::Literal(ActivityValue::OptionalId(
                                            Some(domain_key(choice.domain)),
                                        )),
                                    ),
                                ]
                                .into_boxed_slice(),
                            )),
                            ActivityOperation::Traverse(next),
                        ],
                    )
                })
                .collect(),
        }]
    }

    pub(super) fn resolve(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Option<BattleRewardDomain>, GraphActivityCommandError> {
        let slot = view
            .slots()
            .iter()
            .find(|slot| slot.id() == BATTLE_DOMAIN_SLOT)
            .ok_or_else(invalid)?;
        let ActivityValue::OptionalId(value) = slot.value() else {
            return Err(invalid());
        };
        let Some(value) = value else {
            return Ok(None);
        };
        self.definitions
            .iter()
            .find(|choice| domain_key(choice.domain) == *value)
            .map(|choice| Some(choice.domain))
            .ok_or_else(invalid)
    }
}

impl DivergentUniverseFlowInstance {
    /// Immutable definitions for the exact public domain Route decision. Original
    /// source-room membership is not implied. Reading never samples or mutates.
    pub fn offered_battle_domain_choices(
        &self,
        activity: &GraphActivity,
    ) -> Result<Option<&[DomainChoiceDefinition]>, GraphActivityCommandError> {
        if activity.definition().identity() != self.definition().identity()
            || activity.definition().graph().digest() != self.definition().graph().digest()
        {
            return Err(invalid());
        }
        let Some(pool) = &self.encounter_pool else {
            return Ok(None);
        };
        let count = u32::try_from(self.layers().len()).map_err(|_| invalid())?;
        if !(2..=count)
            .any(|ordinal| domain_choice_node(count, ordinal).ok() == Some(activity.current_node()))
        {
            return Ok(None);
        }
        let view = activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Route)
            .ok_or_else(invalid)?;
        if pool.domains.resolve(&view)?.is_some()
            || decision.options().len() != pool.domains.definitions.len()
            || !decision.options().iter().all(|option| {
                pool.domains
                    .definitions
                    .iter()
                    .any(|choice| u64::from(choice.ordinal) == option.id().get())
            })
        {
            return Err(invalid());
        }
        Ok(Some(&pool.domains.definitions))
    }

    /// Commits one offered domain, its intrinsic Curio grants, allowance consumption/discard and
    /// encounter construction atomically. Stale, foreign or duplicate choices
    /// leave all state, events and RNG unchanged. Direct generic option execution
    /// cannot satisfy the domain marker required by this option's typed program.
    pub fn choose_battle_domain(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        let choices = self
            .offered_battle_domain_choices(activity)?
            .ok_or_else(invalid)?;
        let domain = choices
            .iter()
            .find(|choice| u64::from(choice.ordinal) == option.get())
            .map(|choice| choice.domain)
            .ok_or_else(invalid)?;
        let pool = self.encounter_pool.as_ref().ok_or_else(invalid)?;
        activity.choose_option_with_generated_prefix(expected, decision, option, |view, _| {
            let mut operations = pool
                .domains
                .curios
                .domain_entry_operations(view)
                .map_err(|_| invalid())?;
            operations.push(set_domain(Some(domain)));
            Ok((operations, ()))
        })?;
        Ok(())
    }

    /// Authenticated domain shared by a bound room's encounter and result.
    /// Legacy profiles select once per layer; explicit position profiles select
    /// independently per room. None means no domain is bound at this node.
    pub fn current_battle_domain(
        &self,
        activity: &GraphActivity,
    ) -> Result<Option<BattleRewardDomain>, GraphActivityCommandError> {
        if activity.definition().identity() != self.definition().identity()
            || activity.definition().graph().digest() != self.definition().graph().digest()
            || self
                .position_battles
                .as_ref()
                .is_some_and(|rooms| !rooms.matches(activity))
        {
            return Err(invalid());
        }
        self.domain_from_view(&activity.player_view())
    }

    pub(super) fn domain_from_view(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Option<BattleRewardDomain>, GraphActivityCommandError> {
        if let Some(rooms) = &self.position_battles {
            rooms.domain(view)
        } else if let Some(pool) = &self.encounter_pool {
            pool.domains.resolve(view)
        } else if self.has_runtime_battle_route() {
            Ok(Some(BattleRewardDomain::Combat))
        } else {
            Ok(None)
        }
    }
}

pub(super) fn set_domain(domain: Option<BattleRewardDomain>) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot: BATTLE_DOMAIN_SLOT,
        value: ActivityExpression::Literal(ActivityValue::OptionalId(domain.map(domain_key))),
    }
}

fn domain_key(domain: BattleRewardDomain) -> u64 {
    // Starclock-owned persisted keys, deliberately not upstream composition IDs.
    match domain {
        BattleRewardDomain::Combat => 1,
        BattleRewardDomain::Elite => 2,
        BattleRewardDomain::Aberration => 3,
        BattleRewardDomain::Boss => 4,
    }
}

fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
