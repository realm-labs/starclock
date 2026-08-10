use crate::{
    AbilityId, TeamSide, UnitId,
    battle::{model::BattlePhase, state::BattleState},
    catalog::CombatCatalog,
};

use super::{
    legal,
    model::{ActionFrameInput, Command, CommandError, CommandErrorKind},
};

pub(crate) enum ValidatedCommand {
    StartBattle,
    Advance,
    UseAbility {
        actor: UnitId,
        ability: AbilityId,
        primary_target: Option<UnitId>,
    },
    RequestUltimate {
        actor: UnitId,
        ability: AbilityId,
    },
    CommitPreparedAction {
        primary_target: Option<UnitId>,
    },
    CancelPreparedAction,
    CommitActionFrame {
        input: ActionFrameInput,
    },
    Concede,
}

pub(crate) fn validate(
    state: &BattleState,
    catalog: &CombatCatalog,
    command: &Command,
) -> Result<ValidatedCommand, CommandError> {
    if state.phase.is_terminal() {
        return Err(CommandError::new(CommandErrorKind::TerminalBattle));
    }
    if state.phase == BattlePhase::Resolving {
        return Err(CommandError::new(CommandErrorKind::ResolutionInProgress));
    }
    if let Some(command_decision) = command.decision() {
        let decision = state
            .decision
            .as_ref()
            .ok_or_else(|| CommandError::new(CommandErrorKind::WrongPhase))?;
        if command_decision != decision.id() {
            return Err(CommandError::new(CommandErrorKind::StaleDecision));
        }
        if !decision.contains(command) {
            return Err(CommandError::new(CommandErrorKind::NotOffered));
        }
    } else {
        let boundary = state
            .timeline
            .boundary
            .as_ref()
            .ok_or_else(|| CommandError::new(CommandErrorKind::WrongPhase))?;
        if command.boundary() != Some(boundary.id) {
            return Err(CommandError::new(CommandErrorKind::StaleActionBoundary));
        }
    }
    match (state.phase, command) {
        (BattlePhase::Initializing, Command::StartBattle { .. }) => {
            Ok(ValidatedCommand::StartBattle)
        }
        (BattlePhase::ReadyToAdvance | BattlePhase::AwaitingCommand, Command::Advance { .. }) => {
            Ok(ValidatedCommand::Advance)
        }
        (
            BattlePhase::AwaitingCommand,
            Command::UseAbility {
                actor,
                ability,
                primary_target,
                ..
            },
        ) => Ok(ValidatedCommand::UseAbility {
            actor: *actor,
            ability: *ability,
            primary_target: *primary_target,
        }),
        (BattlePhase::AwaitingCommand, Command::RequestUltimate { actor, ability, .. })
        | (BattlePhase::ReadyToAdvance, Command::RequestUltimate { actor, ability, .. }) => {
            let offered = legal::ultimate_options(
                TeamSide::Player,
                &state.units,
                &state.formations,
                &state.teams,
                &state.effects,
                catalog,
            )
            .into_iter()
            .any(|option| option.actor() == *actor && option.ability() == *ability);
            if !offered {
                return Err(CommandError::new(CommandErrorKind::NotOffered));
            }
            Ok(ValidatedCommand::RequestUltimate {
                actor: *actor,
                ability: *ability,
            })
        }
        (BattlePhase::AwaitingCommand, Command::CommitPreparedAction { primary_target, .. })
            if state.timeline.prepared_action.is_some() =>
        {
            Ok(ValidatedCommand::CommitPreparedAction {
                primary_target: *primary_target,
            })
        }
        (BattlePhase::AwaitingCommand, Command::CancelPreparedAction { .. })
            if state.timeline.prepared_action.is_some() =>
        {
            Ok(ValidatedCommand::CancelPreparedAction)
        }
        (BattlePhase::AwaitingCommand, Command::CommitActionFrame { input, .. })
            if state.timeline.action_frame.is_some() =>
        {
            Ok(ValidatedCommand::CommitActionFrame { input: *input })
        }
        (BattlePhase::AwaitingCommand, Command::Concede { .. }) => Ok(ValidatedCommand::Concede),
        _ => Err(CommandError::new(CommandErrorKind::WrongPhase)),
    }
}
