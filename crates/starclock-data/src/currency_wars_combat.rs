//! Production shared-combat inputs for Currency Wars battle assembly.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};
use starclock_combat::{
    CombatantSpecDigest, EnemyDefinitionId, Energy, Hp, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Rounding, UnitLevel, catalog::definition::EnemyDefinition,
};
use starclock_mode_currency_wars::{
    CurrencyWarsAvatarBattleBehaviorProgramInput, CurrencyWarsBattleBehaviorFallbackRank,
    CurrencyWarsBattleBehaviorProgramInput, CurrencyWarsBattleOverrideDefinition,
    CurrencyWarsBattleProgramBinding, CurrencyWarsBattleProgramBindingArchetype,
    CurrencyWarsBattleProgramBindingInput, CurrencyWarsBattleResourceParts,
    CurrencyWarsBattleResources, CurrencyWarsCatalog, CurrencyWarsCharacterOverrideBinding,
    CurrencyWarsEnemyAffixDefinition, CurrencyWarsEnemyAiConfigurationInput,
    CurrencyWarsEnemyAiConfigurationRuntimeBinding, CurrencyWarsEnemyBehaviorSource,
    CurrencyWarsEnemyCharacterConfigurationInput,
    CurrencyWarsEnemyCharacterConfigurationRuntimeBinding, CurrencyWarsEnemyCombatInput,
    CurrencyWarsEnemySlotDefinition, CurrencyWarsMechanicProgramDisposition,
};

use crate::{
    catalog,
    currency_wars::{CurrencyWarsDataError, debug_error, error},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const STAT_DIFFICULTY: &str = "standard-universe-v1";
const MODE_ENEMY_BASE: u32 = 0x7d40_0000;
const MINION_DONOR: &str = "enemy.juvenile-sting.minionlv2.variant.01";
const ELITE_DONOR: &str = "enemy.automaton-direwolf.elite.variant.01";
const BOSS_DONOR: &str = "enemy.cocolia-complete.littleboss.variant.01";

#[derive(Clone, Copy)]
struct EnemyBinding {
    definition: EnemyDefinitionId,
    behavior_source: EnemyDefinitionId,
    source: CurrencyWarsEnemyBehaviorSource,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EnemyFamilyKey {
    name: Box<str>,
    rank: Box<str>,
}

#[derive(Clone, Debug)]
struct EnemyFamilyCandidate {
    stable_key: Box<str>,
    complete: bool,
    definition: EnemyDefinitionId,
}

pub fn load_currency_wars_battle_resources(
    catalog: &CurrencyWarsCatalog,
) -> Result<CurrencyWarsBattleResources, CurrencyWarsDataError> {
    let core = catalog::load(CORE_BUNDLE).map_err(debug_error)?;
    let levels = catalog
        .encounter_catalog()
        .released_stages()
        .map(|stage| {
            UnitLevel::new(stage.level)
                .ok_or_else(|| error("Currency Wars released enemy level is invalid"))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let stable_keys = catalog
        .encounter_catalog()
        .enemy_slots
        .iter()
        .filter_map(|slot| match &slot.definition {
            CurrencyWarsEnemySlotDefinition::Monster {
                shared_enemy_key, ..
            } => Some(shared_enemy_key.clone()),
            CurrencyWarsEnemySlotDefinition::EliteScaling { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let configuration_stable_keys = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedEnemyCharacterConfiguration(
                configuration,
            ) => Some(configuration.bindings.iter()),
            _ => None,
        })
        .flatten()
        .map(|binding| binding.shared_enemy_key.clone())
        .collect::<BTreeSet<_>>();
    let identities = stable_keys
        .iter()
        .flat_map(|stable_key| {
            levels
                .iter()
                .copied()
                .map(move |level| (stable_key.clone(), level))
        })
        .collect::<BTreeSet<_>>();
    let mut bindings = BTreeMap::new();
    let mut aliases = Vec::new();
    let behavior_families = behavior_families(&core);
    for stable_key in stable_keys {
        let binding = resolve_enemy_binding(&core, &behavior_families, &stable_key, &mut aliases)?;
        bindings.insert(stable_key, binding);
    }
    for stable_key in configuration_stable_keys {
        if bindings.contains_key(&stable_key) {
            continue;
        }
        let binding = resolve_enemy_binding(&core, &behavior_families, &stable_key, &mut aliases)?;
        bindings.insert(stable_key, binding);
    }
    let mut inputs = Vec::with_capacity(identities.len());
    for (stable_key, level) in identities {
        let binding = bindings
            .get(&stable_key)
            .copied()
            .ok_or_else(|| error("Currency Wars enemy binding is missing"))?;
        let definition = core
            .enemy(binding.behavior_source)
            .ok_or_else(|| error("Currency Wars enemy behavior source is missing"))?;
        let (combatant, stat_source_level) =
            resolved_enemy_combatant(&core, definition, &stable_key, level)?;
        inputs.push(CurrencyWarsEnemyCombatInput {
            stable_key,
            definition: binding.definition,
            level,
            stat_source_level,
            behavior_source: binding.source,
            combatant,
        });
    }
    let policy_level = levels
        .iter()
        .next()
        .copied()
        .ok_or_else(|| error("Currency Wars battle behavior policy level is missing"))?;
    let battle_behavior_programs = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattlePolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .map(|(program, policy)| {
            let (definition, behavior_source) = policy_behavior(
                &core,
                &behavior_families,
                policy.family_key.as_deref(),
                policy.fallback_rank,
            )?;
            let (combatant, _) =
                resolved_enemy_combatant(&core, definition, &program.stable_key, policy_level)?;
            Ok(CurrencyWarsBattleBehaviorProgramInput {
                stable_key: program.stable_key.clone(),
                source_path: program.source_path.clone(),
                source_sha256: program.source_sha256.clone(),
                archetype: policy.archetype,
                definition: definition.id(),
                behavior_source,
                combatant,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let mut available_battle_events = catalog
        .empowerment_catalog()
        .battle_overrides()
        .iter()
        .filter_map(|override_| match &override_.definition {
            CurrencyWarsBattleOverrideDefinition::BackBattleEvent(event) => Some(event.event_id),
            CurrencyWarsBattleOverrideDefinition::SummonBattleEventOverride(override_) => {
                Some(override_.battle_event_id)
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    available_battle_events.extend(
        catalog
            .role_override_catalog()
            .programs()
            .iter()
            .flat_map(|program| program.bindings.iter())
            .filter_map(|binding| match binding {
                CurrencyWarsCharacterOverrideBinding::SummonBattleEvent { unit_id, .. } => {
                    Some(*unit_id)
                }
                CurrencyWarsCharacterOverrideBinding::RoleStar { .. }
                | CurrencyWarsCharacterOverrideBinding::ServantStar { .. } => None,
            }),
    );
    let avatar_battle_behavior_programs = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedAvatarBattlePolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .map(|(program, policy)| {
            if policy
                .battle_event_ids
                .iter()
                .any(|event| !available_battle_events.contains(event))
            {
                return Err(error(
                    "Currency Wars avatar battle policy references an unknown BattleEvent",
                ));
            }
            Ok(CurrencyWarsAvatarBattleBehaviorProgramInput {
                stable_key: program.stable_key.clone(),
                source_path: program.source_path.clone(),
                source_sha256: program.source_sha256.clone(),
                archetype: policy.archetype,
                binding_policy: policy.binding_policy,
                role_ids: policy.role_ids.clone(),
                avatar_ids: policy.avatar_ids.clone(),
                battle_event_ids: policy.battle_event_ids.clone(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let battle_program_bindings = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .map(|(program, policy)| {
            for binding in &policy.bindings {
                validate_battle_program_binding(
                    catalog,
                    &core,
                    &available_battle_events,
                    policy.archetype,
                    &policy.bindings,
                    *binding,
                )?;
            }
            Ok(CurrencyWarsBattleProgramBindingInput {
                stable_key: program.stable_key.clone(),
                source_path: program.source_path.clone(),
                source_sha256: program.source_sha256.clone(),
                archetype: policy.archetype,
                bindings: policy.bindings.clone(),
                runtime_definition_count: u16::try_from(
                    policy
                        .bindings
                        .iter()
                        .filter(|binding| {
                            binding_is_runtime_controller(policy.archetype, **binding)
                        })
                        .count(),
                )
                .map_err(debug_error)?,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let enemy_character_configurations = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedEnemyCharacterConfiguration(
                configuration,
            ) => Some((program, configuration)),
            _ => None,
        })
        .map(|(program, configuration)| {
            let bindings = configuration
                .bindings
                .iter()
                .map(|binding| {
                    let definition = bindings.get(&binding.shared_enemy_key).ok_or_else(|| {
                        error("Currency Wars enemy character configuration is unresolved")
                    })?;
                    Ok(CurrencyWarsEnemyCharacterConfigurationRuntimeBinding {
                        shared_enemy_key: binding.shared_enemy_key.clone(),
                        source_template_id: binding.source_template_id,
                        definition: definition.definition,
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(CurrencyWarsEnemyCharacterConfigurationInput {
                stable_key: program.stable_key.clone(),
                source_path: program.source_path.clone(),
                source_sha256: program.source_sha256.clone(),
                bindings: bindings.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let enemy_ai_configurations = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedEnemyAiConfiguration(configuration) => {
                Some((program, configuration))
            }
            _ => None,
        })
        .map(|(program, configuration)| {
            let runtime_bindings = configuration
                .bindings
                .iter()
                .map(|binding| {
                    let definition = bindings.get(&binding.shared_enemy_key).ok_or_else(|| {
                        error("Currency Wars enemy AI configuration is unresolved")
                    })?;
                    Ok(CurrencyWarsEnemyAiConfigurationRuntimeBinding {
                        shared_enemy_key: binding.shared_enemy_key.clone(),
                        source_template_id: binding.source_template_id,
                        definition: definition.definition,
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(CurrencyWarsEnemyAiConfigurationInput {
                stable_key: program.stable_key.clone(),
                source_path: program.source_path.clone(),
                source_sha256: program.source_sha256.clone(),
                bindings: runtime_bindings.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let role_elements = catalog
        .roles()
        .iter()
        .map(|role| {
            let form = core
                .character_form_for_source_avatar(role.avatar_id)
                .ok_or_else(|| error("Currency Wars role has no released character form"))?;
            let character = core
                .character(form)
                .ok_or_else(|| error("Currency Wars released character form is missing"))?;
            Ok((role.id, character.element()))
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    CurrencyWarsBattleResources::new(
        core.combat_catalog(),
        CurrencyWarsBattleResourceParts {
            enemies: inputs,
            battle_behavior_programs,
            avatar_battle_behavior_programs,
            battle_program_bindings,
            enemy_character_configurations,
            enemy_ai_configurations,
            aliases,
            role_elements,
        },
    )
    .map_err(debug_error)
}

fn validate_battle_program_binding(
    catalog: &CurrencyWarsCatalog,
    core: &catalog::SimulationCatalog,
    available_battle_events: &BTreeSet<u32>,
    archetype: CurrencyWarsBattleProgramBindingArchetype,
    policy_bindings: &[CurrencyWarsBattleProgramBinding],
    binding: CurrencyWarsBattleProgramBinding,
) -> Result<(), CurrencyWarsDataError> {
    let exists = match binding {
        CurrencyWarsBattleProgramBinding::Role(id) => catalog.role(id).is_some(),
        CurrencyWarsBattleProgramBinding::Avatar(id) => {
            catalog.roles().iter().any(|role| role.avatar_id == id)
                || core.character_form_for_source_avatar(id).is_some()
                || matches!(
                    archetype,
                    CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent
                ) && policy_bindings.iter().any(|candidate| {
                    matches!(
                        candidate,
                        CurrencyWarsBattleProgramBinding::BattleEvent(event)
                            if available_battle_events.contains(event)
                    )
                })
        }
        CurrencyWarsBattleProgramBinding::Servant(id) => catalog
            .role_override_catalog()
            .programs()
            .iter()
            .flat_map(|program| program.bindings.iter())
            .any(|candidate| {
                matches!(
                    candidate,
                    CurrencyWarsCharacterOverrideBinding::ServantStar { servant_id, .. }
                        if *servant_id == id
                )
            }),
        CurrencyWarsBattleProgramBinding::BattleEvent(id) => available_battle_events.contains(&id),
        CurrencyWarsBattleProgramBinding::Bond(id) => catalog.bond_catalog().bond(id).is_some(),
        CurrencyWarsBattleProgramBinding::AugmentMazeBuff(id) => catalog
            .augment_catalog()
            .maze_buffs()
            .iter()
            .any(|maze_buff| maze_buff.source_id == id),
        CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(id) => catalog
            .encounter_catalog()
            .enemy_affixes
            .iter()
            .any(|affix| {
                matches!(
                    affix.definition,
                    CurrencyWarsEnemyAffixDefinition::MazeBuff { source_id, .. }
                        if source_id == id
                )
            }),
        CurrencyWarsBattleProgramBinding::Equipment(id) => catalog
            .build_catalog()
            .runtime_equipment_definition(id)
            .is_some(),
    };
    if exists {
        Ok(())
    } else {
        Err(error(&format!(
            "Currency Wars battle-program policy references an unknown runtime definition: {binding:?}",
        )))
    }
}

fn binding_is_runtime_controller(
    archetype: CurrencyWarsBattleProgramBindingArchetype,
    binding: CurrencyWarsBattleProgramBinding,
) -> bool {
    matches!(
        (archetype, binding),
        (
            CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility,
            CurrencyWarsBattleProgramBinding::Avatar(_)
        ) | (
            CurrencyWarsBattleProgramBindingArchetype::ServantAbility,
            CurrencyWarsBattleProgramBinding::Servant(_)
        ) | (
            CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent,
            CurrencyWarsBattleProgramBinding::BattleEvent(_)
        ) | (
            CurrencyWarsBattleProgramBindingArchetype::BondStageAbility,
            CurrencyWarsBattleProgramBinding::Bond(_)
        ) | (
            CurrencyWarsBattleProgramBindingArchetype::AugmentStageAbility,
            CurrencyWarsBattleProgramBinding::AugmentMazeBuff(_)
        ) | (
            CurrencyWarsBattleProgramBindingArchetype::MonsterTagController,
            CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(_)
        ) | (
            CurrencyWarsBattleProgramBindingArchetype::EquipmentController,
            CurrencyWarsBattleProgramBinding::Equipment(_)
        )
    )
}

fn behavior_families(
    catalog: &catalog::SimulationCatalog,
) -> BTreeMap<EnemyFamilyKey, Vec<EnemyFamilyCandidate>> {
    let mut families = BTreeMap::<EnemyFamilyKey, Vec<EnemyFamilyCandidate>>::new();
    for (stable_key, definition) in catalog.enemy_stable_definitions() {
        let Some((key, complete)) = enemy_family(stable_key) else {
            continue;
        };
        families.entry(key).or_default().push(EnemyFamilyCandidate {
            stable_key: stable_key.into(),
            complete,
            definition: definition.id(),
        });
    }
    for candidates in families.values_mut() {
        candidates.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
    }
    families
}

fn resolve_enemy_binding(
    catalog: &catalog::SimulationCatalog,
    families: &BTreeMap<EnemyFamilyKey, Vec<EnemyFamilyCandidate>>,
    stable_key: &str,
    aliases: &mut Vec<EnemyDefinition>,
) -> Result<EnemyBinding, CurrencyWarsDataError> {
    if let Some(definition) = catalog.enemy_by_stable_key(stable_key) {
        return Ok(EnemyBinding {
            definition: definition.id(),
            behavior_source: definition.id(),
            source: CurrencyWarsEnemyBehaviorSource::ExactVariant,
        });
    }
    let (donor, source) = family_behavior(catalog, families, stable_key)?.map_or_else(
        || {
            catalog
                .enemy_by_stable_key(behavior_donor(stable_key))
                .map(|definition| {
                    (
                        definition,
                        CurrencyWarsEnemyBehaviorSource::GenericRankFallbackPolicy,
                    )
                })
                .ok_or_else(|| error("Currency Wars enemy behavior donor is missing"))
        },
        |definition| {
            Ok((
                definition,
                CurrencyWarsEnemyBehaviorSource::SameReleasedFamilyPolicy,
            ))
        },
    )?;
    let alias_id = EnemyDefinitionId::new(
        MODE_ENEMY_BASE
            .checked_add(u32::try_from(aliases.len() + 1).map_err(debug_error)?)
            .ok_or_else(|| error("Currency Wars enemy alias ID overflow"))?,
    )
    .ok_or_else(|| error("Currency Wars enemy alias ID is invalid"))?;
    aliases.push(clone_enemy(donor, alias_id)?);
    Ok(EnemyBinding {
        definition: alias_id,
        behavior_source: donor.id(),
        source,
    })
}

fn family_behavior<'a>(
    catalog: &'a catalog::SimulationCatalog,
    families: &BTreeMap<EnemyFamilyKey, Vec<EnemyFamilyCandidate>>,
    stable_key: &str,
) -> Result<Option<&'a EnemyDefinition>, CurrencyWarsDataError> {
    let Some((key, complete)) = enemy_family(stable_key) else {
        return Ok(None);
    };
    let Some(candidate) = families.get(&key).and_then(|candidates| {
        candidates.iter().min_by_key(|candidate| {
            (
                candidate.complete != complete,
                candidate.stable_key.as_ref(),
            )
        })
    }) else {
        return Ok(None);
    };
    catalog
        .enemy(candidate.definition)
        .map(Some)
        .ok_or_else(|| error("Currency Wars family behavior source is missing"))
}

fn policy_behavior<'a>(
    catalog: &'a catalog::SimulationCatalog,
    families: &BTreeMap<EnemyFamilyKey, Vec<EnemyFamilyCandidate>>,
    family_key: Option<&str>,
    fallback_rank: CurrencyWarsBattleBehaviorFallbackRank,
) -> Result<(&'a EnemyDefinition, CurrencyWarsEnemyBehaviorSource), CurrencyWarsDataError> {
    let family = family_key.and_then(|name| {
        families
            .iter()
            .filter(|(key, _)| {
                key.name.as_ref() == name && policy_rank_matches(&key.rank, fallback_rank)
            })
            .flat_map(|(_, candidates)| candidates)
            .min_by_key(|candidate| (!candidate.complete, candidate.stable_key.as_ref()))
    });
    if let Some(candidate) = family {
        let definition = catalog
            .enemy(candidate.definition)
            .ok_or_else(|| error("Currency Wars policy family behavior source is missing"))?;
        return Ok((
            definition,
            CurrencyWarsEnemyBehaviorSource::SameReleasedFamilyPolicy,
        ));
    }
    let definition = catalog
        .enemy_by_stable_key(policy_behavior_donor(fallback_rank))
        .ok_or_else(|| error("Currency Wars policy behavior donor is missing"))?;
    Ok((
        definition,
        CurrencyWarsEnemyBehaviorSource::GenericRankFallbackPolicy,
    ))
}

fn policy_rank_matches(rank: &str, fallback_rank: CurrencyWarsBattleBehaviorFallbackRank) -> bool {
    match fallback_rank {
        CurrencyWarsBattleBehaviorFallbackRank::Minion => matches!(rank, "minion" | "minionlv2"),
        CurrencyWarsBattleBehaviorFallbackRank::Elite => rank == "elite",
        CurrencyWarsBattleBehaviorFallbackRank::Boss => matches!(rank, "littleboss" | "bigboss"),
    }
}

const fn policy_behavior_donor(rank: CurrencyWarsBattleBehaviorFallbackRank) -> &'static str {
    match rank {
        CurrencyWarsBattleBehaviorFallbackRank::Minion => MINION_DONOR,
        CurrencyWarsBattleBehaviorFallbackRank::Elite => ELITE_DONOR,
        CurrencyWarsBattleBehaviorFallbackRank::Boss => BOSS_DONOR,
    }
}

fn resolved_enemy_combatant(
    catalog: &catalog::SimulationCatalog,
    definition: &EnemyDefinition,
    digest_key: &str,
    level: UnitLevel,
) -> Result<(ResolvedCombatantSpec, UnitLevel), CurrencyWarsDataError> {
    let stats = catalog
        .enemy_runtime_stat(definition.id(), level, STAT_DIFFICULTY)
        .or_else(|| catalog.nearest_enemy_runtime_stat(definition.id(), level, STAT_DIFFICULTY))
        .ok_or_else(|| {
            error(&format!(
                "Currency Wars released enemy runtime stats are missing for {digest_key} at level {}",
                level.get()
            ))
        })?;
    let profile = catalog
        .enemy_runtime_profile(definition.id())
        .ok_or_else(|| error("Currency Wars released enemy profile is missing"))?;
    let hp = stats
        .hp()
        .rounded_integer(Rounding::NearestTiesAway)
        .map_err(debug_error)
        .and_then(|value| Hp::new(value).map_err(debug_error))?;
    let combatant = ResolvedCombatantSpec::new(
        definition.unit(),
        level,
        hp,
        stats.speed(),
        ResolvedDefinitionBindings::new(definition.abilities().to_vec(), Vec::new(), Vec::new())
            .map_err(debug_error)?,
        CombatantSpecDigest::new(enemy_digest(
            catalog.combat_catalog().digest().bytes(),
            digest_key,
            level,
            stats.hp().scaled(),
        ))
        .expect("SHA-256 Currency Wars enemy digest is non-zero"),
    )
    .map_err(debug_error)?
    .with_base_attack_defense(stats.attack(), stats.defense())
    .with_base_effect_stats(stats.effect_hit_rate(), stats.effect_resistance())
    .with_energy(Energy::ZERO, Energy::ZERO)
    .and_then(|value| {
        value.with_toughness(
            profile.rank(),
            profile.weaknesses().to_vec(),
            profile.toughness_layers().to_vec(),
        )
    })
    .map_err(debug_error)?;
    Ok((combatant, stats.level()))
}

fn enemy_family(stable_key: &str) -> Option<(EnemyFamilyKey, bool)> {
    let prefix = stable_key.rsplit_once(".variant.")?.0;
    let segments = prefix.split('.').collect::<Vec<_>>();
    if segments.first().copied() != Some("enemy") {
        return None;
    }
    let rank_index = segments.iter().rposition(|segment| {
        matches!(
            *segment,
            "minion" | "minionlv2" | "elite" | "littleboss" | "bigboss"
        )
    })?;
    if rank_index <= 1 {
        return None;
    }
    let authored_name = segments[1..rank_index].join(".");
    let (name, complete) = authored_name
        .strip_suffix("-complete")
        .map_or((authored_name.as_str(), false), |name| (name, true));
    Some((
        EnemyFamilyKey {
            name: name.into(),
            rank: segments[rank_index].into(),
        },
        complete,
    ))
}

fn behavior_donor(stable_key: &str) -> &'static str {
    if stable_key.contains(".littleboss.") || stable_key.contains(".bigboss.") {
        BOSS_DONOR
    } else if stable_key.contains(".elite.") {
        ELITE_DONOR
    } else {
        MINION_DONOR
    }
}

fn clone_enemy(
    donor: &EnemyDefinition,
    id: EnemyDefinitionId,
) -> Result<EnemyDefinition, CurrencyWarsDataError> {
    let mut definition = EnemyDefinition::new(id, donor.unit(), donor.abilities().to_vec());
    if !donor.links().is_empty() {
        definition = definition
            .with_links(donor.links().to_vec())
            .ok_or_else(|| error("Currency Wars enemy alias links are invalid"))?;
    }
    if let Some(ai) = donor.ai_graph() {
        definition = definition
            .with_orchestration(ai, donor.phases().to_vec())
            .ok_or_else(|| error("Currency Wars enemy alias orchestration is invalid"))?;
    }
    Ok(definition)
}

fn enemy_digest(catalog: [u8; 32], stable_key: &str, level: UnitLevel, hp: i64) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.enemy-input.v1");
    hash.update(catalog);
    hash.update((stable_key.len() as u64).to_le_bytes());
    hash.update(stable_key.as_bytes());
    hash.update([level.get()]);
    hash.update(hp.to_le_bytes());
    hash.finalize().into()
}
