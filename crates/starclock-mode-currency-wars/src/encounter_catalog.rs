//! Line-limit exception: released encounter joins and the 1,847-program exact-once validation pass stay adjacent.
use std::collections::BTreeSet;

use starclock_combat::{Ratio, Scalar};

use crate::{
    CurrencyWarsBondId, CurrencyWarsCharacterOverrideProgram, CurrencyWarsComplexAiGlobalFactors,
    CurrencyWarsEnemyAffixBehavior, CurrencyWarsEquipmentId,
    CurrencyWarsGlobalTaskTemplateDefinition, CurrencyWarsGlobalTaskTemplateLibrary,
    CurrencyWarsProgressionProgram, CurrencyWarsRoleId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterGroup {
    pub stable_key: Box<str>,
    pub source_id: u16,
    pub plane_id: Box<str>,
    pub difficulty_id: Box<str>,
    pub rank: Box<str>,
    pub candidate_stage_ids: Box<[Box<str>]>,
    pub monster_ids: Box<[u32]>,
    pub battle_area_ids: Box<[u32]>,
    pub boss_battle_area_id: Option<u32>,
    pub randomization: CurrencyWarsEncounterRandomization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterRandomization {
    pub initial_code: u8,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsReleasedStageEnemy {
    pub formation: u8,
    pub source_monster_id: u32,
    pub shared_enemy_key: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsReleasedStageWave {
    pub enemies: Box<[CurrencyWarsReleasedStageEnemy]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsReleasedStage {
    pub stage_id: u32,
    pub stage_type: Box<str>,
    pub level: u8,
    pub elite_group: Option<u32>,
    pub stage_abilities_json: Box<str>,
    pub waves: Box<[CurrencyWarsReleasedStageWave]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterSourceObligation {
    pub stable_key: Box<str>,
    pub parent_kind: Box<str>,
    pub parent_id: Box<str>,
    pub resolution_state: Box<str>,
    pub camp_ids: Box<[u16]>,
    pub stage: Option<CurrencyWarsReleasedStage>,
    pub replacement_condition: Option<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterWave {
    pub stable_key: Box<str>,
    pub wave_index: u16,
    pub maximum_teammates: u8,
    pub ability: Option<Box<str>>,
    pub parameters: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemySlot {
    pub stable_key: Box<str>,
    pub definition: CurrencyWarsEnemySlotDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsEnemySlotDefinition {
    EliteScaling {
        group: u16,
        ratios: CurrencyWarsEnemyStatRatios,
    },
    Monster {
        source_monster_id: u32,
        tier: Option<u8>,
        star_scaling_groups: [u16; 4],
        shared_enemy_key: Box<str>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyStatRatios {
    pub hp: Ratio,
    pub attack: Ratio,
    pub defense: Ratio,
    pub speed: Ratio,
    pub stance: Ratio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAffix {
    pub stable_key: Box<str>,
    pub definition: CurrencyWarsEnemyAffixDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsEnemyAffixDefinition {
    Affix {
        source_id: u32,
        maze_buff_ids: Box<[u32]>,
        config_path: Box<str>,
        parameters: Box<[Scalar]>,
    },
    MazeBuff {
        source_id: u32,
        modifier: Box<str>,
        binding_type: CurrencyWarsEnemyAffixBindingType,
        binding_key: Box<str>,
        level: u8,
        maximum_level: u8,
        parameters: Box<[Scalar]>,
    },
    Scaling(CurrencyWarsEnemyScaling),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyWarsEnemyAffixBindingType {
    BeforeCharacterBorn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyScaling {
    pub chapter: u8,
    pub difficulty_level: u16,
    pub hp_ratio: Ratio,
    pub attack_ratio: Ratio,
    pub defense_ratio: Ratio,
    pub speed_ratio: Ratio,
    pub stance_ratio: Ratio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBossPool {
    pub stable_key: Box<str>,
    pub source_id: u16,
    pub plane_id: Box<str>,
    pub difficulty_id: Box<str>,
    pub candidate_monster_ids: Box<[Box<str>]>,
    pub selection_policy: Box<str>,
    pub boss_battle_area_id: u32,
    pub candidate_stage_ids: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsMechanicScope {
    CrossBattleActivity,
    BattleVisibleOrBattleBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicShapeCount {
    pub shape: Box<str>,
    pub count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicPresentationAudit {
    pub reason: CurrencyWarsMechanicPresentationKind,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub operation_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub tutorial_keys: Box<[Box<str>]>,
    pub custom_time_types: Box<[Box<str>]>,
    pub player_action_types: Box<[Box<str>]>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicLayoutAudit {
    pub root_keys: Box<[Box<str>]>,
    pub descriptor_entry_count: u32,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicUnreachableCharacterOverrideAudit {
    pub configuration_kind: Box<str>,
    pub parent_config_path: Box<str>,
    pub ability_count: u32,
    pub skill_count: u32,
    pub dynamic_source_count: u32,
    pub mechanical_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicUnreachableBattleConfigurationAudit {
    pub reason: Box<str>,
    pub ability_names: Box<[Box<str>]>,
    pub global_modifier_names: Box<[Box<str>]>,
    pub callback_event_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicEmptyConfigurationAudit {
    pub reason: Box<str>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsEnemyCharacterConfigurationBinding {
    pub shared_enemy_key: Box<str>,
    pub source_template_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyCharacterConfiguration {
    pub bindings: Box<[CurrencyWarsEnemyCharacterConfigurationBinding]>,
    pub ability_names: Box<[Box<str>]>,
    pub skill_names: Box<[Box<str>]>,
    pub skill_ability_count: u32,
    pub dynamic_source_count: u32,
    pub mechanical_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsEnemyAiConfigurationBinding {
    pub shared_enemy_key: Box<str>,
    pub source_template_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAiConfiguration {
    pub ai_name: Box<str>,
    pub bindings: Box<[CurrencyWarsEnemyAiConfigurationBinding]>,
    pub variable_names: Box<[Box<str>]>,
    pub decision_names: Box<[Box<str>]>,
    pub skill_names: Box<[Box<str>]>,
    pub node_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub mechanical_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicRolePresentationAudit {
    pub reason: Box<str>,
    pub record_key: Box<str>,
    pub text_hash: Box<str>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicStructuredPresentationAudit {
    pub reason: Box<str>,
    pub record_key: Box<str>,
    pub root_keys: Box<[Box<str>]>,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub descriptor_entry_count: u32,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsMechanicMetadataAudit {
    Presentation(CurrencyWarsMechanicPresentationAudit),
    LayoutDescriptor(CurrencyWarsMechanicLayoutAudit),
    RolePresentation(CurrencyWarsMechanicRolePresentationAudit),
    StructuredPresentation(CurrencyWarsMechanicStructuredPresentationAudit),
    UnreachableCharacterOverride(CurrencyWarsMechanicUnreachableCharacterOverrideAudit),
    UnreachableBattleConfiguration(CurrencyWarsMechanicUnreachableBattleConfigurationAudit),
    EmptyConfiguration(CurrencyWarsMechanicEmptyConfigurationAudit),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsMechanicActivityProgram {
    Progression(CurrencyWarsProgressionProgram),
    CharacterOverride(CurrencyWarsCharacterOverrideProgram),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsBattleBehaviorArchetype {
    BossPhaseController,
    MultiPhaseEnemy,
    PartnerAssist,
    MechanicalTrait,
    ShieldAndResourceTrait,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsBattleBehaviorFallbackRank {
    Minion,
    Elite,
    Boss,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleBehaviorPolicy {
    pub policy_id: Box<str>,
    pub archetype: CurrencyWarsBattleBehaviorArchetype,
    pub family_key: Option<Box<str>>,
    pub fallback_rank: CurrencyWarsBattleBehaviorFallbackRank,
    pub ability_names: Box<[Box<str>]>,
    pub global_modifier_names: Box<[Box<str>]>,
    pub callback_event_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub selected_behavior: Box<str>,
    pub unresolved_field: Box<str>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsAvatarBattleBehaviorArchetype {
    RoleBattleEvent,
    AugmentBattleEvent,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsAvatarBattleBehaviorBindingPolicy {
    ExactBattleEvent,
    SameFamilyBattleEventFallback,
    TypedAugmentController,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAvatarBattleBehaviorPolicy {
    pub policy_id: Box<str>,
    pub archetype: CurrencyWarsAvatarBattleBehaviorArchetype,
    pub binding_policy: CurrencyWarsAvatarBattleBehaviorBindingPolicy,
    pub role_ids: Box<[CurrencyWarsRoleId]>,
    pub avatar_ids: Box<[u32]>,
    pub battle_event_ids: Box<[u32]>,
    pub ability_names: Box<[Box<str>]>,
    pub global_modifier_names: Box<[Box<str>]>,
    pub callback_event_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub selected_behavior: Box<str>,
    pub unresolved_field: Box<str>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsBattleConfigurationArchetype {
    CommonBattleKernel,
    SharedModifierDefinitions,
    MonsterTagController,
    CharacterController,
    MonsterController,
    StageController,
    SeasonController,
    CurrentEquipmentController,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleConfigurationPolicy {
    pub policy_id: Box<str>,
    pub archetype: CurrencyWarsBattleConfigurationArchetype,
    pub ability_names: Box<[Box<str>]>,
    pub global_modifier_names: Box<[Box<str>]>,
    pub callback_event_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub selected_behavior: Box<str>,
    pub unresolved_field: Box<str>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsBondBattleBehaviorArchetype {
    BondStageAbilityController,
    MultiBondStageAbilityController,
    WolfHuntSummonController,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBondBattleBehaviorPolicy {
    pub policy_id: Box<str>,
    pub archetype: CurrencyWarsBondBattleBehaviorArchetype,
    pub bond_ids: Box<[CurrencyWarsBondId]>,
    pub ability_names: Box<[Box<str>]>,
    pub global_modifier_names: Box<[Box<str>]>,
    pub callback_event_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub selected_behavior: Box<str>,
    pub unresolved_field: Box<str>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsBattleProgramBindingArchetype {
    CoreAvatarAbility,
    ServantAbility,
    RoleBattleEvent,
    BondStageAbility,
    AugmentStageAbility,
    MonsterTagController,
    EquipmentController,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBattleProgramBinding {
    Role(CurrencyWarsRoleId),
    Avatar(u32),
    Servant(u32),
    BattleEvent(u32),
    Bond(CurrencyWarsBondId),
    AugmentMazeBuff(u32),
    EnemyAffixMazeBuff(u32),
    Equipment(CurrencyWarsEquipmentId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleProgramBindingPolicy {
    pub policy_id: Box<str>,
    pub archetype: CurrencyWarsBattleProgramBindingArchetype,
    pub bindings: Box<[CurrencyWarsBattleProgramBinding]>,
    pub ability_names: Box<[Box<str>]>,
    pub global_modifier_names: Box<[Box<str>]>,
    pub callback_event_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub configuration_type_counts: Box<[CurrencyWarsMechanicShapeCount]>,
    pub selected_behavior: Box<str>,
    pub unresolved_field: Box<str>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
    pub ordered_shape_sha256: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsMechanicPresentationKind {
    TutorialAndInputGuidance,
    WorldPropAndUiEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsMechanicProgramDisposition {
    PendingExactSource { ordered_operations_json: Box<str> },
    MetadataOnly(CurrencyWarsMechanicMetadataAudit),
    ExecutedActivity(CurrencyWarsMechanicActivityProgram),
    ExecutedBattlePolicy(CurrencyWarsBattleBehaviorPolicy),
    ExecutedAvatarBattlePolicy(CurrencyWarsAvatarBattleBehaviorPolicy),
    ExecutedBattleConfigurationPolicy(CurrencyWarsBattleConfigurationPolicy),
    ExecutedBondBattlePolicy(CurrencyWarsBondBattleBehaviorPolicy),
    ExecutedBattleProgramBindingPolicy(CurrencyWarsBattleProgramBindingPolicy),
    ExecutedEnemyCharacterConfiguration(CurrencyWarsEnemyCharacterConfiguration),
    ExecutedEnemyAiConfiguration(CurrencyWarsEnemyAiConfiguration),
    ExecutedComplexAiGlobalFactors(CurrencyWarsComplexAiGlobalFactors),
    ExecutedGlobalTaskTemplates(CurrencyWarsGlobalTaskTemplateLibrary),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMechanicProgram {
    pub stable_key: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub mechanic_family: Box<str>,
    pub scope: CurrencyWarsMechanicScope,
    pub trigger: Box<str>,
    pub state_lifecycle: Box<str>,
    pub disposition: CurrencyWarsMechanicProgramDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterCatalog {
    pub groups: Box<[CurrencyWarsEncounterGroup]>,
    pub source_obligations: Box<[CurrencyWarsEncounterSourceObligation]>,
    pub waves: Box<[CurrencyWarsEncounterWave]>,
    pub enemy_slots: Box<[CurrencyWarsEnemySlot]>,
    pub enemy_affixes: Box<[CurrencyWarsEnemyAffix]>,
    pub boss_pools: Box<[CurrencyWarsBossPool]>,
    pub mechanic_programs: Box<[CurrencyWarsMechanicProgram]>,
}

impl CurrencyWarsEncounterCatalog {
    pub fn new(
        mut parts: CurrencyWarsEncounterCatalogParts,
    ) -> Result<Self, CurrencyWarsEncounterCatalogError> {
        parts.groups.sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        parts
            .source_obligations
            .sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        parts.waves.sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        parts
            .enemy_slots
            .sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        parts
            .enemy_affixes
            .sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        parts
            .boss_pools
            .sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        parts
            .mechanic_programs
            .sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        let mut keys = BTreeSet::new();
        let stage_ids = parts
            .source_obligations
            .iter()
            .filter_map(|obligation| obligation.stage.as_ref().map(|stage| stage.stage_id))
            .collect::<BTreeSet<_>>();
        if parts.groups.is_empty()
            || parts.source_obligations.is_empty()
            || !valid_formation_waves(&parts.waves)
            || parts.enemy_slots.is_empty()
            || parts.enemy_affixes.is_empty()
            || parts.boss_pools.is_empty()
            || parts.mechanic_programs.is_empty()
            || stage_ids.is_empty()
            || parts.source_obligations.iter().any(|obligation| {
                obligation.stage.as_ref().is_some_and(|stage| {
                    stage.waves.is_empty() || stage.waves.iter().any(|wave| wave.enemies.is_empty())
                })
            })
            || parts.groups.iter().any(|group| {
                group.monster_ids.is_empty()
                    || group.candidate_stage_ids.iter().any(|stage| {
                        stage
                            .parse::<u32>()
                            .map_or(true, |id| !stage_ids.contains(&id))
                    })
            })
            || parts
                .boss_pools
                .iter()
                .any(|pool| !valid_boss_pool(pool, &parts.groups))
            || parts.groups.iter().any(|group| {
                group.boss_battle_area_id.is_some_and(|battle_area_id| {
                    !parts
                        .boss_pools
                        .iter()
                        .any(|pool| pool.boss_battle_area_id == battle_area_id)
                })
            })
            || !valid_enemy_slots(&parts.enemy_slots, &parts.groups)
            || !valid_enemy_affixes(&parts.enemy_affixes)
            || parts.mechanic_programs.iter().any(|program| {
                !valid_digest(&program.source_sha256)
                    || !keys.insert(program.stable_key.as_ref())
                    || !valid_mechanic_disposition(program)
            })
        {
            return Err(error(
                "Currency Wars encounter catalog inventory is invalid",
            ));
        }
        Ok(Self {
            groups: parts.groups.into_boxed_slice(),
            source_obligations: parts.source_obligations.into_boxed_slice(),
            waves: parts.waves.into_boxed_slice(),
            enemy_slots: parts.enemy_slots.into_boxed_slice(),
            enemy_affixes: parts.enemy_affixes.into_boxed_slice(),
            boss_pools: parts.boss_pools.into_boxed_slice(),
            mechanic_programs: parts.mechanic_programs.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn mechanic_programs(&self) -> &[CurrencyWarsMechanicProgram] {
        &self.mechanic_programs
    }

    #[must_use]
    pub fn groups(&self) -> &[CurrencyWarsEncounterGroup] {
        &self.groups
    }

    #[must_use]
    pub fn boss_pool(&self, battle_area_id: u32) -> Option<&CurrencyWarsBossPool> {
        self.boss_pools
            .iter()
            .find(|pool| pool.boss_battle_area_id == battle_area_id)
    }

    #[must_use]
    pub fn formation_wave(&self, maximum_teammates: u8) -> Option<&CurrencyWarsEncounterWave> {
        self.waves
            .iter()
            .find(|wave| wave.maximum_teammates == maximum_teammates)
    }

    pub fn released_stages(&self) -> impl Iterator<Item = &CurrencyWarsReleasedStage> {
        self.source_obligations
            .iter()
            .filter_map(|obligation| obligation.stage.as_ref())
    }

    #[must_use]
    pub fn released_stage(&self, id: u32) -> Option<&CurrencyWarsReleasedStage> {
        self.source_obligations
            .iter()
            .filter_map(|obligation| obligation.stage.as_ref())
            .find(|stage| stage.stage_id == id)
    }

    #[must_use]
    pub fn enemy_slot(&self, source_monster_id: u32) -> Option<&CurrencyWarsEnemySlot> {
        self.enemy_slots.iter().find(|slot| {
            matches!(
                slot.definition,
                CurrencyWarsEnemySlotDefinition::Monster {
                    source_monster_id: candidate,
                    ..
                } if candidate == source_monster_id
            )
        })
    }

    #[must_use]
    pub fn enemy_star_scaling(
        &self,
        source_monster_id: u32,
        star: u8,
    ) -> Option<CurrencyWarsEnemyStatRatios> {
        let slot = self.enemy_slot(source_monster_id)?;
        self.slot_star_scaling(slot, star)
    }

    #[must_use]
    pub fn enemy_star_scaling_for_key(
        &self,
        shared_enemy_key: &str,
        star: u8,
    ) -> Option<CurrencyWarsEnemyStatRatios> {
        let slot = self.enemy_slots.iter().find(|slot| {
            matches!(
                &slot.definition,
                CurrencyWarsEnemySlotDefinition::Monster {
                    shared_enemy_key: candidate,
                    ..
                } if candidate.as_ref() == shared_enemy_key
            )
        })?;
        self.slot_star_scaling(slot, star)
    }

    fn slot_star_scaling(
        &self,
        slot: &CurrencyWarsEnemySlot,
        star: u8,
    ) -> Option<CurrencyWarsEnemyStatRatios> {
        let CurrencyWarsEnemySlotDefinition::Monster {
            star_scaling_groups,
            ..
        } = &slot.definition
        else {
            return None;
        };
        let group = *star_scaling_groups.get(usize::from(star.checked_sub(1)?))?;
        self.enemy_slots
            .iter()
            .find_map(|slot| match slot.definition {
                CurrencyWarsEnemySlotDefinition::EliteScaling {
                    group: candidate,
                    ratios,
                } if candidate == group => Some(ratios),
                CurrencyWarsEnemySlotDefinition::EliteScaling { .. }
                | CurrencyWarsEnemySlotDefinition::Monster { .. } => None,
            })
    }

    #[must_use]
    pub fn enemy_scaling(
        &self,
        chapter: u8,
        difficulty_level: u16,
    ) -> Option<CurrencyWarsEnemyScaling> {
        self.enemy_affixes
            .iter()
            .filter_map(|affix| match affix.definition {
                CurrencyWarsEnemyAffixDefinition::Scaling(scaling) => Some(scaling),
                CurrencyWarsEnemyAffixDefinition::Affix { .. }
                | CurrencyWarsEnemyAffixDefinition::MazeBuff { .. } => None,
            })
            .find(|scaling| {
                scaling.chapter == chapter && scaling.difficulty_level == difficulty_level
            })
    }

    pub fn enemy_scalings(&self) -> impl Iterator<Item = CurrencyWarsEnemyScaling> + '_ {
        self.enemy_affixes
            .iter()
            .filter_map(|affix| match affix.definition {
                CurrencyWarsEnemyAffixDefinition::Scaling(scaling) => Some(scaling),
                CurrencyWarsEnemyAffixDefinition::Affix { .. }
                | CurrencyWarsEnemyAffixDefinition::MazeBuff { .. } => None,
            })
    }

    pub fn enemy_affix_definitions(&self) -> impl Iterator<Item = &CurrencyWarsEnemyAffix> {
        self.enemy_affixes.iter().filter(|affix| {
            matches!(
                affix.definition,
                CurrencyWarsEnemyAffixDefinition::Affix { .. }
            )
        })
    }

    #[must_use]
    pub fn enemy_affix_definition(&self, source_id: u32) -> Option<&CurrencyWarsEnemyAffix> {
        self.enemy_affix_definitions().find(|affix| {
            matches!(
                affix.definition,
                CurrencyWarsEnemyAffixDefinition::Affix {
                    source_id: candidate,
                    ..
                } if candidate == source_id
            )
        })
    }
}

#[cfg(test)]
impl CurrencyWarsEncounterCatalog {
    pub(crate) fn test_fixture() -> Self {
        Self::new(CurrencyWarsEncounterCatalogParts {
            groups: vec![CurrencyWarsEncounterGroup {
                stable_key: "group.fixture".into(),
                source_id: 1,
                plane_id: "fixture".into(),
                difficulty_id: "fixture".into(),
                rank: "fixture".into(),
                candidate_stage_ids: Box::new(["100".into()]),
                monster_ids: Box::new([1]),
                battle_area_ids: Box::new([1]),
                boss_battle_area_id: Some(1),
                randomization: CurrencyWarsEncounterRandomization {
                    initial_code: 1,
                    enabled: true,
                },
            }],
            source_obligations: vec![CurrencyWarsEncounterSourceObligation {
                stable_key: "source.fixture".into(),
                parent_kind: "fixture".into(),
                parent_id: "fixture".into(),
                resolution_state: "fixture".into(),
                camp_ids: Box::new([1]),
                stage: Some(CurrencyWarsReleasedStage {
                    stage_id: 100,
                    stage_type: "fixture".into(),
                    level: 1,
                    elite_group: None,
                    stage_abilities_json: "[]".into(),
                    waves: Box::new([CurrencyWarsReleasedStageWave {
                        enemies: Box::new([CurrencyWarsReleasedStageEnemy {
                            formation: 0,
                            source_monster_id: 1,
                            shared_enemy_key: "enemy.fixture".into(),
                        }]),
                    }]),
                }),
                replacement_condition: None,
            }],
            waves: (1_u8..=5)
                .map(|maximum_teammates| CurrencyWarsEncounterWave {
                    stable_key: format!("wave.fixture.{maximum_teammates}").into(),
                    wave_index: u16::from(maximum_teammates),
                    maximum_teammates,
                    ability: None,
                    parameters: Box::new([]),
                })
                .collect(),
            enemy_slots: vec![
                CurrencyWarsEnemySlot {
                    stable_key: "scaling.fixture".into(),
                    definition: CurrencyWarsEnemySlotDefinition::EliteScaling {
                        group: 1,
                        ratios: CurrencyWarsEnemyStatRatios {
                            hp: starclock_combat::Ratio::ONE,
                            attack: starclock_combat::Ratio::ONE,
                            defense: starclock_combat::Ratio::ONE,
                            speed: starclock_combat::Ratio::ONE,
                            stance: starclock_combat::Ratio::ONE,
                        },
                    },
                },
                CurrencyWarsEnemySlot {
                    stable_key: "slot.fixture".into(),
                    definition: CurrencyWarsEnemySlotDefinition::Monster {
                        source_monster_id: 1,
                        tier: Some(1),
                        star_scaling_groups: [1; 4],
                        shared_enemy_key: "enemy.fixture".into(),
                    },
                },
            ],
            enemy_affixes: vec![CurrencyWarsEnemyAffix {
                stable_key: "affix.fixture".into(),
                definition: CurrencyWarsEnemyAffixDefinition::Scaling(CurrencyWarsEnemyScaling {
                    chapter: 1,
                    difficulty_level: 0,
                    hp_ratio: Ratio::ONE,
                    attack_ratio: Ratio::ONE,
                    defense_ratio: Ratio::ONE,
                    speed_ratio: Ratio::ONE,
                    stance_ratio: Ratio::ONE,
                }),
            }],
            boss_pools: vec![CurrencyWarsBossPool {
                stable_key: "boss.fixture".into(),
                source_id: 1,
                plane_id: "fixture".into(),
                difficulty_id: "fixture".into(),
                candidate_monster_ids: Box::new(["1".into()]),
                selection_policy: "fixture".into(),
                boss_battle_area_id: 1,
                candidate_stage_ids: Box::new(["100".into()]),
            }],
            mechanic_programs: vec![CurrencyWarsMechanicProgram {
                stable_key: "mechanic.fixture".into(),
                source_path: "fixture".into(),
                source_sha256: "00".repeat(32).into(),
                mechanic_family: "fixture".into(),
                scope: CurrencyWarsMechanicScope::CrossBattleActivity,
                trigger: "fixture".into(),
                state_lifecycle: "fixture".into(),
                disposition: CurrencyWarsMechanicProgramDisposition::PendingExactSource {
                    ordered_operations_json: "[]".into(),
                },
            }],
        })
        .expect("test encounter catalog is valid")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterCatalogParts {
    pub groups: Vec<CurrencyWarsEncounterGroup>,
    pub source_obligations: Vec<CurrencyWarsEncounterSourceObligation>,
    pub waves: Vec<CurrencyWarsEncounterWave>,
    pub enemy_slots: Vec<CurrencyWarsEnemySlot>,
    pub enemy_affixes: Vec<CurrencyWarsEnemyAffix>,
    pub boss_pools: Vec<CurrencyWarsBossPool>,
    pub mechanic_programs: Vec<CurrencyWarsMechanicProgram>,
}

fn valid_mechanic_disposition(definition: &CurrencyWarsMechanicProgram) -> bool {
    match &definition.disposition {
        CurrencyWarsMechanicProgramDisposition::PendingExactSource {
            ordered_operations_json,
        } => !ordered_operations_json.is_empty(),
        CurrencyWarsMechanicProgramDisposition::ExecutedBattlePolicy(policy) => {
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref() == "BattleOwnedTypedEnemyBehaviorPolicy"
                && policy.policy_id.as_ref() == "mechanic.configuration_program"
                && !policy.ability_names.is_empty()
                && all_unique(&policy.ability_names)
                && ordered_unique(&policy.global_modifier_names)
                && valid_optional_shape_counts(&policy.callback_event_counts)
                && valid_optional_shape_counts(&policy.configuration_type_counts)
                && !policy.selected_behavior.is_empty()
                && !policy.unresolved_field.is_empty()
                && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
                && !policy.replacement_condition.is_empty()
                && valid_digest(&policy.ordered_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedAvatarBattlePolicy(policy) => {
            let valid_binding = match policy.archetype {
                CurrencyWarsAvatarBattleBehaviorArchetype::RoleBattleEvent => {
                    !policy.battle_event_ids.is_empty()
                        && matches!(
                            policy.binding_policy,
                            CurrencyWarsAvatarBattleBehaviorBindingPolicy::ExactBattleEvent
                                | CurrencyWarsAvatarBattleBehaviorBindingPolicy::SameFamilyBattleEventFallback
                        )
                }
                CurrencyWarsAvatarBattleBehaviorArchetype::AugmentBattleEvent => {
                    policy.binding_policy
                        == CurrencyWarsAvatarBattleBehaviorBindingPolicy::TypedAugmentController
                        && policy.role_ids.is_empty()
                        && policy.avatar_ids.is_empty()
                        && policy.battle_event_ids.is_empty()
                }
            };
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref() == "BattleOwnedTypedAvatarBehaviorPolicy"
                && policy.policy_id.as_ref() == "mechanic.configuration_program"
                && valid_binding
                && ordered_unique_values(&policy.role_ids)
                && ordered_unique_values(&policy.avatar_ids)
                && ordered_unique_values(&policy.battle_event_ids)
                && !policy.ability_names.is_empty()
                && all_unique(&policy.ability_names)
                && ordered_unique(&policy.global_modifier_names)
                && valid_optional_shape_counts(&policy.callback_event_counts)
                && valid_shape_counts(&policy.configuration_type_counts)
                && !policy.selected_behavior.is_empty()
                && !policy.unresolved_field.is_empty()
                && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
                && !policy.replacement_condition.is_empty()
                && valid_digest(&policy.ordered_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedBattleConfigurationPolicy(policy) => {
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref()
                    == "BattleOwnedTypedConfigurationFamilyPolicy"
                && policy.policy_id.as_ref() == "mechanic.configuration_program"
                && (!policy.ability_names.is_empty() || !policy.global_modifier_names.is_empty())
                && all_unique(&policy.ability_names)
                && ordered_unique(&policy.global_modifier_names)
                && valid_optional_shape_counts(&policy.callback_event_counts)
                && valid_shape_counts(&policy.configuration_type_counts)
                && !policy.selected_behavior.is_empty()
                && !policy.unresolved_field.is_empty()
                && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
                && !policy.replacement_condition.is_empty()
                && valid_digest(&policy.ordered_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedBondBattlePolicy(policy) => {
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref() == "BattleOwnedTypedBondBehaviorPolicy"
                && policy.policy_id.as_ref() == "mechanic.configuration_program"
                && !policy.bond_ids.is_empty()
                && ordered_unique_values(&policy.bond_ids)
                && !policy.ability_names.is_empty()
                && all_unique(&policy.ability_names)
                && ordered_unique(&policy.global_modifier_names)
                && valid_optional_shape_counts(&policy.callback_event_counts)
                && valid_optional_shape_counts(&policy.configuration_type_counts)
                && !policy.selected_behavior.is_empty()
                && !policy.unresolved_field.is_empty()
                && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
                && !policy.replacement_condition.is_empty()
                && valid_digest(&policy.ordered_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(policy) => {
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref() == "BattleOwnedTypedProgramBindingPolicy"
                && policy.policy_id.as_ref() == "mechanic.configuration_program"
                && valid_battle_program_bindings(policy)
                && (policy.archetype == CurrencyWarsBattleProgramBindingArchetype::BondStageAbility
                    || !policy.ability_names.is_empty())
                && all_unique(&policy.ability_names)
                && ordered_unique(&policy.global_modifier_names)
                && valid_optional_shape_counts(&policy.callback_event_counts)
                && valid_shape_counts(&policy.configuration_type_counts)
                && !policy.selected_behavior.is_empty()
                && !policy.unresolved_field.is_empty()
                && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
                && !policy.replacement_condition.is_empty()
                && valid_digest(&policy.ordered_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedEnemyCharacterConfiguration(config) => {
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref()
                    == "BattleOwnedTypedEnemyCharacterConfiguration"
                && !config.bindings.is_empty()
                && ordered_unique_values(&config.bindings)
                && config.bindings.iter().all(|binding| {
                    !binding.shared_enemy_key.is_empty() && binding.source_template_id > 0
                })
                && !config.ability_names.is_empty()
                && all_unique(&config.ability_names)
                && !config.skill_names.is_empty()
                && all_unique(&config.skill_names)
                && valid_digest(&config.mechanical_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedEnemyAiConfiguration(config) => {
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref() == "BattleOwnedTypedEnemyAiConfiguration"
                && !config.ai_name.is_empty()
                && !config.bindings.is_empty()
                && ordered_unique_values(&config.bindings)
                && config.bindings.iter().all(|binding| {
                    !binding.shared_enemy_key.is_empty() && binding.source_template_id > 0
                })
                && all_unique(&config.variable_names)
                && !config.decision_names.is_empty()
                && config.decision_names.iter().all(|name| !name.is_empty())
                && !config.skill_names.is_empty()
                && config.skill_names.iter().all(|name| !name.is_empty())
                && valid_shape_counts(&config.node_type_counts)
                && valid_digest(&config.mechanical_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedComplexAiGlobalFactors(factors) => {
            let expected_group_count = match factors.mapper_policy_id.as_ref() {
                "currency-wars.complex-ai-multirange-policy.v1" => 2,
                "currency-wars.complex-ai-source-and-multirange-policy.v1" => 9,
                _ => 0,
            };
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref() == "BattleOwnedTypedComplexAiFactorPolicy"
                && expected_group_count > 0
                && factors.groups.len() == expected_group_count
                && factors
                    .groups
                    .windows(2)
                    .all(|pair| pair[0].stable_key < pair[1].stable_key)
                && factors.groups.iter().all(|group| {
                    !group.stable_key.is_empty()
                        && !group.factors.is_empty()
                        && group.factors.iter().all(|factor| !factor.ranges.is_empty())
                })
                && !factors.selected_behavior.is_empty()
                && !factors.unresolved_field.is_empty()
                && factors.confidence.as_ref() == "PolicyOnlyNotObservedParity"
                && !factors.replacement_condition.is_empty()
                && valid_digest(&factors.mechanical_shape_sha256)
        }
        CurrencyWarsMechanicProgramDisposition::ExecutedGlobalTaskTemplates(library) => {
            definition.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                && definition.state_lifecycle.as_ref()
                    == "BattleOwnedTypedGlobalTaskTemplateLibrary"
                && library.templates().len() == 13
                && library
                    .templates()
                    .iter()
                    .filter(|template| {
                        matches!(
                            &template.definition,
                            CurrencyWarsGlobalTaskTemplateDefinition::ApplyModifier(_)
                        )
                    })
                    .count()
                    == 6
                && valid_digest(library.mechanical_shape_sha256())
        }
        CurrencyWarsMechanicProgramDisposition::MetadataOnly(audit) => match audit {
            CurrencyWarsMechanicMetadataAudit::Presentation(audit) => {
                definition.state_lifecycle.as_ref() == "PresentationOnlyNoAuthoritativeState"
                    && valid_digest(&audit.ordered_shape_sha256)
                    && valid_shape_counts(&audit.configuration_type_counts)
                    && valid_optional_shape_counts(&audit.operation_type_counts)
                    && ordered_unique(&audit.tutorial_keys)
                    && ordered_unique(&audit.custom_time_types)
                    && ordered_unique(&audit.player_action_types)
            }
            CurrencyWarsMechanicMetadataAudit::LayoutDescriptor(audit) => {
                definition.state_lifecycle.as_ref() == "MetadataOnlyNoAuthoritativeState"
                    && valid_digest(&audit.ordered_shape_sha256)
                    && audit.descriptor_entry_count > 0
                    && !audit.root_keys.is_empty()
                    && ordered_unique(&audit.root_keys)
            }
            CurrencyWarsMechanicMetadataAudit::RolePresentation(audit) => {
                definition.state_lifecycle.as_ref() == "MetadataOnlyNoAuthoritativeState"
                    && valid_digest(&audit.ordered_shape_sha256)
                    && !audit.reason.is_empty()
                    && !audit.record_key.is_empty()
                    && !audit.text_hash.is_empty()
                    && audit.text_hash.bytes().all(|byte| byte.is_ascii_digit())
            }
            CurrencyWarsMechanicMetadataAudit::StructuredPresentation(audit) => {
                definition.state_lifecycle.as_ref() == "MetadataOnlyNoAuthoritativeState"
                    && valid_digest(&audit.ordered_shape_sha256)
                    && !audit.reason.is_empty()
                    && !audit.record_key.is_empty()
                    && !audit.root_keys.is_empty()
                    && ordered_unique(&audit.root_keys)
                    && valid_optional_shape_counts(&audit.configuration_type_counts)
                    && audit.descriptor_entry_count > 0
            }
            CurrencyWarsMechanicMetadataAudit::UnreachableCharacterOverride(audit) => {
                definition.state_lifecycle.as_ref() == "MetadataOnlyNoAuthoritativeState"
                    && valid_digest(&audit.mechanical_shape_sha256)
                    && !audit.configuration_kind.is_empty()
                    && audit.skill_count > 0
            }
            CurrencyWarsMechanicMetadataAudit::UnreachableBattleConfiguration(audit) => {
                definition.state_lifecycle.as_ref() == "MetadataOnlyNoAuthoritativeState"
                    && audit.reason.as_ref() == "NoVersion44EquipmentAbilityBinding"
                    && !audit.ability_names.is_empty()
                    && all_unique(&audit.ability_names)
                    && ordered_unique(&audit.global_modifier_names)
                    && valid_optional_shape_counts(&audit.callback_event_counts)
                    && valid_shape_counts(&audit.configuration_type_counts)
                    && valid_digest(&audit.ordered_shape_sha256)
            }
            CurrencyWarsMechanicMetadataAudit::EmptyConfiguration(audit) => {
                definition.state_lifecycle.as_ref() == "MetadataOnlyNoAuthoritativeState"
                    && audit.reason.as_ref() == "NoAbilityModifierCallbackOrConfigurationNode"
                    && valid_digest(&audit.ordered_shape_sha256)
            }
        },
        CurrencyWarsMechanicProgramDisposition::ExecutedActivity(program) => match program {
            CurrencyWarsMechanicActivityProgram::Progression(
                CurrencyWarsProgressionProgram::RoleCostAvailability(_),
            ) => {
                program_state(program) == "ShopCandidateEligibilityByRunPosition"
                    && definition.state_lifecycle.as_ref()
                        == "ShopCandidateEligibilityByRunPosition"
            }
            CurrencyWarsMechanicActivityProgram::Progression(
                CurrencyWarsProgressionProgram::SeasonScoreAndExperience(_),
            ) => {
                program_state(program) == "SettlementProjectionNoRunMutation"
                    && definition.state_lifecycle.as_ref() == "SettlementProjectionNoRunMutation"
            }
            CurrencyWarsMechanicActivityProgram::Progression(
                CurrencyWarsProgressionProgram::ModuleRoleBan(_),
            ) => definition.state_lifecycle.as_ref() == "ShopAndRosterRoleEligibilityByModule",
            CurrencyWarsMechanicActivityProgram::Progression(
                CurrencyWarsProgressionProgram::SeasonRolePool(_),
            ) => definition.state_lifecycle.as_ref() == "ShopAndRosterRoleEligibilityBySeason",
            CurrencyWarsMechanicActivityProgram::Progression(
                CurrencyWarsProgressionProgram::SeasonTraitRolePool(_),
            ) => definition.state_lifecycle.as_ref() == "ControllerRoleTraitIndex",
            CurrencyWarsMechanicActivityProgram::Progression(
                CurrencyWarsProgressionProgram::RoleReferenceScore(_),
            ) => definition.state_lifecycle.as_ref() == "ControllerRoleReferenceRanking",
            CurrencyWarsMechanicActivityProgram::CharacterOverride(_) => {
                definition.state_lifecycle.as_ref()
                    == "ContributionSnapshotCharacterOverrideSelection"
            }
        },
    }
}

fn valid_battle_program_bindings(policy: &CurrencyWarsBattleProgramBindingPolicy) -> bool {
    if policy.bindings.is_empty()
        || !ordered_unique_values(&policy.bindings)
        || policy.bindings.iter().any(|binding| match binding {
            CurrencyWarsBattleProgramBinding::Avatar(id)
            | CurrencyWarsBattleProgramBinding::Servant(id)
            | CurrencyWarsBattleProgramBinding::BattleEvent(id)
            | CurrencyWarsBattleProgramBinding::AugmentMazeBuff(id)
            | CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(id) => *id == 0,
            CurrencyWarsBattleProgramBinding::Role(_)
            | CurrencyWarsBattleProgramBinding::Bond(_)
            | CurrencyWarsBattleProgramBinding::Equipment(_) => false,
        })
    {
        return false;
    }
    let count = |predicate: fn(&CurrencyWarsBattleProgramBinding) -> bool| {
        policy
            .bindings
            .iter()
            .filter(|binding| predicate(binding))
            .count()
    };
    let roles = count(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::Role(_)));
    let avatars = count(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::Avatar(_)));
    let servants = count(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::Servant(_)));
    let battle_events =
        count(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::BattleEvent(_)));
    let bonds = count(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::Bond(_)));
    let augment_maze_buffs = count(|binding| {
        matches!(
            binding,
            CurrencyWarsBattleProgramBinding::AugmentMazeBuff(_)
        )
    });
    let enemy_affix_maze_buffs = count(|binding| {
        matches!(
            binding,
            CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(_)
        )
    });
    let equipment =
        count(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::Equipment(_)));
    let bindings = policy.bindings.len();
    match policy.archetype {
        CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility => {
            roles > 0 && avatars > 0 && roles + avatars == bindings
        }
        CurrencyWarsBattleProgramBindingArchetype::ServantAbility => {
            roles > 0 && avatars > 0 && servants > 0 && roles + avatars + servants == bindings
        }
        CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent => {
            battle_events > 0 && roles + avatars + battle_events == bindings
        }
        CurrencyWarsBattleProgramBindingArchetype::BondStageAbility => bonds == bindings,
        CurrencyWarsBattleProgramBindingArchetype::AugmentStageAbility => {
            augment_maze_buffs == bindings
        }
        CurrencyWarsBattleProgramBindingArchetype::MonsterTagController => {
            enemy_affix_maze_buffs == bindings
        }
        CurrencyWarsBattleProgramBindingArchetype::EquipmentController => equipment == bindings,
    }
}

fn valid_boss_pool(pool: &CurrencyWarsBossPool, groups: &[CurrencyWarsEncounterGroup]) -> bool {
    let Some(group) = groups
        .iter()
        .find(|group| group.boss_battle_area_id == Some(pool.boss_battle_area_id))
    else {
        return false;
    };
    let expected_stages = group
        .candidate_stage_ids
        .iter()
        .filter(|stage| {
            stage.parse::<u32>().ok().map(|id| id / 100) == Some(pool.boss_battle_area_id)
        })
        .map(Box::as_ref)
        .collect::<Vec<_>>();
    expected_stages
        == pool
            .candidate_stage_ids
            .iter()
            .map(Box::as_ref)
            .collect::<Vec<_>>()
        && group.monster_ids.len() == pool.candidate_monster_ids.len()
        && group
            .monster_ids
            .iter()
            .zip(pool.candidate_monster_ids.iter())
            .all(|(expected, actual)| actual.parse::<u32>() == Ok(*expected))
}

fn valid_enemy_slots(
    slots: &[CurrencyWarsEnemySlot],
    groups: &[CurrencyWarsEncounterGroup],
) -> bool {
    let mut scaling_groups = BTreeSet::new();
    let mut monsters = BTreeSet::new();
    let mut monster_keys = BTreeSet::new();
    for slot in slots {
        match &slot.definition {
            CurrencyWarsEnemySlotDefinition::EliteScaling { group, .. } => {
                if !scaling_groups.insert(*group) {
                    return false;
                }
            }
            CurrencyWarsEnemySlotDefinition::Monster {
                source_monster_id,
                tier,
                shared_enemy_key,
                ..
            } => {
                if tier.is_some_and(|tier| tier == 0)
                    || shared_enemy_key.is_empty()
                    || !monsters.insert(*source_monster_id)
                    || !monster_keys.insert(shared_enemy_key.as_ref())
                {
                    return false;
                }
            }
        }
    }
    if slots.iter().any(|slot| match &slot.definition {
        CurrencyWarsEnemySlotDefinition::EliteScaling { .. } => false,
        CurrencyWarsEnemySlotDefinition::Monster {
            star_scaling_groups,
            ..
        } => star_scaling_groups
            .iter()
            .any(|group| !scaling_groups.contains(group)),
    }) {
        return false;
    }
    groups
        .iter()
        .flat_map(|group| group.monster_ids.iter())
        .all(|monster| monsters.contains(monster))
        && !monster_keys.is_empty()
}

fn valid_enemy_affixes(affixes: &[CurrencyWarsEnemyAffix]) -> bool {
    let mut stable_keys = BTreeSet::new();
    let mut affix_ids = BTreeSet::new();
    let mut maze_buff_ids = BTreeSet::new();
    let mut referenced_maze_buff_ids = BTreeSet::new();
    let mut scaling_keys = BTreeSet::new();
    for affix in affixes {
        if affix.stable_key.is_empty() || !stable_keys.insert(affix.stable_key.as_ref()) {
            return false;
        }
        match &affix.definition {
            CurrencyWarsEnemyAffixDefinition::Affix {
                source_id,
                maze_buff_ids,
                config_path,
                ..
            } => {
                if *source_id == 0
                    || config_path.is_empty()
                    || !affix_ids.insert(*source_id)
                    || maze_buff_ids.contains(&0)
                    || CurrencyWarsEnemyAffixBehavior::compile(affix).is_err()
                {
                    return false;
                }
                referenced_maze_buff_ids.extend(maze_buff_ids);
            }
            CurrencyWarsEnemyAffixDefinition::MazeBuff {
                source_id,
                modifier,
                binding_key,
                level,
                maximum_level,
                ..
            } => {
                if *source_id == 0
                    || modifier.is_empty()
                    || binding_key.is_empty()
                    || *level == 0
                    || level > maximum_level
                    || !maze_buff_ids.insert(*source_id)
                {
                    return false;
                }
            }
            CurrencyWarsEnemyAffixDefinition::Scaling(scaling) => {
                if scaling.chapter == 0
                    || !scaling_keys.insert((scaling.chapter, scaling.difficulty_level))
                {
                    return false;
                }
            }
        }
    }
    referenced_maze_buff_ids.is_subset(&maze_buff_ids)
}

fn valid_formation_waves(waves: &[CurrencyWarsEncounterWave]) -> bool {
    waves.len() == 5
        && waves.iter().enumerate().all(|(index, wave)| {
            let expected = u8::try_from(index + 1).expect("five formation waves fit in u8");
            wave.wave_index == u16::from(expected)
                && wave.maximum_teammates == expected
                && wave.ability.is_none()
                && wave.parameters.is_empty()
        })
}

fn program_state(program: &CurrencyWarsMechanicActivityProgram) -> &'static str {
    match program {
        CurrencyWarsMechanicActivityProgram::Progression(
            CurrencyWarsProgressionProgram::RoleCostAvailability(_),
        ) => "ShopCandidateEligibilityByRunPosition",
        CurrencyWarsMechanicActivityProgram::Progression(
            CurrencyWarsProgressionProgram::SeasonScoreAndExperience(_),
        ) => "SettlementProjectionNoRunMutation",
        CurrencyWarsMechanicActivityProgram::Progression(
            CurrencyWarsProgressionProgram::ModuleRoleBan(_),
        ) => "ShopAndRosterRoleEligibilityByModule",
        CurrencyWarsMechanicActivityProgram::Progression(
            CurrencyWarsProgressionProgram::SeasonRolePool(_),
        ) => "ShopAndRosterRoleEligibilityBySeason",
        CurrencyWarsMechanicActivityProgram::Progression(
            CurrencyWarsProgressionProgram::SeasonTraitRolePool(_),
        ) => "ControllerRoleTraitIndex",
        CurrencyWarsMechanicActivityProgram::Progression(
            CurrencyWarsProgressionProgram::RoleReferenceScore(_),
        ) => "ControllerRoleReferenceRanking",
        CurrencyWarsMechanicActivityProgram::CharacterOverride(_) => {
            "ContributionSnapshotCharacterOverrideSelection"
        }
    }
}

fn valid_shape_counts(counts: &[CurrencyWarsMechanicShapeCount]) -> bool {
    !counts.is_empty() && valid_optional_shape_counts(counts)
}

fn valid_optional_shape_counts(counts: &[CurrencyWarsMechanicShapeCount]) -> bool {
    counts.iter().all(|entry| entry.count > 0)
        && counts.windows(2).all(|pair| pair[0].shape < pair[1].shape)
}

fn ordered_unique(values: &[Box<str>]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn ordered_unique_values<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn all_unique(values: &[Box<str>]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEncounterCatalogError {
    message: Box<str>,
}
impl std::fmt::Display for CurrencyWarsEncounterCatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for CurrencyWarsEncounterCatalogError {}
fn error(message: &'static str) -> CurrencyWarsEncounterCatalogError {
    CurrencyWarsEncounterCatalogError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CurrencyWarsEncounterCatalog, CurrencyWarsEncounterCatalogParts,
        CurrencyWarsMechanicMetadataAudit, CurrencyWarsMechanicPresentationAudit,
        CurrencyWarsMechanicPresentationKind, CurrencyWarsMechanicProgramDisposition,
    };

    #[test]
    fn malformed_presentation_audit_is_rejected() {
        let catalog = CurrencyWarsEncounterCatalog::test_fixture();
        let mut parts = CurrencyWarsEncounterCatalogParts {
            groups: catalog.groups.into_vec(),
            source_obligations: catalog.source_obligations.into_vec(),
            waves: catalog.waves.into_vec(),
            enemy_slots: catalog.enemy_slots.into_vec(),
            enemy_affixes: catalog.enemy_affixes.into_vec(),
            boss_pools: catalog.boss_pools.into_vec(),
            mechanic_programs: catalog.mechanic_programs.into_vec(),
        };
        parts.mechanic_programs[0].state_lifecycle = "PresentationOnlyNoAuthoritativeState".into();
        parts.mechanic_programs[0].disposition =
            CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::Presentation(
                    CurrencyWarsMechanicPresentationAudit {
                        reason: CurrencyWarsMechanicPresentationKind::TutorialAndInputGuidance,
                        configuration_type_counts: Box::new([]),
                        operation_type_counts: Box::new([]),
                        tutorial_keys: Box::new([]),
                        custom_time_types: Box::new([]),
                        player_action_types: Box::new([]),
                        ordered_shape_sha256: "00".repeat(32).into(),
                    },
                ),
            );

        let error = CurrencyWarsEncounterCatalog::new(parts).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Currency Wars encounter catalog inventory is invalid"
        );
    }
}
