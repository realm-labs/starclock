//! Participant carry mutations lowered by generic Activity operations.

use starclock_combat::Ratio;

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
