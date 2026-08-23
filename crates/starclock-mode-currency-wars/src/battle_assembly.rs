//! Line-limit exception: selection, immutable assembly identity and cache validation form one boundary owner.
//! Deterministic Currency Wars encounter selection and immutable battle assembly.

mod affix;
pub(super) mod combatant_overlay;
mod participant;
mod resources;
mod scaling;
mod static_contribution;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

use sha2::{Digest, Sha256};
use starclock_combat::{
    ActionValue, AssemblyDigest, Battle, BattleClockSpec, BattleSeed, BattleSpec, ConcedePolicy,
    EncounterWaveId, Energy, FormationIndex, KeyedTeamResourceSpec,
    LethalRescueHpPolicy as CombatLethalRescueHpPolicy, PlayerLethalRescueSpec,
    ResolvedCombatantSpec, SourceDefinitionId, TeamResourceSpec, TeamResourceWavePolicy, UnitLevel,
    catalog::{
        CombatCatalog,
        builder::CombatCatalogBuilder,
        definition::EncounterDefinition,
        encounter::{EncounterWaveDefinition, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy},
    },
};

const ASSIST_USE_RESOURCE_ID: u32 = 0x7da0_0001;
const ASSIST_USE_RESOURCE_KEY: &str = "assist-use";
const ASSIST_USE_RESOURCE_MAXIMUM: u16 = 4;

use crate::{
    CurrencyWarsBattleConfigurationArchetype, CurrencyWarsBattleProgramBinding,
    CurrencyWarsBattleProgramBindingArchetype, CurrencyWarsBondBattleBehaviorArchetype,
    CurrencyWarsBondId, CurrencyWarsBossPool, CurrencyWarsCharacterOverrideBinding,
    CurrencyWarsContributionDigest, CurrencyWarsContributionSnapshot, CurrencyWarsEncounterCatalog,
    CurrencyWarsEncounterGroup, CurrencyWarsEnemyAffixDefinition,
    CurrencyWarsEnemyAffixSelectionSource, CurrencyWarsEnemyAffixSemantic,
    CurrencyWarsEnemyScaling, CurrencyWarsEnemySlot, CurrencyWarsEnemySlotDefinition,
    CurrencyWarsLethalRescueHpPolicy, CurrencyWarsMechanicProgramDisposition, CurrencyWarsNodeKind,
    CurrencyWarsReleasedStage, CurrencyWarsRoleId, ENEMY_AFFIX_SELECTION_POLICY_ID,
    ENEMY_AFFIX_SELECTION_REPLACEMENT_CONDITION, automatic_technique, back_battle_event,
};

use self::{
    affix::{install_reactions, install_static_modifiers},
    participant::{
        CurrencyWarsEnemyParticipantInputs, enemy_participants, player_participants,
        time_assassin_formation,
    },
    static_contribution::install_static_contributions,
};

pub use self::{
    resources::{
        CurrencyWarsAvatarBattleBehaviorProgramInput, CurrencyWarsBattleBehaviorProgramInput,
        CurrencyWarsBattleProgramBindingInput, CurrencyWarsBattleResourceParts,
        CurrencyWarsBattleResources, CurrencyWarsEnemyAiConfigurationInput,
        CurrencyWarsEnemyAiConfigurationRuntimeBinding, CurrencyWarsEnemyBehaviorSource,
        CurrencyWarsEnemyCharacterConfigurationInput,
        CurrencyWarsEnemyCharacterConfigurationRuntimeBinding, CurrencyWarsEnemyCombatInput,
    },
    static_contribution::CurrencyWarsBattleContributionReceipt,
};

const POLICY_ID: &str = "currency-wars.encounter-selection-policy.v1";
const REPLACEMENT_CONDITION: &str =
    "Replace when released evidence defines the node-to-camp and camp-to-stage draw algorithm.";
const ENEMY_BEHAVIOR_POLICY_ID: &str = "currency-wars.enemy-behavior-fallback-policy.v1";
const ENEMY_BEHAVIOR_REPLACEMENT_CONDITION: &str = "Replace each fallback when its released enemy behavior is lowered into the shared combat catalog.";
const ENEMY_STAT_POLICY_ID: &str = "currency-wars.enemy-stat-level-fallback-policy.v1";
const ENEMY_STAT_REPLACEMENT_CONDITION: &str =
    "Replace each nearest-level fallback when an exact released runtime stat row is available.";
const TEAM_RESOURCE_POLICY_ID: &str = "currency-wars.initial-skill-point-policy.v1";
const TEAM_RESOURCE_REPLACEMENT_CONDITION: &str =
    "Replace when released Currency Wars evidence identifies initial and maximum Skill Points.";
const ENEMY_STAR_POLICY_ID: &str = "currency-wars.enemy-star-selection-policy.v1";
const ENEMY_STAR_REPLACEMENT_CONDITION: &str = "Replace when released executable evidence identifies the enemy-star selection input instead of the current Plane 1-3 and Boss mapping.";
const FORMATION_WAVE_POLICY_ID: &str = "currency-wars.formation-wave-selection-policy.v1";
const FORMATION_WAVE_REPLACEMENT_CONDITION: &str = "Replace when released executable evidence identifies a GridFightFormationWave selector other than the released StageConfig wave slot count.";
const ENEMY_ROSTER_POLICY_ID: &str = "currency-wars.camp-enemy-roster-policy.v1";
const ENEMY_ROSTER_REPLACEMENT_CONDITION: &str = "Replace when released executable evidence identifies the exact GridFightCamp MonsterList draw algorithm, including InitialRandomCode and IfRandomEnabled semantics.";
const TIME_ASSASSIN_POLICY_ID: &str = "currency-wars.time-assassin-spawn-policy.v1";
const TIME_ASSASSIN_REPLACEMENT_CONDITION: &str = "Replace the deterministic one-in-four eligible-node draw when released executable evidence identifies the exact Time Assassin spawn probability and placement algorithm.";
const TIME_ASSASSIN_MONSTER_ID: u32 = 4_032_028;
const REGULAR_ENEMY_DEFEAT_ENERGY_SCALED: i64 = 10_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleConfigurationExecution {
    pub source_path: Box<str>,
    pub archetype: CurrencyWarsBattleConfigurationArchetype,
    pub active_binding_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleConfigurationExecutionReceipt {
    executions: Box<[CurrencyWarsBattleConfigurationExecution]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondBattleBehaviorExecution {
    pub source_path: Box<str>,
    pub archetype: CurrencyWarsBondBattleBehaviorArchetype,
    pub bond_ids: Box<[CurrencyWarsBondId]>,
    pub registered_binding_count: u16,
    pub active_binding_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondBattleBehaviorExecutionReceipt {
    executions: Box<[CurrencyWarsBondBattleBehaviorExecution]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleProgramBindingExecution {
    pub source_path: Box<str>,
    pub archetype: CurrencyWarsBattleProgramBindingArchetype,
    pub bindings: Box<[CurrencyWarsBattleProgramBinding]>,
    pub registered_binding_count: u16,
    pub active_binding_count: u16,
    pub runtime_definition_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleProgramBindingExecutionReceipt {
    executions: Box<[CurrencyWarsBattleProgramBindingExecution]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyCharacterConfigurationExecution {
    pub source_path: Box<str>,
    pub registered_binding_count: u16,
    pub active_binding_count: u16,
    pub runtime_definition_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyCharacterConfigurationExecutionReceipt {
    executions: Box<[CurrencyWarsEnemyCharacterConfigurationExecution]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAiConfigurationExecution {
    pub source_path: Box<str>,
    pub registered_binding_count: u16,
    pub active_binding_count: u16,
    pub runtime_definition_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAiConfigurationExecutionReceipt {
    executions: Box<[CurrencyWarsEnemyAiConfigurationExecution]>,
}

impl CurrencyWarsEnemyAiConfigurationExecutionReceipt {
    #[must_use]
    pub fn executions(&self) -> &[CurrencyWarsEnemyAiConfigurationExecution] {
        &self.executions
    }
}

impl CurrencyWarsEnemyCharacterConfigurationExecutionReceipt {
    #[must_use]
    pub fn executions(&self) -> &[CurrencyWarsEnemyCharacterConfigurationExecution] {
        &self.executions
    }
}

impl CurrencyWarsBattleProgramBindingExecutionReceipt {
    #[must_use]
    pub fn executions(&self) -> &[CurrencyWarsBattleProgramBindingExecution] {
        &self.executions
    }
}

impl CurrencyWarsBondBattleBehaviorExecutionReceipt {
    #[must_use]
    pub fn executions(&self) -> &[CurrencyWarsBondBattleBehaviorExecution] {
        &self.executions
    }
}

impl CurrencyWarsBattleConfigurationExecutionReceipt {
    #[must_use]
    pub fn executions(&self) -> &[CurrencyWarsBattleConfigurationExecution] {
        &self.executions
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterSelectionReceipt {
    pub group_source_id: u16,
    pub boss_pool_source_id: Option<u16>,
    pub stage_id: u32,
    pub formation_wave_limits: Box<[u8]>,
    pub selected_monster_ids: Box<[u32]>,
    pub selected_enemy_affix_ids: Box<[u32]>,
    pub enemy_affix_selection_source: CurrencyWarsEnemyAffixSelectionSource,
    pub enemy_star: u8,
    pub enemy_difficulty_level: u16,
    pub fallback_behavior_slots: u16,
    pub fallback_stat_slots: u16,
    pub initial_phase_slots: u16,
    pub multi_phase_slots: u16,
    pub time_assassin_spawned: bool,
}

impl CurrencyWarsEncounterSelectionReceipt {
    #[must_use]
    pub const fn policy_id(&self) -> &'static str {
        POLICY_ID
    }

    #[must_use]
    pub const fn replacement_condition(&self) -> &'static str {
        REPLACEMENT_CONDITION
    }

    #[must_use]
    pub const fn enemy_behavior_policy_id(&self) -> Option<&'static str> {
        if self.fallback_behavior_slots == 0 {
            None
        } else {
            Some(ENEMY_BEHAVIOR_POLICY_ID)
        }
    }

    #[must_use]
    pub const fn enemy_behavior_replacement_condition(&self) -> Option<&'static str> {
        if self.fallback_behavior_slots == 0 {
            None
        } else {
            Some(ENEMY_BEHAVIOR_REPLACEMENT_CONDITION)
        }
    }

    #[must_use]
    pub const fn enemy_stat_policy_id(&self) -> Option<&'static str> {
        if self.fallback_stat_slots == 0 {
            None
        } else {
            Some(ENEMY_STAT_POLICY_ID)
        }
    }

    #[must_use]
    pub const fn enemy_affix_policy_id(&self) -> Option<&'static str> {
        match self.enemy_affix_selection_source {
            CurrencyWarsEnemyAffixSelectionSource::Explicit => None,
            CurrencyWarsEnemyAffixSelectionSource::DeterministicProjectPolicy => {
                Some(ENEMY_AFFIX_SELECTION_POLICY_ID)
            }
        }
    }

    #[must_use]
    pub const fn enemy_affix_replacement_condition(&self) -> Option<&'static str> {
        match self.enemy_affix_selection_source {
            CurrencyWarsEnemyAffixSelectionSource::Explicit => None,
            CurrencyWarsEnemyAffixSelectionSource::DeterministicProjectPolicy => {
                Some(ENEMY_AFFIX_SELECTION_REPLACEMENT_CONDITION)
            }
        }
    }

    #[must_use]
    pub const fn enemy_stat_replacement_condition(&self) -> Option<&'static str> {
        if self.fallback_stat_slots == 0 {
            None
        } else {
            Some(ENEMY_STAT_REPLACEMENT_CONDITION)
        }
    }

    #[must_use]
    pub const fn team_resource_policy_id(&self) -> &'static str {
        TEAM_RESOURCE_POLICY_ID
    }

    #[must_use]
    pub const fn team_resource_replacement_condition(&self) -> &'static str {
        TEAM_RESOURCE_REPLACEMENT_CONDITION
    }

    #[must_use]
    pub const fn enemy_star_policy_id(&self) -> &'static str {
        ENEMY_STAR_POLICY_ID
    }

    #[must_use]
    pub const fn enemy_star_replacement_condition(&self) -> &'static str {
        ENEMY_STAR_REPLACEMENT_CONDITION
    }

    #[must_use]
    pub const fn formation_wave_policy_id(&self) -> &'static str {
        FORMATION_WAVE_POLICY_ID
    }

    #[must_use]
    pub const fn formation_wave_replacement_condition(&self) -> &'static str {
        FORMATION_WAVE_REPLACEMENT_CONDITION
    }

    #[must_use]
    pub const fn enemy_roster_policy_id(&self) -> &'static str {
        ENEMY_ROSTER_POLICY_ID
    }

    #[must_use]
    pub const fn enemy_roster_replacement_condition(&self) -> &'static str {
        ENEMY_ROSTER_REPLACEMENT_CONDITION
    }

    #[must_use]
    pub fn time_assassin_policy_id(&self) -> Option<&'static str> {
        self.selected_enemy_affix_ids
            .contains(&4013)
            .then_some(TIME_ASSASSIN_POLICY_ID)
    }

    #[must_use]
    pub fn time_assassin_replacement_condition(&self) -> Option<&'static str> {
        self.selected_enemy_affix_ids
            .contains(&4013)
            .then_some(TIME_ASSASSIN_REPLACEMENT_CONDITION)
    }
}

#[derive(Clone, Debug)]
pub struct CurrencyWarsBattleMaterialization {
    combat_catalog: Arc<CombatCatalog>,
    battle_spec: BattleSpec,
    contribution_digest: CurrencyWarsContributionDigest,
    contribution_receipt: CurrencyWarsBattleContributionReceipt,
    configuration_execution_receipt: CurrencyWarsBattleConfigurationExecutionReceipt,
    bond_execution_receipt: CurrencyWarsBondBattleBehaviorExecutionReceipt,
    program_binding_execution_receipt: CurrencyWarsBattleProgramBindingExecutionReceipt,
    enemy_character_configuration_execution_receipt:
        CurrencyWarsEnemyCharacterConfigurationExecutionReceipt,
    enemy_ai_configuration_execution_receipt: CurrencyWarsEnemyAiConfigurationExecutionReceipt,
    selection: CurrencyWarsEncounterSelectionReceipt,
}

impl CurrencyWarsBattleMaterialization {
    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }

    #[must_use]
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.battle_spec
    }

    #[must_use]
    pub const fn contribution_digest(&self) -> CurrencyWarsContributionDigest {
        self.contribution_digest
    }

    #[must_use]
    pub const fn contribution_receipt(&self) -> CurrencyWarsBattleContributionReceipt {
        self.contribution_receipt
    }

    #[must_use]
    pub const fn configuration_execution_receipt(
        &self,
    ) -> &CurrencyWarsBattleConfigurationExecutionReceipt {
        &self.configuration_execution_receipt
    }

    #[must_use]
    pub const fn bond_execution_receipt(&self) -> &CurrencyWarsBondBattleBehaviorExecutionReceipt {
        &self.bond_execution_receipt
    }

    #[must_use]
    pub const fn program_binding_execution_receipt(
        &self,
    ) -> &CurrencyWarsBattleProgramBindingExecutionReceipt {
        &self.program_binding_execution_receipt
    }

    #[must_use]
    pub const fn enemy_character_configuration_execution_receipt(
        &self,
    ) -> &CurrencyWarsEnemyCharacterConfigurationExecutionReceipt {
        &self.enemy_character_configuration_execution_receipt
    }

    #[must_use]
    pub const fn enemy_ai_configuration_execution_receipt(
        &self,
    ) -> &CurrencyWarsEnemyAiConfigurationExecutionReceipt {
        &self.enemy_ai_configuration_execution_receipt
    }

    #[must_use]
    pub fn selection(&self) -> CurrencyWarsEncounterSelectionReceipt {
        self.selection.clone()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CurrencyWarsBattleCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
}

#[derive(Debug)]
pub struct CurrencyWarsBattleAssembler {
    resources: Arc<CurrencyWarsBattleResources>,
    capacity: usize,
    cache: BTreeMap<[u8; 32], CurrencyWarsBattleMaterialization>,
    order: VecDeque<[u8; 32]>,
    hits: u64,
    misses: u64,
}

impl CurrencyWarsBattleAssembler {
    pub fn new(
        resources: Arc<CurrencyWarsBattleResources>,
        capacity: usize,
    ) -> Result<Self, CurrencyWarsBattleAssemblyError> {
        if !(1..=256).contains(&capacity) {
            return Err(error("Currency Wars battle cache capacity is invalid"));
        }
        Ok(Self {
            resources,
            capacity,
            cache: BTreeMap::new(),
            order: VecDeque::new(),
            hits: 0,
            misses: 0,
        })
    }

    pub fn materialize(
        &mut self,
        snapshot: &CurrencyWarsContributionSnapshot,
        encounters: &CurrencyWarsEncounterCatalog,
        scaling: CurrencyWarsEnemyScaling,
    ) -> Result<CurrencyWarsBattleMaterialization, CurrencyWarsBattleAssemblyError> {
        validate_scaling(snapshot, scaling)?;
        let selected = select_encounter(snapshot, encounters)?;
        let cache_key = assembly_digest(&self.resources, snapshot, encounters, &selected, scaling);
        if let Some(cached) = self.cache.get(&cache_key) {
            self.hits = self.hits.saturating_add(1);
            return Ok(cached.clone());
        }
        let materialization = assemble(
            &self.resources,
            snapshot,
            encounters,
            selected,
            scaling,
            cache_key,
        )?;
        self.misses = self.misses.saturating_add(1);
        if self.cache.len() == self.capacity {
            let oldest = self
                .order
                .pop_front()
                .expect("non-empty bounded cache has an insertion order");
            self.cache.remove(&oldest);
        }
        self.order.push_back(cache_key);
        self.cache.insert(cache_key, materialization.clone());
        Ok(materialization)
    }

    #[must_use]
    pub fn cache_stats(&self) -> CurrencyWarsBattleCacheStats {
        CurrencyWarsBattleCacheStats {
            hits: self.hits,
            misses: self.misses,
            entries: self.cache.len(),
        }
    }

    #[must_use]
    pub const fn resources(&self) -> &Arc<CurrencyWarsBattleResources> {
        &self.resources
    }
}

struct CurrencyWarsSelectedEncounter<'a> {
    group: &'a CurrencyWarsEncounterGroup,
    boss_pool: Option<&'a CurrencyWarsBossPool>,
    stage: CurrencyWarsReleasedStage,
    enemies: Vec<&'a CurrencyWarsEnemySlot>,
    time_assassin: Option<&'a CurrencyWarsEnemySlot>,
}

fn validate_scaling(
    snapshot: &CurrencyWarsContributionSnapshot,
    scaling: CurrencyWarsEnemyScaling,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    if snapshot.node.plane != scaling.chapter
        || scaling.difficulty_level < snapshot.difficulty.enemy_scaling.enemy_difficulty_level
    {
        return Err(error(
            "Currency Wars enemy scaling does not match the snapshot",
        ));
    }
    Ok(())
}

fn select_encounter<'a>(
    snapshot: &CurrencyWarsContributionSnapshot,
    encounters: &'a CurrencyWarsEncounterCatalog,
) -> Result<CurrencyWarsSelectedEncounter<'a>, CurrencyWarsBattleAssemblyError> {
    let eligible = encounters
        .groups()
        .iter()
        .map(|group| {
            let boss_pool = match snapshot.node.kind {
                CurrencyWarsNodeKind::Boss => group
                    .boss_battle_area_id
                    .and_then(|battle_area_id| encounters.boss_pool(battle_area_id)),
                CurrencyWarsNodeKind::Monster
                | CurrencyWarsNodeKind::CampMonster
                | CurrencyWarsNodeKind::EliteBranch
                | CurrencyWarsNodeKind::Supply => None,
            };
            Ok((
                group,
                boss_pool,
                stage_candidates(snapshot.node.kind, group, boss_pool)?,
            ))
        })
        .filter_map(|result| match result {
            Ok((_, _, candidates)) if candidates.is_empty() => None,
            result => Some(result),
        })
        .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
    let (group, boss_pool, mut candidates) =
        select(&eligible, selection_word(snapshot.digest, b"camp"))
            .cloned()
            .ok_or_else(|| error("Currency Wars encounter group candidate set is empty"))?;
    let desired_tier = snapshot.team_level.level.saturating_sub(1).min(6);
    let tier_candidates = candidates
        .iter()
        .copied()
        .filter(|id| id % 10 == u32::from(desired_tier))
        .collect::<Vec<_>>();
    if !tier_candidates.is_empty() {
        candidates = tier_candidates;
    }
    let stage_id = select(&candidates, selection_word(snapshot.digest, b"stage"))
        .copied()
        .ok_or_else(|| error("Currency Wars released stage candidate set is empty"))?;
    let stage = encounters
        .released_stage(stage_id)
        .cloned()
        .ok_or_else(|| error("Currency Wars selected released stage is missing"))?;
    let enemies = select_enemy_roster(snapshot, encounters, group, boss_pool, &stage)?;
    let time_assassin = if should_spawn_time_assassin(snapshot) {
        Some(
            encounters
                .enemy_slot(TIME_ASSASSIN_MONSTER_ID)
                .ok_or_else(|| error("Currency Wars Time Assassin enemy slot is missing"))?,
        )
    } else {
        None
    };
    Ok(CurrencyWarsSelectedEncounter {
        group,
        boss_pool,
        stage,
        enemies,
        time_assassin,
    })
}

fn should_spawn_time_assassin(snapshot: &CurrencyWarsContributionSnapshot) -> bool {
    let eligible_node = matches!(
        snapshot.node.kind,
        CurrencyWarsNodeKind::Monster
            | CurrencyWarsNodeKind::CampMonster
            | CurrencyWarsNodeKind::EliteBranch
    );
    eligible_node
        && snapshot
            .enemy_affix_behaviors
            .iter()
            .any(|behavior| behavior.semantic == CurrencyWarsEnemyAffixSemantic::TimeAssassin)
        && time_assassin_word(snapshot.digest).is_multiple_of(4)
}

fn time_assassin_word(digest: CurrencyWarsContributionDigest) -> u64 {
    let mut hash = Sha256::new();
    hash.update(TIME_ASSASSIN_POLICY_ID.as_bytes());
    hash.update(digest.bytes());
    let bytes: [u8; 32] = hash.finalize().into();
    u64::from_le_bytes(
        bytes[..8]
            .try_into()
            .expect("SHA-256 prefix has eight bytes"),
    )
}

fn stage_candidates(
    node_kind: CurrencyWarsNodeKind,
    group: &CurrencyWarsEncounterGroup,
    boss_pool: Option<&CurrencyWarsBossPool>,
) -> Result<Vec<u32>, CurrencyWarsBattleAssemblyError> {
    let candidates = match node_kind {
        CurrencyWarsNodeKind::Boss => boss_pool
            .map(|pool| pool.candidate_stage_ids.as_ref())
            .unwrap_or_default(),
        CurrencyWarsNodeKind::Monster
        | CurrencyWarsNodeKind::CampMonster
        | CurrencyWarsNodeKind::EliteBranch
        | CurrencyWarsNodeKind::Supply => &group.candidate_stage_ids,
    };
    candidates
        .iter()
        .map(|id| id.parse::<u32>().map_err(debug_error))
        .filter(|result| match result {
            Err(_) => true,
            Ok(stage_id) => {
                let battle_area_id = stage_id / 100;
                match node_kind {
                    CurrencyWarsNodeKind::Boss => group.boss_battle_area_id == Some(battle_area_id),
                    CurrencyWarsNodeKind::Monster
                    | CurrencyWarsNodeKind::CampMonster
                    | CurrencyWarsNodeKind::EliteBranch => {
                        group.battle_area_ids.contains(&battle_area_id)
                            && group.boss_battle_area_id != Some(battle_area_id)
                    }
                    CurrencyWarsNodeKind::Supply => false,
                }
            }
        })
        .collect()
}

fn select_enemy_roster<'a>(
    snapshot: &CurrencyWarsContributionSnapshot,
    encounters: &'a CurrencyWarsEncounterCatalog,
    group: &CurrencyWarsEncounterGroup,
    boss_pool: Option<&CurrencyWarsBossPool>,
    stage: &CurrencyWarsReleasedStage,
) -> Result<Vec<&'a CurrencyWarsEnemySlot>, CurrencyWarsBattleAssemblyError> {
    let candidate_ids = match boss_pool {
        Some(pool) => pool
            .candidate_monster_ids
            .iter()
            .map(|value| value.parse::<u32>().map_err(debug_error))
            .collect::<Result<Vec<_>, _>>()?,
        None => group.monster_ids.to_vec(),
    };
    let candidates = candidate_ids
        .iter()
        .map(|source_monster_id| {
            encounters
                .enemy_slot(*source_monster_id)
                .ok_or_else(|| error("Currency Wars camp enemy slot is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut selected = Vec::new();
    for (wave_index, wave) in stage.waves.iter().enumerate() {
        let maximum_teammates = u8::try_from(wave.enemies.len()).map_err(debug_error)?;
        encounters
            .formation_wave(maximum_teammates)
            .ok_or_else(|| error("Currency Wars formation wave is missing"))?;
        if wave.enemies.len() > candidates.len() {
            return Err(error(
                "Currency Wars camp enemy pool cannot fill the formation wave",
            ));
        }
        let mut available = candidates.clone();
        for slot_index in 0..wave.enemies.len() {
            let index = if group.randomization.enabled {
                enemy_selection_word(
                    snapshot.digest,
                    group,
                    stage.stage_id,
                    wave_index,
                    slot_index,
                ) as usize
                    % available.len()
            } else {
                (usize::from(group.randomization.initial_code) + slot_index) % available.len()
            };
            selected.push(available.remove(index));
        }
    }
    Ok(selected)
}

fn monster_identity(
    slot: &CurrencyWarsEnemySlot,
) -> Result<(u32, &str), CurrencyWarsBattleAssemblyError> {
    match &slot.definition {
        CurrencyWarsEnemySlotDefinition::Monster {
            source_monster_id,
            shared_enemy_key,
            ..
        } => Ok((*source_monster_id, shared_enemy_key)),
        CurrencyWarsEnemySlotDefinition::EliteScaling { .. } => Err(error(
            "Currency Wars selected enemy slot is an elite scaling definition",
        )),
    }
}

fn enemy_selection_word(
    digest: CurrencyWarsContributionDigest,
    group: &CurrencyWarsEncounterGroup,
    stage: u32,
    wave: usize,
    slot: usize,
) -> u64 {
    let mut hash = Sha256::new();
    hash.update(ENEMY_ROSTER_POLICY_ID.as_bytes());
    hash.update(digest.bytes());
    hash.update(group.source_id.to_le_bytes());
    hash.update([group.randomization.initial_code]);
    hash.update(stage.to_le_bytes());
    hash.update((wave as u64).to_le_bytes());
    hash.update((slot as u64).to_le_bytes());
    let bytes: [u8; 32] = hash.finalize().into();
    u64::from_le_bytes(
        bytes[..8]
            .try_into()
            .expect("SHA-256 prefix has eight bytes"),
    )
}

fn assemble(
    resources: &CurrencyWarsBattleResources,
    snapshot: &CurrencyWarsContributionSnapshot,
    encounters: &CurrencyWarsEncounterCatalog,
    selected: CurrencyWarsSelectedEncounter<'_>,
    scaling: CurrencyWarsEnemyScaling,
    digest: [u8; 32],
) -> Result<CurrencyWarsBattleMaterialization, CurrencyWarsBattleAssemblyError> {
    let CurrencyWarsSelectedEncounter {
        group,
        boss_pool,
        stage,
        enemies: selected_enemies,
        time_assassin,
    } = selected;
    let stage_id = stage.stage_id;
    let level = UnitLevel::new(stage.level)
        .ok_or_else(|| error("Currency Wars released stage level is invalid"))?;
    let effective_difficulty = scaling.difficulty_level;
    let all_selected_enemies = selected_enemies
        .iter()
        .copied()
        .chain(time_assassin)
        .collect::<Vec<_>>();
    let fallback_behavior_slots = all_selected_enemies
        .iter()
        .filter(|enemy| {
            let (_, stable_key) = monster_identity(enemy)
                .expect("selected enemy roster contains only validated monster slots");
            resources
                .enemy(stable_key, level)
                .is_some_and(|input| input.behavior_source.is_fallback())
        })
        .count();
    let fallback_stat_slots = all_selected_enemies
        .iter()
        .filter(|enemy| {
            let (_, stable_key) = monster_identity(enemy)
                .expect("selected enemy roster contains only validated monster slots");
            resources
                .enemy(stable_key, level)
                .is_some_and(|input| input.stat_source_level != level)
        })
        .count();
    let initial_phase_slots = all_selected_enemies
        .iter()
        .filter(|enemy| {
            monster_identity(enemy)
                .ok()
                .and_then(|(_, stable_key)| resources.enemy(stable_key, level))
                .and_then(|input| resources.combat().enemy(input.definition))
                .is_some_and(|definition| !definition.phases().is_empty())
        })
        .count();
    let multi_phase_slots = all_selected_enemies
        .iter()
        .filter(|enemy| {
            monster_identity(enemy)
                .ok()
                .and_then(|(_, stable_key)| resources.enemy(stable_key, level))
                .and_then(|input| resources.combat().enemy(input.definition))
                .is_some_and(|definition| definition.phases().len() > 1)
        })
        .count();
    let encounter = encounter_definition(
        resources,
        snapshot,
        &stage,
        &selected_enemies,
        time_assassin,
        level,
    )?;
    let mut builder = CombatCatalogBuilder::from_catalog(resources.combat(), digest);
    builder.add_encounter(encounter);
    let mut player_combatants =
        automatic_technique::install(&mut builder, resources.combat(), snapshot)?;
    back_battle_event::install(&mut builder, snapshot, &mut player_combatants)?;
    let contribution_receipt =
        install_static_contributions(&mut builder, snapshot, &mut player_combatants)?;
    let affix_overlays = install_static_modifiers(&mut builder, snapshot, &mut player_combatants)?;
    install_reactions(&mut builder, resources, snapshot, &mut player_combatants)?;
    let combat_catalog = builder.build().map_err(debug_error)?;
    let mut participants = player_participants(snapshot, &player_combatants)?;
    let enemy_star = enemy_star(snapshot);
    participants.extend(enemy_participants(CurrencyWarsEnemyParticipantInputs {
        resources,
        encounters,
        snapshot,
        stage: &stage,
        selected_enemies: &selected_enemies,
        time_assassin,
        level,
        scaling,
        enemy_star,
        root_digest: digest,
        affix_overlays: &affix_overlays,
    })?);
    let player_resources = player_resources(&combat_catalog, &player_combatants)?;
    let defeat_energy = snapshot
        .battle_overrides
        .scale_defeat_energy(
            Energy::from_scaled(REGULAR_ENEMY_DEFEAT_ENERGY_SCALED)
                .expect("the released regular enemy-defeat Energy is non-negative"),
        )
        .map_err(debug_error)?;
    let mut battle_spec = BattleSpec::new(
        AssemblyDigest::new(digest).expect("SHA-256 assembly digest is non-zero"),
        snapshot.node.encounter,
        participants,
        player_resources,
        TeamResourceSpec::new(0, 0).expect("empty enemy resource is valid"),
        ConcedePolicy::Allowed,
    )
    .map_err(debug_error)?;
    if let Some(clock) = snapshot.battle_clock {
        battle_spec = battle_spec.with_clock(clock);
    }
    battle_spec = battle_spec
        .with_enemy_defeat_energy(defeat_energy)
        .ok_or_else(|| error("Currency Wars enemy-defeat Energy is invalid"))?;
    if let Some(rescue) = player_lethal_rescue(
        snapshot.battle_clock,
        snapshot.battle_overrides.lethal_rescue_hp_policy,
        snapshot.battle_overrides.lethal_rescue_action_value,
    )? {
        battle_spec = battle_spec
            .with_player_lethal_rescue(rescue)
            .ok_or_else(|| error("Currency Wars lethal-rescue clock binding is invalid"))?;
    }
    Battle::create(
        Arc::clone(&combat_catalog),
        battle_spec.clone(),
        BattleSeed::new([0x43; 32]),
    )
    .map_err(debug_error)?;
    let configuration_execution_receipt = battle_configuration_execution_receipt(
        encounters,
        snapshot,
        &stage,
        all_selected_enemies.len(),
        contribution_receipt,
    )?;
    let bond_execution_receipt = bond_battle_behavior_execution_receipt(encounters, snapshot)?;
    let program_binding_execution_receipt =
        battle_program_binding_execution_receipt(resources, snapshot)?;
    let enemy_character_configuration_execution_receipt =
        enemy_character_configuration_execution_receipt(resources, &all_selected_enemies)?;
    let enemy_ai_configuration_execution_receipt =
        enemy_ai_configuration_execution_receipt(resources, &all_selected_enemies)?;
    Ok(CurrencyWarsBattleMaterialization {
        combat_catalog,
        battle_spec,
        contribution_digest: snapshot.digest,
        contribution_receipt,
        configuration_execution_receipt,
        bond_execution_receipt,
        program_binding_execution_receipt,
        enemy_character_configuration_execution_receipt,
        enemy_ai_configuration_execution_receipt,
        selection: CurrencyWarsEncounterSelectionReceipt {
            group_source_id: group.source_id,
            boss_pool_source_id: boss_pool.map(|pool| pool.source_id),
            stage_id,
            formation_wave_limits: stage
                .waves
                .iter()
                .enumerate()
                .map(|(index, wave)| {
                    u8::try_from(
                        wave.enemies.len() + usize::from(index == 0 && time_assassin.is_some()),
                    )
                    .map_err(debug_error)
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            selected_monster_ids: selected_enemies
                .iter()
                .copied()
                .chain(time_assassin)
                .map(|slot| monster_identity(slot).map(|(source_monster_id, _)| source_monster_id))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            selected_enemy_affix_ids: snapshot
                .enemy_affixes
                .iter()
                .filter_map(|affix| match affix.definition {
                    CurrencyWarsEnemyAffixDefinition::Affix { source_id, .. } => Some(source_id),
                    CurrencyWarsEnemyAffixDefinition::MazeBuff { .. }
                    | CurrencyWarsEnemyAffixDefinition::Scaling(_) => None,
                })
                .collect(),
            enemy_affix_selection_source: snapshot.enemy_affix_selection_source,
            enemy_star,
            enemy_difficulty_level: effective_difficulty,
            fallback_behavior_slots: u16::try_from(fallback_behavior_slots).map_err(debug_error)?,
            fallback_stat_slots: u16::try_from(fallback_stat_slots).map_err(debug_error)?,
            initial_phase_slots: u16::try_from(initial_phase_slots).map_err(debug_error)?,
            multi_phase_slots: u16::try_from(multi_phase_slots).map_err(debug_error)?,
            time_assassin_spawned: time_assassin.is_some(),
        },
    })
}

fn player_lethal_rescue(
    clock: Option<BattleClockSpec>,
    hp_policy: CurrencyWarsLethalRescueHpPolicy,
    action_value_loss: ActionValue,
) -> Result<Option<PlayerLethalRescueSpec>, CurrencyWarsBattleAssemblyError> {
    match clock {
        None => Ok(None),
        Some(BattleClockSpec::ActionValue(_)) => {
            let hp = match hp_policy {
                CurrencyWarsLethalRescueHpPolicy::FullMaximumHp => {
                    CombatLethalRescueHpPolicy::MaximumHp
                }
            };
            PlayerLethalRescueSpec::new(hp, Some(action_value_loss))
                .map(Some)
                .ok_or_else(|| error("Currency Wars lethal-rescue input is invalid"))
        }
        Some(BattleClockSpec::Cycles(_)) => Err(error(
            "Currency Wars lethal rescue requires an Action Value clock",
        )),
    }
}

#[cfg(test)]
mod tests {
    use starclock_combat::{ActionValueClockSpec, BattleClockExpiry};

    use super::*;

    #[test]
    fn lethal_rescue_is_bound_only_to_finite_action_value_battles() {
        let loss = ActionValue::from_scaled(20_000_000).unwrap();
        assert_eq!(
            player_lethal_rescue(None, CurrencyWarsLethalRescueHpPolicy::FullMaximumHp, loss,)
                .unwrap(),
            None,
        );

        let clock = BattleClockSpec::ActionValue(
            ActionValueClockSpec::new(
                ActionValue::from_scaled(100_000_000).unwrap(),
                BattleClockExpiry::Lose,
            )
            .unwrap(),
        );
        let rescue = player_lethal_rescue(
            Some(clock),
            CurrencyWarsLethalRescueHpPolicy::FullMaximumHp,
            loss,
        )
        .unwrap()
        .unwrap();
        assert_eq!(rescue.hp(), CombatLethalRescueHpPolicy::MaximumHp);
        assert_eq!(rescue.action_value_loss(), Some(loss));
    }

    #[test]
    fn front_special_resources_do_not_override_team_skill_points() {
        let resources = TeamResourceSpec::new(3, 5).unwrap();

        assert_eq!(resources.skill_points(), 3);
        assert_eq!(resources.maximum_skill_points(), 5);
    }

    #[test]
    fn assist_uses_are_a_bounded_persistent_team_resource() {
        let resource = supported_team_resource(ASSIST_USE_RESOURCE_KEY).unwrap();

        assert_eq!(
            resource.id(),
            SourceDefinitionId::new(ASSIST_USE_RESOURCE_ID).unwrap()
        );
        assert_eq!(resource.stable_key(), Some(ASSIST_USE_RESOURCE_KEY));
        assert_eq!(resource.initial(), 0);
        assert_eq!(resource.maximum(), ASSIST_USE_RESOURCE_MAXIMUM);
        assert_eq!(resource.wave(), TeamResourceWavePolicy::Persist);
    }
}

fn enemy_character_configuration_execution_receipt(
    resources: &CurrencyWarsBattleResources,
    selected_enemies: &[&CurrencyWarsEnemySlot],
) -> Result<CurrencyWarsEnemyCharacterConfigurationExecutionReceipt, CurrencyWarsBattleAssemblyError>
{
    let active_enemy_keys = selected_enemies
        .iter()
        .map(|enemy| monster_identity(enemy).map(|(_, stable_key)| stable_key))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let executions = resources
        .enemy_character_configurations()
        .iter()
        .map(|input| {
            let registered_binding_count =
                u16::try_from(input.bindings.len()).map_err(debug_error)?;
            let active_binding_count = u16::try_from(
                input
                    .bindings
                    .iter()
                    .filter(|binding| active_enemy_keys.contains(binding.shared_enemy_key.as_ref()))
                    .count(),
            )
            .map_err(debug_error)?;
            Ok(CurrencyWarsEnemyCharacterConfigurationExecution {
                source_path: input.source_path.clone(),
                registered_binding_count,
                active_binding_count,
                runtime_definition_count: registered_binding_count,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
    Ok(CurrencyWarsEnemyCharacterConfigurationExecutionReceipt {
        executions: executions.into_boxed_slice(),
    })
}

fn enemy_ai_configuration_execution_receipt(
    resources: &CurrencyWarsBattleResources,
    selected_enemies: &[&CurrencyWarsEnemySlot],
) -> Result<CurrencyWarsEnemyAiConfigurationExecutionReceipt, CurrencyWarsBattleAssemblyError> {
    let active_enemy_keys = selected_enemies
        .iter()
        .map(|enemy| monster_identity(enemy).map(|(_, stable_key)| stable_key))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let executions = resources
        .enemy_ai_configurations()
        .iter()
        .map(|input| {
            let registered_binding_count =
                u16::try_from(input.bindings.len()).map_err(debug_error)?;
            let active_binding_count = u16::try_from(
                input
                    .bindings
                    .iter()
                    .filter(|binding| active_enemy_keys.contains(binding.shared_enemy_key.as_ref()))
                    .count(),
            )
            .map_err(debug_error)?;
            Ok(CurrencyWarsEnemyAiConfigurationExecution {
                source_path: input.source_path.clone(),
                registered_binding_count,
                active_binding_count,
                runtime_definition_count: registered_binding_count,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
    Ok(CurrencyWarsEnemyAiConfigurationExecutionReceipt {
        executions: executions.into_boxed_slice(),
    })
}

fn battle_program_binding_execution_receipt(
    resources: &CurrencyWarsBattleResources,
    snapshot: &CurrencyWarsContributionSnapshot,
) -> Result<CurrencyWarsBattleProgramBindingExecutionReceipt, CurrencyWarsBattleAssemblyError> {
    let executions = resources
        .battle_program_bindings()
        .iter()
        .map(|input| {
            let registered_binding_count =
                u16::try_from(input.bindings.len()).map_err(debug_error)?;
            let active_binding_count = u16::try_from(
                input
                    .bindings
                    .iter()
                    .filter(|binding| battle_program_binding_is_active(**binding, input, snapshot))
                    .count(),
            )
            .map_err(debug_error)?;
            Ok(CurrencyWarsBattleProgramBindingExecution {
                source_path: input.source_path.clone(),
                archetype: input.archetype,
                bindings: input.bindings.clone(),
                registered_binding_count,
                active_binding_count,
                runtime_definition_count: input.runtime_definition_count,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
    Ok(CurrencyWarsBattleProgramBindingExecutionReceipt {
        executions: executions.into_boxed_slice(),
    })
}

fn battle_program_binding_is_active(
    binding: CurrencyWarsBattleProgramBinding,
    input: &CurrencyWarsBattleProgramBindingInput,
    snapshot: &CurrencyWarsContributionSnapshot,
) -> bool {
    match binding {
        CurrencyWarsBattleProgramBinding::Role(id) => snapshot
            .roles
            .iter()
            .any(|role| role.role_state.role() == id),
        CurrencyWarsBattleProgramBinding::Avatar(id) => {
            snapshot.roles.iter().any(|role| role.role.avatar_id == id)
        }
        CurrencyWarsBattleProgramBinding::Servant(_) => input.bindings.iter().any(|candidate| {
            matches!(
                candidate,
                CurrencyWarsBattleProgramBinding::Role(id)
                    if snapshot.roles.iter().any(|role| role.role_state.role() == *id)
            )
        }),
        CurrencyWarsBattleProgramBinding::BattleEvent(id) => {
            snapshot
                .battle_overrides
                .back_battle_events
                .iter()
                .any(|event| event.event_id == id)
                || snapshot
                    .battle_overrides
                    .external_battle_event_ids
                    .contains(&id)
                || snapshot
                    .battle_overrides
                    .summon_battle_event_overrides
                    .iter()
                    .any(|override_| override_.battle_event_id == id)
                || snapshot
                    .summon_battle_event_overrides
                    .iter()
                    .any(|program| {
                        program.bindings.iter().any(|binding| {
                            matches!(binding,
                            CurrencyWarsCharacterOverrideBinding::SummonBattleEvent {
                                unit_id,
                                ..
                            } if *unit_id == id)
                        })
                    })
        }
        CurrencyWarsBattleProgramBinding::Bond(id) => {
            snapshot.bonds.active_bonds.iter().any(|bond| bond.id == id)
        }
        CurrencyWarsBattleProgramBinding::AugmentMazeBuff(id) => snapshot
            .augment_maze_buffs
            .iter()
            .any(|maze_buff| maze_buff.source_id == id),
        CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(id) => {
            snapshot.enemy_affixes.iter().any(|affix| {
                matches!(
                    affix.definition,
                    CurrencyWarsEnemyAffixDefinition::MazeBuff { source_id, .. }
                        if source_id == id
                )
            })
        }
        CurrencyWarsBattleProgramBinding::Equipment(id) => snapshot
            .roles
            .iter()
            .flat_map(|role| role.equipment.iter())
            .any(|equipment| equipment.runtime.id == id),
    }
}

fn bond_battle_behavior_execution_receipt(
    encounters: &CurrencyWarsEncounterCatalog,
    snapshot: &CurrencyWarsContributionSnapshot,
) -> Result<CurrencyWarsBondBattleBehaviorExecutionReceipt, CurrencyWarsBattleAssemblyError> {
    let executions = encounters
        .mechanic_programs()
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBondBattlePolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .map(|(program, policy)| {
            let registered_binding_count =
                u16::try_from(policy.bond_ids.len()).map_err(debug_error)?;
            let active_binding_count = u16::try_from(
                policy
                    .bond_ids
                    .iter()
                    .filter(|bond| {
                        snapshot
                            .bonds
                            .active_bonds
                            .iter()
                            .any(|active| active.id == **bond)
                    })
                    .count(),
            )
            .map_err(debug_error)?;
            Ok(CurrencyWarsBondBattleBehaviorExecution {
                source_path: program.source_path.clone(),
                archetype: policy.archetype,
                bond_ids: policy.bond_ids.clone(),
                registered_binding_count,
                active_binding_count,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
    Ok(CurrencyWarsBondBattleBehaviorExecutionReceipt {
        executions: executions.into_boxed_slice(),
    })
}

fn battle_configuration_execution_receipt(
    encounters: &CurrencyWarsEncounterCatalog,
    snapshot: &CurrencyWarsContributionSnapshot,
    stage: &CurrencyWarsReleasedStage,
    selected_enemy_count: usize,
    contribution: CurrencyWarsBattleContributionReceipt,
) -> Result<CurrencyWarsBattleConfigurationExecutionReceipt, CurrencyWarsBattleAssemblyError> {
    let executions = encounters
        .mechanic_programs()
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleConfigurationPolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .map(|(program, policy)| {
            let active_binding_count = match policy.archetype {
                CurrencyWarsBattleConfigurationArchetype::CommonBattleKernel => 1,
                CurrencyWarsBattleConfigurationArchetype::SharedModifierDefinitions => {
                    usize::from(contribution.modifier_binding_count > 0)
                }
                CurrencyWarsBattleConfigurationArchetype::MonsterTagController => {
                    selected_enemy_count + snapshot.enemy_affix_behaviors.len()
                }
                CurrencyWarsBattleConfigurationArchetype::CharacterController => {
                    usize::from(contribution.front_role_count)
                }
                CurrencyWarsBattleConfigurationArchetype::MonsterController => selected_enemy_count,
                CurrencyWarsBattleConfigurationArchetype::StageController => stage.waves.len(),
                CurrencyWarsBattleConfigurationArchetype::SeasonController => {
                    1 + snapshot.season_talents.len()
                }
                CurrencyWarsBattleConfigurationArchetype::CurrentEquipmentController => snapshot
                    .roles
                    .iter()
                    .flat_map(|role| role.equipment.iter())
                    .filter(|equipment| {
                        equipment
                            .runtime
                            .ability_name
                            .as_deref()
                            .is_some_and(|ability| {
                                policy
                                    .ability_names
                                    .iter()
                                    .any(|candidate| candidate.as_ref() == ability)
                            })
                    })
                    .count(),
            };
            Ok(CurrencyWarsBattleConfigurationExecution {
                source_path: program.source_path.clone(),
                archetype: policy.archetype,
                active_binding_count: u16::try_from(active_binding_count).map_err(debug_error)?,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
    Ok(CurrencyWarsBattleConfigurationExecutionReceipt {
        executions: executions.into_boxed_slice(),
    })
}

fn player_resources(
    catalog: &CombatCatalog,
    combatants: &BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<TeamResourceSpec, CurrencyWarsBattleAssemblyError> {
    let referenced = combatants
        .values()
        .flat_map(ResolvedCombatantSpec::abilities)
        .filter_map(|ability| catalog.ability(*ability))
        .filter_map(|ability| ability.action())
        .flat_map(|action| action.resources().team_resource_costs())
        .map(|cost| cost.stable_key())
        .collect::<BTreeSet<_>>();
    let keyed = referenced
        .into_iter()
        .map(supported_team_resource)
        .collect::<Result<Vec<_>, _>>()?;
    TeamResourceSpec::new(3, 5)
        .and_then(|resources| resources.with_keyed(keyed))
        .ok_or_else(|| error("Currency Wars player Skill Points are invalid"))
}

fn supported_team_resource(
    stable_key: &str,
) -> Result<KeyedTeamResourceSpec, CurrencyWarsBattleAssemblyError> {
    match stable_key {
        ASSIST_USE_RESOURCE_KEY => KeyedTeamResourceSpec::new(
            SourceDefinitionId::new(ASSIST_USE_RESOURCE_ID)
                .expect("the mode-owned Assist-use resource ID is non-zero"),
            0,
            ASSIST_USE_RESOURCE_MAXIMUM,
            TeamResourceWavePolicy::Persist,
        )
        .and_then(|resource| resource.with_stable_key(ASSIST_USE_RESOURCE_KEY))
        .ok_or_else(|| error("Currency Wars Assist-use resource definition is invalid")),
        _ => Err(error(
            "Currency Wars selected combatant references an unsupported team resource",
        )),
    }
}

fn encounter_definition(
    resources: &CurrencyWarsBattleResources,
    snapshot: &CurrencyWarsContributionSnapshot,
    stage: &CurrencyWarsReleasedStage,
    selected_enemies: &[&CurrencyWarsEnemySlot],
    time_assassin: Option<&CurrencyWarsEnemySlot>,
    level: UnitLevel,
) -> Result<EncounterDefinition, CurrencyWarsBattleAssemblyError> {
    let mut selected_enemies = selected_enemies.iter();
    let waves = stage
        .waves
        .iter()
        .enumerate()
        .map(|(wave_index, wave)| {
            let mut slots = wave
                .enemies
                .iter()
                .enumerate()
                .map(|(slot_index, enemy)| {
                    let selected = selected_enemies
                        .next()
                        .ok_or_else(|| error("Currency Wars selected enemy roster is truncated"))?;
                    let (_, stable_key) = monster_identity(selected)?;
                    let input = resources
                        .enemy(stable_key, level)
                        .ok_or_else(|| error("Currency Wars enemy combat input is missing"))?;
                    let initial_phase = resources
                        .combat()
                        .enemy(input.definition)
                        .and_then(|enemy| enemy.phases().first())
                        .map(starclock_combat::catalog::encounter::EnemyPhaseDefinition::id);
                    WaveSlotDefinition::new(
                        u16::try_from(slot_index + 1).map_err(debug_error)?,
                        FormationIndex::new(enemy.formation)
                            .ok_or_else(|| error("Currency Wars enemy formation is invalid"))?,
                        input.definition,
                        Some(stage.level),
                        initial_phase,
                        true,
                    )
                    .ok_or_else(|| error("Currency Wars wave slot is invalid"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if wave_index == 0
                && let Some(selected) = time_assassin
            {
                let (_, stable_key) = monster_identity(selected)?;
                let input = resources
                    .enemy(stable_key, level)
                    .ok_or_else(|| error("Currency Wars Time Assassin combat input is missing"))?;
                let initial_phase = resources
                    .combat()
                    .enemy(input.definition)
                    .and_then(|enemy| enemy.phases().first())
                    .map(starclock_combat::catalog::encounter::EnemyPhaseDefinition::id);
                slots.push(
                    WaveSlotDefinition::new(
                        u16::try_from(slots.len() + 1).map_err(debug_error)?,
                        time_assassin_formation(wave)?,
                        input.definition,
                        Some(stage.level),
                        initial_phase,
                        true,
                    )
                    .ok_or_else(|| error("Currency Wars Time Assassin wave slot is invalid"))?,
                );
            }
            EncounterWaveDefinition::new(
                EncounterWaveId::new(
                    snapshot
                        .node
                        .encounter
                        .get()
                        .checked_add(u32::try_from(wave_index + 1).map_err(debug_error)?)
                        .ok_or_else(|| error("Currency Wars wave ID overflow"))?,
                )
                .ok_or_else(|| error("Currency Wars wave ID is invalid"))?,
                u16::try_from(wave_index + 1).map_err(debug_error)?,
                None,
                None,
                WaveCarry::CARRY_ALL,
                slots,
            )
            .ok_or_else(|| error("Currency Wars encounter wave is invalid"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    EncounterDefinition::new(snapshot.node.encounter, Vec::new(), Vec::new())
        .with_authored_waves(WaveTransitionPolicy::AfterAction, waves)
        .ok_or_else(|| error("Currency Wars encounter definition is invalid"))
}

const fn enemy_star(snapshot: &CurrencyWarsContributionSnapshot) -> u8 {
    match snapshot.node.kind {
        CurrencyWarsNodeKind::Boss => 4,
        CurrencyWarsNodeKind::Monster
        | CurrencyWarsNodeKind::CampMonster
        | CurrencyWarsNodeKind::EliteBranch
        | CurrencyWarsNodeKind::Supply => snapshot.node.plane,
    }
}

fn select<T>(values: &[T], word: u64) -> Option<&T> {
    (!values.is_empty()).then(|| &values[word as usize % values.len()])
}

fn selection_word(digest: CurrencyWarsContributionDigest, label: &[u8]) -> u64 {
    let mut hash = Sha256::new();
    hash.update(POLICY_ID.as_bytes());
    hash.update(label);
    hash.update(digest.bytes());
    let bytes: [u8; 32] = hash.finalize().into();
    u64::from_le_bytes(
        bytes[..8]
            .try_into()
            .expect("SHA-256 prefix has eight bytes"),
    )
}

fn assembly_digest(
    resources: &CurrencyWarsBattleResources,
    snapshot: &CurrencyWarsContributionSnapshot,
    encounters: &CurrencyWarsEncounterCatalog,
    selected: &CurrencyWarsSelectedEncounter<'_>,
    scaling: CurrencyWarsEnemyScaling,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.battle-assembly.v1");
    hash.update(resources.digest());
    hash.update(snapshot.digest.bytes());
    hash.update(ENEMY_STAR_POLICY_ID.as_bytes());
    let star = enemy_star(snapshot);
    hash.update([star]);
    hash.update(FORMATION_WAVE_POLICY_ID.as_bytes());
    hash.update(ENEMY_ROSTER_POLICY_ID.as_bytes());
    hash.update(selected.group.source_id.to_le_bytes());
    hash.update(
        selected
            .boss_pool
            .map_or(0_u16, |pool| pool.source_id)
            .to_le_bytes(),
    );
    hash.update(selected.stage.stage_id.to_le_bytes());
    hash.update([selected.stage.level]);
    for wave in &selected.stage.waves {
        hash.update((wave.enemies.len() as u64).to_le_bytes());
    }
    for (enemy, selected_enemy) in selected
        .stage
        .waves
        .iter()
        .flat_map(|wave| wave.enemies.iter())
        .zip(selected.enemies.iter())
    {
        hash.update([enemy.formation]);
        let (source_monster_id, stable_key) = monster_identity(selected_enemy)
            .expect("selected enemy roster contains only validated monster slots");
        hash.update(source_monster_id.to_le_bytes());
        hash.update((stable_key.len() as u64).to_le_bytes());
        hash.update(stable_key.as_bytes());
        if let Some(ratios) = encounters.enemy_star_scaling(source_monster_id, star) {
            hash.update(ratios.hp.scaled().to_le_bytes());
            hash.update(ratios.attack.scaled().to_le_bytes());
            hash.update(ratios.defense.scaled().to_le_bytes());
            hash.update(ratios.speed.scaled().to_le_bytes());
            hash.update(ratios.stance.scaled().to_le_bytes());
        }
    }
    hash.update(TIME_ASSASSIN_POLICY_ID.as_bytes());
    hash.update([u8::from(selected.time_assassin.is_some())]);
    if let Some(time_assassin) = selected.time_assassin {
        let (source_monster_id, stable_key) = monster_identity(time_assassin)
            .expect("selected Time Assassin contains a validated monster slot");
        hash.update(source_monster_id.to_le_bytes());
        hash.update((stable_key.len() as u64).to_le_bytes());
        hash.update(stable_key.as_bytes());
        if let Some(ratios) = encounters.enemy_star_scaling(source_monster_id, star) {
            hash.update(ratios.hp.scaled().to_le_bytes());
            hash.update(ratios.attack.scaled().to_le_bytes());
            hash.update(ratios.defense.scaled().to_le_bytes());
            hash.update(ratios.speed.scaled().to_le_bytes());
            hash.update(ratios.stance.scaled().to_le_bytes());
        }
    }
    hash.update([scaling.chapter]);
    hash.update(scaling.difficulty_level.to_le_bytes());
    hash.update(scaling.hp_ratio.scaled().to_le_bytes());
    hash.update(scaling.attack_ratio.scaled().to_le_bytes());
    hash.update(scaling.defense_ratio.scaled().to_le_bytes());
    hash.update(scaling.speed_ratio.scaled().to_le_bytes());
    hash.update(scaling.stance_ratio.scaled().to_le_bytes());
    hash.finalize().into()
}

fn combatant_digest(root: [u8; 32], wave: usize, slot: usize) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.enemy-combatant.v1");
    hash.update(root);
    hash.update((wave as u64).to_le_bytes());
    hash.update((slot as u64).to_le_bytes());
    hash.finalize().into()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleAssemblyError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsBattleAssemblyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsBattleAssemblyError {}

pub(super) fn error(message: &str) -> CurrencyWarsBattleAssemblyError {
    CurrencyWarsBattleAssemblyError {
        message: message.into(),
    }
}

pub(super) fn debug_error(value: impl std::fmt::Debug) -> CurrencyWarsBattleAssemblyError {
    CurrencyWarsBattleAssemblyError {
        message: format!("{value:?}").into(),
    }
}
