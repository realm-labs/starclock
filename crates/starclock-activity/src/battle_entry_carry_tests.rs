use starclock_combat::{Energy, Hp, LifeState, PresenceState};

use crate::{
    ActivityBattleSettlementError, ActivityParticipantCarryDefinition,
    ActivityParticipantCarryState, EnergyCarryPolicy, HpCarryPolicy, LifeCarryPolicy,
    ParticipantBattleState, ParticipantId, PresenceCarryPolicy,
    battle_entry_carry::rebind_entry_carry, battle_settlement::apply_carry,
};

fn definition(hp: HpCarryPolicy, energy: EnergyCarryPolicy) -> ActivityParticipantCarryDefinition {
    ActivityParticipantCarryDefinition::new(
        ParticipantId::new(1).unwrap(),
        hp,
        energy,
        LifeCarryPolicy::CarryExact,
        PresenceCarryPolicy::CarryExact,
    )
}

fn previous(hp: i64, energy_scaled: i64, life: LifeState) -> ActivityParticipantCarryState {
    apply_carry(
        definition(HpCarryPolicy::CarryExact, EnergyCarryPolicy::CarryExact),
        ParticipantBattleState::new(
            ParticipantId::new(1).unwrap(),
            Hp::new(hp).unwrap(),
            Hp::new(1_000).unwrap(),
            Energy::from_scaled(energy_scaled).unwrap(),
            Energy::from_scaled(100_000_000).unwrap(),
            life,
            PresenceState::Departed,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn entry_capacity_vectors_preserve_absolute_values_and_fractional_energy() {
    for hp_policy in [HpCarryPolicy::CarryExact, HpCarryPolicy::CarryClamped] {
        for energy_policy in [
            EnergyCarryPolicy::CarryExact,
            EnergyCarryPolicy::CarryClamped,
        ] {
            for (maximum_hp, maximum_energy) in [(700, 60_250_001), (2_000, 200_000_000)] {
                let prior = previous(700, 60_250_001, LifeState::Alive);
                let result = rebind_entry_carry(
                    definition(hp_policy, energy_policy),
                    prior,
                    Hp::new(maximum_hp).unwrap(),
                    Energy::from_scaled(maximum_energy).unwrap(),
                )
                .unwrap();
                assert_eq!(result.current_hp(), prior.current_hp());
                assert_eq!(result.current_energy(), prior.current_energy());
                assert_eq!(result.maximum_hp(), Hp::new(maximum_hp).unwrap());
                assert_eq!(
                    result.maximum_energy(),
                    Energy::from_scaled(maximum_energy).unwrap()
                );
                assert_eq!(result.life(), prior.life());
                assert_eq!(result.presence(), prior.presence());
            }
        }
    }
}

#[test]
fn entry_exact_capacity_rejects_one_unit_over_either_destination() {
    for (maximum_hp, maximum_energy) in [(699, 60_250_001), (700, 60_250_000)] {
        assert_eq!(
            rebind_entry_carry(
                definition(HpCarryPolicy::CarryExact, EnergyCarryPolicy::CarryExact),
                previous(700, 60_250_001, LifeState::Alive),
                Hp::new(maximum_hp).unwrap(),
                Energy::from_scaled(maximum_energy).unwrap(),
            ),
            Err(ActivityBattleSettlementError::CarryInvariant)
        );
    }
}

#[test]
fn entry_clamped_capacity_includes_zero_energy_and_one_hp() {
    let result = rebind_entry_carry(
        definition(HpCarryPolicy::CarryClamped, EnergyCarryPolicy::CarryClamped),
        previous(700, 60_250_001, LifeState::Alive),
        Hp::new(1).unwrap(),
        Energy::ZERO,
    )
    .unwrap();
    assert_eq!(result.current_hp(), Hp::new(1).unwrap());
    assert_eq!(result.current_energy(), Energy::ZERO);
    assert_eq!(result.maximum_energy(), Energy::ZERO);
}

#[test]
fn entry_full_and_zero_policies_use_new_capacity_and_explicit_revive() {
    let policy = ActivityParticipantCarryDefinition::new(
        ParticipantId::new(1).unwrap(),
        HpCarryPolicy::RestoreFull,
        EnergyCarryPolicy::ResetZero,
        LifeCarryPolicy::RestoreAlive,
        PresenceCarryPolicy::RestorePresent,
    );
    let result = rebind_entry_carry(
        policy,
        previous(0, 60_250_001, LifeState::Defeated),
        Hp::new(500).unwrap(),
        Energy::ZERO,
    )
    .unwrap();
    assert_eq!(result.current_hp(), Hp::new(500).unwrap());
    assert_eq!(result.current_energy(), Energy::ZERO);
    assert_eq!(result.life(), LifeState::Alive);
    assert_eq!(result.presence(), PresenceState::Present);
}

#[test]
fn entry_incompatible_life_policy_is_not_an_implicit_revive() {
    assert_eq!(
        rebind_entry_carry(
            definition(HpCarryPolicy::RestoreFull, EnergyCarryPolicy::ResetZero),
            previous(0, 0, LifeState::Defeated),
            Hp::new(500).unwrap(),
            Energy::ZERO,
        ),
        Err(ActivityBattleSettlementError::CarryInvariant)
    );
}

#[test]
fn entry_zero_hp_defeat_and_departure_are_explicit() {
    let policy = ActivityParticipantCarryDefinition::new(
        ParticipantId::new(1).unwrap(),
        HpCarryPolicy::CarryClamped,
        EnergyCarryPolicy::CarryClamped,
        LifeCarryPolicy::DefeatOnZero,
        PresenceCarryPolicy::DepartIfDefeated,
    );
    let result = rebind_entry_carry(
        policy,
        previous(0, 0, LifeState::Downed),
        Hp::new(500).unwrap(),
        Energy::ZERO,
    )
    .unwrap();
    assert_eq!(result.current_hp(), Hp::new(0).unwrap());
    assert_eq!(result.life(), LifeState::Defeated);
    assert_eq!(result.presence(), PresenceState::Departed);
}
