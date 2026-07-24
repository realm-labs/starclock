//! Canonical identities for Standard Universe battle materialization.

use starclock_combat::{EnemyDefinitionId, ParticipantSpec, ResolvedCombatantSpec, UnitLevel};

use crate::{
    battle_contribution::UniverseBattleContributionSet,
    battle_technique::CompiledUniverseBattleTechnique, catalog::UniverseCatalog, digest::Encoder,
};

use super::{
    DIFFICULTY_BINDING_COUNT, MEMBER_COUNT, UNIVERSE_BATTLE_MATERIALIZATION_REVISION,
    UNIVERSE_ENEMY_RUNTIME_STAT_POLICY, UniverseBattleRoster, UniverseEnemyMaterialization,
};

pub(super) fn root_digest(
    universe: &UniverseCatalog,
    roster: &UniverseBattleRoster,
    contributions: &UniverseBattleContributionSet,
    enemies: &[UniverseEnemyMaterialization],
    technique: Option<&CompiledUniverseBattleTechnique>,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-materialization.root.v1");
    encoder.text(UNIVERSE_BATTLE_MATERIALIZATION_REVISION);
    encoder.digest(universe.identity().universe_bundle_digest().bytes());
    encoder.digest(roster.participant_lock().bytes());
    encoder.digest(contributions.digest());
    if let Some(technique) = technique {
        encoder.u8(1);
        encoder.digest(technique.digest());
    }
    encoder.u32(enemies.len() as u32);
    for enemy in enemies {
        encoder.text(enemy.stable_key());
        encoder.u8(enemy.definition_match() as u8);
        encoder.u32(enemy.combat_enemy().get());
        encoder.optional_text(enemy.proxy_stable_key());
    }
    encoder.finish()
}

pub(super) fn combatant_digest(
    base: &ResolvedCombatantSpec,
    contributions: &UniverseBattleContributionSet,
    technique: Option<&CompiledUniverseBattleTechnique>,
) -> [u8; 32] {
    let mut encoder =
        Encoder::new(b"starclock.standard-universe.player-combatant-materialization.v1");
    encoder.digest(base.digest().bytes());
    encoder.digest(contributions.digest());
    if let Some(technique) = technique {
        encoder.u8(1);
        encoder.digest(technique.digest());
    }
    encoder.finish()
}

pub(super) fn technique_variant_digest(contributions: [u8; 32], technique: [u8; 32]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-technique-contribution.v1");
    encoder.digest(contributions);
    encoder.digest(technique);
    encoder.finish()
}

pub(super) fn enemy_digest(
    enemy: EnemyDefinitionId,
    level: UnitLevel,
    wave_index: usize,
    slot_index: usize,
    source_key: &str,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.enemy-proxy-combatant.v1");
    encoder.text(UNIVERSE_ENEMY_RUNTIME_STAT_POLICY);
    encoder.text(source_key);
    encoder.u32(enemy.get());
    encoder.u8(level.get());
    encoder.u32(wave_index as u32);
    encoder.u32(slot_index as u32);
    encoder.finish()
}

pub(super) fn spec_digest(
    root: [u8; 32],
    kind: u8,
    identity: u32,
    participants: &[ParticipantSpec],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-spec.v1");
    encoder.digest(root);
    encoder.u8(kind);
    encoder.u32(identity);
    encoder.u32(participants.len() as u32);
    for participant in participants {
        encoder.u8(participant.side() as u8);
        encoder.u8(participant.formation().get());
        encoder.u32(u32::from(participant.wave()));
        encoder.digest(participant.combatant().digest().bytes());
    }
    encoder.finish()
}

pub(super) fn coverage_digest(
    wave_count: usize,
    enemy_slot_count: usize,
    exact: usize,
    declared_rules: usize,
    materialized_rules: usize,
    enemies: &[UniverseEnemyMaterialization],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-coverage.v1");
    encoder.u32(MEMBER_COUNT as u32);
    encoder.u32(wave_count as u32);
    encoder.u32(enemy_slot_count as u32);
    encoder.u32(DIFFICULTY_BINDING_COUNT as u32);
    encoder.u32(enemies.len() as u32);
    encoder.u32(exact as u32);
    encoder.u32((enemies.len() - exact) as u32);
    encoder.u32(declared_rules as u32);
    encoder.u32(materialized_rules as u32);
    encoder.text(UNIVERSE_ENEMY_RUNTIME_STAT_POLICY);
    for enemy in enemies {
        encoder.text(enemy.stable_key());
        encoder.u8(enemy.definition_match() as u8);
        encoder.u32(enemy.combat_enemy().get());
        encoder.optional_text(enemy.proxy_stable_key());
    }
    encoder.finish()
}
