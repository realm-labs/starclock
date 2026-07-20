use starclock_combat::{
    AbilityId, AiCandidateId, AiStateId, Command, UnitId,
    catalog::encounter::{AiCandidateDefinition, AiCandidateSelection},
    rng::{engine::DeterministicRng, types::DrawSample},
};

use crate::{EnemyDecision, EnemyDecisionError};

pub(super) fn candidate<'a>(
    rng: &mut DeterministicRng,
    tied: &[&'a AiCandidateDefinition],
) -> Result<(&'a AiCandidateDefinition, Option<DrawSample>), EnemyDecisionError> {
    Ok(match tied[0].selection() {
        AiCandidateSelection::FirstLegal => (tied[0], None),
        AiCandidateSelection::WeightedDraw { purpose, .. } => {
            let weights = tied
                .iter()
                .map(|candidate| match candidate.selection() {
                    AiCandidateSelection::WeightedDraw {
                        weight,
                        purpose: candidate_purpose,
                    } if candidate_purpose == purpose => Ok(u64::from(weight)),
                    _ => Err(EnemyDecisionError::InvalidWeightedGroup),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let selected = rng
                .choose_weighted(purpose, &weights)
                .map_err(|_| EnemyDecisionError::Random)?
                .ok_or(EnemyDecisionError::InvalidWeightedGroup)?;
            let index = usize::try_from(selected.index())
                .map_err(|_| EnemyDecisionError::InvalidWeightedGroup)?;
            (tied[index], Some(selected.range().sample()))
        }
    })
}

pub(super) fn fallback(
    commands: &[Command],
    actor: UnitId,
    state: AiStateId,
    ability: AbilityId,
    candidate: Option<AiCandidateId>,
) -> Result<EnemyDecision, EnemyDecisionError> {
    let command = offered(commands, actor, ability).ok_or(EnemyDecisionError::NoLegalFallback)?;
    Ok(EnemyDecision {
        state,
        candidate,
        command,
        draw: None,
    })
}

pub(super) fn offered(commands: &[Command], actor: UnitId, ability: AbilityId) -> Option<Command> {
    commands
        .iter()
        .find(|command| {
            matches!(command,
                Command::UseAbility { actor: offered_actor, ability: offered_ability, .. }
                    if *offered_actor == actor && *offered_ability == ability
            )
        })
        .cloned()
}
