//! Policy-authored public initial Equation selection over shared graph offers.

use super::{
    DivergentUniverseEntryFlowError, DivergentUniverseFlowInstance,
    DivergentUniverseRuntimeFactory,
    equation_offer::DivergentUniverseEquationOfferRuntime,
    state::{EQUATIONS_SLOT, INITIAL_EQUATION_ACCEPTED_SLOT},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId,
    ActivityExpression, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityRandomOffer, ActivityRngLabel, ActivityStateHash, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError, NodeId,
};
use starclock_data::{
    divergent_universe_decisions::InitialEquationPolicy,
    divergent_universe_equation_catalog::DivergentUniverseEquationId,
};

#[derive(Clone, Debug)]
pub(super) struct InitialEquations {
    policy: InitialEquationPolicy,
    pool: Box<[(u64, DivergentUniverseEquationId)]>,
    acquisition: DivergentUniverseEquationOfferRuntime,
}

impl InitialEquations {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let policy = factory.decision_catalog().initial_equations().clone();
        let pool = factory
            .bundle
            .equation_catalog()
            .equations()
            .iter()
            .enumerate()
            .filter(|(_, equation)| equation.category == policy.category)
            .map(|(index, equation)| {
                Ok((
                    u64::try_from(index + 1)
                        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?,
                    equation.id.clone(),
                ))
            })
            .collect::<Result<Box<[_]>, DivergentUniverseEntryFlowError>>()?;
        let acquisition = factory
            .equation_offer_runtime()
            .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        Ok(Self {
            policy,
            pool,
            acquisition,
        })
    }

    pub(super) fn program(&self, next: ActivityEdgeId) -> Vec<ActivityOperation> {
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Preparation,
            options: self
                .pool
                .iter()
                .map(|(key, _)| {
                    ActivityOptionDefinition::new(
                        option(*key),
                        0,
                        ActivityCondition::Not(Box::new(ActivityCondition::OrderedIdSetContains {
                            slot: EQUATIONS_SLOT,
                            id: *key,
                        })),
                        vec![
                            ActivityOperation::Require(ActivityCondition::Boolean(
                                ActivityExpression::Slot(INITIAL_EQUATION_ACCEPTED_SLOT),
                            )),
                            ActivityOperation::Traverse(next),
                        ],
                    )
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }]
    }

    pub(super) fn random_offer(
        &self,
        node: NodeId,
    ) -> Result<ActivityRandomOffer, DivergentUniverseEntryFlowError> {
        ActivityRandomOffer::new(
            node,
            ActivityRngLabel::Reward,
            23_801,
            self.policy.offer_width,
            self.pool.iter().map(|(key, _)| (option(*key), 1)).collect(),
            None,
        )
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
    }
}

impl DivergentUniverseFlowInstance {
    #[must_use]
    pub fn initial_equation_policy(&self) -> Option<&InitialEquationPolicy> {
        self.initial_equations.as_ref().map(|value| &value.policy)
    }

    /// Authenticates a visible initial Equation choice, then atomically acquires
    /// it, refreshes progress, executes grants and enters the first checkpoint.
    /// Rejection restores both the offer and RNG; raw graph choices cannot skip acquisition.
    pub fn choose_initial_equation(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if activity.definition().identity() != self.definition.identity()
            || activity.definition().graph().digest() != self.definition.graph().digest()
        {
            return Err(invalid());
        }
        let runtime = self.initial_equations.as_ref().ok_or_else(invalid)?;
        let view = activity.player_view();
        let node = u32::try_from(self.layers().len())
            .ok()
            .and_then(|value| value.checked_add(7));
        if Some(view.current_node().get()) != node
            || view
                .decision()
                .is_none_or(|value| value.kind() != ActivityDecisionKind::Preparation)
        {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        activity.choose_option_with_generated_prefix(
            expected,
            decision,
            selected,
            |view, rng| {
                let (_, id) = runtime
                    .pool
                    .iter()
                    .find(|(key, _)| *key == selected.get())
                    .ok_or_else(invalid)?;
                let mut operations = runtime
                    .acquisition
                    .accepted_acquisition_operations(view, id, rng)
                    .map_err(|_| invalid())?;
                operations.push(ActivityOperation::SetSlot {
                    slot: INITIAL_EQUATION_ACCEPTED_SLOT,
                    value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
                });
                Ok((operations, ()))
            },
        )?;
        Ok(())
    }
}

fn option(key: u64) -> ActivityOptionId {
    ActivityOptionId::new(key).expect("compiled Equation state key is nonzero")
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
