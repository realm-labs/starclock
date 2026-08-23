mod replay;
mod trace;

pub use replay::{
    CurrencyWarsReplayDivergence, CurrencyWarsReplayDivergenceKind, CurrencyWarsReplayError,
    CurrencyWarsReplayGambit, CurrencyWarsReplayIdentity, CurrencyWarsReplayRequest,
    decode_currency_wars_replay_request, encode_currency_wars_replay, verify_currency_wars_replay,
};
pub use trace::{
    CurrencyWarsBaselineActivityAction, CurrencyWarsBaselineActivityTraceEntry,
    CurrencyWarsBaselineBattleReport, CurrencyWarsBaselineRunReport,
    CurrencyWarsBaselineTraceController, CurrencyWarsBaselineTraceEntry,
};

use std::{collections::BTreeMap, num::NonZeroU32, sync::Arc};

use sha2::{Digest, Sha256};
use starclock_activity::{
    ActivityBattleHandoff, ActivityDecisionKind, BattleOutcome, BattleResult, EventDigest,
    MetricValue, ParticipantBattleState, ProjectedValue, ProjectionField,
};
use starclock_combat::{
    ActionBoundaryView, ActionValue, ActiveTurnView, Battle, BattlePhase, BattleSpec,
    BattleStateHash, Command, DecisionKind, DecisionOwner, LifeState, ParticipantInitialState,
    ParticipantSpec, Ratio, TeamSide,
    catalog::{CombatCatalog, action::AbilityKind, encounter::AiTransitionTiming},
    rng::types::RngSeed,
    rule::model::ConditionExpr,
};
use starclock_mode_currency_wars::{
    CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY, CURRENCY_WARS_BATTLE_PROGRESS_KEY,
    CurrencyWarsBattleAssembler, CurrencyWarsRun, CurrencyWarsRuntimeError,
};
use starclock_replay::{
    battle::encode_battle_command_payload, battle_event::encode_battle_event_payload,
};

use crate::{
    EnemyController, EnemyDecisionError,
    baseline::{
        BaselineAbilityClass, BaselineAbilityHint, BaselineController, BaselineDecisionError,
        BaselineHints, BaselineScoreComponents, BaselineTargetHint,
    },
};

pub const CURRENCY_WARS_BASELINE_ACTIVITY_STEP_BUDGET: u32 = 1_024;
pub const CURRENCY_WARS_BASELINE_BATTLE_COMMAND_BUDGET: u32 = 10_000;
pub const CURRENCY_WARS_BASELINE_CONCEDE_COMMAND_LIMIT: u32 = 64;
const RATIO_SCALE: i128 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBaselineController {
    activity_step_budget: u32,
    battle_command_budget: u32,
    concede_command_limit: Option<NonZeroU32>,
}

impl Default for CurrencyWarsBaselineController {
    fn default() -> Self {
        Self {
            activity_step_budget: CURRENCY_WARS_BASELINE_ACTIVITY_STEP_BUDGET,
            battle_command_budget: CURRENCY_WARS_BASELINE_BATTLE_COMMAND_BUDGET,
            concede_command_limit: NonZeroU32::new(CURRENCY_WARS_BASELINE_CONCEDE_COMMAND_LIMIT),
        }
    }
}

impl CurrencyWarsBaselineController {
    pub const ID: &'static str = "currency-wars-baseline-controller-v1";

    #[must_use]
    pub fn identity_digest() -> [u8; 32] {
        Sha256::digest(Self::ID.as_bytes()).into()
    }

    #[must_use]
    pub const fn with_budgets(
        activity_step_budget: u32,
        battle_command_budget: u32,
    ) -> Option<Self> {
        Self::with_limits(
            activity_step_budget,
            battle_command_budget,
            NonZeroU32::new(CURRENCY_WARS_BASELINE_CONCEDE_COMMAND_LIMIT),
        )
    }

    /// Builds a controller with explicit execution limits.
    ///
    /// `None` disables deterministic concession. The battle command budget
    /// remains the hard bound for every battle.
    #[must_use]
    pub const fn with_limits(
        activity_step_budget: u32,
        battle_command_budget: u32,
        concede_command_limit: Option<NonZeroU32>,
    ) -> Option<Self> {
        if activity_step_budget == 0 || battle_command_budget == 0 {
            None
        } else {
            Some(Self {
                activity_step_budget,
                battle_command_budget,
                concede_command_limit,
            })
        }
    }

    pub fn run_to_terminal(
        self,
        run: &mut CurrencyWarsRun,
        assembler: &mut CurrencyWarsBattleAssembler,
    ) -> Result<CurrencyWarsBaselineRunReport, CurrencyWarsBaselineControllerError> {
        let mut battles = Vec::new();
        let mut activity_trace = Vec::new();
        let mut supply_decisions = 0_u32;
        let mut route_decisions = 0_u32;
        let mut activity_steps = 0_u32;
        loop {
            let view = run.player_view();
            if let Some(terminal) = view.terminal() {
                return Ok(CurrencyWarsBaselineRunReport {
                    terminal,
                    final_state_hash: view.state_hash(),
                    activity_steps,
                    supply_decisions,
                    route_decisions,
                    activity_trace: activity_trace.into_boxed_slice(),
                    battles: battles.into_boxed_slice(),
                });
            }
            let decision = view
                .decision()
                .ok_or(CurrencyWarsBaselineControllerError::MissingActivityDecision)?;
            match decision.kind() {
                ActivityDecisionKind::Encounter => {
                    if self.activity_step_budget.saturating_sub(activity_steps) < 2 {
                        return Err(CurrencyWarsBaselineControllerError::ActivityStepBudget);
                    }
                    let attempt = u32::try_from(battles.len())
                        .ok()
                        .and_then(|value| value.checked_add(1))
                        .and_then(starclock_activity::AttemptId::new)
                        .ok_or(CurrencyWarsBaselineControllerError::ActivityStepBudget)?;
                    let preparation = run
                        .engage_current_node(attempt, assembler)
                        .map_err(CurrencyWarsBaselineControllerError::Runtime)?;
                    let catalog = Arc::clone(preparation.materialization().combat_catalog());
                    activity_steps += 1;
                    activity_trace.push(CurrencyWarsBaselineActivityTraceEntry {
                        action: CurrencyWarsBaselineActivityAction::EngageEncounter,
                        state_hash: run.state_hash(),
                        battle_index: None,
                    });
                    run.choose_prepared_battle()
                        .map_err(CurrencyWarsBaselineControllerError::Runtime)?;
                    let handoff = run
                        .start_pending_battle()
                        .map_err(CurrencyWarsBaselineControllerError::Runtime)?;
                    let (result, report) = self.execute_battle(catalog, &handoff)?;
                    run.submit_battle_result(result)
                        .map_err(CurrencyWarsBaselineControllerError::Runtime)?;
                    battles.push(report);
                    activity_steps += 1;
                    let battle_index = u32::try_from(battles.len())
                        .map_err(|_| CurrencyWarsBaselineControllerError::ActivityStepBudget)?;
                    activity_trace.push(CurrencyWarsBaselineActivityTraceEntry {
                        action: CurrencyWarsBaselineActivityAction::PrepareBattle,
                        state_hash: run.state_hash(),
                        battle_index: Some(battle_index),
                    });
                }
                ActivityDecisionKind::Shop => {
                    if activity_steps == self.activity_step_budget {
                        return Err(CurrencyWarsBaselineControllerError::ActivityStepBudget);
                    }
                    run.continue_supply()
                        .map_err(CurrencyWarsBaselineControllerError::Runtime)?;
                    activity_steps += 1;
                    supply_decisions = supply_decisions
                        .checked_add(1)
                        .ok_or(CurrencyWarsBaselineControllerError::ActivityStepBudget)?;
                    activity_trace.push(CurrencyWarsBaselineActivityTraceEntry {
                        action: CurrencyWarsBaselineActivityAction::ContinueSupply,
                        state_hash: run.state_hash(),
                        battle_index: None,
                    });
                }
                ActivityDecisionKind::Route => {
                    if activity_steps == self.activity_step_budget {
                        return Err(CurrencyWarsBaselineControllerError::ActivityStepBudget);
                    }
                    run.continue_plane()
                        .map_err(CurrencyWarsBaselineControllerError::Runtime)?;
                    activity_steps += 1;
                    route_decisions = route_decisions
                        .checked_add(1)
                        .ok_or(CurrencyWarsBaselineControllerError::ActivityStepBudget)?;
                    activity_trace.push(CurrencyWarsBaselineActivityTraceEntry {
                        action: CurrencyWarsBaselineActivityAction::ContinuePlane,
                        state_hash: run.state_hash(),
                        battle_index: None,
                    });
                }
                other => {
                    return Err(
                        CurrencyWarsBaselineControllerError::UnsupportedActivityDecision(other),
                    );
                }
            }
        }
    }

    /// Resolves one already-started Currency Wars battle using only commands
    /// offered by the authoritative combat decision boundary.
    pub fn execute_battle(
        self,
        catalog: Arc<CombatCatalog>,
        handoff: &ActivityBattleHandoff,
    ) -> Result<(BattleResult, CurrencyWarsBaselineBattleReport), CurrencyWarsBaselineControllerError>
    {
        let spec = carried_spec(handoff)?;
        let mut battle = Battle::create(Arc::clone(&catalog), spec, handoff.identity().seed())
            .map_err(|_| CurrencyWarsBaselineControllerError::BattleBuild)?;
        let mut enemy = EnemyController::new(enemy_seed(handoff));
        let mut commitment = EventCommitment::new(&catalog, handoff);
        let mut trace = Vec::new();
        for command_index in 0..self.battle_command_budget {
            if battle.view().phase().is_terminal() {
                let event_digest = commitment.finish();
                let (result, report) = project_result(
                    &battle,
                    catalog.digest().bytes(),
                    handoff,
                    event_digest,
                    trace,
                )?;
                return Ok((result, report));
            }
            let (command, controller, enemy_action) =
                if battle.view().phase() == BattlePhase::ReadyToAdvance {
                    (
                        battle
                            .advance_command()
                            .ok_or(CurrencyWarsBaselineControllerError::MissingBattleDecision)?,
                        CurrencyWarsBaselineTraceController::System,
                        None,
                    )
                } else {
                    let decision = battle
                        .decision()
                        .cloned()
                        .ok_or(CurrencyWarsBaselineControllerError::MissingBattleDecision)?;
                    if self
                        .concede_command_limit
                        .is_some_and(|limit| command_index >= limit.get())
                        && decision.owner() == DecisionOwner::Team(TeamSide::Player)
                        && let Some(command) = decision
                            .legal_commands()
                            .iter()
                            .find(|command| matches!(command, Command::Concede { .. }))
                            .cloned()
                    {
                        (command, CurrencyWarsBaselineTraceController::Player, None)
                    } else {
                        select_battle_command(&battle, &catalog, &mut enemy, &decision)?
                    }
                };
            let resolution = battle
                .apply(command.clone())
                .map_err(|_| CurrencyWarsBaselineControllerError::BattleCommandRejected)?;
            commitment.push(&command, &resolution)?;
            trace.push(CurrencyWarsBaselineTraceEntry {
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
                    .map_err(CurrencyWarsBaselineControllerError::EnemyDecision)?;
            }
        }
        let view = battle.view();
        let decision = battle.decision();
        let living_player_units = view
            .units_by_id()
            .filter(|unit| {
                unit.side() == TeamSide::Player
                    && unit.life() == LifeState::Alive
                    && unit.presence().is_active()
            })
            .count();
        let living_enemy_units = view
            .units_by_id()
            .filter(|unit| {
                unit.side() == TeamSide::Enemy
                    && unit.life() == LifeState::Alive
                    && unit.presence().is_active()
            })
            .count();
        let targetable_enemy_units = view
            .units_by_id()
            .filter(|unit| {
                unit.side() == TeamSide::Enemy
                    && unit.life() == LifeState::Alive
                    && unit.presence().is_targetable()
            })
            .count();
        let player_hp = view
            .units_by_id()
            .filter(|unit| unit.side() == TeamSide::Player && unit.presence().is_active())
            .fold((0_i128, 0_i128), |(current, maximum), unit| {
                (
                    current + i128::from(unit.current_hp().get()),
                    maximum + i128::from(unit.maximum_hp().get()),
                )
            });
        let enemy_hp = view
            .units_by_id()
            .filter(|unit| unit.side() == TeamSide::Enemy && unit.presence().is_active())
            .fold((0_i128, 0_i128), |(current, maximum), unit| {
                (
                    current + i128::from(unit.current_hp().get()),
                    maximum + i128::from(unit.maximum_hp().get()),
                )
            });
        let active_player_actors = view
            .timeline_actors()
            .filter(|actor| {
                actor.is_active()
                    && view
                        .units_by_id()
                        .find(|unit| unit.id() == actor.owner())
                        .is_some_and(|unit| unit.side() == TeamSide::Player)
            })
            .count();
        let active_enemy_actors = view
            .timeline_actors()
            .filter(|actor| {
                actor.is_active()
                    && view
                        .units_by_id()
                        .find(|unit| unit.id() == actor.owner())
                        .is_some_and(|unit| unit.side() == TeamSide::Enemy)
            })
            .count();
        Err(CurrencyWarsBaselineControllerError::BattleCommandBudget(
            Box::new(CurrencyWarsBattleCommandBudgetDiagnostic {
                commands: self.battle_command_budget,
                phase: view.phase(),
                state_hash: battle.state_hash(),
                remaining_action_value_scaled: view
                    .clock()
                    .and_then(|clock| clock.remaining_action_value_scaled()),
                decision_kind: decision.map(starclock_combat::DecisionPoint::kind),
                decision_owner: decision.map(starclock_combat::DecisionPoint::owner),
                active_turn: view.active_turn(),
                action_boundary: view.action_boundary(),
                living_player_units,
                active_player_actors,
                living_enemy_units,
                targetable_enemy_units,
                active_enemy_actors,
                player_hp,
                enemy_hp,
                last_player_commands: trace
                    .iter()
                    .rev()
                    .filter(|entry| {
                        entry.controller() == CurrencyWarsBaselineTraceController::Player
                    })
                    .take(8)
                    .map(|entry| entry.command().clone())
                    .collect(),
                last_commands: trace
                    .iter()
                    .rev()
                    .take(8)
                    .map(|entry| entry.command().clone())
                    .collect(),
            }),
        ))
    }
}

type EnemyAction<'a> = (
    starclock_combat::UnitId,
    &'a starclock_combat::catalog::encounter::AiGraphDefinition,
);

fn select_battle_command<'a>(
    battle: &Battle,
    catalog: &'a CombatCatalog,
    enemy: &mut EnemyController,
    decision: &starclock_combat::DecisionPoint,
) -> Result<
    (
        Command,
        CurrencyWarsBaselineTraceController,
        Option<EnemyAction<'a>>,
    ),
    CurrencyWarsBaselineControllerError,
> {
    if decision.kind() == DecisionKind::BattleChoice && decision.legal_commands().len() == 1 {
        return Ok((
            decision.legal_commands()[0].clone(),
            trace_controller(decision.owner()),
            None,
        ));
    }
    match decision.owner() {
        DecisionOwner::System => {
            let command = decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::StartBattle { .. }))
                .cloned()
                .ok_or(CurrencyWarsBaselineControllerError::UnsupportedBattleDecision)?;
            Ok((command, CurrencyWarsBaselineTraceController::System, None))
        }
        DecisionOwner::Team(TeamSide::Player) => {
            let hints = baseline_hints(catalog, battle, decision)?;
            let selected = BaselineController
                .decide(battle.view(), decision, &hints)
                .map_err(CurrencyWarsBaselineControllerError::PlayerDecision)?;
            Ok((
                selected.command().clone(),
                CurrencyWarsBaselineTraceController::Player,
                None,
            ))
        }
        DecisionOwner::Team(TeamSide::Enemy) => {
            let actor = decision
                .legal_commands()
                .iter()
                .find_map(command_actor)
                .ok_or(CurrencyWarsBaselineControllerError::MissingEnemyController)?;
            let (graph_id, initial_state, _) = battle
                .view()
                .units_by_id()
                .find(|unit| unit.id() == actor)
                .and_then(|unit| unit.enemy_ai_state())
                .ok_or(CurrencyWarsBaselineControllerError::MissingEnemyController)?;
            let graph = catalog
                .ai_graph(graph_id)
                .ok_or(CurrencyWarsBaselineControllerError::MissingEnemyController)?;
            let (command, settle) =
                match enemy.decide(graph, initial_state, actor, decision, static_condition) {
                    Ok(selected) => (selected.command().clone(), true),
                    Err(EnemyDecisionError::NoLegalFallback) => (
                        decision.legal_commands().first().cloned().ok_or(
                            CurrencyWarsBaselineControllerError::UnsupportedBattleDecision,
                        )?,
                        false,
                    ),
                    Err(error) => {
                        return Err(CurrencyWarsBaselineControllerError::EnemyDecision(error));
                    }
                };
            Ok((
                command,
                CurrencyWarsBaselineTraceController::Enemy,
                settle.then_some((actor, graph)),
            ))
        }
    }
}

fn baseline_hints(
    catalog: &CombatCatalog,
    battle: &Battle,
    decision: &starclock_combat::DecisionPoint,
) -> Result<BaselineHints, CurrencyWarsBaselineControllerError> {
    let mut abilities = BTreeMap::new();
    for command in decision.legal_commands() {
        let (ability, class) = match command {
            Command::UseAbility { ability, .. } => {
                let action = catalog
                    .ability(*ability)
                    .and_then(|definition| definition.action())
                    .ok_or(CurrencyWarsBaselineControllerError::InvalidBattleHints)?;
                let class = match action.kind() {
                    AbilityKind::Basic => BaselineAbilityClass::Basic,
                    _ => BaselineAbilityClass::Skill,
                };
                (*ability, class)
            }
            Command::RequestUltimate { ability, .. } => (*ability, BaselineAbilityClass::Interrupt),
            Command::StartBattle { .. }
            | Command::CommitPreparedAction { .. }
            | Command::CancelPreparedAction { .. }
            | Command::CommitActionFrame { .. }
            | Command::Advance { .. }
            | Command::Concede { .. } => continue,
        };
        if abilities
            .insert(ability, class)
            .is_some_and(|previous| previous != class)
        {
            return Err(CurrencyWarsBaselineControllerError::InvalidBattleHints);
        }
    }
    let abilities = abilities
        .into_iter()
        .map(|(ability, class)| {
            let components = BaselineScoreComponents::new(0, 0, 0, 0, 0, false)
                .expect("Currency Wars baseline priorities are bounded");
            BaselineAbilityHint::new(ability, class, components)
        })
        .collect();
    let targets = battle
        .view()
        .units_by_id()
        .map(|unit| BaselineTargetHint::new(unit.id(), 0))
        .collect::<Option<Vec<_>>>()
        .ok_or(CurrencyWarsBaselineControllerError::InvalidBattleHints)?;
    BaselineHints::new(abilities, targets)
        .map_err(|_| CurrencyWarsBaselineControllerError::InvalidBattleHints)
}

fn trace_controller(owner: DecisionOwner) -> CurrencyWarsBaselineTraceController {
    match owner {
        DecisionOwner::System => CurrencyWarsBaselineTraceController::System,
        DecisionOwner::Team(TeamSide::Player) => CurrencyWarsBaselineTraceController::Player,
        DecisionOwner::Team(TeamSide::Enemy) => CurrencyWarsBaselineTraceController::Enemy,
    }
}

fn command_actor(command: &Command) -> Option<starclock_combat::UnitId> {
    match command {
        Command::UseAbility { actor, .. } | Command::RequestUltimate { actor, .. } => Some(*actor),
        Command::StartBattle { .. }
        | Command::CommitPreparedAction { .. }
        | Command::CancelPreparedAction { .. }
        | Command::CommitActionFrame { .. }
        | Command::Advance { .. }
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
        | ConditionExpr::HighestDamageDealer(_)
        | ConditionExpr::EnemyRank { .. }
        | ConditionExpr::EnemyRankEliteOrBoss { .. } => false,
    }
}

fn carried_spec(
    handoff: &ActivityBattleHandoff,
) -> Result<BattleSpec, CurrencyWarsBaselineControllerError> {
    if handoff.participants().len() != handoff.participant_carry().len() {
        return Err(CurrencyWarsBaselineControllerError::ParticipantMapping);
    }
    let authored = handoff.battle_spec();
    let participants = authored
        .participants()
        .iter()
        .cloned()
        .map(|participant| {
            if participant.side() == TeamSide::Player {
                carried_participant(participant, handoff)
            } else {
                Ok(participant)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut spec = BattleSpec::new(
        authored.assembly_digest(),
        authored.encounter(),
        participants,
        authored.resources(TeamSide::Player).clone(),
        authored.resources(TeamSide::Enemy).clone(),
        authored.concede_policy(),
    )
    .map_err(|_| CurrencyWarsBaselineControllerError::BattleBuild)?;
    if let Some(clock) = authored.clock() {
        spec = spec.with_clock(clock);
    }
    if let Some(energy) = authored.enemy_defeat_energy() {
        spec = spec
            .with_enemy_defeat_energy(energy)
            .ok_or(CurrencyWarsBaselineControllerError::BattleBuild)?;
    }
    if let Some(rescue) = authored.player_lethal_rescue() {
        spec = spec
            .with_player_lethal_rescue(rescue)
            .ok_or(CurrencyWarsBaselineControllerError::BattleBuild)?;
    }
    Ok(spec)
}

fn carried_participant(
    participant: ParticipantSpec,
    handoff: &ActivityBattleHandoff,
) -> Result<ParticipantSpec, CurrencyWarsBaselineControllerError> {
    let binding = handoff
        .participants()
        .iter()
        .find(|binding| binding.formation() == participant.formation())
        .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)?;
    let carry = handoff
        .participant_carry()
        .iter()
        .find(|carry| carry.participant() == binding.participant())
        .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)?;
    let initial = ParticipantInitialState::new(
        carry.current_hp(),
        participant.combatant().maximum_hp(),
        carry.current_energy(),
        participant.combatant().maximum_energy(),
        carry.life(),
        carry.presence(),
    )
    .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)?;
    participant
        .with_initial_state(initial)
        .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)
}

fn project_result(
    battle: &Battle,
    catalog_digest: [u8; 32],
    handoff: &ActivityBattleHandoff,
    event_digest: EventDigest,
    trace: Vec<CurrencyWarsBaselineTraceEntry>,
) -> Result<(BattleResult, CurrencyWarsBaselineBattleReport), CurrencyWarsBaselineControllerError> {
    let view = battle.view();
    let outcome = match view.phase() {
        BattlePhase::Won => BattleOutcome::Won,
        BattlePhase::Lost => BattleOutcome::Lost,
        BattlePhase::Faulted => BattleOutcome::Faulted,
        BattlePhase::Finalized => BattleOutcome::Finalized,
        BattlePhase::Initializing
        | BattlePhase::ReadyToAdvance
        | BattlePhase::AwaitingCommand
        | BattlePhase::Resolving => {
            return Err(CurrencyWarsBaselineControllerError::MissingBattleDecision);
        }
    };
    let progress = battle_progress(battle, outcome)?;
    let remaining_action_value = projected_remaining_action_value(
        outcome,
        view.clock()
            .and_then(|clock| clock.remaining_action_value_scaled()),
    )?;
    let mut values = Vec::with_capacity(handoff.projection().fields().len());
    for field in handoff.projection().fields() {
        values.push(match field {
            ProjectionField::Outcome => ProjectedValue::Outcome(outcome),
            ProjectionField::FinalStateHash => ProjectedValue::FinalStateHash(battle.state_hash()),
            ProjectionField::EventDigest => ProjectedValue::EventDigest(event_digest),
            ProjectionField::TerminalFault => ProjectedValue::TerminalFault(view.fault()),
            ProjectionField::ParticipantState(participant) => {
                ProjectedValue::ParticipantState(participant_state(battle, handoff, *participant)?)
            }
            ProjectionField::Metric { key, kind }
                if key.as_ref() == CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY
                    && *kind == starclock_activity::MetricValueKind::ActionValue =>
            {
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::ActionValue(remaining_action_value.scaled()),
                }
            }
            ProjectionField::Metric { key, kind }
                if key.as_ref() == CURRENCY_WARS_BATTLE_PROGRESS_KEY
                    && *kind == starclock_activity::MetricValueKind::Ratio =>
            {
                ProjectedValue::Metric {
                    key: key.clone(),
                    value: MetricValue::Ratio(progress.scaled()),
                }
            }
            ProjectionField::Metric { .. } => {
                return Err(CurrencyWarsBaselineControllerError::Projection);
            }
        });
    }
    let result = BattleResult::seal(handoff.identity(), values);
    let report = CurrencyWarsBaselineBattleReport {
        catalog_digest,
        combat_input_digest: handoff.identity().combat_input_digest().bytes(),
        assembly_digest: handoff.identity().assembly_digest().bytes(),
        outcome,
        final_state_hash: battle.state_hash(),
        event_digest,
        progress,
        remaining_action_value,
        trace: trace.into_boxed_slice(),
    };
    Ok((result, report))
}

fn projected_remaining_action_value(
    outcome: BattleOutcome,
    remaining_scaled: Option<i64>,
) -> Result<ActionValue, CurrencyWarsBaselineControllerError> {
    if outcome == BattleOutcome::Lost {
        return Ok(ActionValue::ZERO);
    }
    remaining_scaled
        .map(ActionValue::from_scaled)
        .transpose()
        .map_err(|_| CurrencyWarsBaselineControllerError::Projection)
        .map(|remaining| remaining.unwrap_or(ActionValue::ZERO))
}

fn participant_state(
    battle: &Battle,
    handoff: &ActivityBattleHandoff,
    participant: starclock_activity::ParticipantId,
) -> Result<ParticipantBattleState, CurrencyWarsBaselineControllerError> {
    let formation = handoff
        .participants()
        .iter()
        .find(|binding| binding.participant() == participant)
        .map(|binding| binding.formation())
        .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)?;
    let view = battle.view();
    let unit_id = view
        .formation(TeamSide::Player)
        .find(|entry| entry.index() == formation)
        .map(|entry| entry.unit())
        .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)?;
    let unit = view
        .units_by_id()
        .find(|unit| unit.id() == unit_id)
        .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)?;
    let authored = handoff
        .battle_spec()
        .participants()
        .iter()
        .find(|participant| {
            participant.side() == TeamSide::Player && participant.formation() == formation
        })
        .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)?;
    let maximum_hp = authored.combatant().maximum_hp();
    let maximum_energy = authored.combatant().maximum_energy();
    ParticipantBattleState::new(
        participant,
        starclock_combat::Hp::new(unit.current_hp().get().min(maximum_hp.get()))
            .expect("clamped HP remains non-negative"),
        maximum_hp,
        starclock_combat::Energy::from_scaled(
            unit.current_energy().scaled().min(maximum_energy.scaled()),
        )
        .map_err(|_| CurrencyWarsBaselineControllerError::ParticipantMapping)?,
        maximum_energy,
        unit.life(),
        unit.presence(),
    )
    .ok_or(CurrencyWarsBaselineControllerError::ParticipantMapping)
}

fn battle_progress(
    battle: &Battle,
    outcome: BattleOutcome,
) -> Result<Ratio, CurrencyWarsBaselineControllerError> {
    if outcome == BattleOutcome::Won {
        return Ok(Ratio::ONE);
    }
    let mut maximum = 0_i128;
    let mut remaining = 0_i128;
    for unit in battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Enemy)
    {
        maximum = maximum
            .checked_add(i128::from(unit.maximum_hp().get()))
            .ok_or(CurrencyWarsBaselineControllerError::Projection)?;
        let current = if unit.life() == LifeState::Defeated {
            0
        } else {
            unit.current_hp().get()
        };
        remaining = remaining
            .checked_add(i128::from(current))
            .ok_or(CurrencyWarsBaselineControllerError::Projection)?;
    }
    if maximum <= 0 || remaining < 0 || remaining > maximum {
        return Err(CurrencyWarsBaselineControllerError::Projection);
    }
    let scaled = maximum
        .checked_sub(remaining)
        .and_then(|depleted| depleted.checked_mul(RATIO_SCALE))
        .and_then(|value| value.checked_div(maximum))
        .and_then(|value| i64::try_from(value).ok())
        .ok_or(CurrencyWarsBaselineControllerError::Projection)?;
    Ok(Ratio::from_scaled(scaled))
}

fn enemy_seed(handoff: &ActivityBattleHandoff) -> RngSeed {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.baseline-enemy-controller.v1");
    hash.update(handoff.identity().seed().bytes());
    hash.update(handoff.identity().assembly_digest().bytes());
    RngSeed::new(hash.finalize().into())
}

struct EventCommitment(Sha256);

impl EventCommitment {
    fn new(catalog: &CombatCatalog, handoff: &ActivityBattleHandoff) -> Self {
        let mut hash = Sha256::new();
        hash.update(b"starclock.currency-wars.baseline-event-commitment.v1");
        hash.update(catalog.digest().bytes());
        hash.update(handoff.identity().seed().bytes());
        hash.update(handoff.identity().assembly_digest().bytes());
        Self(hash)
    }

    fn push(
        &mut self,
        command: &Command,
        resolution: &starclock_combat::Resolution,
    ) -> Result<(), CurrencyWarsBaselineControllerError> {
        let command = encode_battle_command_payload(command)
            .map_err(|_| CurrencyWarsBaselineControllerError::Projection)?;
        hash_bytes(&mut self.0, &command)?;
        self.0.update(resolution.state_hash().bytes());
        self.0.update(resolution.root_command().get().to_le_bytes());
        self.0.update(
            u32::try_from(resolution.events().len())
                .map_err(|_| CurrencyWarsBaselineControllerError::Projection)?
                .to_le_bytes(),
        );
        for event in resolution.events() {
            let payload = encode_battle_event_payload(event)
                .map_err(|_| CurrencyWarsBaselineControllerError::Projection)?;
            hash_bytes(&mut self.0, &payload)?;
        }
        Ok(())
    }

    fn finish(self) -> EventDigest {
        EventDigest::new(self.0.finalize().into()).expect("SHA-256 output is non-zero")
    }
}

fn hash_bytes(hash: &mut Sha256, value: &[u8]) -> Result<(), CurrencyWarsBaselineControllerError> {
    hash.update(
        u32::try_from(value.len())
            .map_err(|_| CurrencyWarsBaselineControllerError::Projection)?
            .to_le_bytes(),
    );
    hash.update(value);
    Ok(())
}

#[derive(Debug)]
pub struct CurrencyWarsBattleCommandBudgetDiagnostic {
    pub commands: u32,
    pub phase: BattlePhase,
    pub state_hash: BattleStateHash,
    pub remaining_action_value_scaled: Option<i64>,
    pub decision_kind: Option<DecisionKind>,
    pub decision_owner: Option<DecisionOwner>,
    pub active_turn: Option<ActiveTurnView>,
    pub action_boundary: Option<ActionBoundaryView>,
    pub living_player_units: usize,
    pub active_player_actors: usize,
    pub living_enemy_units: usize,
    pub targetable_enemy_units: usize,
    pub active_enemy_actors: usize,
    pub player_hp: (i128, i128),
    pub enemy_hp: (i128, i128),
    pub last_player_commands: Box<[Command]>,
    pub last_commands: Box<[Command]>,
}

#[derive(Debug)]
pub enum CurrencyWarsBaselineControllerError {
    ActivityStepBudget,
    BattleCommandBudget(Box<CurrencyWarsBattleCommandBudgetDiagnostic>),
    MissingActivityDecision,
    UnsupportedActivityDecision(ActivityDecisionKind),
    Runtime(CurrencyWarsRuntimeError),
    BattleBuild,
    MissingBattleDecision,
    UnsupportedBattleDecision,
    BattleCommandRejected,
    MissingEnemyController,
    InvalidBattleHints,
    ParticipantMapping,
    Projection,
    BattleFaulted(Option<starclock_combat::BattleFault>),
    PlayerDecision(BaselineDecisionError),
    EnemyDecision(EnemyDecisionError),
}

impl std::fmt::Display for CurrencyWarsBaselineControllerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CurrencyWarsBaselineControllerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_identity_and_budgets_are_stable_and_bounded() {
        assert_ne!(CurrencyWarsBaselineController::identity_digest(), [0; 32]);
        assert!(CurrencyWarsBaselineController::with_budgets(1, 1).is_some());
        assert!(CurrencyWarsBaselineController::with_budgets(0, 1).is_none());
        assert!(CurrencyWarsBaselineController::with_budgets(1, 0).is_none());
        assert_eq!(
            CurrencyWarsBaselineController::with_limits(1, 1, None)
                .expect("non-zero execution budgets are valid")
                .concede_command_limit,
            None,
        );
        assert_eq!(
            CurrencyWarsBaselineController::with_limits(1, 1, NonZeroU32::new(7),)
                .expect("non-zero execution budgets are valid")
                .concede_command_limit,
            NonZeroU32::new(7),
        );
    }

    #[test]
    fn lost_battles_project_an_exhausted_action_value_boundary() {
        assert_eq!(
            projected_remaining_action_value(BattleOutcome::Lost, Some(12_000_000)).unwrap(),
            ActionValue::ZERO,
        );
        assert_eq!(
            projected_remaining_action_value(BattleOutcome::Won, Some(12_000_000)).unwrap(),
            ActionValue::from_scaled(12_000_000).unwrap(),
        );
    }
}
