//! Authored encounter offers; shared Activity owns every random draw and rollback.

use super::tawot_service::ENCOUNTER as TAWOT_ENCOUNTER_NODE;
use super::{
    DivergentUniverseEntryFlowError, DivergentUniverseFlowInstance, domain_choices::DomainChoices,
    room_lifecycle::exit_condition,
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeId, ActivityExpression, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityRandomOffer, ActivityRngLabel,
    ActivityValue, GraphActivity, GraphActivityCommandError, NodeId,
};
use starclock_data::{
    divergent_universe_decisions::{EncounterPoolPolicy, EncounterPoolPolicyKind},
    divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId,
};

#[derive(Clone, Debug)]
pub(super) struct EncounterPool {
    pub(super) policy: EncounterPoolPolicy,
    pub(super) domains: DomainChoices,
}

impl EncounterPool {
    pub(super) fn new(policy: EncounterPoolPolicy, domains: DomainChoices) -> Self {
        match policy.kind {
            EncounterPoolPolicyKind::VersionedProjectPolicyFixedFirstUniformLaterLayersWithReplacement => {}
        }
        Self { policy, domains }
    }

    pub(super) fn options(&self, edge: ActivityEdgeId) -> Box<[ActivityOptionDefinition]> {
        self.policy
            .candidate_stages
            .iter()
            .enumerate()
            .map(|(index, _)| {
                ActivityOptionDefinition::new(
                    option(index),
                    0,
                    ActivityCondition::Boolean(ActivityExpression::Literal(
                        ActivityValue::Boolean(true),
                    )),
                    vec![
                        ActivityOperation::Require(exit_condition()),
                        ActivityOperation::Traverse(edge),
                    ],
                )
            })
            .collect()
    }

    pub(super) fn random_offers(
        &self,
        layer_count: usize,
    ) -> Result<Vec<ActivityRandomOffer>, DivergentUniverseEntryFlowError> {
        let count = u32::try_from(layer_count)
            .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        (2..=count)
            .map(|ordinal| {
                ActivityRandomOffer::new(
                    NodeId::new(ordinal).expect("later layer ordinal is positive"),
                    ActivityRngLabel::Encounter,
                    23_901,
                    1,
                    self.policy
                        .candidate_stages
                        .iter()
                        .enumerate()
                        .map(|(index, _)| (option(index), 1))
                        .collect(),
                    None,
                )
                .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
            })
            .collect()
    }
}

impl DivergentUniverseFlowInstance {
    #[must_use]
    pub fn encounter_pool_policy(&self) -> Option<&EncounterPoolPolicy> {
        self.encounter_pool.as_ref().map(|pool| &pool.policy)
    }

    /// Returns the sole public encounter's immutable authored group/stage binding.
    /// Does not draw RNG or alter state. A bound pool rejects non-Encounter,
    /// foreign-definition and malformed offers; unbound low-level flows return None.
    pub fn offered_encounter(
        &self,
        activity: &GraphActivity,
    ) -> Result<Option<(&DivergentUniverseEncounterGroupId, &str)>, GraphActivityCommandError> {
        let Some(pool) = &self.encounter_pool else {
            return Ok(None);
        };
        let invalid = || GraphActivityCommandError::DecisionNotOffered;
        if activity.definition().identity() != self.definition().identity()
            || activity.definition().graph().digest() != self.definition().graph().digest()
        {
            return Err(invalid());
        }
        let view = activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Encounter)
            .ok_or_else(invalid)?;
        let [selected] = decision.options() else {
            return Err(invalid());
        };
        let layer =
            if self.tawot_service.is_some() && view.current_node().get() == TAWOT_ENCOUNTER_NODE {
                1
            } else {
                usize::try_from(view.current_node().get()).map_err(|_| invalid())?
            };
        if layer == 0 || layer > self.layers().len() {
            return Err(invalid());
        }
        let stage = if layer == 1 {
            if selected.id().get() != 1 {
                return Err(invalid());
            }
            pool.policy.first_stage.as_ref()
        } else {
            let index = selected
                .id()
                .get()
                .checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(invalid)?;
            pool.policy
                .candidate_stages
                .get(index)
                .ok_or_else(invalid)?
                .as_ref()
        };
        Ok(Some((&pool.policy.encounter_group, stage)))
    }
}

fn option(index: usize) -> ActivityOptionId {
    ActivityOptionId::new(
        u64::try_from(index).expect("validated pool has at most 256 candidates") + 1,
    )
    .expect("compiled encounter option is positive")
}
