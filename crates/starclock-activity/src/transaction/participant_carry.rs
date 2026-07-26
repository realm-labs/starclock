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
            ActivityCarryMutationError::ArithmeticOverflow => ActivityFault::ArithmeticOverflow,
        })
}

pub(super) fn set_energy(
    carry: &mut ActivityCarryLedger,
    participant: ParticipantId,
    energy: Energy,
) -> Result<(), ActivityFault> {
    carry
        .set_participant_energy(participant, energy)
        .map_err(|error| match error {
            ActivityCarryMutationError::MissingParticipant => {
                ActivityFault::MissingParticipant(participant)
            }
            ActivityCarryMutationError::ArithmeticOverflow => ActivityFault::ArithmeticOverflow,
            ActivityCarryMutationError::ParticipantNotDefeated
            | ActivityCarryMutationError::InvalidRestoreRatio => ActivityFault::TypeMismatch,
        })
}
