//! Character build-row lowering into generated-type-free domain definitions.

use std::collections::BTreeMap;

use starclock_combat::{AbilityId, Scalar, UnitDefinitionId};

use crate::{
    catalog::{
        CatalogLoadError, CombatDefinitions, IdentityDefinition, IdentityKind, LoadMode,
        contiguous, domain_fail, parse_decimal, positive, positive_u16, require_identity,
    },
    generated::SoraConfig,
};

#[derive(Debug)]
pub(super) struct BuildDefinitions {
    pub(super) characters: Box<[CharacterDefinition]>,
}

#[derive(Debug)]
pub(super) struct CharacterDefinition {
    pub(super) id: UnitDefinitionId,
    rarity: u8,
    path: u8,
    element: u8,
    pub(super) base_energy: Scalar,
    pub(super) base_aggro: Scalar,
    pub(super) stats: Box<[CharacterStatDefinition]>,
    pub(super) abilities: Box<[CharacterAbilityDefinition]>,
}

#[derive(Debug)]
pub(super) struct CharacterStatDefinition {
    level: u16,
    promotion: u8,
    hp: Scalar,
    attack: Scalar,
    defense: Scalar,
    speed: Scalar,
}

#[derive(Debug)]
pub(super) struct CharacterAbilityDefinition {
    sequence: u16,
    slot: u8,
    pub(super) ability: AbilityId,
    invested_level_cap: u16,
}

impl BuildDefinitions {
    pub(super) fn len(&self) -> usize {
        self.characters.len()
    }

    pub(super) fn violates_invariants(&self, combat: &CombatDefinitions) -> bool {
        self.characters.iter().any(|character| {
            !(4..=5).contains(&character.rarity)
                || character.path > 8
                || character.element > 6
                || character.base_energy.scaled() < 0
                || character.base_aggro.scaled() < 0
                || character.stats.is_empty()
                || character.abilities.is_empty()
                || character.abilities.iter().any(|binding| {
                    binding.slot > 8
                        || combat
                            .ability_level_cap(binding.ability)
                            .is_none_or(|level_cap| {
                                binding.invested_level_cap == 0
                                    || binding.invested_level_cap > level_cap
                            })
                })
        })
    }
}

pub(super) fn convert(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    combat: &CombatDefinitions,
) -> Result<BuildDefinitions, CatalogLoadError> {
    let mut characters = Vec::new();
    for row in config.character().ordered_rows() {
        let id = positive(row.id, "Character.id")?;
        require_identity(identities, id, IdentityKind::Character, mode)?;
        let rarity = u8::try_from(row.rarity)
            .ok()
            .filter(|rarity| (4..=5).contains(rarity))
            .ok_or_else(|| domain_fail("invalid character rarity"))?;
        let mut stats = config
            .character_stat()
            .iter()
            .filter(|stat| stat.character_id == row.id)
            .map(|stat| {
                Ok(CharacterStatDefinition {
                    level: positive_u16(stat.level, "CharacterStat.level")?,
                    promotion: u8::try_from(stat.promotion)
                        .map_err(|_| domain_fail("invalid character promotion"))?,
                    hp: Scalar::from_scaled(parse_decimal(&stat.hp_decimal)?),
                    attack: Scalar::from_scaled(parse_decimal(&stat.atk_decimal)?),
                    defense: Scalar::from_scaled(parse_decimal(&stat.def_decimal)?),
                    speed: Scalar::from_scaled(parse_decimal(&stat.spd_decimal)?),
                })
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        stats.sort_unstable_by_key(|stat| (stat.level, stat.promotion));
        if stats
            .iter()
            .any(|stat| stat.level > 80 || stat.promotion > 6)
            || !stats
                .iter()
                .any(|stat| stat.level == 1 && stat.promotion == 0)
            || !stats
                .iter()
                .any(|stat| stat.level == 80 && stat.promotion == 6)
            || stats.iter().any(|stat| {
                stat.hp.scaled() <= 0
                    || stat.attack.scaled() <= 0
                    || stat.defense.scaled() <= 0
                    || stat.speed.scaled() <= 0
            })
        {
            return Err(domain_fail(format!(
                "character {} lacks valid level 1/80 stat boundaries",
                row.id
            )));
        }
        let mut abilities = config
            .character_ability_binding()
            .iter()
            .filter(|binding| binding.character_id == row.id)
            .map(|binding| {
                let ability_raw =
                    positive(binding.ability_id, "CharacterAbilityBinding.ability_id")?;
                let ability = AbilityId::new(ability_raw).expect("positive ability ID");
                if combat.ability_level_cap(ability).is_none() {
                    return Err(domain_fail("character binding refers to a missing ability"));
                }
                Ok(CharacterAbilityDefinition {
                    sequence: positive_u16(binding.sequence, "CharacterAbilityBinding.sequence")?,
                    slot: character_ability_slot(binding.slot),
                    ability,
                    invested_level_cap: positive_u16(
                        binding.invested_level_cap,
                        "CharacterAbilityBinding.invested_level_cap",
                    )?,
                })
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        abilities.sort_unstable_by_key(|binding| binding.sequence);
        contiguous(
            abilities.iter().map(|binding| binding.sequence),
            "character abilities",
        )?;
        if abilities.is_empty() {
            return Err(domain_fail(format!(
                "character {} has no ability binding",
                row.id
            )));
        }
        characters.push(CharacterDefinition {
            id: UnitDefinitionId::new(id).expect("positive u32 is a valid UnitDefinitionId"),
            rarity,
            path: combat_path(row.path),
            element: combat_element(row.element),
            base_energy: Scalar::from_scaled(parse_decimal(&row.base_energy_decimal)?),
            base_aggro: Scalar::from_scaled(parse_decimal(&row.base_aggro_decimal)?),
            stats: stats.into_boxed_slice(),
            abilities: abilities.into_boxed_slice(),
        });
    }
    characters.sort_unstable_by_key(|character| character.id);
    Ok(BuildDefinitions {
        characters: characters.into_boxed_slice(),
    })
}

fn character_ability_slot(
    value: crate::generated::character_ability_slot::CharacterAbilitySlot,
) -> u8 {
    use crate::generated::character_ability_slot::CharacterAbilitySlot as V;
    match value {
        V::Basic => 0,
        V::Skill => 1,
        V::Ultimate => 2,
        V::Talent => 3,
        V::Technique => 4,
        V::Enhanced => 5,
        V::Summon => 6,
        V::Memosprite => 7,
        V::Passive => 8,
    }
}

fn combat_path(value: crate::generated::combat_path::CombatPath) -> u8 {
    use crate::generated::combat_path::CombatPath as V;
    match value {
        V::Destruction => 0,
        V::Hunt => 1,
        V::Erudition => 2,
        V::Harmony => 3,
        V::Nihility => 4,
        V::Preservation => 5,
        V::Abundance => 6,
        V::Remembrance => 7,
        V::Elation => 8,
    }
}

fn combat_element(value: crate::generated::combat_element::CombatElement) -> u8 {
    use crate::generated::combat_element::CombatElement as V;
    match value {
        V::Physical => 0,
        V::Fire => 1,
        V::Ice => 2,
        V::Lightning => 3,
        V::Wind => 4,
        V::Quantum => 5,
        V::Imaginary => 6,
    }
}
