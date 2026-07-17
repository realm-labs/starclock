use crate::battle::{model::BattlePhase, state::BattleState};

use super::model::{Command, CommandError, CommandErrorKind};

pub(crate) enum ValidatedCommand {
    StartBattle,
    PassInterruptWindow,
    UseAbility {
        actor: crate::UnitId,
        ability: crate::AbilityId,
    },
    Concede,
}

pub(crate) fn validate(
    state: &BattleState,
    command: &Command,
) -> Result<ValidatedCommand, CommandError> {
    if state.phase.is_terminal() {
        return Err(CommandError::new(CommandErrorKind::TerminalBattle));
    }
    if state.phase == BattlePhase::Resolving {
        return Err(CommandError::new(CommandErrorKind::ResolutionInProgress));
    }
    let decision = state
        .decision
        .as_ref()
        .ok_or_else(|| CommandError::new(CommandErrorKind::WrongPhase))?;
    if command.decision() != decision.id() {
        return Err(CommandError::new(CommandErrorKind::StaleDecision));
    }
    if !decision.contains(command) {
        return Err(CommandError::new(CommandErrorKind::NotOffered));
    }
    match (state.phase, command) {
        (BattlePhase::Initializing, Command::StartBattle { .. }) => {
            Ok(ValidatedCommand::StartBattle)
        }
        (BattlePhase::AwaitingCommand, Command::PassInterruptWindow { .. }) => {
            Ok(ValidatedCommand::PassInterruptWindow)
        }
        (
            BattlePhase::AwaitingCommand,
            Command::UseAbility {
                actor,
                ability,
                primary_target: None,
                ..
            },
        ) => Ok(ValidatedCommand::UseAbility {
            actor: *actor,
            ability: *ability,
        }),
        (BattlePhase::AwaitingCommand, Command::Concede { .. }) => Ok(ValidatedCommand::Concede),
        _ => Err(CommandError::new(CommandErrorKind::WrongPhase)),
    }
}
