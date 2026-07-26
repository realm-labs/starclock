//! Participant carry mutations lowered by generic Activity operations.

use starclock_combat::{Energy, Ratio};

use crate::{
    ActivityFault, ParticipantId,
    battle_settlement::{ActivityCarryLedger, ActivityCarryMutationError},
};

pub(super) fn restore(
    carry: &mut ActivityCarryLedger,
    participant: ParticipantId,
    hp_ratio: Ratio,
) -> Result<(), ActivityFault> {
    carry
        .restore_participant(participant, hp_ratio)
        .map_err(|error| match error {
            ActivityCarryMutationError::MissingParticipant => {
                ActivityFault::MissingParticipant(participant)
            }
            ActivityCarryMutationError::ParticipantNotDefeated => {
                ActivityFault::InvalidParticipantState(participant)
            }
            ActivityCarryMutationError::InvalidRestoreRatio => ActivityFault::TypeMismatch,
            ActivityCarryMutationError::ParticipantNotAlive
            | ActivityCarryMutationError::InvalidMinimumHp => ActivityFault::TypeMismatch,
            ActivityCarryMutationError::ArithmeticOverflow => ActivityFault::ArithmeticOverflow,
        })
}

pub(super) fn heal_maximum_hp_ratio(
    carry: &mut ActivityCarryLedger,
    participant: ParticipantId,
    hp_ratio: Ratio,
) -> Result<(), ActivityFault> {
    carry
        .heal_participant_maximum_hp_ratio(participant, hp_ratio)
        .map_err(|error| mutation_fault(error, participant))
}

pub(super) fn lose_current_hp_ratio(
    carry: &mut ActivityCarryLedger,
    participant: ParticipantId,
    hp_ratio: Ratio,
    minimum_hp: starclock_combat::Hp,
) -> Result<(), ActivityFault> {
    carry
        .lose_participant_current_hp_ratio(participant, hp_ratio, minimum_hp)
        .map_err(|error| mutation_fault(error, participant))
}

pub(super) fn set_energy(
    carry: &mut ActivityCarryLedger,
    participant: ParticipantId,
    energy: Energy,
) -> Result<(), ActivityFault> {
    carry
        .set_participant_energy(participant, energy)
        .map_err(|error| mutation_fault(error, participant))
}

fn mutation_fault(error: ActivityCarryMutationError, participant: ParticipantId) -> ActivityFault {
    match error {
        ActivityCarryMutationError::MissingParticipant => {
            ActivityFault::MissingParticipant(participant)
        }
        ActivityCarryMutationError::ParticipantNotDefeated
        | ActivityCarryMutationError::ParticipantNotAlive => {
            ActivityFault::InvalidParticipantState(participant)
        }
        ActivityCarryMutationError::InvalidRestoreRatio
        | ActivityCarryMutationError::InvalidMinimumHp => ActivityFault::TypeMismatch,
        ActivityCarryMutationError::ArithmeticOverflow => ActivityFault::ArithmeticOverflow,
    }
}
