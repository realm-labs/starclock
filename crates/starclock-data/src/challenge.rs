//! Typed Sora lowering for rotating challenge profiles.

use starclock_combat::{
    ActionValue, BattleClockExpiry, EncounterId, FormationIndex, Hp, RawToughness, Rounding,
    RuleBundleId, Scalar, Speed, StatValue, UnitLevel,
    formula::{model::CombatElement, toughness::EnemyRank},
};
use starclock_mode_challenge::{
    ActionValueClockRule, ApocalypticCombatDefinitions, ApocalypticEncounter,
    ApocalypticEnemyBinding, ApocalypticEnemyBindingId, ApocalypticEnemySlot, ApocalypticNode,
    ApocalypticProfile, ApocalypticStage, ChallengeNodeId, ChallengeProfileId, ChallengeStageId,
    CycleClockRule, MemoryCombatDefinitions, MemoryEncounter, MemoryEnemyBinding,
    MemoryEnemyBindingId, MemoryEnemySlot, MemoryEnemyStats, MemoryEnemyStatsInput, MemoryWave,
    Objective, ObjectiveId, ObjectiveKind, PolicyConfidence, ProjectPolicy,
    PureFictionCombatDefinitions, PureFictionEncounter, PureFictionEnemyBinding,
    PureFictionEnemyBindingId, PureFictionEnemySlot, PureFictionNode, PureFictionProfile,
    PureFictionSpawnEnd, PureFictionStage, PureFictionWave,
    memory_of_chaos::{MemoryNode, MemoryProfile, MemoryStage},
};

use crate::challenge_generated::{
    SoraConfig, challenge_clock_expiry::ChallengeClockExpiry,
    challenge_combat_element::ChallengeCombatElement, challenge_enemy_rank::ChallengeEnemyRank,
    challenge_objective_kind::ChallengeObjectiveKind,
    challenge_policy_confidence::ChallengePolicyConfidence,
    pure_fiction_spawn_end::PureFictionSpawnEnd as GeneratedPureFictionSpawnEnd,
    runtime::SoraBundle,
};

pub(crate) const PRODUCTION_BUNDLE: &[u8] =
    include_bytes!("../../../config/challenge-runtime-generated/config.sora");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChallengeDataError {
    message: Box<str>,
}

impl std::fmt::Display for ChallengeDataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ChallengeDataError {}

/// Loads the production challenge bundle and lowers its Memory of Chaos rows.
pub fn memory_of_chaos() -> Result<MemoryProfile, ChallengeDataError> {
    load_memory_of_chaos(PRODUCTION_BUNDLE)
}

/// Loads the exact ordinary and Starward encounter topology and reviewed behavior bindings.
pub fn memory_of_chaos_combat_definitions() -> Result<MemoryCombatDefinitions, ChallengeDataError> {
    load_memory_of_chaos_combat_definitions(PRODUCTION_BUNDLE)
}

/// Loads the active ordinary and Starward Apocalyptic Shadow profile.
pub fn apocalyptic_shadow() -> Result<ApocalypticProfile, ChallengeDataError> {
    load_apocalyptic_shadow(PRODUCTION_BUNDLE)
}

/// Loads the typed ordinary and Starward boss encounter closure.
pub fn apocalyptic_shadow_combat_definitions()
-> Result<ApocalypticCombatDefinitions, ChallengeDataError> {
    load_apocalyptic_shadow_combat_definitions(PRODUCTION_BUNDLE)
}

/// Loads the active released Pure Fiction ordinary and Starward profile.
pub fn pure_fiction() -> Result<PureFictionProfile, ChallengeDataError> {
    load_pure_fiction(PRODUCTION_BUNDLE)
}

/// Loads the exact three-wave Pure Fiction encounter closure.
pub fn pure_fiction_combat_definitions() -> Result<PureFictionCombatDefinitions, ChallengeDataError>
{
    load_pure_fiction_combat_definitions(PRODUCTION_BUNDLE)
}

pub use crate::challenge_anomaly::{anomaly_arbitration, load_anomaly_arbitration};

pub fn load_pure_fiction(bytes: &[u8]) -> Result<PureFictionProfile, ChallengeDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(error)?;
    let config = SoraConfig::from_source(&bundle).map_err(error)?;
    let profile = config
        .pf_runtime_profiles()
        .ordered_rows()
        .next()
        .ok_or_else(|| message("Pure Fiction runtime profile is missing"))?;
    if config.pf_runtime_profiles().len() != 1
        || profile.node_score_maximum != 40_000
        || profile.stage_score_maximum != 120_000
    {
        return Err(message("Pure Fiction score constant denominator drift"));
    }
    let expiry = match profile.expiry {
        ChallengeClockExpiry::Lose => BattleClockExpiry::Lose,
        ChallengeClockExpiry::Finalize => BattleClockExpiry::Finalize,
    };
    let objectives = config
        .pf_runtime_objectives()
        .ordered_rows()
        .map(|row| {
            if row.kind != ChallengeObjectiveKind::ScoreAtLeast {
                return Err(message("Pure Fiction objective is not score-based"));
            }
            Ok(Objective::new(
                ObjectiveId::new(unsigned(row.id, "objective id")?)
                    .ok_or_else(|| message("objective id must be non-zero"))?,
                ObjectiveKind::ScoreAtLeast(row.threshold),
            ))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let cacophonies = config
        .pf_runtime_cacophonies()
        .ordered_rows()
        .map(|row| {
            RuleBundleId::new(unsigned(row.upstream_buff_id, "Cacophony id")?)
                .ok_or_else(|| message("Cacophony id must be non-zero"))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let mut stages = Vec::new();
    for row in config.pf_runtime_stages().ordered_rows() {
        let clock = CycleClockRule::new(
            u16::try_from(row.initial_cycles)
                .map_err(|_| message("Pure Fiction cycle budget exceeds u16"))?,
            action_value(profile.first_window_scaled)?,
            action_value(profile.later_window_scaled)?,
            false,
            expiry,
        )
        .ok_or_else(|| message("invalid Pure Fiction clock"))?;
        let mut nodes = config
            .pf_runtime_nodes()
            .ordered_rows()
            .filter(|node| node.stage_id == row.id)
            .map(|node| {
                Ok(PureFictionNode {
                    id: ChallengeNodeId::new(unsigned(node.id, "node id")?)
                        .ok_or_else(|| message("node id must be non-zero"))?,
                    encounter: EncounterId::new(unsigned(node.encounter_id, "encounter id")?)
                        .ok_or_else(|| message("encounter id must be non-zero"))?,
                    team_index: u8::try_from(node.team_index)
                        .map_err(|_| message("team index exceeds u8"))?,
                    score_cap: i64::from(profile.node_score_maximum),
                    cacophony_bundles: cacophonies.clone().into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        nodes.sort_by_key(|node| node.team_index);
        if !(2..=3).contains(&nodes.len())
            || nodes
                .iter()
                .enumerate()
                .any(|(index, node)| usize::from(node.team_index) != index)
        {
            return Err(message(
                "Pure Fiction stage requires two or three canonically indexed nodes",
            ));
        }
        let stage_objectives = objectives
            .iter()
            .copied()
            .filter(|objective| {
                if nodes.len() == 3 {
                    objective.id().get() >= 4_000
                } else {
                    objective.id().get() < 4_000
                }
            })
            .collect::<Vec<_>>();
        if stage_objectives.len() != 3 {
            return Err(message("Pure Fiction objective family denominator drift"));
        }
        stages.push(PureFictionStage {
            id: ChallengeStageId::new(unsigned(row.upstream_stage_id, "stage id")?)
                .ok_or_else(|| message("stage id must be non-zero"))?,
            clock,
            clear_score: i64::from(row.clear_score),
            nodes: nodes.into_boxed_slice(),
            objectives: stage_objectives.into_boxed_slice(),
        });
    }
    if stages.len() != 5 || cacophonies.len() != 3 {
        return Err(message("Pure Fiction profile denominator drift"));
    }
    Ok(PureFictionProfile {
        id: ChallengeProfileId::new(unsigned(profile.id, "profile id")?)
            .ok_or_else(|| message("profile id must be non-zero"))?,
        stages: stages.into_boxed_slice(),
        policies: pure_fiction_policies(&config),
    })
}

pub fn load_pure_fiction_combat_definitions(
    bytes: &[u8],
) -> Result<PureFictionCombatDefinitions, ChallengeDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(error)?;
    let config = SoraConfig::from_source(&bundle).map_err(error)?;
    let enemies = config
        .pf_runtime_enemies()
        .ordered_rows()
        .map(|row| {
            PureFictionEnemyBinding::new(
                PureFictionEnemyBindingId::new(unsigned(row.id, "enemy id")?)
                    .ok_or_else(|| message("enemy id must be non-zero"))?,
                unsigned(row.upstream_monster_id, "upstream monster id")?,
                row.stable_key.clone(),
                row.behavior_source_key.clone(),
                row.behavior_exact,
            )
            .ok_or_else(|| message("invalid Pure Fiction enemy binding"))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let mut encounters = Vec::new();
    for encounter in config.pf_runtime_encounters().ordered_rows() {
        let mut waves = Vec::new();
        for wave in config
            .pf_runtime_waves()
            .ordered_rows()
            .filter(|wave| wave.encounter_id == encounter.id)
        {
            let slots = config
                .pf_runtime_enemy_slots()
                .ordered_rows()
                .filter(|slot| slot.wave_id == wave.id)
                .map(|slot| pure_fiction_slot(&config, slot))
                .collect::<Result<Vec<_>, ChallengeDataError>>()?;
            let spawn_end = match wave.spawn_end {
                GeneratedPureFictionSpawnEnd::DefeatQuota => PureFictionSpawnEnd::DefeatQuota(
                    u16::try_from(
                        wave.defeat_quota
                            .ok_or_else(|| message("defeat-quota wave has no quota"))?,
                    )
                    .map_err(|_| message("defeat quota exceeds u16"))?,
                ),
                GeneratedPureFictionSpawnEnd::RequiredSlots => {
                    PureFictionSpawnEnd::RequiredSlotsDefeated
                }
            };
            waves.push(
                PureFictionWave::new(
                    u16::try_from(wave.sequence)
                        .map_err(|_| message("wave sequence exceeds u16"))?,
                    slots,
                    spawn_end,
                    wave.refill_source_wave
                        .map(|value| {
                            u16::try_from(value)
                                .map_err(|_| message("refill source wave exceeds u16"))
                        })
                        .transpose()?,
                    u8::try_from(wave.maximum_simultaneous)
                        .map_err(|_| message("maximum simultaneous exceeds u8"))?,
                    i64::from(wave.score_cap),
                    wave.normal_defeat_true_damage_scaled,
                )
                .ok_or_else(|| message("invalid Pure Fiction wave"))?,
            );
        }
        encounters.push(
            PureFictionEncounter::new(
                EncounterId::new(unsigned(encounter.id, "encounter id")?)
                    .ok_or_else(|| message("encounter id must be non-zero"))?,
                UnitLevel::new(
                    u8::try_from(encounter.level).map_err(|_| message("level exceeds u8"))?,
                )
                .ok_or_else(|| message("level is outside combat bounds"))?,
                waves,
            )
            .ok_or_else(|| message("invalid Pure Fiction encounter"))?,
        );
    }
    if enemies.len() != 42 || encounters.len() != 9 {
        return Err(message("Pure Fiction combat definition denominator drift"));
    }
    PureFictionCombatDefinitions::new(enemies, encounters)
        .ok_or_else(|| message("invalid Pure Fiction combat definitions"))
}

fn pure_fiction_slot(
    config: &SoraConfig,
    slot: &crate::challenge_generated::pf_runtime_enemy_slots::PfRuntimeEnemySlots,
) -> Result<PureFictionEnemySlot, ChallengeDataError> {
    let enemy = config
        .pf_runtime_enemies()
        .get(&slot.enemy_id)
        .ok_or_else(|| message("Pure Fiction slot enemy is missing"))?;
    let weaknesses = enemy
        .weaknesses
        .as_deref()
        .unwrap_or_default()
        .iter()
        .copied()
        .map(combat_element)
        .collect();
    let stats = MemoryEnemyStats::new(MemoryEnemyStatsInput {
        maximum_hp: Hp::new(slot.maximum_hp)
            .map_err(|_| message("Pure Fiction enemy HP exceeds bounds"))?,
        attack: StatValue::from_scaled(slot.attack_scaled)
            .map_err(|_| message("Pure Fiction Attack exceeds bounds"))?,
        defense: StatValue::from_scaled(slot.defense_scaled)
            .map_err(|_| message("Pure Fiction Defense exceeds bounds"))?,
        speed: Speed::from_scaled(slot.speed_scaled)
            .map_err(|_| message("Pure Fiction Speed exceeds bounds"))?,
        effect_hit_rate: Scalar::from_scaled(slot.effect_hit_rate_scaled),
        effect_resistance: Scalar::from_scaled(slot.effect_resistance_scaled),
        rank: match enemy.rank {
            ChallengeEnemyRank::Normal => EnemyRank::Normal,
            ChallengeEnemyRank::Elite => EnemyRank::Elite,
            ChallengeEnemyRank::Boss => EnemyRank::Boss,
        },
        weaknesses,
        toughness: RawToughness::from_scalar(
            Scalar::from_scaled(slot.toughness_scaled),
            Rounding::NearestTiesEven,
        )
        .map_err(|_| message("Pure Fiction Toughness exceeds bounds"))?,
    })
    .ok_or_else(|| message("invalid Pure Fiction enemy stats"))?;
    Ok(PureFictionEnemySlot::new(
        PureFictionEnemyBindingId::new(unsigned(slot.enemy_id, "enemy id")?)
            .ok_or_else(|| message("enemy id must be non-zero"))?,
        u16::try_from(slot.spawn_sequence).map_err(|_| message("spawn sequence exceeds u16"))?,
        FormationIndex::new(
            u8::try_from(slot.formation_index).map_err(|_| message("formation exceeds u8"))?,
        )
        .ok_or_else(|| message("formation is outside combat bounds"))?,
        stats,
    ))
}

fn pure_fiction_policies(config: &SoraConfig) -> Box<[ProjectPolicy]> {
    config
        .pf_runtime_policies()
        .ordered_rows()
        .map(|row| ProjectPolicy {
            id: row.stable_key.clone().into_boxed_str(),
            known_facts: row.known_facts.clone().into_boxed_str(),
            selected_behavior: row.selected_behavior.clone().into_boxed_str(),
            rejected_alternatives: row
                .rejected_alternatives
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            rationale: row.rationale.clone().into_boxed_str(),
            affected_tests: row
                .affected_tests
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            confidence: match row.confidence {
                ChallengePolicyConfidence::Low => PolicyConfidence::Low,
                ChallengePolicyConfidence::Medium => PolicyConfidence::Medium,
                ChallengePolicyConfidence::High => PolicyConfidence::High,
            },
            replacement_condition: row.replacement_condition.clone().into_boxed_str(),
        })
        .collect()
}

pub fn load_apocalyptic_shadow(bytes: &[u8]) -> Result<ApocalypticProfile, ChallengeDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(error)?;
    let config = SoraConfig::from_source(&bundle).map_err(error)?;
    let profile = config
        .aps_runtime_profiles()
        .ordered_rows()
        .next()
        .ok_or_else(|| message("Apocalyptic runtime profile is missing"))?;
    if config.aps_runtime_profiles().len() != 1
        || profile.boss_progress_maximum != 2_000
        || profile.action_value_score_maximum != 2_000
    {
        return Err(message("Apocalyptic score constant denominator drift"));
    }
    let profile_id = ChallengeProfileId::new(unsigned(profile.id, "profile id")?)
        .ok_or_else(|| message("profile id must be non-zero"))?;
    let expiry = match profile.expiry {
        ChallengeClockExpiry::Lose => BattleClockExpiry::Lose,
        ChallengeClockExpiry::Finalize => BattleClockExpiry::Finalize,
    };
    let clock =
        ActionValueClockRule::new(action_value(profile.initial_action_value_scaled)?, expiry)
            .ok_or_else(|| message("invalid Apocalyptic clock"))?;
    let objectives = config
        .aps_runtime_objectives()
        .ordered_rows()
        .map(|row| {
            let id = ObjectiveId::new(unsigned(row.id, "objective id")?)
                .ok_or_else(|| message("objective id must be non-zero"))?;
            if row.kind != ChallengeObjectiveKind::ScoreAtLeast {
                return Err(message("Apocalyptic objective is not score-based"));
            }
            Ok(Objective::new(
                id,
                ObjectiveKind::ScoreAtLeast(row.threshold),
            ))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let mut stages = Vec::new();
    for row in config.aps_runtime_stages().ordered_rows() {
        let mut nodes = config
            .aps_runtime_nodes()
            .ordered_rows()
            .filter(|node| node.stage_id == row.id)
            .map(|node| {
                Ok(ApocalypticNode {
                    id: ChallengeNodeId::new(unsigned(node.id, "node id")?)
                        .ok_or_else(|| message("node id must be non-zero"))?,
                    encounter: EncounterId::new(unsigned(node.encounter_id, "encounter id")?)
                        .ok_or_else(|| message("encounter id must be non-zero"))?,
                    team_index: u8::try_from(node.team_index)
                        .map_err(|_| message("team index exceeds u8"))?,
                    axiom_bundles: node
                        .axiom_bundle_ids
                        .iter()
                        .map(|id| {
                            RuleBundleId::new(unsigned(*id, "Axiom bundle id")?)
                                .ok_or_else(|| message("Axiom bundle id must be non-zero"))
                        })
                        .collect::<Result<Vec<_>, ChallengeDataError>>()?
                        .into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        nodes.sort_by_key(|node| node.team_index);
        if !(2..=3).contains(&nodes.len()) {
            return Err(message(
                "Apocalyptic stage requires two ordinary or three Starward nodes",
            ));
        }
        let stage_objectives = objectives
            .iter()
            .filter(|objective| {
                if nodes.len() == 3 {
                    objective.id().get() >= 5_000
                } else {
                    objective.id().get() < 5_000
                }
            })
            .cloned()
            .collect::<Vec<_>>();
        stages.push(ApocalypticStage {
            id: ChallengeStageId::new(unsigned(row.upstream_stage_id, "stage id")?)
                .ok_or_else(|| message("stage id must be non-zero"))?,
            nodes: nodes.into_boxed_slice(),
            objectives: stage_objectives.into_boxed_slice(),
        });
    }
    if stages.len() != 5 {
        return Err(message(
            "Apocalyptic ordinary/Starward stage denominator drift",
        ));
    }
    let policies = config
        .aps_runtime_policies()
        .ordered_rows()
        .map(|row| ProjectPolicy {
            id: row.stable_key.clone().into_boxed_str(),
            known_facts: row.known_facts.clone().into_boxed_str(),
            selected_behavior: row.selected_behavior.clone().into_boxed_str(),
            rejected_alternatives: row
                .rejected_alternatives
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            rationale: row.rationale.clone().into_boxed_str(),
            affected_tests: row
                .affected_tests
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            confidence: match row.confidence {
                ChallengePolicyConfidence::Low => PolicyConfidence::Low,
                ChallengePolicyConfidence::Medium => PolicyConfidence::Medium,
                ChallengePolicyConfidence::High => PolicyConfidence::High,
            },
            replacement_condition: row.replacement_condition.clone().into_boxed_str(),
        })
        .collect();
    Ok(ApocalypticProfile {
        id: profile_id,
        clock,
        stages: stages.into_boxed_slice(),
        policies,
    })
}

pub fn load_apocalyptic_shadow_combat_definitions(
    bytes: &[u8],
) -> Result<ApocalypticCombatDefinitions, ChallengeDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(error)?;
    let config = SoraConfig::from_source(&bundle).map_err(error)?;
    let enemies = config
        .aps_runtime_enemies()
        .ordered_rows()
        .map(|row| {
            ApocalypticEnemyBinding::new(
                ApocalypticEnemyBindingId::new(unsigned(row.id, "enemy binding id")?)
                    .ok_or_else(|| message("enemy binding id must be non-zero"))?,
                unsigned(row.upstream_monster_id, "upstream monster id")?,
                row.stable_key.clone(),
                row.behavior_source_key.clone(),
                row.behavior_exact,
            )
            .ok_or_else(|| message("invalid Apocalyptic enemy binding"))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let mut encounters = Vec::new();
    for row in config.aps_runtime_encounters().ordered_rows() {
        let slots = config
            .aps_runtime_enemy_slots()
            .ordered_rows()
            .filter(|slot| slot.encounter_id == row.id)
            .map(|slot| {
                let enemy = config
                    .aps_runtime_enemies()
                    .get(&slot.enemy_id)
                    .ok_or_else(|| message("Apocalyptic slot enemy is missing"))?;
                let weaknesses = enemy
                    .weaknesses
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .copied()
                    .map(combat_element)
                    .collect();
                let stats = MemoryEnemyStats::new(MemoryEnemyStatsInput {
                    maximum_hp: Hp::new(slot.maximum_hp)
                        .map_err(|_| message("Apocalyptic enemy HP exceeds bounds"))?,
                    attack: StatValue::from_scaled(slot.attack_scaled)
                        .map_err(|_| message("Apocalyptic enemy Attack exceeds bounds"))?,
                    defense: StatValue::from_scaled(slot.defense_scaled)
                        .map_err(|_| message("Apocalyptic enemy Defense exceeds bounds"))?,
                    speed: Speed::from_scaled(slot.speed_scaled)
                        .map_err(|_| message("Apocalyptic enemy Speed exceeds bounds"))?,
                    effect_hit_rate: Scalar::ZERO,
                    effect_resistance: Scalar::ZERO,
                    rank: match enemy.rank {
                        ChallengeEnemyRank::Normal => EnemyRank::Normal,
                        ChallengeEnemyRank::Elite => EnemyRank::Elite,
                        ChallengeEnemyRank::Boss => EnemyRank::Boss,
                    },
                    weaknesses,
                    toughness: RawToughness::from_scalar(
                        Scalar::from_scaled(slot.toughness_scaled),
                        Rounding::NearestTiesEven,
                    )
                    .map_err(|_| message("Apocalyptic Toughness exceeds bounds"))?,
                })
                .ok_or_else(|| message("invalid Apocalyptic enemy stats"))?;
                Ok(ApocalypticEnemySlot::new(
                    ApocalypticEnemyBindingId::new(unsigned(slot.enemy_id, "enemy id")?)
                        .ok_or_else(|| message("enemy id must be non-zero"))?,
                    FormationIndex::new(
                        u8::try_from(slot.formation_index)
                            .map_err(|_| message("formation exceeds u8"))?,
                    )
                    .ok_or_else(|| message("formation is outside combat bounds"))?,
                    slot.score_included,
                    stats,
                ))
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        encounters.push(
            ApocalypticEncounter::new(
                EncounterId::new(unsigned(row.id, "encounter id")?)
                    .ok_or_else(|| message("encounter id must be non-zero"))?,
                UnitLevel::new(u8::try_from(row.level).map_err(|_| message("level exceeds u8"))?)
                    .ok_or_else(|| message("level is outside combat bounds"))?,
                slots,
            )
            .ok_or_else(|| message("invalid Apocalyptic encounter"))?,
        );
    }
    if enemies.len() != 10 || encounters.len() != 9 {
        return Err(message("Apocalyptic combat definition denominator drift"));
    }
    ApocalypticCombatDefinitions::new(enemies, encounters)
        .ok_or_else(|| message("invalid Apocalyptic combat definitions"))
}

fn combat_element(element: ChallengeCombatElement) -> CombatElement {
    match element {
        ChallengeCombatElement::Physical => CombatElement::Physical,
        ChallengeCombatElement::Fire => CombatElement::Fire,
        ChallengeCombatElement::Ice => CombatElement::Ice,
        ChallengeCombatElement::Lightning => CombatElement::Lightning,
        ChallengeCombatElement::Wind => CombatElement::Wind,
        ChallengeCombatElement::Quantum => CombatElement::Quantum,
        ChallengeCombatElement::Imaginary => CombatElement::Imaginary,
    }
}

pub fn load_memory_of_chaos(bytes: &[u8]) -> Result<MemoryProfile, ChallengeDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(error)?;
    let config = SoraConfig::from_source(&bundle).map_err(error)?;
    let profile_row = config
        .moc_runtime_profiles()
        .ordered_rows()
        .next()
        .ok_or_else(|| message("Memory of Chaos runtime profile is missing"))?;
    if config.moc_runtime_profiles().len() != 1 {
        return Err(message(
            "Memory of Chaos requires exactly one runtime profile",
        ));
    }
    let profile_id = ChallengeProfileId::new(unsigned(profile_row.id, "profile id")?)
        .ok_or_else(|| message("profile id must be non-zero"))?;
    let initial_cycles = u16::try_from(profile_row.initial_cycles)
        .map_err(|_| message("initial cycles exceed u16"))?;
    let expiry = match profile_row.expiry {
        ChallengeClockExpiry::Lose => BattleClockExpiry::Lose,
        ChallengeClockExpiry::Finalize => BattleClockExpiry::Finalize,
    };
    let clock = CycleClockRule::new(
        initial_cycles,
        action_value(profile_row.first_window_scaled)?,
        action_value(profile_row.later_window_scaled)?,
        profile_row.reset_window_on_wave,
        expiry,
    )
    .ok_or_else(|| message("invalid Memory of Chaos clock"))?;
    let objectives = config
        .moc_runtime_objectives()
        .ordered_rows()
        .map(|row| {
            let id = ObjectiveId::new(unsigned(row.id, "objective id")?)
                .ok_or_else(|| message("objective id must be non-zero"))?;
            let kind = match row.kind {
                ChallengeObjectiveKind::Complete => ObjectiveKind::Complete,
                ChallengeObjectiveKind::NoDefeatedParticipants => {
                    ObjectiveKind::NoDefeatedParticipants
                }
                ChallengeObjectiveKind::RemainingCyclesAtLeast => {
                    ObjectiveKind::RemainingCyclesAtLeast(
                        u16::try_from(row.threshold)
                            .map_err(|_| message("cycle objective threshold exceeds u16"))?,
                    )
                }
                ChallengeObjectiveKind::ScoreAtLeast => ObjectiveKind::ScoreAtLeast(row.threshold),
            };
            Ok(Objective::new(id, kind))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let mut stages = Vec::new();
    for stage in config.moc_runtime_stages().ordered_rows() {
        let stage_clock = CycleClockRule::new(
            u16::try_from(stage.initial_cycles)
                .map_err(|_| message("stage cycle budget exceeds u16"))?,
            action_value(profile_row.first_window_scaled)?,
            action_value(profile_row.later_window_scaled)?,
            profile_row.reset_window_on_wave,
            expiry,
        )
        .ok_or_else(|| message("invalid Memory stage clock"))?;
        let stage_id = ChallengeStageId::new(unsigned(stage.upstream_stage_id, "stage id")?)
            .ok_or_else(|| message("stage id must be non-zero"))?;
        let mut nodes = config
            .moc_runtime_nodes()
            .ordered_rows()
            .filter(|node| node.stage_id == stage.id)
            .map(|node| {
                let id = ChallengeNodeId::new(unsigned(node.id, "node id")?)
                    .ok_or_else(|| message("node id must be non-zero"))?;
                let encounter =
                    EncounterId::new(unsigned(node.upstream_encounter_id, "encounter id")?)
                        .ok_or_else(|| message("encounter id must be non-zero"))?;
                let turbulence = RuleBundleId::new(unsigned(
                    stage.turbulence_upstream_id,
                    "turbulence rule bundle id",
                )?)
                .ok_or_else(|| message("turbulence rule bundle id must be non-zero"))?;
                MemoryNode::new(
                    id,
                    encounter,
                    u8::try_from(node.team_index).map_err(|_| message("team index exceeds u8"))?,
                    vec![turbulence],
                )
                .ok_or_else(|| message("invalid Memory of Chaos node"))
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        nodes.sort_by_key(MemoryNode::team_index);
        if !(2..=3).contains(&nodes.len()) {
            return Err(message(
                "Memory of Chaos stage requires two ordinary or three Starward nodes",
            ));
        }
        let stage_objectives = objectives
            .iter()
            .copied()
            .filter(|objective| {
                if nodes.len() == 3 {
                    objective.id().get() > 3
                } else {
                    objective.id().get() <= 3
                }
            })
            .collect::<Vec<_>>();
        stages.push(
            MemoryStage::new(stage_id, stage_clock, nodes, stage_objectives)
                .ok_or_else(|| message("invalid Memory of Chaos stage"))?,
        );
    }
    let policies = config
        .moc_runtime_policies()
        .ordered_rows()
        .map(|row| ProjectPolicy {
            id: row.stable_key.clone().into_boxed_str(),
            known_facts: row.known_facts.clone().into_boxed_str(),
            selected_behavior: row.selected_behavior.clone().into_boxed_str(),
            rejected_alternatives: row
                .rejected_alternatives
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            rationale: row.rationale.clone().into_boxed_str(),
            affected_tests: row
                .affected_tests
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            confidence: match row.confidence {
                ChallengePolicyConfidence::Low => PolicyConfidence::Low,
                ChallengePolicyConfidence::Medium => PolicyConfidence::Medium,
                ChallengePolicyConfidence::High => PolicyConfidence::High,
            },
            replacement_condition: row.replacement_condition.clone().into_boxed_str(),
        })
        .collect();
    MemoryProfile::new(profile_id, clock, stages, policies)
        .ok_or_else(|| message("invalid Memory of Chaos profile"))
}

pub fn load_memory_of_chaos_combat_definitions(
    bytes: &[u8],
) -> Result<MemoryCombatDefinitions, ChallengeDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(error)?;
    let config = SoraConfig::from_source(&bundle).map_err(error)?;
    let enemies = config
        .moc_runtime_enemy_bindings()
        .ordered_rows()
        .map(|row| {
            MemoryEnemyBinding::new(
                MemoryEnemyBindingId::new(unsigned(row.id, "enemy binding id")?)
                    .ok_or_else(|| message("enemy binding id must be non-zero"))?,
                unsigned(row.upstream_variant_id, "upstream enemy variant id")?,
                row.stable_key.clone(),
                row.behavior_source_key.clone(),
                row.behavior_exact,
            )
            .ok_or_else(|| message("invalid Memory enemy behavior binding"))
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let mut encounters = Vec::new();
    for row in config.moc_runtime_encounters().ordered_rows() {
        let node = config
            .moc_runtime_nodes()
            .get(&row.node_id)
            .ok_or_else(|| message("Memory encounter refers to a missing node"))?;
        if node.upstream_encounter_id != row.id {
            return Err(message("Memory node and encounter identity differ"));
        }
        let mut waves = Vec::new();
        for wave in config
            .moc_runtime_waves()
            .ordered_rows()
            .filter(|wave| wave.encounter_id == row.id)
        {
            let slots = config
                .moc_runtime_enemy_slots()
                .ordered_rows()
                .filter(|slot| slot.wave_id == wave.id)
                .map(|slot| {
                    MemoryEnemySlot::new(
                        MemoryEnemyBindingId::new(unsigned(
                            slot.enemy_binding_id,
                            "enemy slot binding id",
                        )?)
                        .ok_or_else(|| message("enemy slot binding id must be non-zero"))?,
                        u16::try_from(slot.spawn_sequence)
                            .map_err(|_| message("enemy spawn sequence exceeds u16"))?,
                        FormationIndex::new(
                            u8::try_from(slot.formation_index)
                                .map_err(|_| message("enemy formation exceeds u8"))?,
                        )
                        .ok_or_else(|| message("enemy formation is outside combat bounds"))?,
                        enemy_stats(slot)?,
                    )
                    .ok_or_else(|| message("invalid Memory enemy slot"))
                })
                .collect::<Result<Vec<_>, ChallengeDataError>>()?;
            waves.push(
                MemoryWave::new(
                    u16::try_from(wave.sequence)
                        .map_err(|_| message("Memory wave sequence exceeds u16"))?,
                    slots,
                )
                .ok_or_else(|| message("invalid Memory wave"))?,
            );
        }
        encounters.push(
            MemoryEncounter::new(
                EncounterId::new(unsigned(row.id, "Memory encounter id")?)
                    .ok_or_else(|| message("Memory encounter id must be non-zero"))?,
                UnitLevel::new(
                    u8::try_from(row.level)
                        .map_err(|_| message("Memory encounter level exceeds u8"))?,
                )
                .ok_or_else(|| message("Memory encounter level is outside combat bounds"))?,
                u16::try_from(row.hard_level_group)
                    .map_err(|_| message("Memory hard-level group exceeds u16"))?,
                waves,
            )
            .ok_or_else(|| message("invalid Memory encounter"))?,
        );
    }
    if enemies.len() != 41
        || enemies
            .iter()
            .filter(|enemy| enemy.behavior_exact())
            .count()
            != 19
        || encounters.len() != 25
    {
        return Err(message("Memory combat definition denominator drift"));
    }
    MemoryCombatDefinitions::new(enemies, encounters)
        .ok_or_else(|| message("invalid Memory combat definitions"))
}

fn enemy_stats(
    row: &crate::challenge_generated::moc_runtime_enemy_slots::MocRuntimeEnemySlots,
) -> Result<MemoryEnemyStats, ChallengeDataError> {
    let weaknesses = row
        .weaknesses
        .as_deref()
        .unwrap_or_default()
        .iter()
        .copied()
        .map(|element| match element {
            ChallengeCombatElement::Physical => CombatElement::Physical,
            ChallengeCombatElement::Fire => CombatElement::Fire,
            ChallengeCombatElement::Ice => CombatElement::Ice,
            ChallengeCombatElement::Lightning => CombatElement::Lightning,
            ChallengeCombatElement::Wind => CombatElement::Wind,
            ChallengeCombatElement::Quantum => CombatElement::Quantum,
            ChallengeCombatElement::Imaginary => CombatElement::Imaginary,
        })
        .collect();
    MemoryEnemyStats::new(MemoryEnemyStatsInput {
        maximum_hp: Hp::from_scalar(
            Scalar::from_scaled(row.maximum_hp_scaled),
            Rounding::NearestTiesEven,
        )
        .map_err(|_| message("Memory enemy HP exceeds combat bounds"))?,
        attack: StatValue::from_scaled(row.attack_scaled)
            .map_err(|_| message("Memory enemy Attack exceeds combat bounds"))?,
        defense: StatValue::from_scaled(row.defense_scaled)
            .map_err(|_| message("Memory enemy Defense exceeds combat bounds"))?,
        speed: Speed::from_scaled(row.speed_scaled)
            .map_err(|_| message("Memory enemy Speed exceeds combat bounds"))?,
        effect_hit_rate: Scalar::from_scaled(row.effect_hit_rate_scaled),
        effect_resistance: Scalar::from_scaled(row.effect_resistance_scaled),
        rank: match row.rank {
            ChallengeEnemyRank::Normal => EnemyRank::Normal,
            ChallengeEnemyRank::Elite => EnemyRank::Elite,
            ChallengeEnemyRank::Boss => EnemyRank::Boss,
        },
        weaknesses,
        toughness: RawToughness::from_scalar(
            Scalar::from_scaled(row.toughness_scaled),
            Rounding::NearestTiesEven,
        )
        .map_err(|_| message("Memory enemy Toughness exceeds combat bounds"))?,
    })
    .ok_or_else(|| message("invalid Memory enemy runtime stats"))
}

fn action_value(value: i64) -> Result<ActionValue, ChallengeDataError> {
    ActionValue::from_scaled(value).map_err(|_| message("Action Value must be non-negative"))
}

fn unsigned(value: i32, field: &str) -> Result<u32, ChallengeDataError> {
    u32::try_from(value).map_err(|_| message(&format!("{field} must be non-negative")))
}

fn error(error: impl std::fmt::Display) -> ChallengeDataError {
    message(&error.to_string())
}

pub(crate) fn message(message: &str) -> ChallengeDataError {
    ChallengeDataError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use starclock_mode_challenge::ObjectiveKind;

    use super::{
        apocalyptic_shadow, apocalyptic_shadow_combat_definitions, memory_of_chaos,
        memory_of_chaos_combat_definitions, pure_fiction, pure_fiction_combat_definitions,
    };

    #[test]
    fn production_memory_profile_lowers_all_active_stages_and_policies() {
        let profile = memory_of_chaos().expect("typed challenge bundle loads");
        assert_eq!(profile.stages().len(), 13);
        assert_eq!(profile.stages()[0].id().get(), 5_201);
        assert_eq!(profile.stages()[11].id().get(), 5_212);
        assert_eq!(profile.stages()[12].id().get(), 5_213);
        assert_eq!(profile.stages()[12].nodes().len(), 3);
        assert_eq!(profile.stages()[12].initial_cycles(), 45);
        assert_eq!(profile.stages()[0].nodes()[0].encounter().get(), 30_123_011);
        assert_eq!(profile.stages()[0].nodes()[1].team_index(), 1);
        assert_eq!(profile.policies().len(), 8);
        assert!(
            profile
                .policies()
                .iter()
                .all(|policy| !policy.replacement_condition.is_empty())
        );
    }

    #[test]
    fn production_memory_combat_definitions_lower_exact_closure() {
        let combat = memory_of_chaos_combat_definitions().expect("typed combat rows lower");
        assert_eq!(combat.enemies().len(), 41);
        assert_eq!(
            combat
                .enemies()
                .iter()
                .filter(|enemy| enemy.behavior_exact())
                .count(),
            19
        );
        assert_eq!(combat.encounters().len(), 25);
        let first = &combat.encounters()[0].waves()[0].slots()[0];
        assert_eq!(first.stats().maximum_hp().get(), 15_291);
        assert_eq!(first.stats().speed().scaled(), 132_000_000);
        assert_eq!(first.stats().effect_hit_rate().scaled(), 144_000);
        assert_eq!(first.stats().toughness().get(), 60);
        assert!(combat.encounters().iter().all(|encounter| {
            encounter.waves().len() == 2
                && encounter
                    .waves()
                    .iter()
                    .all(|wave| !wave.slots().is_empty())
        }));
    }

    #[test]
    fn production_apocalyptic_profile_lowers_all_playable_stages() {
        let profile = apocalyptic_shadow().expect("typed Apocalyptic profile loads");
        assert_eq!(profile.stages.len(), 5);
        assert_eq!(profile.stages[0].id.get(), 30_191);
        assert_eq!(profile.stages[3].id.get(), 30_194);
        assert_eq!(profile.stages[0].nodes[0].encounter.get(), 420_471);
        assert_eq!(profile.stages[0].nodes[0].axiom_bundles.len(), 3);
        assert_eq!(profile.stages[4].nodes.len(), 3);
        assert_eq!(profile.stages[4].nodes[2].encounter.get(), 420_494);
        assert_eq!(
            profile.stages[4].objectives[2].kind(),
            ObjectiveKind::ScoreAtLeast(9_900)
        );
        assert_eq!(profile.clock.initial().scaled(), 2_000_000_000);
        assert_eq!(profile.policies.len(), 5);
        assert!(
            profile
                .policies
                .iter()
                .all(|policy| !policy.replacement_condition.is_empty())
        );
    }

    #[test]
    fn production_apocalyptic_combat_definitions_lower() {
        let combat = apocalyptic_shadow_combat_definitions()
            .expect("typed Apocalyptic combat definitions lower");
        assert_eq!(combat.enemies().len(), 10);
        assert!(combat.enemies().iter().all(|enemy| !enemy.behavior_exact()));
        assert_eq!(combat.encounters().len(), 9);
        let final_first = combat
            .encounters()
            .iter()
            .find(|encounter| encounter.id().get() == 420_474)
            .expect("final first node exists");
        assert_eq!(final_first.slots().len(), 2);
        assert_eq!(final_first.slots()[0].stats().maximum_hp().get(), 4_000_000);
        assert_eq!(final_first.slots()[1].stats().maximum_hp().get(), 1_000_000);
    }

    #[test]
    fn production_pure_fiction_profile_lowers_all_playable_stages() {
        let profile = pure_fiction().expect("typed Pure Fiction profile loads");
        assert_eq!(profile.stages.len(), 5);
        assert_eq!(profile.stages[0].clock.initial_cycles(), 5);
        assert_eq!(profile.stages[3].clock.initial_cycles(), 4);
        assert_eq!(profile.stages[0].nodes[0].score_cap, 40_000);
        assert_eq!(profile.stages[0].nodes[0].cacophony_bundles.len(), 3);
        assert_eq!(profile.stages[4].nodes.len(), 3);
        assert_eq!(profile.stages[4].nodes[2].encounter.get(), 30_322_043);
        assert_eq!(profile.stages[4].clear_score, 45_000);
        assert_eq!(profile.stages[4].objectives.len(), 3);
        assert_eq!(profile.policies.len(), 5);
    }

    #[test]
    fn production_pure_fiction_combat_definitions_lower_exact_topology() {
        let definitions =
            pure_fiction_combat_definitions().expect("typed Pure Fiction combat rows lower");
        assert_eq!(definitions.enemies().len(), 42);
        assert_eq!(definitions.encounters().len(), 9);
        assert!(
            definitions
                .encounters()
                .iter()
                .all(|encounter| encounter.waves().len() == 3)
        );
        assert!(definitions.encounters().iter().all(|encounter| {
            encounter.waves()[0].slots().len() == 5
                && encounter.waves()[1].slots().len() == 1
                && encounter.waves()[2].slots().len() == 1
        }));
    }
}
