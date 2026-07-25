//! Construction of one executable battle request from a validated materialization.

use std::collections::BTreeMap;

use starclock_combat::{
    AssemblyDigest, BattleSpec, CombatantSpecDigest, ConcedePolicy, EnemyDefinitionId, Energy, Hp,
    KeyedTeamResourceSpec, ParticipantSource, ParticipantSpec, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamResourceWavePolicy, TeamSide,
    UnitLevel, catalog::CombatCatalog,
};

use crate::{
    battle_contribution::UniverseBattleContributionSet,
    battle_rule_lowering::{RESONANCE_RESOURCE_ID, RESONANCE_RESOURCE_KEY, RuleAttachment},
    encounter::{DifficultyEnemyBinding, EncounterMemberDefinition},
};

use super::{
    UniverseBattleMaterializationError, checked_formation, checked_level, checked_sequence,
    difficulty_encounter_id, materialization_digest::enemy_digest,
    materialization_digest::spec_digest, member_encounter_id,
};

pub(super) fn member_spec(
    member: &EncounterMemberDefinition,
    players: &[ParticipantSpec],
    enemy_map: &BTreeMap<&str, EnemyDefinitionId>,
    catalog: &CombatCatalog,
    revision: &str,
    root_digest: [u8; 32],
    contributions: &UniverseBattleContributionSet,
) -> Result<BattleSpec, UniverseBattleMaterializationError> {
    let mut participants = players.to_vec();
    for (wave_index, wave) in member.waves().iter().enumerate() {
        for (slot_index, slot) in wave.enemies().iter().enumerate() {
            let enemy = *enemy_map
                .get(slot.enemy_variant_key())
                .ok_or(UniverseBattleMaterializationError::MissingEnemyMapping)?;
            participants.push(enemy_participant(
                catalog,
                enemy,
                checked_level(member.stage_level())?,
                wave_index,
                slot_index,
                slot.enemy_variant_key(),
                contributions,
            )?);
        }
    }
    BattleSpec::new(
        revision,
        AssemblyDigest::new(spec_digest(
            root_digest,
            0,
            member.id().get(),
            &participants,
        ))
        .expect("SHA-256 digest is non-zero"),
        member_encounter_id(member.id())?,
        participants,
        player_resources(contributions)?,
        TeamResourceSpec::new(0, 0).expect("empty enemy resources are valid"),
        ConcedePolicy::Allowed,
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleSpec)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn difficulty_spec(
    index: usize,
    binding: &DifficultyEnemyBinding,
    players: &[ParticipantSpec],
    enemy_map: &BTreeMap<&str, EnemyDefinitionId>,
    catalog: &CombatCatalog,
    revision: &str,
    root_digest: [u8; 32],
    contributions: &UniverseBattleContributionSet,
) -> Result<BattleSpec, UniverseBattleMaterializationError> {
    let enemy = *enemy_map
        .get(binding.enemy_variant_key())
        .ok_or(UniverseBattleMaterializationError::MissingEnemyMapping)?;
    let mut participants = players.to_vec();
    participants.push(enemy_participant(
        catalog,
        enemy,
        checked_level(binding.level())?,
        0,
        0,
        binding.enemy_variant_key(),
        contributions,
    )?);
    BattleSpec::new(
        revision,
        AssemblyDigest::new(spec_digest(
            root_digest,
            1,
            u32::try_from(index + 1)
                .map_err(|_| UniverseBattleMaterializationError::IdentityOverflow)?,
            &participants,
        ))
        .expect("SHA-256 digest is non-zero"),
        difficulty_encounter_id(index)?,
        participants,
        player_resources(contributions)?,
        TeamResourceSpec::new(0, 0).expect("empty enemy resources are valid"),
        ConcedePolicy::Allowed,
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleSpec)
}

fn player_resources(
    contributions: &UniverseBattleContributionSet,
) -> Result<TeamResourceSpec, UniverseBattleMaterializationError> {
    let resources = TeamResourceSpec::new(3, 5).expect("standard player resources are valid");
    let Some(resonance) = contributions.resonance() else {
        return Ok(resources);
    };
    let resonance = KeyedTeamResourceSpec::new(
        RESONANCE_RESOURCE_ID,
        resonance.initial_energy(),
        resonance.maximum_energy(),
        TeamResourceWavePolicy::Persist,
    )
    .and_then(|resource| resource.with_stable_key(RESONANCE_RESOURCE_KEY))
    .ok_or(UniverseBattleMaterializationError::InvalidCombatant)?;
    resources
        .with_keyed(vec![resonance])
        .ok_or(UniverseBattleMaterializationError::InvalidCombatant)
}

fn enemy_participant(
    catalog: &CombatCatalog,
    enemy_id: EnemyDefinitionId,
    level: UnitLevel,
    wave_index: usize,
    slot_index: usize,
    source_key: &str,
    contributions: &UniverseBattleContributionSet,
) -> Result<ParticipantSpec, UniverseBattleMaterializationError> {
    let enemy = catalog
        .enemy(enemy_id)
        .ok_or(UniverseBattleMaterializationError::MissingProxyEnemy)?;
    let enemy_rules = contributions
        .executable_rules()
        .iter()
        .filter(|rule| rule.attachment() == RuleAttachment::EveryEnemy)
        .map(|rule| rule.bundle().id())
        .collect::<Vec<_>>();
    let combatant = ResolvedCombatantSpec::new(
        enemy.unit(),
        level,
        Hp::new(1).expect("Goal 01 executable proxy HP is positive"),
        Speed::from_scaled(50_000_000).expect("static proxy Speed is valid"),
        ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), enemy_rules, Vec::new())
            .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?,
        CombatantSpecDigest::new(enemy_digest(
            enemy_id, level, wave_index, slot_index, source_key,
        ))
        .expect("SHA-256 digest is non-zero"),
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?
    .with_energy(Energy::ZERO, Energy::ZERO)
    .map_err(|_| UniverseBattleMaterializationError::InvalidCombatant)?;
    ParticipantSpec::new(
        TeamSide::Enemy,
        checked_formation(slot_index)?,
        ParticipantSource::EncounterEnemy(enemy_id),
        combatant,
    )
    .with_wave(checked_sequence(wave_index)?)
    .ok_or(UniverseBattleMaterializationError::InvalidBattleSpec)
}
