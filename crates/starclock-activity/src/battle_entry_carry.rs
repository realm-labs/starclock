//! Rebind verified cross-battle resources to the accepted destination capacity.

use starclock_combat::{Energy, Hp};

use crate::{
    ActivityBattleSettlementError, ActivityParticipantCarryDefinition,
    ActivityParticipantCarryState, EnergyCarryPolicy, HpCarryPolicy, ParticipantBattleState,
    battle_settlement::apply_carry,
};

pub(crate) fn rebind_entry_carry(
    definition: ActivityParticipantCarryDefinition,
    previous: ActivityParticipantCarryState,
    maximum_hp: Hp,
    maximum_energy: Energy,
) -> Result<ActivityParticipantCarryState, ActivityBattleSettlementError> {
    if definition.participant() != previous.participant()
        || (definition.hp() == HpCarryPolicy::CarryExact && previous.current_hp() > maximum_hp)
        || (definition.energy() == EnergyCarryPolicy::CarryExact
            && previous.current_energy() > maximum_energy)
    {
        return Err(ActivityBattleSettlementError::CarryInvariant);
    }
    // Exact values have already passed their bounds check. The other policies
    // permit bounding before applying the existing life/presence projection.
    let state = ParticipantBattleState::new(
        previous.participant(),
        previous.current_hp().min(maximum_hp),
        maximum_hp,
        previous.current_energy().min(maximum_energy),
        maximum_energy,
        previous.life(),
        previous.presence(),
    )
    .ok_or(ActivityBattleSettlementError::CarryInvariant)?;
    apply_carry(definition, state)
}

#[cfg(test)]
#[path = "battle_entry_carry_tests.rs"]
mod tests;
