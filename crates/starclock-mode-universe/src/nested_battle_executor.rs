//! Production synchronous execution of an Activity battle handoff.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, BattleOutcome, BattleResult, EventDigest, GraphActivityBattleError,
    MetricValue, MetricValueKind, ParticipantBattleState, ProjectedValue, ProjectionField,
};
use starclock_ai::{EnemyController, EnemyDecisionError};
use starclock_combat::{
    Battle, BattlePhase, BattleStateHash, Command, DecisionKind, DecisionOwner, LifeState,
    ParticipantInitialState, ParticipantSpec, TeamSide,
    catalog::{CombatCatalog, encounter::AiTransitionTiming},
    rng::types::RngSeed,
    rule::model::ConditionExpr,
};

use crate::{
    baseline_runner::{
        DynamicNestedBattleExecutor, NestedBattleExecutionError, NestedBattleExecutor,
    },
    digest::Encoder,
    dynamic_battle_assembler::StandardUniverseDynamicBattleStart,
    dynamic_battle_assembler::{
        StandardUniverseBattleAssembler, StandardUniverseDynamicBattleError,
    },
};

pub const DEFAULT_NESTED_BATTLE_COMMAND_BUDGET: u32 = 10_000;

/// Controller responsible for one accepted nested-battle command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum NestedBattleController {
    System = 0,
    BaselinePlayer = 1,
    AuthoredEnemy = 2,
}

/// One accepted command boundary with payload-direct replay evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedBattleTraceEntry {
    controller: NestedBattleController,
    command: Command,
    state_hash: BattleStateHash,
    events: Box<[starclock_combat::BattleEvent]>,
}

impl NestedBattleTraceEntry {
    #[must_use]
    pub const fn controller(&self) -> NestedBattleController {
        self.controller
    }
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
    #[must_use]
    pub const fn state_hash(&self) -> BattleStateHash {
        self.state_hash
    }
    #[must_use]
    pub const fn emitted_events(&self) -> u32 {
        self.events.len() as u32
    }
    #[must_use]
    pub fn events(&self) -> &[starclock_combat::BattleEvent] {
        &self.events
    }
}

/// Complete diagnostics for the most recently executed handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedBattleExecutionReport {
    outcome: BattleOutcome,
    final_state_hash: BattleStateHash,
    event_digest: EventDigest,
    terminal_fault: Option<starclock_combat::BattleFault>,
    trace: Box<[NestedBattleTraceEntry]>,
}

impl NestedBattleExecutionReport {
    #[must_use]
    pub const fn outcome(&self) -> BattleOutcome {
        self.outcome
    }
    #[must_use]
    pub const fn final_state_hash(&self) -> BattleStateHash {
        self.final_state_hash
    }
    #[must_use]
    pub const fn event_digest(&self) -> EventDigest {
        self.event_digest
    }
    #[must_use]
    pub const fn terminal_fault(&self) -> Option<starclock_combat::BattleFault> {
        self.terminal_fault
    }
    #[must_use]
    pub fn trace(&self) -> &[NestedBattleTraceEntry] {
        &self.trace
    }
}

/// Deterministic production executor over the generic combat aggregate.
pub struct UniverseNestedBattleExecutor {
    compatibility_catalog: Option<Arc<CombatCatalog>>,
    command_budget: u32,
    reports: Vec<NestedBattleExecutionReport>,
}

impl UniverseNestedBattleExecutor {
    #[must_use]
    pub fn new(catalog: Arc<CombatCatalog>) -> Self {
        Self {
            compatibility_catalog: Some(catalog),
            command_budget: DEFAULT_NESTED_BATTLE_COMMAND_BUDGET,
            reports: Vec::new(),
        }
    }

    /// Creates an executor that accepts only assembler-paired dynamic starts.
    #[must_use]
    pub fn dynamic() -> Self {
        Self {
            compatibility_catalog: None,
            command_budget: DEFAULT_NESTED_BATTLE_COMMAND_BUDGET,
            reports: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_command_budget(mut self, command_budget: u32) -> Option<Self> {
        if command_budget == 0 {
            None
        } else {
            self.command_budget = command_budget;
            Some(self)
        }
    }

    #[must_use]
    pub const fn command_budget(&self) -> u32 {
        self.command_budget
    }

    #[must_use]
    pub fn last_report(&self) -> Option<&NestedBattleExecutionReport> {
        self.reports.last()
    }

    #[must_use]
    pub fn reports(&self) -> &[NestedBattleExecutionReport] {
        &self.reports
    }

    /// Starts, executes and settles the current Activity handoff as one adapter
    /// operation. Infrastructure or settlement failure restores the unstarted
    /// Activity boundary and does not append a report.
    pub fn execute_pending_activity_battle(
        &mut self,
        activity: &mut crate::runtime::StandardUniverseActivity,
    ) -> Result<SettledNestedBattle, ActivityNestedBattleExecutionError> {
        let catalog = self.compatibility_catalog.as_ref().ok_or(
            ActivityNestedBattleExecutionError::Execution(NestedBattleExecutionError::BattleBuild),
        )?;
        let handoff = activity
            .start_pending_battle(activity.view().state_hash())
            .map_err(ActivityNestedBattleExecutionError::Start)?;
        let (result, report) = match self.execute_checked(catalog, &handoff) {
            Ok(value) => value,
            Err(error) => {
                let restored = activity.rollback_pending_battle_start();
                debug_assert!(restored, "this adapter started the exact pending handoff");
                return Err(ActivityNestedBattleExecutionError::Execution(error));
            }
        };
        if let Err(error) =
            activity.submit_pending_battle_result(activity.view().state_hash(), result.clone())
        {
            let restored = activity.rollback_pending_battle_start();
            debug_assert!(restored, "failed settlement retains the started handoff");
            return Err(ActivityNestedBattleExecutionError::Settlement(error));
        }
        self.reports.push(report.clone());
        Ok(SettledNestedBattle { result, report })
    }

    /// Dynamically assembles, executes and settles the current Activity battle
    /// with rollback on every post-start failure.
    pub fn execute_dynamic_pending_activity_battle(
        &mut self,
        activity: &mut crate::runtime::StandardUniverseActivity,
        assembler: &StandardUniverseBattleAssembler,
    ) -> Result<SettledNestedBattle, ActivityNestedBattleExecutionError> {
        let start = assembler
            .start_pending_battle(activity)
            .map_err(ActivityNestedBattleExecutionError::DynamicStart)?;
        let (result, report) = match self.execute_checked(start.combat_catalog(), start.handoff()) {
            Ok(value) => value,
            Err(error) => {
                let restored = activity.rollback_pending_battle_start();
                debug_assert!(restored, "the assembler started the exact pending handoff");
                return Err(ActivityNestedBattleExecutionError::Execution(error));
            }
        };
        if let Err(error) =
            activity.submit_pending_battle_result(activity.view().state_hash(), result.clone())
        {
            let restored = activity.rollback_pending_battle_start();
            debug_assert!(restored, "failed settlement retains the started handoff");
            return Err(ActivityNestedBattleExecutionError::Settlement(error));
        }
        self.reports.push(report.clone());
        Ok(SettledNestedBattle { result, report })
    }

    fn execute_checked(
        &self,
        catalog: &Arc<CombatCatalog>,
        handoff: &ActivityBattleHandoff,
    ) -> Result<(BattleResult, NestedBattleExecutionReport), NestedBattleExecutionError> {
        let mut battle = create_nested_battle(Arc::clone(catalog), handoff)?;
        let mut enemy = EnemyController::new(enemy_controller_seed(handoff));
        let mut commitment = EventCommitment::new(catalog, handoff);
        let mut trace = Vec::new();
        let mut timeline_elapsed_scaled = 0_i64;

        for _ in 0..self.command_budget {
            if battle.view().phase().is_terminal() {
                let result = project_result(
                    &battle,
                    handoff,
                    commitment.finish(),
                    timeline_elapsed_scaled,
                )?;
                let report = report(&battle, &result, trace)?;
                return Ok((result, report));
            }
            let decision = battle
                .decision()
                .cloned()
                .ok_or(NestedBattleExecutionError::MissingDecision)?;
            let (command, controller, enemy_action) =
                select_command(&battle, catalog, &mut enemy, &decision)?;
            let resolution = battle
                .apply(command.clone())
                .map_err(|_| NestedBattleExecutionError::CommandRejected)?;
            timeline_elapsed_scaled = timeline_elapsed_scaled
                .checked_add(resolution.timeline_elapsed_scaled())
                .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
            commitment.push(&command, &resolution);
            trace.push(NestedBattleTraceEntry {
                controller,
                command,
                state_hash: resolution.state_hash(),
                events: resolution.events().to_vec().into_boxed_slice(),
            });
            if let Some((actor, graph)) = enemy_action {
                enemy
                    .settle(
                        graph,
                        actor,
                        AiTransitionTiming::AfterAction,
                        static_condition,
                    )
                    .map_err(NestedBattleExecutionError::EnemyDecision)?;
            }
        }
        Err(NestedBattleExecutionError::StepBudgetExceeded)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettledNestedBattle {
    result: BattleResult,
    report: NestedBattleExecutionReport,
}

impl SettledNestedBattle {
    #[must_use]
    pub const fn result(&self) -> &BattleResult {
        &self.result
    }
    #[must_use]
    pub const fn report(&self) -> &NestedBattleExecutionReport {
        &self.report
    }
}

#[derive(Debug)]
pub enum ActivityNestedBattleExecutionError {
    Start(crate::runtime::StandardUniverseBattleStartError),
    DynamicStart(StandardUniverseDynamicBattleError),
    Execution(NestedBattleExecutionError),
    Settlement(GraphActivityBattleError),
}

impl NestedBattleExecutor for UniverseNestedBattleExecutor {
    fn execute(
        &mut self,
        handoff: &ActivityBattleHandoff,
    ) -> Result<BattleResult, NestedBattleExecutionError> {
        let catalog = self
            .compatibility_catalog
            .as_ref()
            .ok_or(NestedBattleExecutionError::BattleBuild)?;
        let (result, report) = self.execute_checked(catalog, handoff)?;
        self.reports.push(report);
        Ok(result)
    }
}

impl DynamicNestedBattleExecutor for UniverseNestedBattleExecutor {
    fn execute_dynamic(
        &mut self,
        start: &StandardUniverseDynamicBattleStart,
    ) -> Result<BattleResult, NestedBattleExecutionError> {
        let (result, report) = self.execute_checked(start.combat_catalog(), start.handoff())?;
        self.reports.push(report);
        Ok(result)
    }
}

pub(crate) fn create_nested_battle(
    catalog: Arc<CombatCatalog>,
    handoff: &ActivityBattleHandoff,
) -> Result<Battle, NestedBattleExecutionError> {
    let spec = carried_spec(handoff)?;
    Battle::create(catalog, spec, handoff.identity().seed())
        .map_err(|_| NestedBattleExecutionError::BattleBuild)
}

fn carried_spec(
    handoff: &ActivityBattleHandoff,
) -> Result<starclock_combat::BattleSpec, NestedBattleExecutionError> {
    if handoff.participants().len() != handoff.participant_carry().len() {
        return Err(NestedBattleExecutionError::ParticipantMapping);
    }
    let mut participants = Vec::with_capacity(handoff.battle_spec().participants().len());
    for participant in handoff.battle_spec().participants() {
        let mut participant = participant.clone();
        if participant.side() == TeamSide::Player {
            participant = carried_participant(participant, handoff)?;
        }
        participants.push(participant);
    }
    starclock_combat::BattleSpec::new(
        handoff.identity().assembly_digest(),
        handoff.battle_spec().encounter(),
        participants,
        handoff.battle_spec().resources(TeamSide::Player).clone(),
        handoff.battle_spec().resources(TeamSide::Enemy).clone(),
        handoff.battle_spec().concede_policy(),
    )
    .map_err(|_| NestedBattleExecutionError::BattleBuild)
}

fn carried_participant(
    participant: ParticipantSpec,
    handoff: &ActivityBattleHandoff,
) -> Result<ParticipantSpec, NestedBattleExecutionError> {
    let binding = handoff
        .participants()
        .iter()
        .find(|binding| binding.formation() == participant.formation())
        .ok_or(NestedBattleExecutionError::ParticipantMapping)?;
    let carry = handoff
        .participant_carry()
        .iter()
        .find(|carry| carry.participant() == binding.participant())
        .ok_or(NestedBattleExecutionError::ParticipantMapping)?;
    let initial = ParticipantInitialState::new(
        carry.current_hp(),
        participant.combatant().maximum_hp(),
        carry.current_energy(),
        participant.combatant().maximum_energy(),
        carry.life(),
        carry.presence(),
    )
    .ok_or(NestedBattleExecutionError::ParticipantMapping)?;
    participant
        .with_initial_state(initial)
        .ok_or(NestedBattleExecutionError::ParticipantMapping)
}

type EnemyAction<'a> = (
    starclock_combat::UnitId,
    &'a starclock_combat::catalog::encounter::AiGraphDefinition,
);

fn select_command<'a>(
    battle: &Battle,
    catalog: &'a CombatCatalog,
    enemy: &mut EnemyController,
    decision: &starclock_combat::DecisionPoint,
) -> Result<(Command, NestedBattleController, Option<EnemyAction<'a>>), NestedBattleExecutionError>
{
    if decision.kind() == DecisionKind::InterruptWindow
        && let Some(command) = decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
    {
        let controller = match decision.owner() {
            DecisionOwner::System => NestedBattleController::System,
            DecisionOwner::Team(TeamSide::Player) => NestedBattleController::BaselinePlayer,
            DecisionOwner::Team(TeamSide::Enemy) => NestedBattleController::AuthoredEnemy,
        };
        return Ok((command.clone(), controller, None));
    }
    if decision.kind() == DecisionKind::BattleChoice && decision.legal_commands().len() == 1 {
        let controller = match decision.owner() {
            DecisionOwner::System => NestedBattleController::System,
            DecisionOwner::Team(TeamSide::Player) => NestedBattleController::BaselinePlayer,
            DecisionOwner::Team(TeamSide::Enemy) => NestedBattleController::AuthoredEnemy,
        };
        return Ok((decision.legal_commands()[0].clone(), controller, None));
    }
    match decision.owner() {
        DecisionOwner::System => Ok((
            system_command(decision)?,
            NestedBattleController::System,
            None,
        )),
        DecisionOwner::Team(TeamSide::Player) => Ok((
            player_command(catalog, battle.view(), decision)?,
            NestedBattleController::BaselinePlayer,
            None,
        )),
        DecisionOwner::Team(TeamSide::Enemy) => {
            let actor = decision
                .legal_commands()
                .iter()
                .find_map(command_actor)
                .ok_or(NestedBattleExecutionError::MissingEnemyController)?;
            let (_, initial_state, _) = battle
                .view()
                .units_by_id()
                .find(|unit| unit.id() == actor)
                .and_then(|unit| unit.enemy_ai_state())
                .ok_or(NestedBattleExecutionError::MissingEnemyController)?;
            let graph_id = battle
                .view()
                .units_by_id()
                .find(|unit| unit.id() == actor)
                .and_then(|unit| unit.enemy_ai_state())
                .map(|(graph, _, _)| graph)
                .ok_or(NestedBattleExecutionError::MissingEnemyController)?;
            let graph = catalog
                .ai_graph(graph_id)
                .ok_or(NestedBattleExecutionError::MissingEnemyController)?;
            let (command, settle_ai_state) =
                match enemy.decide(graph, initial_state, actor, decision, static_condition) {
                    Ok(selected) => (selected.command().clone(), true),
                    Err(EnemyDecisionError::NoLegalFallback) => (
                        decision
                            .legal_commands()
                            .first()
                            .cloned()
                            .ok_or(NestedBattleExecutionError::UnsupportedDecision)?,
                        false,
                    ),
                    Err(source) => return Err(NestedBattleExecutionError::EnemyDecision(source)),
                };
            Ok((
                command,
                NestedBattleController::AuthoredEnemy,
                settle_ai_state.then_some((actor, graph)),
            ))
        }
    }
}

fn system_command(
    decision: &starclock_combat::DecisionPoint,
) -> Result<Command, NestedBattleExecutionError> {
    let selected = match decision.kind() {
        DecisionKind::BattleStart => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::StartBattle { .. })),
        DecisionKind::InterruptWindow => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. })),
        DecisionKind::NormalAction | DecisionKind::BattleChoice => None,
    };
    selected
        .cloned()
        .ok_or(NestedBattleExecutionError::UnsupportedDecision)
}

fn player_command(
    catalog: &CombatCatalog,
    view: starclock_combat::BattleView<'_>,
    decision: &starclock_combat::DecisionPoint,
) -> Result<Command, NestedBattleExecutionError> {
    let selected = match decision.kind() {
        DecisionKind::InterruptWindow => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
            .or_else(|| {
                decision.legal_commands().iter().find(|command| {
                    matches!(command, Command::UseInterrupt { .. })
                        && command_is_affordable(catalog, view, command)
                })
            }),
        DecisionKind::NormalAction | DecisionKind::BattleChoice => decision
            .legal_commands()
            .iter()
            .find(|command| {
                let Command::UseAbility { ability, .. } = command else {
                    return false;
                };
                catalog
                    .ability(*ability)
                    .and_then(|definition| definition.action())
                    .is_some_and(|action| {
                        action.kind() == starclock_combat::catalog::action::AbilityKind::Basic
                            && command_is_affordable(catalog, view, command)
                    })
            })
            .or_else(|| {
                decision
                    .legal_commands()
                    .iter()
                    .find(|command| command_is_affordable(catalog, view, command))
            }),
        DecisionKind::BattleStart => None,
    };
    selected
        .cloned()
        .ok_or(NestedBattleExecutionError::UnsupportedDecision)
}

fn command_is_affordable(
    catalog: &CombatCatalog,
    view: starclock_combat::BattleView<'_>,
    command: &Command,
) -> bool {
    let (actor, ability) = match command {
        Command::UseAbility { actor, ability, .. }
        | Command::UseInterrupt { actor, ability, .. } => (*actor, *ability),
        _ => return true,
    };
    let Some(action) = catalog
        .ability(ability)
        .and_then(|definition| definition.action())
    else {
        return false;
    };
    let policy = action.resources();
    let Some(unit) = view.units_by_id().find(|unit| unit.id() == actor) else {
        return false;
    };
    if unit.current_energy() < policy.energy_cost()
        || policy.character_resource_costs().iter().any(|cost| {
            unit.character_resource(cost.stable_key())
                .is_none_or(|(current, _)| current < cost.amount())
        })
        || !policy.team_resource_costs().is_empty()
    {
        return false;
    }
    let team = view.team(unit.side());
    match policy.skill_point_payment() {
        starclock_combat::catalog::action::SkillPointPaymentPolicy::TeamSkillPoints => {
            team.skill_points() >= policy.skill_point_cost()
        }
        starclock_combat::catalog::action::SkillPointPaymentPolicy::Suppressed => true,
        starclock_combat::catalog::action::SkillPointPaymentPolicy::TeamResource(resource) => team
            .keyed_resource(resource)
            .is_some_and(|(current, _)| current >= policy.skill_point_cost()),
    }
}

fn command_actor(command: &Command) -> Option<starclock_combat::UnitId> {
    match command {
        Command::UseAbility { actor, .. } | Command::UseInterrupt { actor, .. } => Some(*actor),
        Command::StartBattle { .. }
        | Command::PassInterruptWindow { .. }
        | Command::Concede { .. } => None,
    }
}

fn static_condition(condition: &ConditionExpr) -> bool {
    match condition {
        ConditionExpr::Literal(value) => *value,
        ConditionExpr::Not(value) => !static_condition(value),
        ConditionExpr::All(values) => values.iter().all(static_condition),
        ConditionExpr::Any(values) => values.iter().any(static_condition),
        ConditionExpr::Compare { .. }
        | ConditionExpr::EventKind(_)
        | ConditionExpr::SourceTag(_)
        | ConditionExpr::SelectorCardinality { .. }
        | ConditionExpr::LifePresence { .. }
        | ConditionExpr::EffectExists { .. }
        | ConditionExpr::IsFrozen(_)
        | ConditionExpr::HasWeakness { .. }
        | ConditionExpr::IsBroken(_)
        | ConditionExpr::CurrentTargetIsBroken
        | ConditionExpr::EnemyRank { .. } => false,
    }
}

pub(crate) fn project_result(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    event_digest: EventDigest,
    timeline_elapsed_scaled: i64,
) -> Result<BattleResult, NestedBattleExecutionError> {
    let view = battle.view();
    let outcome = match view.phase() {
        BattlePhase::Won => BattleOutcome::Won,
        BattlePhase::Lost => BattleOutcome::Lost,
        BattlePhase::Faulted => BattleOutcome::Faulted,
        BattlePhase::Initializing | BattlePhase::AwaitingCommand | BattlePhase::Resolving => {
            return Err(NestedBattleExecutionError::MissingDecision);
        }
    };
    let mut values = Vec::with_capacity(handoff.projection().fields().len());
    for field in handoff.projection().fields() {
        values.push(match field {
            ProjectionField::Outcome => ProjectedValue::Outcome(outcome),
            ProjectionField::FinalStateHash => ProjectedValue::FinalStateHash(battle.state_hash()),
            ProjectionField::EventDigest => ProjectedValue::EventDigest(event_digest),
            ProjectionField::TerminalFault => ProjectedValue::TerminalFault(view.fault()),
            ProjectionField::ParticipantState(participant) => {
                let formation = handoff
                    .participants()
                    .iter()
                    .find(|binding| binding.participant() == *participant)
                    .map(|binding| binding.formation())
                    .ok_or(NestedBattleExecutionError::ParticipantMapping)?;
                let unit_id = view
                    .formation(TeamSide::Player)
                    .find(|entry| entry.index() == formation)
                    .map(|entry| entry.unit())
                    .ok_or(NestedBattleExecutionError::ParticipantMapping)?;
                let unit = view
                    .units_by_id()
                    .find(|unit| unit.id() == unit_id)
                    .ok_or(NestedBattleExecutionError::ParticipantMapping)?;
                let state = ParticipantBattleState::new(
                    *participant,
                    unit.current_hp(),
                    unit.maximum_hp(),
                    unit.current_energy(),
                    unit.maximum_energy(),
                    unit.life(),
                    unit.presence(),
                )
                .ok_or(NestedBattleExecutionError::ParticipantMapping)?;
                ProjectedValue::ParticipantState(state)
            }
            ProjectionField::Metric { key, kind }
                if key.as_ref() == crate::battle_materialization::DEFEATED_ENEMY_COUNT_METRIC
                    && *kind == MetricValueKind::BoundedInteger =>
            {
                let defeated = view
                    .units_by_id()
                    .filter(|unit| {
                        unit.side() == TeamSide::Enemy && unit.life() == LifeState::Defeated
                    })
                    .count();
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::BoundedInteger(
                        i64::try_from(defeated)
                            .map_err(|_| NestedBattleExecutionError::UnsupportedProjection)?,
                    ),
                }
            }
            ProjectionField::Metric { key, kind }
                if *kind == MetricValueKind::BoundedInteger
                    && key
                        .strip_prefix(
                            crate::battle_materialization::FIXED_BLESSING_COUNT_METRIC_PREFIX,
                        )
                        .is_some() =>
            {
                let count = key
                    .strip_prefix(crate::battle_materialization::FIXED_BLESSING_COUNT_METRIC_PREFIX)
                    .and_then(|value| value.parse::<i64>().ok())
                    .filter(|value| (1..=4).contains(value))
                    .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::BoundedInteger(count),
                }
            }
            ProjectionField::Metric { key, kind }
                if *kind == MetricValueKind::BoundedInteger
                    && key
                        .strip_prefix(
                            crate::battle_materialization::CYCLE_BLESSING_COUNT_METRIC_PREFIX,
                        )
                        .is_some() =>
            {
                let (cycles, base, bonus) = parse_cycle_reward_metric(key)?;
                let count =
                    cycle_reward_count(outcome, timeline_elapsed_scaled, cycles, base, bonus)?;
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::BoundedInteger(count),
                }
            }
            ProjectionField::Metric { .. } => {
                return Err(NestedBattleExecutionError::UnsupportedProjection);
            }
        });
    }
    Ok(BattleResult::seal(handoff.identity(), values))
}

fn cycle_reward_count(
    outcome: BattleOutcome,
    timeline_elapsed_scaled: i64,
    cycles: u8,
    base: u8,
    bonus: u8,
) -> Result<i64, NestedBattleExecutionError> {
    let cycle_limit = i64::from(cycles)
        .checked_mul(100_000_000)
        .and_then(|value| value.checked_add(50_000_000))
        .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
    if timeline_elapsed_scaled < 0 {
        return Err(NestedBattleExecutionError::UnsupportedProjection);
    }
    let earned_bonus = if outcome == BattleOutcome::Won && timeline_elapsed_scaled <= cycle_limit {
        bonus
    } else {
        0
    };
    Ok(i64::from(base) + i64::from(earned_bonus))
}

fn parse_cycle_reward_metric(key: &str) -> Result<(u8, u8, u8), NestedBattleExecutionError> {
    let value = key
        .strip_prefix(crate::battle_materialization::CYCLE_BLESSING_COUNT_METRIC_PREFIX)
        .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
    let mut parts = value.split('.');
    let cycles = parts
        .next()
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|value| (1..=8).contains(value))
        .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
    if parts.next() != Some("base") {
        return Err(NestedBattleExecutionError::UnsupportedProjection);
    }
    let base = parts
        .next()
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|value| (1..=4).contains(value))
        .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
    if parts.next() != Some("bonus") {
        return Err(NestedBattleExecutionError::UnsupportedProjection);
    }
    let bonus = parts
        .next()
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|value| (1..=4).contains(value))
        .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
    if parts.next().is_some() || base.saturating_add(bonus) > 4 {
        return Err(NestedBattleExecutionError::UnsupportedProjection);
    }
    Ok((cycles, base, bonus))
}

fn report(
    battle: &Battle,
    result: &BattleResult,
    trace: Vec<NestedBattleTraceEntry>,
) -> Result<NestedBattleExecutionReport, NestedBattleExecutionError> {
    let outcome = result
        .values()
        .iter()
        .find_map(|value| match value {
            ProjectedValue::Outcome(value) => Some(*value),
            _ => None,
        })
        .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
    let event_digest = result
        .values()
        .iter()
        .find_map(|value| match value {
            ProjectedValue::EventDigest(value) => Some(*value),
            _ => None,
        })
        .ok_or(NestedBattleExecutionError::UnsupportedProjection)?;
    Ok(NestedBattleExecutionReport {
        outcome,
        final_state_hash: battle.state_hash(),
        event_digest,
        terminal_fault: battle.view().fault(),
        trace: trace.into_boxed_slice(),
    })
}

fn enemy_controller_seed(handoff: &ActivityBattleHandoff) -> RngSeed {
    let mut encoder = Encoder::new(b"starclock-standard-universe-enemy-controller-v1");
    encoder.digest(handoff.identity().seed().bytes());
    encoder.digest(handoff.identity().assembly_digest().bytes());
    RngSeed::new(encoder.finish())
}

/// This commitment hashes the complete deterministic input trace, every
/// resulting canonical state hash, every emitted event identity/cause and its
/// typed family. The inputs deterministically imply the full event payload.
pub(crate) struct EventCommitment(Encoder);

impl EventCommitment {
    pub(crate) fn new(catalog: &CombatCatalog, handoff: &ActivityBattleHandoff) -> Self {
        let mut encoder = Encoder::new(b"starclock-universe-battle-event-commitment");
        encoder.digest(catalog.digest().bytes());
        encoder.digest(handoff.identity().seed().bytes());
        encoder.digest(handoff.identity().assembly_digest().bytes());
        Self(encoder)
    }

    pub(crate) fn push(&mut self, command: &Command, resolution: &starclock_combat::Resolution) {
        encode_command(&mut self.0, command);
        self.0.digest(resolution.state_hash().bytes());
        self.0.u64(resolution.root_command().get());
        self.0.u64(resolution.events().len() as u64);
        for event in resolution.events() {
            self.0.u64(event.id().get());
            let cause = event.cause();
            optional_u64(&mut self.0, cause.parent_event().map(|value| value.get()));
            self.0.u64(cause.root_command().get());
            optional_u64(&mut self.0, cause.action().map(|value| value.get()));
            optional_u64(&mut self.0, cause.phase().map(|value| value.get()));
            optional_u64(&mut self.0, cause.hit().map(|value| value.get()));
            optional_u64(&mut self.0, cause.owner().map(|value| value.get()));
            match cause.actor() {
                None => self.0.u8(0),
                Some(starclock_combat::CauseActor::Unit(value)) => {
                    self.0.u8(1);
                    self.0.u64(value.get());
                }
                Some(starclock_combat::CauseActor::TimelineActor(value)) => {
                    self.0.u8(2);
                    self.0.u64(value.get());
                }
            }
            optional_u64(&mut self.0, cause.applier().map(|value| value.get()));
            optional_u32(
                &mut self.0,
                cause.source_definition().map(|value| value.get()),
            );
            optional_u64(&mut self.0, cause.primary_target().map(|value| value.get()));
            // Preserve the released commitment byte shape. This option never
            // had a writer and is not part of event payload v2.
            optional_u32(&mut self.0, None);
            self.0.u8(event_family(event.kind()));
        }
    }

    pub(crate) fn finish(self) -> EventDigest {
        EventDigest::new(self.0.finish()).expect("SHA-256 output is non-zero")
    }
}

fn event_family(kind: &starclock_combat::BattleEventKind) -> u8 {
    match kind {
        starclock_combat::BattleEventKind::Battle(_) => 0,
        starclock_combat::BattleEventKind::Decision(_) => 1,
        starclock_combat::BattleEventKind::Turn(_) => 2,
        starclock_combat::BattleEventKind::Action(_) => 3,
        starclock_combat::BattleEventKind::Phase(_) => 4,
        starclock_combat::BattleEventKind::Hit(_) => 5,
        starclock_combat::BattleEventKind::Damage(_) => 6,
        starclock_combat::BattleEventKind::Heal(_) => 7,
        starclock_combat::BattleEventKind::HpConsumption(_) => 8,
        starclock_combat::BattleEventKind::Shield(_) => 9,
        starclock_combat::BattleEventKind::Toughness(_) => 10,
        starclock_combat::BattleEventKind::BreakDamage(_) => 11,
        starclock_combat::BattleEventKind::Unit(_) => 12,
        starclock_combat::BattleEventKind::Wave(_) => 13,
        starclock_combat::BattleEventKind::EnemyPhase(_) => 14,
        starclock_combat::BattleEventKind::Resource(_) => 15,
        starclock_combat::BattleEventKind::Effect(_) => 16,
        starclock_combat::BattleEventKind::RuleState(_) => 17,
        starclock_combat::BattleEventKind::RuleSignal(_) => 18,
        starclock_combat::BattleEventKind::Fault(_) => 19,
        _ => u8::MAX,
    }
}

fn encode_command(encoder: &mut Encoder, command: &Command) {
    match command {
        Command::StartBattle { decision } => {
            encoder.u8(0);
            encoder.u64(decision.get());
        }
        Command::UseAbility {
            decision,
            actor,
            ability,
            primary_target,
        } => {
            encoder.u8(1);
            action_command(encoder, *decision, *actor, *ability, *primary_target);
        }
        Command::UseInterrupt {
            decision,
            actor,
            ability,
            primary_target,
        } => {
            encoder.u8(2);
            action_command(encoder, *decision, *actor, *ability, *primary_target);
        }
        Command::PassInterruptWindow { decision } => {
            encoder.u8(3);
            encoder.u64(decision.get());
        }
        Command::Concede { decision } => {
            encoder.u8(4);
            encoder.u64(decision.get());
        }
    }
}

fn action_command(
    encoder: &mut Encoder,
    decision: starclock_combat::DecisionId,
    actor: starclock_combat::UnitId,
    ability: starclock_combat::AbilityId,
    primary_target: Option<starclock_combat::UnitId>,
) {
    encoder.u64(decision.get());
    encoder.u64(actor.get());
    encoder.u32(ability.get());
    optional_u64(encoder, primary_target.map(|value| value.get()));
}

fn optional_u64(encoder: &mut Encoder, value: Option<u64>) {
    encoder.bool(value.is_some());
    if let Some(value) = value {
        encoder.u64(value);
    }
}

fn optional_u32(encoder: &mut Encoder, value: Option<u32>) {
    encoder.bool(value.is_some());
    if let Some(value) = value {
        encoder.u32(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_reward_metric_is_strict_and_boundary_inclusive() {
        assert_eq!(
            parse_cycle_reward_metric("occurrence.blessing-reward.within-cycles.4.base.1.bonus.1"),
            Ok((4, 1, 1))
        );
        assert!(
            parse_cycle_reward_metric("occurrence.blessing-reward.within-cycles.4.base.3.bonus.2")
                .is_err()
        );
        assert_eq!(
            cycle_reward_count(BattleOutcome::Won, 450_000_000, 4, 1, 1),
            Ok(2)
        );
        assert_eq!(
            cycle_reward_count(BattleOutcome::Won, 450_000_001, 4, 1, 1),
            Ok(1)
        );
        assert_eq!(cycle_reward_count(BattleOutcome::Lost, 0, 4, 1, 1), Ok(1));
        assert!(cycle_reward_count(BattleOutcome::Won, -1, 4, 1, 1).is_err());
    }
}
