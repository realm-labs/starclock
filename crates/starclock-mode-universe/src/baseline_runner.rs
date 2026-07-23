//! Deterministic baseline orchestration over the Standard Universe facade.

use crate::baseline_controller::{
    ActivityBaselineController, ActivityBaselineDecision, ActivityBaselineHints,
    ActivityDecisionError,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityOptionId, ActivityStateHash,
    ActivityTerminalOutcome, BattleOutcome, BattleResult, BattleResultDigest, BattleResultIdentity,
    GraphActivityBattleError, GraphActivityCommandError, GraphActivityEncounterError,
};

use crate::runtime::{
    StandardUniverseActivity, StandardUniverseBattleStartError, StandardUniverseEncounterError,
};

/// Synchronous authoritative battle boundary used by CLI, services and tests.
///
/// Runtime failures belong in the returned battle projection as an explicit
/// fault. Infrastructure cancellation is deliberately outside the accepted
/// Activity-command transaction.
pub trait NestedBattleExecutor {
    fn execute(&mut self, handoff: &starclock_activity::ActivityBattleHandoff) -> BattleResult;
}

impl<F> NestedBattleExecutor for F
where
    F: FnMut(&starclock_activity::ActivityBattleHandoff) -> BattleResult,
{
    fn execute(&mut self, handoff: &starclock_activity::ActivityBattleHandoff) -> BattleResult {
        self(handoff)
    }
}

/// Immutable baseline policy shared by all automatic Standard Universe runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardUniverseBaselinePolicy {
    hints: ActivityBaselineHints,
    technique_points: u16,
    max_steps: u32,
}

impl StandardUniverseBaselinePolicy {
    pub fn new(
        hints: ActivityBaselineHints,
        technique_points: u16,
        max_steps: u32,
    ) -> Result<Self, StandardUniverseBaselinePolicyError> {
        if max_steps == 0 {
            return Err(StandardUniverseBaselinePolicyError::ZeroStepBudget);
        }
        Ok(Self {
            hints,
            technique_points,
            max_steps,
        })
    }

    #[must_use]
    pub const fn hints(&self) -> &ActivityBaselineHints {
        &self.hints
    }
    #[must_use]
    pub const fn technique_points(&self) -> u16 {
        self.technique_points
    }
    #[must_use]
    pub const fn max_steps(&self) -> u32 {
        self.max_steps
    }
}

impl Default for StandardUniverseBaselinePolicy {
    fn default() -> Self {
        Self {
            hints: ActivityBaselineHints::default(),
            technique_points: 5,
            max_steps: 10_000,
        }
    }
}

/// One accepted automatic command boundary and its controller diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StandardUniverseBaselineStep {
    Decision {
        decision: ActivityBaselineDecision,
        state_hash: ActivityStateHash,
    },
    Preparation {
        option: ActivityOptionId,
        state_hash: ActivityStateHash,
    },
    Battle {
        identity: BattleResultIdentity,
        result_digest: BattleResultDigest,
        outcome: BattleOutcome,
        state_hash: ActivityStateHash,
    },
}

/// Terminal run diagnostics. The final hash is the hash after all accepted
/// commands and nested battle settlements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardUniverseBaselineReport {
    terminal: ActivityTerminalOutcome,
    final_state_hash: ActivityStateHash,
    steps: Box<[StandardUniverseBaselineStep]>,
}

impl StandardUniverseBaselineReport {
    #[must_use]
    pub const fn terminal(&self) -> ActivityTerminalOutcome {
        self.terminal
    }
    #[must_use]
    pub const fn final_state_hash(&self) -> ActivityStateHash {
        self.final_state_hash
    }
    #[must_use]
    pub fn steps(&self) -> &[StandardUniverseBaselineStep] {
        &self.steps
    }
}

/// Stateless driver. Every mutation is delegated to the checked Activity
/// facade, using the exact state hash, decision ID and option ID in its view.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StandardUniverseBaselineRunner {
    controller: ActivityBaselineController,
}

impl StandardUniverseBaselineRunner {
    pub const REVISION: &'static str = "standard-universe-baseline-runner-v1";

    pub fn advance<E: NestedBattleExecutor>(
        self,
        activity: &mut StandardUniverseActivity,
        policy: &StandardUniverseBaselinePolicy,
        executor: &mut E,
    ) -> Result<StandardUniverseBaselineStep, StandardUniverseBaselineError> {
        let view = activity.view();
        if view.terminal().is_some() {
            return Err(StandardUniverseBaselineError::AlreadyTerminal);
        }
        if view.pending_battle().is_some() {
            let handoff = activity
                .start_pending_battle(view.state_hash())
                .map_err(StandardUniverseBaselineError::BattleStart)?;
            let identity = handoff.identity();
            let result = executor.execute(&handoff);
            let result_digest = result.actual_digest();
            let settled = activity
                .submit_pending_battle_result(activity.view().state_hash(), result)
                .map_err(StandardUniverseBaselineError::BattleSettlement)?;
            return Ok(StandardUniverseBaselineStep::Battle {
                identity,
                result_digest,
                outcome: settled.settlement().outcome(),
                state_hash: settled.state_hash(),
            });
        }
        if let Some(preparation) = view.preparation() {
            let option = self
                .controller
                .decide_preparation(preparation, policy.hints())
                .map_err(StandardUniverseBaselineError::Controller)?;
            activity
                .choose_preparation_option(view.state_hash(), option)
                .map_err(StandardUniverseBaselineError::Preparation)?;
            return Ok(StandardUniverseBaselineStep::Preparation {
                option,
                state_hash: activity.view().state_hash(),
            });
        }
        let decision_view = view
            .decision()
            .ok_or(StandardUniverseBaselineError::MissingBoundary)?;
        let selected = self
            .controller
            .decide(decision_view, policy.hints())
            .map_err(StandardUniverseBaselineError::Controller)?;
        apply_decision(
            activity,
            view.state_hash(),
            &selected,
            policy.technique_points(),
        )?;
        Ok(StandardUniverseBaselineStep::Decision {
            decision: selected,
            state_hash: activity.view().state_hash(),
        })
    }

    pub fn run_to_terminal<E: NestedBattleExecutor>(
        self,
        activity: &mut StandardUniverseActivity,
        policy: &StandardUniverseBaselinePolicy,
        executor: &mut E,
    ) -> Result<StandardUniverseBaselineReport, StandardUniverseBaselineError> {
        let mut steps = Vec::new();
        for _ in 0..policy.max_steps() {
            let view = activity.view();
            if let Some(terminal) = view.terminal() {
                return Ok(StandardUniverseBaselineReport {
                    terminal,
                    final_state_hash: view.state_hash(),
                    steps: steps.into_boxed_slice(),
                });
            }
            steps.push(self.advance(activity, policy, executor)?);
        }
        Err(StandardUniverseBaselineError::StepBudgetExceeded)
    }
}

fn apply_decision(
    activity: &mut StandardUniverseActivity,
    state_hash: ActivityStateHash,
    selected: &ActivityBaselineDecision,
    technique_points: u16,
) -> Result<(), StandardUniverseBaselineError> {
    match selected.kind() {
        ActivityDecisionKind::Encounter => {
            activity
                .engage_encounter(
                    state_hash,
                    selected.decision(),
                    selected.option(),
                    technique_points,
                )
                .map_err(StandardUniverseBaselineError::Encounter)?;
        }
        ActivityDecisionKind::ExternalOutcome => {
            let outcome = ActivityExternalOutcomeId::new(selected.option().get())
                .expect("offered Activity option IDs are non-zero");
            activity
                .submit_external_outcome(state_hash, selected.decision(), outcome)
                .map_err(StandardUniverseBaselineError::Command)?;
        }
        _ => {
            activity
                .choose_option(state_hash, selected.decision(), selected.option())
                .map_err(StandardUniverseBaselineError::Command)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseBaselinePolicyError {
    ZeroStepBudget,
}

#[derive(Debug)]
pub enum StandardUniverseBaselineError {
    AlreadyTerminal,
    MissingBoundary,
    StepBudgetExceeded,
    Controller(ActivityDecisionError),
    Command(GraphActivityCommandError),
    Encounter(StandardUniverseEncounterError),
    Preparation(GraphActivityEncounterError),
    BattleStart(StandardUniverseBattleStartError),
    BattleSettlement(GraphActivityBattleError),
}
