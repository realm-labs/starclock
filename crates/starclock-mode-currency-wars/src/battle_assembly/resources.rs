//! Immutable shared-combat inputs used by Currency Wars battle assembly.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use sha2::{Digest, Sha256};
use starclock_combat::{
    EnemyDefinitionId, ResolvedCombatantSpec, UnitDefinitionId, UnitLevel,
    catalog::{CombatCatalog, builder::CombatCatalogBuilder, definition::EnemyDefinition},
    formula::model::CombatElement,
};

use crate::{
    CurrencyWarsAvatarBattleBehaviorArchetype, CurrencyWarsAvatarBattleBehaviorBindingPolicy,
    CurrencyWarsBattleBehaviorArchetype, CurrencyWarsBattleProgramBinding,
    CurrencyWarsBattleProgramBindingArchetype, CurrencyWarsRoleId,
};

use super::{CurrencyWarsBattleAssemblyError, debug_error, error};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyCombatInput {
    pub stable_key: Box<str>,
    pub definition: EnemyDefinitionId,
    pub level: UnitLevel,
    pub stat_source_level: UnitLevel,
    pub behavior_source: CurrencyWarsEnemyBehaviorSource,
    pub combatant: ResolvedCombatantSpec,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurrencyWarsEnemyBehaviorSource {
    ExactVariant,
    SameReleasedFamilyPolicy,
    GenericRankFallbackPolicy,
}

impl CurrencyWarsEnemyBehaviorSource {
    pub(super) const fn is_fallback(self) -> bool {
        !matches!(self, Self::ExactVariant)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleBehaviorProgramInput {
    pub stable_key: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub archetype: CurrencyWarsBattleBehaviorArchetype,
    pub definition: EnemyDefinitionId,
    pub behavior_source: CurrencyWarsEnemyBehaviorSource,
    pub combatant: ResolvedCombatantSpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAvatarBattleBehaviorProgramInput {
    pub stable_key: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub archetype: CurrencyWarsAvatarBattleBehaviorArchetype,
    pub binding_policy: CurrencyWarsAvatarBattleBehaviorBindingPolicy,
    pub role_ids: Box<[CurrencyWarsRoleId]>,
    pub avatar_ids: Box<[u32]>,
    pub battle_event_ids: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleProgramBindingInput {
    pub stable_key: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub archetype: CurrencyWarsBattleProgramBindingArchetype,
    pub bindings: Box<[CurrencyWarsBattleProgramBinding]>,
    pub runtime_definition_count: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsEnemyCharacterConfigurationRuntimeBinding {
    pub shared_enemy_key: Box<str>,
    pub source_template_id: u32,
    pub definition: EnemyDefinitionId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyCharacterConfigurationInput {
    pub stable_key: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub bindings: Box<[CurrencyWarsEnemyCharacterConfigurationRuntimeBinding]>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsEnemyAiConfigurationRuntimeBinding {
    pub shared_enemy_key: Box<str>,
    pub source_template_id: u32,
    pub definition: EnemyDefinitionId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnemyAiConfigurationInput {
    pub stable_key: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub bindings: Box<[CurrencyWarsEnemyAiConfigurationRuntimeBinding]>,
}

#[derive(Clone, Debug)]
pub(super) struct CurrencyWarsResolvedEnemyInput {
    pub(super) definition: EnemyDefinitionId,
    pub(super) combatant: ResolvedCombatantSpec,
    pub(super) stat_source_level: UnitLevel,
    pub(super) behavior_source: CurrencyWarsEnemyBehaviorSource,
}

#[derive(Clone, Debug)]
pub struct CurrencyWarsBattleResources {
    combat: Arc<CombatCatalog>,
    enemies: BTreeMap<(Box<str>, UnitLevel), CurrencyWarsResolvedEnemyInput>,
    battle_behavior_programs: Box<[CurrencyWarsBattleBehaviorProgramInput]>,
    avatar_battle_behavior_programs: Box<[CurrencyWarsAvatarBattleBehaviorProgramInput]>,
    battle_program_bindings: Box<[CurrencyWarsBattleProgramBindingInput]>,
    enemy_character_configurations: Box<[CurrencyWarsEnemyCharacterConfigurationInput]>,
    enemy_ai_configurations: Box<[CurrencyWarsEnemyAiConfigurationInput]>,
    role_elements: BTreeMap<CurrencyWarsRoleId, CombatElement>,
    digest: [u8; 32],
}

pub struct CurrencyWarsBattleResourceParts {
    pub enemies: Vec<CurrencyWarsEnemyCombatInput>,
    pub battle_behavior_programs: Vec<CurrencyWarsBattleBehaviorProgramInput>,
    pub avatar_battle_behavior_programs: Vec<CurrencyWarsAvatarBattleBehaviorProgramInput>,
    pub battle_program_bindings: Vec<CurrencyWarsBattleProgramBindingInput>,
    pub enemy_character_configurations: Vec<CurrencyWarsEnemyCharacterConfigurationInput>,
    pub enemy_ai_configurations: Vec<CurrencyWarsEnemyAiConfigurationInput>,
    pub aliases: Vec<EnemyDefinition>,
    pub role_elements: Vec<(CurrencyWarsRoleId, CombatElement)>,
}

impl CurrencyWarsBattleResources {
    pub fn new(
        base: &CombatCatalog,
        parts: CurrencyWarsBattleResourceParts,
    ) -> Result<Self, CurrencyWarsBattleAssemblyError> {
        let CurrencyWarsBattleResourceParts {
            mut enemies,
            mut battle_behavior_programs,
            mut avatar_battle_behavior_programs,
            mut battle_program_bindings,
            mut enemy_character_configurations,
            mut enemy_ai_configurations,
            mut aliases,
            mut role_elements,
        } = parts;
        enemies.sort_by(|left, right| {
            (left.stable_key.as_ref(), left.level).cmp(&(right.stable_key.as_ref(), right.level))
        });
        battle_behavior_programs.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        avatar_battle_behavior_programs
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        battle_program_bindings.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        enemy_character_configurations
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        enemy_ai_configurations.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        aliases.sort_by_key(EnemyDefinition::id);
        role_elements.sort_unstable_by_key(|(role, _)| *role);
        let alias_ids = aliases
            .iter()
            .map(EnemyDefinition::id)
            .collect::<BTreeSet<_>>();
        if enemies.is_empty()
            || battle_behavior_programs.is_empty()
            || role_elements.is_empty()
            || role_elements.windows(2).any(|pair| pair[0].0 == pair[1].0)
            || enemies.windows(2).any(|pair| {
                pair[0].stable_key == pair[1].stable_key && pair[0].level == pair[1].level
            })
            || battle_behavior_programs.windows(2).any(|pair| {
                pair[0].stable_key == pair[1].stable_key
                    || pair[0].source_path == pair[1].source_path
            })
            || avatar_battle_behavior_programs.windows(2).any(|pair| {
                pair[0].stable_key == pair[1].stable_key
                    || pair[0].source_path == pair[1].source_path
            })
            || battle_program_bindings.windows(2).any(|pair| {
                pair[0].stable_key == pair[1].stable_key
                    || pair[0].source_path == pair[1].source_path
            })
            || enemy_character_configurations.windows(2).any(|pair| {
                pair[0].stable_key == pair[1].stable_key
                    || pair[0].source_path == pair[1].source_path
            })
            || enemy_ai_configurations.windows(2).any(|pair| {
                pair[0].stable_key == pair[1].stable_key
                    || pair[0].source_path == pair[1].source_path
            })
            || aliases.windows(2).any(|pair| pair[0].id() == pair[1].id())
            || aliases.iter().any(|alias| base.enemy(alias.id()).is_some())
            || enemies.iter().any(|input| {
                definition(base, &aliases, &alias_ids, input.definition).is_none_or(|definition| {
                    input.stable_key.is_empty()
                        || definition.unit() != input.combatant.form()
                        || definition.abilities() != input.combatant.abilities()
                })
            })
            || battle_behavior_programs.iter().any(|input| {
                base.enemy(input.definition).is_none_or(|definition| {
                    input.stable_key.is_empty()
                        || input.source_path.is_empty()
                        || !valid_sha256(&input.source_sha256)
                        || input.behavior_source == CurrencyWarsEnemyBehaviorSource::ExactVariant
                        || definition.ai_graph().is_none()
                        || definition.abilities().is_empty()
                        || definition.unit() != input.combatant.form()
                        || definition.abilities() != input.combatant.abilities()
                })
            })
            || avatar_battle_behavior_programs
                .iter()
                .any(invalid_avatar_behavior_program)
            || battle_program_bindings
                .iter()
                .any(invalid_battle_program_binding)
            || enemy_character_configurations.iter().any(|input| {
                invalid_enemy_character_configuration(base, &aliases, &alias_ids, input)
            })
            || enemy_ai_configurations
                .iter()
                .any(|input| invalid_enemy_ai_configuration(base, &aliases, &alias_ids, input))
        {
            return Err(error("Currency Wars battle resources are invalid"));
        }
        let digest = resource_digest(
            base,
            &enemies,
            &battle_behavior_programs,
            &avatar_battle_behavior_programs,
            &battle_program_bindings,
            CurrencyWarsEnemyConfigurationInputs {
                character: &enemy_character_configurations,
                ai: &enemy_ai_configurations,
            },
            &role_elements,
        );
        let mut builder = CombatCatalogBuilder::from_catalog(base, digest);
        for alias in aliases {
            builder.add_enemy(alias);
        }
        let combat = builder.build().map_err(debug_error)?;
        Ok(Self {
            combat,
            enemies: enemies
                .into_iter()
                .map(|input| {
                    (
                        (input.stable_key, input.level),
                        CurrencyWarsResolvedEnemyInput {
                            definition: input.definition,
                            combatant: input.combatant,
                            stat_source_level: input.stat_source_level,
                            behavior_source: input.behavior_source,
                        },
                    )
                })
                .collect(),
            battle_behavior_programs: battle_behavior_programs.into_boxed_slice(),
            avatar_battle_behavior_programs: avatar_battle_behavior_programs.into_boxed_slice(),
            battle_program_bindings: battle_program_bindings.into_boxed_slice(),
            enemy_character_configurations: enemy_character_configurations.into_boxed_slice(),
            enemy_ai_configurations: enemy_ai_configurations.into_boxed_slice(),
            role_elements: role_elements.into_iter().collect(),
            digest,
        })
    }

    #[must_use]
    pub const fn combat(&self) -> &Arc<CombatCatalog> {
        &self.combat
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(super) fn role_element(&self, role: CurrencyWarsRoleId) -> Option<CombatElement> {
        self.role_elements.get(&role).copied()
    }

    pub(super) fn enemy(
        &self,
        stable_key: &str,
        level: UnitLevel,
    ) -> Option<&CurrencyWarsResolvedEnemyInput> {
        self.enemies.get(&(stable_key.into(), level))
    }

    pub(super) fn enemy_form(&self, stable_key: &str) -> Option<UnitDefinitionId> {
        let mut forms = self
            .enemies
            .iter()
            .filter(|((key, _), _)| key.as_ref() == stable_key)
            .map(|(_, input)| input.combatant.form());
        let form = forms.next()?;
        forms.all(|candidate| candidate == form).then_some(form)
    }

    #[must_use]
    pub fn enemy_input_count(&self) -> usize {
        self.enemies.len()
    }

    #[must_use]
    pub fn contains_enemy_input(&self, stable_key: &str, level: UnitLevel) -> bool {
        self.enemy(stable_key, level).is_some()
    }

    #[must_use]
    pub fn behavior_fallback_input_count(&self) -> usize {
        self.enemies
            .values()
            .filter(|input| input.behavior_source.is_fallback())
            .count()
    }

    #[must_use]
    pub fn same_family_behavior_input_count(&self) -> usize {
        self.enemies
            .values()
            .filter(|input| {
                input.behavior_source == CurrencyWarsEnemyBehaviorSource::SameReleasedFamilyPolicy
            })
            .count()
    }

    #[must_use]
    pub fn generic_behavior_fallback_input_count(&self) -> usize {
        self.enemies
            .values()
            .filter(|input| {
                input.behavior_source == CurrencyWarsEnemyBehaviorSource::GenericRankFallbackPolicy
            })
            .count()
    }

    #[must_use]
    pub fn stat_fallback_input_count(&self) -> usize {
        self.enemies
            .values()
            .filter(|input| input.stat_source_level != input.combatant.level())
            .count()
    }

    #[must_use]
    pub fn battle_behavior_programs(&self) -> &[CurrencyWarsBattleBehaviorProgramInput] {
        &self.battle_behavior_programs
    }

    #[must_use]
    pub fn avatar_battle_behavior_programs(
        &self,
    ) -> &[CurrencyWarsAvatarBattleBehaviorProgramInput] {
        &self.avatar_battle_behavior_programs
    }

    #[must_use]
    pub fn battle_program_bindings(&self) -> &[CurrencyWarsBattleProgramBindingInput] {
        &self.battle_program_bindings
    }

    #[must_use]
    pub fn enemy_character_configurations(
        &self,
    ) -> &[CurrencyWarsEnemyCharacterConfigurationInput] {
        &self.enemy_character_configurations
    }

    #[must_use]
    pub fn enemy_ai_configurations(&self) -> &[CurrencyWarsEnemyAiConfigurationInput] {
        &self.enemy_ai_configurations
    }
}

fn invalid_enemy_character_configuration(
    base: &CombatCatalog,
    aliases: &[EnemyDefinition],
    alias_ids: &BTreeSet<EnemyDefinitionId>,
    input: &CurrencyWarsEnemyCharacterConfigurationInput,
) -> bool {
    input.stable_key.is_empty()
        || input.source_path.is_empty()
        || !valid_sha256(&input.source_sha256)
        || input.bindings.is_empty()
        || input.bindings.windows(2).any(|pair| pair[0] >= pair[1])
        || input.bindings.iter().any(|binding| {
            binding.shared_enemy_key.is_empty()
                || binding.source_template_id == 0
                || definition(base, aliases, alias_ids, binding.definition).is_none_or(
                    |definition| {
                        definition.ai_graph().is_none() || definition.abilities().is_empty()
                    },
                )
        })
}

fn invalid_enemy_ai_configuration(
    base: &CombatCatalog,
    aliases: &[EnemyDefinition],
    alias_ids: &BTreeSet<EnemyDefinitionId>,
    input: &CurrencyWarsEnemyAiConfigurationInput,
) -> bool {
    input.stable_key.is_empty()
        || input.source_path.is_empty()
        || !valid_sha256(&input.source_sha256)
        || input.bindings.is_empty()
        || input.bindings.windows(2).any(|pair| pair[0] >= pair[1])
        || input.bindings.iter().any(|binding| {
            binding.shared_enemy_key.is_empty()
                || binding.source_template_id == 0
                || definition(base, aliases, alias_ids, binding.definition).is_none_or(
                    |definition| {
                        definition.ai_graph().is_none() || definition.abilities().is_empty()
                    },
                )
        })
}

fn invalid_battle_program_binding(input: &CurrencyWarsBattleProgramBindingInput) -> bool {
    input.stable_key.is_empty()
        || input.source_path.is_empty()
        || !valid_sha256(&input.source_sha256)
        || input.runtime_definition_count == 0
        || input.bindings.is_empty()
        || input.bindings.windows(2).any(|pair| pair[0] >= pair[1])
}

fn invalid_avatar_behavior_program(input: &CurrencyWarsAvatarBattleBehaviorProgramInput) -> bool {
    let invalid_binding = match input.archetype {
        CurrencyWarsAvatarBattleBehaviorArchetype::RoleBattleEvent => {
            input.battle_event_ids.is_empty()
                || !matches!(
                input.binding_policy,
                CurrencyWarsAvatarBattleBehaviorBindingPolicy::ExactBattleEvent
                    | CurrencyWarsAvatarBattleBehaviorBindingPolicy::SameFamilyBattleEventFallback
            )
        }
        CurrencyWarsAvatarBattleBehaviorArchetype::AugmentBattleEvent => {
            input.binding_policy
                != CurrencyWarsAvatarBattleBehaviorBindingPolicy::TypedAugmentController
                || !input.role_ids.is_empty()
                || !input.avatar_ids.is_empty()
                || !input.battle_event_ids.is_empty()
        }
    };
    input.stable_key.is_empty()
        || input.source_path.is_empty()
        || !valid_sha256(&input.source_sha256)
        || invalid_binding
        || input.role_ids.windows(2).any(|pair| pair[0] >= pair[1])
        || input.avatar_ids.windows(2).any(|pair| pair[0] >= pair[1])
        || input
            .battle_event_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
}

fn definition<'a>(
    base: &'a CombatCatalog,
    aliases: &'a [EnemyDefinition],
    alias_ids: &BTreeSet<EnemyDefinitionId>,
    id: EnemyDefinitionId,
) -> Option<&'a EnemyDefinition> {
    base.enemy(id).or_else(|| {
        alias_ids
            .contains(&id)
            .then(|| aliases.iter().find(|alias| alias.id() == id))
            .flatten()
    })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

struct CurrencyWarsEnemyConfigurationInputs<'a> {
    character: &'a [CurrencyWarsEnemyCharacterConfigurationInput],
    ai: &'a [CurrencyWarsEnemyAiConfigurationInput],
}

fn resource_digest(
    base: &CombatCatalog,
    enemies: &[CurrencyWarsEnemyCombatInput],
    battle_behavior_programs: &[CurrencyWarsBattleBehaviorProgramInput],
    avatar_battle_behavior_programs: &[CurrencyWarsAvatarBattleBehaviorProgramInput],
    battle_program_bindings: &[CurrencyWarsBattleProgramBindingInput],
    enemy_configurations: CurrencyWarsEnemyConfigurationInputs<'_>,
    role_elements: &[(CurrencyWarsRoleId, CombatElement)],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.battle-resources.v7");
    hash.update(base.digest().bytes());
    hash.update((enemies.len() as u64).to_le_bytes());
    for enemy in enemies {
        hash.update((enemy.stable_key.len() as u64).to_le_bytes());
        hash.update(enemy.stable_key.as_bytes());
        hash.update([enemy.level.get()]);
        hash.update([enemy.stat_source_level.get()]);
        hash.update([enemy.behavior_source as u8]);
        hash.update(enemy.combatant.digest().bytes());
    }
    hash.update((battle_behavior_programs.len() as u64).to_le_bytes());
    for program in battle_behavior_programs {
        hash.update((program.stable_key.len() as u64).to_le_bytes());
        hash.update(program.stable_key.as_bytes());
        hash.update((program.source_path.len() as u64).to_le_bytes());
        hash.update(program.source_path.as_bytes());
        hash.update(program.source_sha256.as_bytes());
        hash.update([program.archetype as u8]);
        hash.update(program.definition.get().to_le_bytes());
        hash.update([program.behavior_source as u8]);
        hash.update(program.combatant.digest().bytes());
    }
    hash.update((avatar_battle_behavior_programs.len() as u64).to_le_bytes());
    for program in avatar_battle_behavior_programs {
        hash.update((program.stable_key.len() as u64).to_le_bytes());
        hash.update(program.stable_key.as_bytes());
        hash.update((program.source_path.len() as u64).to_le_bytes());
        hash.update(program.source_path.as_bytes());
        hash.update(program.source_sha256.as_bytes());
        hash.update([program.archetype as u8]);
        hash.update([program.binding_policy as u8]);
        hash.update((program.role_ids.len() as u64).to_le_bytes());
        for role in &program.role_ids {
            hash.update(role.get().to_le_bytes());
        }
        hash.update((program.avatar_ids.len() as u64).to_le_bytes());
        for avatar in &program.avatar_ids {
            hash.update(avatar.to_le_bytes());
        }
        hash.update((program.battle_event_ids.len() as u64).to_le_bytes());
        for event in &program.battle_event_ids {
            hash.update(event.to_le_bytes());
        }
    }
    hash.update((battle_program_bindings.len() as u64).to_le_bytes());
    for program in battle_program_bindings {
        hash.update((program.stable_key.len() as u64).to_le_bytes());
        hash.update(program.stable_key.as_bytes());
        hash.update((program.source_path.len() as u64).to_le_bytes());
        hash.update(program.source_path.as_bytes());
        hash.update(program.source_sha256.as_bytes());
        hash.update([program.archetype as u8]);
        hash.update(program.runtime_definition_count.to_le_bytes());
        hash.update((program.bindings.len() as u64).to_le_bytes());
        for binding in &program.bindings {
            let (tag, raw) = battle_program_binding_digest_key(*binding);
            hash.update([tag]);
            hash.update(raw.to_le_bytes());
        }
    }
    hash.update((enemy_configurations.character.len() as u64).to_le_bytes());
    for program in enemy_configurations.character {
        hash.update((program.stable_key.len() as u64).to_le_bytes());
        hash.update(program.stable_key.as_bytes());
        hash.update((program.source_path.len() as u64).to_le_bytes());
        hash.update(program.source_path.as_bytes());
        hash.update(program.source_sha256.as_bytes());
        hash.update((program.bindings.len() as u64).to_le_bytes());
        for binding in &program.bindings {
            hash.update((binding.shared_enemy_key.len() as u64).to_le_bytes());
            hash.update(binding.shared_enemy_key.as_bytes());
            hash.update(binding.source_template_id.to_le_bytes());
            hash.update(binding.definition.get().to_le_bytes());
        }
    }
    hash.update((enemy_configurations.ai.len() as u64).to_le_bytes());
    for program in enemy_configurations.ai {
        hash.update((program.stable_key.len() as u64).to_le_bytes());
        hash.update(program.stable_key.as_bytes());
        hash.update((program.source_path.len() as u64).to_le_bytes());
        hash.update(program.source_path.as_bytes());
        hash.update(program.source_sha256.as_bytes());
        hash.update((program.bindings.len() as u64).to_le_bytes());
        for binding in &program.bindings {
            hash.update((binding.shared_enemy_key.len() as u64).to_le_bytes());
            hash.update(binding.shared_enemy_key.as_bytes());
            hash.update(binding.source_template_id.to_le_bytes());
            hash.update(binding.definition.get().to_le_bytes());
        }
    }
    hash.update((role_elements.len() as u64).to_le_bytes());
    for (role, element) in role_elements {
        hash.update(role.get().to_le_bytes());
        hash.update([*element as u8]);
    }
    hash.finalize().into()
}

const fn battle_program_binding_digest_key(binding: CurrencyWarsBattleProgramBinding) -> (u8, u32) {
    match binding {
        CurrencyWarsBattleProgramBinding::Role(id) => (0, id.get()),
        CurrencyWarsBattleProgramBinding::Avatar(id) => (1, id),
        CurrencyWarsBattleProgramBinding::Servant(id) => (2, id),
        CurrencyWarsBattleProgramBinding::BattleEvent(id) => (3, id),
        CurrencyWarsBattleProgramBinding::Bond(id) => (4, id.get()),
        CurrencyWarsBattleProgramBinding::AugmentMazeBuff(id) => (5, id),
        CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(id) => (6, id),
        CurrencyWarsBattleProgramBinding::Equipment(id) => (7, id.get()),
    }
}
