//! Deterministic complete-run controller over offered Activity boundaries.

use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityOptionId, ActivityStateHash,
    ActivityTerminalOutcome, AttemptId, BattleOutcome, BattleResultDigest, BattleSequence,
    GraphActivity, GraphActivityCommandError,
};
use starclock_data::{
    catalog::SimulationCatalog, divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId,
};

use crate::{
    baseline_controller::{
        ActivityBaselineController, ActivityBaselineDecision, ActivityBaselineHints,
        ActivityDecisionError,
    },
    nested_battle_executor::NestedBattleExecutionReport,
};

use super::source_deck_selection::NODE as SOURCE_DECK_NODE;
use super::{
    DivergentUniverseBattleAssemblyError, DivergentUniverseBattleAssemblyPolicy,
    DivergentUniverseBattleSettlementError, DivergentUniverseContributionSnapshotError,
    DivergentUniverseEncounterReachabilityError, DivergentUniverseFlowInstance,
    DivergentUniverseRuntimeFactory, occurrence_binding::OccurrenceExecutionError,
};

/// Immutable controller policy. Group/stage are fallback inputs only for
/// unbound low-level flows; production flows resolve their authored public offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBaselinePolicy {
    hints: ActivityBaselineHints,
    encounter_group: DivergentUniverseEncounterGroupId,
    encounter_stage: Box<str>,
    max_steps: u32,
}

impl DivergentUniverseBaselinePolicy {
    pub fn new(
        hints: ActivityBaselineHints,
        encounter_group: DivergentUniverseEncounterGroupId,
        encounter_stage: impl Into<Box<str>>,
        max_steps: u32,
    ) -> Result<Self, DivergentUniverseBaselinePolicyError> {
        let encounter_stage = encounter_stage.into();
        if encounter_stage.is_empty() {
            return Err(DivergentUniverseBaselinePolicyError::EmptyEncounterStage);
        }
        if max_steps == 0 {
            return Err(DivergentUniverseBaselinePolicyError::ZeroStepBudget);
        }
        Ok(Self {
            hints,
            encounter_group,
            encounter_stage,
            max_steps,
        })
    }

    #[must_use]
    pub const fn hints(&self) -> &ActivityBaselineHints {
        &self.hints
    }
    #[must_use]
    pub const fn encounter_group(&self) -> &DivergentUniverseEncounterGroupId {
        &self.encounter_group
    }
    #[must_use]
    pub fn encounter_stage(&self) -> &str {
        &self.encounter_stage
    }
    #[must_use]
    pub const fn max_steps(&self) -> u32 {
        self.max_steps
    }
}

/// One accepted controller boundary. Each decision contains the complete
/// scored offer and each battle contains the actual checked command trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBaselineStep {
    ActivityDecision {
        decision: ActivityBaselineDecision,
        state_hash: ActivityStateHash,
    },
    Battle {
        decision: ActivityBaselineDecision,
        identity: Box<starclock_activity::BattleResultIdentity>,
        encounter_group: DivergentUniverseEncounterGroupId,
        encounter_stage: Box<str>,
        result_digest: BattleResultDigest,
        outcome: BattleOutcome,
        execution: NestedBattleExecutionReport,
        state_hash: ActivityStateHash,
    },
}

impl DivergentUniverseBaselineStep {
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        match self {
            Self::ActivityDecision { state_hash, .. } | Self::Battle { state_hash, .. } => {
                *state_hash
            }
        }
    }
}

/// Terminal receipt for one complete run from a caller-owned fresh Activity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBaselineReport {
    run_family: DivergentUniverseRunFamily,
    terminal: ActivityTerminalOutcome,
    final_state_hash: ActivityStateHash,
    completed_battles: u32,
    steps: Box<[DivergentUniverseBaselineStep]>,
}

impl DivergentUniverseBaselineReport {
    #[must_use]
    pub const fn run_family(&self) -> DivergentUniverseRunFamily {
        self.run_family
    }
    #[must_use]
    pub const fn terminal(&self) -> ActivityTerminalOutcome {
        self.terminal
    }
    #[must_use]
    pub const fn final_state_hash(&self) -> ActivityStateHash {
        self.final_state_hash
    }
    #[must_use]
    pub const fn completed_battles(&self) -> u32 {
        self.completed_battles
    }
    #[must_use]
    pub fn steps(&self) -> &[DivergentUniverseBaselineStep] {
        &self.steps
    }
}

/// Stateless mode facade over the shared Activity scorer and the production
/// Divergent Universe battle assembly/settlement path.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DivergentUniverseBaselineRunner {
    controller: ActivityBaselineController,
}

/// Exact public decision/option identity selected by an external adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOfferedSelection {
    decision: ActivityDecisionId,
    option: ActivityOptionId,
}

impl DivergentUniverseOfferedSelection {
    #[must_use]
    pub const fn new(decision: ActivityDecisionId, option: ActivityOptionId) -> Self {
        Self { decision, option }
    }
}

impl DivergentUniverseBaselineRunner {
    pub const ID: &'static str = "divergent-universe-baseline-runner";

    pub fn advance(
        self,
        factory: &DivergentUniverseRuntimeFactory,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        core: &SimulationCatalog,
        policy: &DivergentUniverseBaselinePolicy,
    ) -> Result<DivergentUniverseBaselineStep, DivergentUniverseBaselineError> {
        let (state_hash, selected) = {
            let view = activity.player_view();
            if view.terminal().is_some() {
                return Err(DivergentUniverseBaselineError::AlreadyTerminal);
            }
            let decision = view
                .decision()
                .ok_or(DivergentUniverseBaselineError::MissingOfferedDecision)?;
            let selected = self
                .controller
                .decide(decision, policy.hints())
                .map_err(DivergentUniverseBaselineError::Controller)?;
            (view.state_hash(), selected)
        };
        advance_decision(factory, flow, activity, core, policy, state_hash, selected)
    }

    /// Applies one exact caller-selected option from the current public offer.
    /// Selection and decision identity are validated before any mutation.
    pub fn advance_selected(
        self,
        factory: &DivergentUniverseRuntimeFactory,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        core: &SimulationCatalog,
        policy: &DivergentUniverseBaselinePolicy,
        selection: DivergentUniverseOfferedSelection,
    ) -> Result<DivergentUniverseBaselineStep, DivergentUniverseBaselineError> {
        let (state_hash, selected) = {
            let view = activity.player_view();
            if view.terminal().is_some() {
                return Err(DivergentUniverseBaselineError::AlreadyTerminal);
            }
            let decision = view
                .decision()
                .ok_or(DivergentUniverseBaselineError::MissingOfferedDecision)?;
            if decision.id() != selection.decision {
                return Err(DivergentUniverseBaselineError::OfferedDecisionMismatch);
            }
            let selected = self
                .controller
                .select_offered(decision, selection.option, policy.hints())
                .map_err(DivergentUniverseBaselineError::Controller)?;
            (view.state_hash(), selected)
        };
        advance_decision(factory, flow, activity, core, policy, state_hash, selected)
    }

    pub fn run_to_terminal(
        self,
        factory: &DivergentUniverseRuntimeFactory,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        core: &SimulationCatalog,
        policy: &DivergentUniverseBaselinePolicy,
    ) -> Result<DivergentUniverseBaselineReport, DivergentUniverseBaselineError> {
        if activity.definition().identity() != flow.definition().identity() {
            return Err(DivergentUniverseBaselineError::DefinitionMismatch);
        }
        if !flow.has_runtime_battle_route() {
            return Err(DivergentUniverseBaselineError::MissingBattleRoute);
        }
        let mut steps = Vec::new();
        for _ in 0..policy.max_steps() {
            let view = activity.player_view();
            if let Some(terminal) = view.terminal() {
                return Ok(DivergentUniverseBaselineReport {
                    run_family: flow.run_family(),
                    terminal,
                    final_state_hash: view.state_hash(),
                    completed_battles: view.completed_battle_count(),
                    steps: steps.into_boxed_slice(),
                });
            }
            steps.push(self.advance(factory, flow, activity, core, policy)?);
        }
        Err(DivergentUniverseBaselineError::StepBudgetExceeded)
    }
}

fn advance_decision(
    factory: &DivergentUniverseRuntimeFactory,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    core: &SimulationCatalog,
    policy: &DivergentUniverseBaselinePolicy,
    state_hash: ActivityStateHash,
    selected: ActivityBaselineDecision,
) -> Result<DivergentUniverseBaselineStep, DivergentUniverseBaselineError> {
    if selected.kind() == ActivityDecisionKind::Route
        && flow
            .has_position_domain_hand(activity)
            .map_err(DivergentUniverseBaselineError::ActivityCommand)?
    {
        flow.choose_position_domain_card(
            activity,
            state_hash,
            selected.decision(),
            selected.option(),
        )
        .map_err(DivergentUniverseBaselineError::ActivityCommand)?;
        return Ok(DivergentUniverseBaselineStep::ActivityDecision {
            decision: selected,
            state_hash: activity.state_hash(),
        });
    }
    if selected.kind() == ActivityDecisionKind::Route
        && flow
            .offered_battle_domain_choices(activity)
            .map_err(DivergentUniverseBaselineError::ActivityCommand)?
            .is_some()
    {
        flow.choose_battle_domain(activity, state_hash, selected.decision(), selected.option())
            .map_err(DivergentUniverseBaselineError::ActivityCommand)?;
        return Ok(DivergentUniverseBaselineStep::ActivityDecision {
            decision: selected,
            state_hash: activity.state_hash(),
        });
    }
    if selected.kind() == ActivityDecisionKind::Service
        && flow.offered_evolution_event(activity).is_some()
    {
        flow.choose_evolution_option(activity, state_hash, selected.decision(), selected.option())
            .map_err(DivergentUniverseBaselineError::ActivityCommand)?;
        return Ok(DivergentUniverseBaselineStep::ActivityDecision {
            decision: selected,
            state_hash: activity.state_hash(),
        });
    }
    if flow.offered_tawot_service(activity).is_some() {
        flow.choose_tawot_service_option(
            activity,
            state_hash,
            selected.decision(),
            selected.option(),
        )
        .map_err(DivergentUniverseBaselineError::ActivityCommand)?;
        return Ok(DivergentUniverseBaselineStep::ActivityDecision {
            decision: selected,
            state_hash: activity.state_hash(),
        });
    }
    if selected.kind() == ActivityDecisionKind::Preparation {
        if activity.current_node() == SOURCE_DECK_NODE {
            flow.choose_source_deck(activity, state_hash, selected.decision(), selected.option())
        } else {
            flow.choose_initial_equation(
                activity,
                state_hash,
                selected.decision(),
                selected.option(),
            )
        }
        .map_err(DivergentUniverseBaselineError::ActivityCommand)?;
        return Ok(DivergentUniverseBaselineStep::ActivityDecision {
            decision: selected,
            state_hash: activity.state_hash(),
        });
    }
    if selected.kind() == ActivityDecisionKind::Encounter {
        return execute_battle(factory, flow, activity, core, policy, selected);
    }
    if selected.kind() == ActivityDecisionKind::Reward {
        flow.choose_battle_blessing(activity, state_hash, selected.decision(), selected.option())
            .map_err(DivergentUniverseBaselineError::ActivityCommand)?;
        return Ok(DivergentUniverseBaselineStep::ActivityDecision {
            decision: selected,
            state_hash: activity.state_hash(),
        });
    }
    if selected.kind() == ActivityDecisionKind::Choice
        && flow.offered_occurrence(activity).is_some()
    {
        flow.choose_occurrence_option(
            factory,
            activity,
            state_hash,
            selected.decision(),
            selected.option(),
        )
        .map_err(DivergentUniverseBaselineError::Occurrence)?;
        return Ok(DivergentUniverseBaselineStep::ActivityDecision {
            decision: selected,
            state_hash: activity.state_hash(),
        });
    }
    activity
        .choose_option(state_hash, selected.decision(), selected.option())
        .map_err(DivergentUniverseBaselineError::ActivityCommand)?;
    Ok(DivergentUniverseBaselineStep::ActivityDecision {
        decision: selected,
        state_hash: activity.state_hash(),
    })
}

pub(super) fn completed_report(
    flow: &DivergentUniverseFlowInstance,
    activity: &GraphActivity,
    steps: Vec<DivergentUniverseBaselineStep>,
) -> Result<DivergentUniverseBaselineReport, DivergentUniverseBaselineError> {
    if activity.definition().identity() != flow.definition().identity() {
        return Err(DivergentUniverseBaselineError::DefinitionMismatch);
    }
    let view = activity.player_view();
    let terminal = view
        .terminal()
        .ok_or(DivergentUniverseBaselineError::NotTerminal)?;
    Ok(DivergentUniverseBaselineReport {
        run_family: flow.run_family(),
        terminal,
        final_state_hash: view.state_hash(),
        completed_battles: view.completed_battle_count(),
        steps: steps.into_boxed_slice(),
    })
}

fn execute_battle(
    factory: &DivergentUniverseRuntimeFactory,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    core: &SimulationCatalog,
    policy: &DivergentUniverseBaselinePolicy,
    decision: ActivityBaselineDecision,
) -> Result<DivergentUniverseBaselineStep, DivergentUniverseBaselineError> {
    let expected = activity.state_hash();
    let (encounter_group, encounter_stage) = flow
        .offered_encounter(activity)
        .map_err(DivergentUniverseBaselineError::ActivityCommand)?
        .unwrap_or((policy.encounter_group(), policy.encounter_stage()));
    let contribution = factory
        .contribution_snapshot_runtime()
        .map_err(DivergentUniverseBaselineError::Contribution)?
        .snapshot(flow, activity)
        .map_err(DivergentUniverseBaselineError::Contribution)?;
    let encounter = factory
        .encounter_reachability_runtime()
        .map_err(DivergentUniverseBaselineError::Encounter)?
        .select_stage_candidate(activity, expected, encounter_group, encounter_stage)
        .map_err(DivergentUniverseBaselineError::Encounter)?;
    let assembled = factory
        .battle_assembly_runtime()
        .resolve_current_battle(
            flow,
            activity,
            core,
            &contribution,
            &encounter,
            DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        )
        .map_err(DivergentUniverseBaselineError::BattleAssembly)?;
    let sequence = activity
        .player_view()
        .completed_battle_count()
        .checked_add(1)
        .ok_or(DivergentUniverseBaselineError::BattleSequenceExhausted)?;
    let resolution = factory
        .battle_settlement_runtime()
        .execute_current_battle(
            flow,
            activity,
            expected,
            AttemptId::new(sequence)
                .ok_or(DivergentUniverseBaselineError::BattleSequenceExhausted)?,
            BattleSequence::new(sequence)
                .ok_or(DivergentUniverseBaselineError::BattleSequenceExhausted)?,
            &assembled,
        )
        .map_err(DivergentUniverseBaselineError::BattleSettlement)?;
    let execution = resolution
        .report()
        .cloned()
        .ok_or(DivergentUniverseBaselineError::MissingBattleReport)?;
    let identity = resolution.result().identity();
    Ok(DivergentUniverseBaselineStep::Battle {
        decision,
        identity: Box::new(identity),
        encounter_group: encounter_group.clone(),
        encounter_stage: encounter_stage.into(),
        result_digest: resolution.result().actual_digest(),
        outcome: resolution.settlement().outcome(),
        execution,
        state_hash: resolution.state_hash(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBaselinePolicyError {
    EmptyEncounterStage,
    ZeroStepBudget,
}

#[derive(Debug)]
pub enum DivergentUniverseBaselineError {
    AlreadyTerminal,
    DefinitionMismatch,
    MissingBattleRoute,
    MissingOfferedDecision,
    OfferedDecisionMismatch,
    NotTerminal,
    StepBudgetExceeded,
    BattleSequenceExhausted,
    MissingBattleReport,
    Controller(ActivityDecisionError),
    ActivityCommand(GraphActivityCommandError),
    Occurrence(OccurrenceExecutionError),
    Contribution(DivergentUniverseContributionSnapshotError),
    Encounter(DivergentUniverseEncounterReachabilityError),
    BattleAssembly(DivergentUniverseBattleAssemblyError),
    BattleSettlement(DivergentUniverseBattleSettlementError),
}
