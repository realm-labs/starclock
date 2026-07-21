//! Character build-row lowering into generated-type-free domain definitions.

use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{AbilityId, ModifierDefinitionId, RuleBundleId, Scalar, UnitDefinitionId};

use crate::{
    catalog::{
        CatalogLoadError, CombatDefinitions, IdentityDefinition, IdentityKind, LoadMode,
        contiguous, domain_fail, parse_decimal, positive, positive_u16, require_identity,
    },
    generated::SoraConfig,
};

#[derive(Debug)]
pub(super) struct BuildDefinitions {
    pub(super) characters: Box<[CharacterDataDefinition]>,
    pub(super) light_cones: Box<[crate::light_cone_lower::LightConeDataDefinition]>,
}

/// Generated-row-free production character definition retained by the data catalog.
#[derive(Debug)]
pub struct CharacterDataDefinition {
    pub(super) id: UnitDefinitionId,
    pub(super) rarity: u8,
    pub(super) path: u8,
    pub(super) element: u8,
    pub(super) base_energy: Scalar,
    pub(super) base_aggro: Scalar,
    pub(super) stats: Box<[CharacterStatDefinition]>,
    pub(super) abilities: Box<[CharacterAbilityDefinition]>,
    pub(super) innate_rule_bundles: Box<[RuleBundleId]>,
    pub(super) resources: Box<[CharacterResourceDefinition]>,
    pub(super) ability_parameters: Box<[AbilityParameterDefinition]>,
    pub(super) traces: Box<[TraceDefinition]>,
    pub(super) eidolons: Box<[EidolonDefinition]>,
    pub(super) complete_progression_required: bool,
}

#[derive(Debug)]
pub(super) struct CharacterStatDefinition {
    pub(super) level: u16,
    pub(super) promotion: u8,
    pub(super) hp: Scalar,
    pub(super) attack: Scalar,
    pub(super) defense: Scalar,
    pub(super) speed: Scalar,
}

#[derive(Debug)]
pub(super) struct CharacterAbilityDefinition {
    pub(super) sequence: u16,
    pub(super) slot: u8,
    pub(super) ability: AbilityId,
    pub(super) invested_level_cap: u16,
    pub(super) effective_level_cap: u16,
}

#[derive(Debug)]
pub(super) struct CharacterResourceDefinition {
    pub(super) sequence: u16,
    pub(super) stable_key: Box<str>,
    pub(super) maximum: Scalar,
    pub(super) initial: Scalar,
}

#[derive(Debug)]
pub(super) struct AbilityParameterDefinition {
    pub(super) ability: AbilityId,
    pub(super) effective_level: u16,
    pub(super) parameter_key: Box<str>,
    pub(super) value: Scalar,
}

#[derive(Debug)]
pub(super) struct TraceDefinition {
    pub(super) id: u32,
    pub(super) kind: u8,
    pub(super) promotion_requirement: u8,
    pub(super) prerequisites: Box<[u32]>,
    pub(super) patches: Box<[DataBuildPatch]>,
}

#[derive(Debug)]
pub(super) struct EidolonDefinition {
    pub(super) id: u32,
    pub(super) rank: u8,
    pub(super) patches: Box<[DataBuildPatch]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DataBuildPatch {
    AddRule(RuleBundleId),
    RemoveRule(RuleBundleId),
    AddModifier(ModifierDefinitionId),
    AddAbility(AbilityId),
    ReplaceAbility {
        old: AbilityId,
        new: AbilityId,
    },
    AdjustAbilityLevel {
        ability: AbilityId,
        bonus: i8,
        cap_delta: i8,
    },
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
                || character.innate_rule_bundles.iter().any(|id| id.get() == 0)
                || (character.complete_progression_required
                    && (character.traces.is_empty() || character.eidolons.len() != 6))
                || character.resources.iter().any(|resource| {
                    resource.sequence == 0
                        || resource.stable_key.is_empty()
                        || resource.maximum.scaled() < 0
                        || resource.initial.scaled() < 0
                        || resource.initial > resource.maximum
                })
                || character.ability_parameters.iter().any(|parameter| {
                    parameter.effective_level == 0
                        || parameter.parameter_key.is_empty()
                        || parameter.value.scaled() < i64::MIN / 2
                })
                || character.traces.iter().any(|trace| {
                    trace.id == 0
                        || trace.kind > 4
                        || trace.promotion_requirement > 6
                        || trace.prerequisites.contains(&trace.id)
                        || trace
                            .patches
                            .iter()
                            .any(DataBuildPatch::violates_invariants)
                })
                || character
                    .eidolons
                    .iter()
                    .enumerate()
                    .any(|(index, eidolon)| {
                        eidolon.id == 0
                            || usize::from(eidolon.rank) != index + 1
                            || eidolon
                                .patches
                                .iter()
                                .any(DataBuildPatch::violates_invariants)
                    })
                || character.abilities.iter().any(|binding| {
                    binding.slot > 8
                        || binding.invested_level_cap == 0
                        || binding.invested_level_cap > binding.effective_level_cap
                        || combat
                            .ability_level_cap(binding.ability)
                            .is_none_or(|level_cap| level_cap != binding.effective_level_cap)
                })
        }) || self
            .light_cones
            .iter()
            .any(crate::light_cone_lower::LightConeDataDefinition::violates_invariants)
    }
}

impl CharacterDataDefinition {
    #[must_use]
    pub const fn id(&self) -> UnitDefinitionId {
        self.id
    }

    #[must_use]
    pub fn stat_row_count(&self) -> usize {
        self.stats.len()
    }

    #[must_use]
    pub fn ability_count(&self) -> usize {
        self.abilities.len()
    }

    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    #[must_use]
    pub fn ability_parameter_count(&self) -> usize {
        self.ability_parameters.len()
    }

    #[must_use]
    pub fn trace_count(&self) -> usize {
        self.traces.len()
    }

    #[must_use]
    pub fn eidolon_count(&self) -> usize {
        self.eidolons.len()
    }
}

impl DataBuildPatch {
    fn violates_invariants(&self) -> bool {
        match *self {
            Self::AddRule(id) | Self::RemoveRule(id) => id.get() == 0,
            Self::AddModifier(id) => id.get() == 0,
            Self::AddAbility(id) => id.get() == 0,
            Self::ReplaceAbility { old, new } => old.get() == 0 || new.get() == 0 || old == new,
            Self::AdjustAbilityLevel {
                ability,
                bonus,
                cap_delta,
            } => ability.get() == 0 || bonus == 0 || cap_delta == 0,
        }
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
                let effective_level_cap = combat
                    .ability_level_cap(ability)
                    .ok_or_else(|| domain_fail("character binding refers to a missing ability"))?;
                if effective_level_cap == 0 {
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
                    effective_level_cap,
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
        let mut innate_rule_bundles = abilities
            .iter()
            .filter_map(|binding| {
                combat
                    .abilities
                    .binary_search_by_key(&binding.ability, |ability| ability.id)
                    .ok()
                    .and_then(|index| combat.abilities[index].entry_rule)
                    .map(|id| RuleBundleId::new(id.get()).expect("rule ID is nonzero"))
            })
            .collect::<Vec<_>>();
        innate_rule_bundles.sort_unstable();
        innate_rule_bundles.dedup();
        let resources = lower_resources(config, row.id)?;
        let ability_parameters = lower_ability_parameters(config, &abilities)?;
        let traces = lower_traces(config, row.id, mode, identities, combat)?;
        let eidolons = lower_eidolons(config, row.id, mode, identities, combat)?;
        characters.push(CharacterDataDefinition {
            id: UnitDefinitionId::new(id).expect("positive u32 is a valid UnitDefinitionId"),
            rarity,
            path: combat_path(row.path),
            element: combat_element(row.element),
            base_energy: Scalar::from_scaled(parse_decimal(&row.base_energy_decimal)?),
            base_aggro: Scalar::from_scaled(parse_decimal(&row.base_aggro_decimal)?),
            stats: stats.into_boxed_slice(),
            abilities: abilities.into_boxed_slice(),
            innate_rule_bundles: innate_rule_bundles.into_boxed_slice(),
            resources: resources.into_boxed_slice(),
            ability_parameters: ability_parameters.into_boxed_slice(),
            traces: traces.into_boxed_slice(),
            eidolons: eidolons.into_boxed_slice(),
            complete_progression_required: mode == LoadMode::Production,
        });
    }
    characters.sort_unstable_by_key(|character| character.id);
    let light_cones = crate::light_cone_lower::convert(config, mode, identities, combat)?;
    Ok(BuildDefinitions {
        characters: characters.into_boxed_slice(),
        light_cones: light_cones.into_boxed_slice(),
    })
}

fn lower_resources(
    config: &SoraConfig,
    character_id: i32,
) -> Result<Vec<CharacterResourceDefinition>, CatalogLoadError> {
    let mut resources = config
        .character_resource()
        .iter()
        .filter(|resource| resource.character_id == character_id)
        .map(|resource| {
            let maximum = Scalar::from_scaled(parse_decimal(&resource.maximum_decimal)?);
            let initial = Scalar::from_scaled(parse_decimal(&resource.initial_decimal)?);
            if resource.stable_key.trim().is_empty()
                || maximum.scaled() < 0
                || initial.scaled() < 0
                || initial > maximum
            {
                return Err(domain_fail("invalid character resource definition"));
            }
            Ok(CharacterResourceDefinition {
                sequence: positive_u16(resource.sequence, "CharacterResource.sequence")?,
                stable_key: resource.stable_key.clone().into_boxed_str(),
                maximum,
                initial,
            })
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    resources.sort_unstable_by_key(|resource| resource.sequence);
    contiguous(
        resources.iter().map(|resource| resource.sequence),
        "character resources",
    )?;
    if resources
        .iter()
        .map(|resource| resource.stable_key.as_ref())
        .collect::<BTreeSet<_>>()
        .len()
        != resources.len()
    {
        return Err(domain_fail("duplicate character resource stable key"));
    }
    Ok(resources)
}

fn lower_ability_parameters(
    config: &SoraConfig,
    abilities: &[CharacterAbilityDefinition],
) -> Result<Vec<AbilityParameterDefinition>, CatalogLoadError> {
    let ability_ids = abilities
        .iter()
        .map(|binding| binding.ability.get())
        .collect::<BTreeSet<_>>();
    let mut parameters = config
        .ability_level_parameter()
        .iter()
        .filter(|parameter| {
            u32::try_from(parameter.ability_id)
                .ok()
                .is_some_and(|id| ability_ids.contains(&id))
        })
        .map(|parameter| {
            if parameter.parameter_key.trim().is_empty() {
                return Err(domain_fail("empty ability parameter key"));
            }
            Ok(AbilityParameterDefinition {
                ability: AbilityId::new(positive(
                    parameter.ability_id,
                    "AbilityLevelParameter.ability_id",
                )?)
                .expect("positive ability ID"),
                effective_level: positive_u16(
                    parameter.effective_level,
                    "AbilityLevelParameter.effective_level",
                )?,
                parameter_key: parameter.parameter_key.clone().into_boxed_str(),
                value: Scalar::from_scaled(parse_decimal(&parameter.value_decimal)?),
            })
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    parameters.sort_unstable_by(|left, right| {
        (
            left.ability,
            left.effective_level,
            left.parameter_key.as_ref(),
        )
            .cmp(&(
                right.ability,
                right.effective_level,
                right.parameter_key.as_ref(),
            ))
    });
    for binding in abilities {
        let has_parameters = parameters
            .iter()
            .any(|parameter| parameter.ability == binding.ability);
        for level in 1..=binding.invested_level_cap {
            if has_parameters
                && !parameters.iter().any(|parameter| {
                    parameter.ability == binding.ability && parameter.effective_level == level
                })
            {
                return Err(domain_fail(format!(
                    "ability {} lacks parameters at level {level}",
                    binding.ability.get()
                )));
            }
        }
    }
    Ok(parameters)
}

fn lower_traces(
    config: &SoraConfig,
    character_id: i32,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    combat: &CombatDefinitions,
) -> Result<Vec<TraceDefinition>, CatalogLoadError> {
    let rows = config
        .trace_node()
        .ordered_rows()
        .filter(|trace| trace.character_id == character_id)
        .collect::<Vec<_>>();
    if rows.is_empty() {
        if mode != LoadMode::Production {
            return Ok(Vec::new());
        }
        return Err(domain_fail(format!(
            "character {character_id} has no Trace graph"
        )));
    }
    let owned_ids = rows
        .iter()
        .map(|row| positive(row.id, "TraceNode.id"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut traces = Vec::with_capacity(rows.len());
    for row in rows {
        let id = positive(row.id, "TraceNode.id")?;
        require_identity(identities, id, IdentityKind::Other, mode)?;
        let mut prerequisites = row
            .prerequisite_trace_ids
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|value| positive(*value, "TraceNode.prerequisite_trace_ids"))
            .collect::<Result<Vec<_>, _>>()?;
        prerequisites.sort_unstable();
        if prerequisites.windows(2).any(|pair| pair[0] >= pair[1])
            || prerequisites.iter().any(|value| !owned_ids.contains(value))
        {
            return Err(domain_fail(format!(
                "Trace {id} prerequisites {prerequisites:?} cross character or duplicate"
            )));
        }
        let patches = lower_patches(
            config
                .trace_patch()
                .iter()
                .filter(|patch| patch.trace_id == row.id)
                .map(|patch| (patch.sequence, &patch.patch)),
            combat,
            "TracePatch",
        )?;
        traces.push(TraceDefinition {
            id,
            kind: trace_kind(row.kind),
            promotion_requirement: u8::try_from(row.promotion_requirement)
                .map_err(|_| domain_fail("Trace promotion does not fit u8"))?,
            prerequisites: prerequisites.into_boxed_slice(),
            patches: patches.into_boxed_slice(),
        });
    }
    traces.sort_unstable_by_key(|trace| trace.id);
    validate_trace_graph(&traces)?;
    Ok(traces)
}

fn lower_eidolons(
    config: &SoraConfig,
    character_id: i32,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    combat: &CombatDefinitions,
) -> Result<Vec<EidolonDefinition>, CatalogLoadError> {
    let mut eidolons = config
        .eidolon()
        .ordered_rows()
        .filter(|eidolon| eidolon.character_id == character_id)
        .map(|row| {
            let id = positive(row.id, "Eidolon.id")?;
            require_identity(identities, id, IdentityKind::Other, mode)?;
            let patches = lower_patches(
                config
                    .eidolon_patch()
                    .iter()
                    .filter(|patch| patch.eidolon_id == row.id)
                    .map(|patch| (patch.sequence, &patch.patch)),
                combat,
                "EidolonPatch",
            )?;
            Ok(EidolonDefinition {
                id,
                rank: u8::try_from(row.rank)
                    .map_err(|_| domain_fail("Eidolon rank does not fit u8"))?,
                patches: patches.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    eidolons.sort_unstable_by_key(|eidolon| eidolon.rank);
    if mode != LoadMode::Production && eidolons.is_empty() {
        return Ok(eidolons);
    }
    if eidolons.len() != 6
        || eidolons
            .iter()
            .enumerate()
            .any(|(index, eidolon)| usize::from(eidolon.rank) != index + 1)
    {
        return Err(domain_fail(format!(
            "character {character_id} lacks an exact E1-E6 set"
        )));
    }
    Ok(eidolons)
}

fn lower_patches<'a>(
    rows: impl Iterator<Item = (i32, &'a crate::generated::build_patch::BuildPatch)>,
    combat: &CombatDefinitions,
    label: &str,
) -> Result<Vec<DataBuildPatch>, CatalogLoadError> {
    let mut rows = rows.collect::<Vec<_>>();
    rows.sort_unstable_by_key(|(sequence, _)| *sequence);
    contiguous(
        rows.iter()
            .map(|(sequence, _)| positive_u16(*sequence, label))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter(),
        label,
    )?;
    rows.into_iter()
        .map(|(_, patch)| match patch {
            crate::generated::build_patch::BuildPatch::AddRule { rule_identity_id } => {
                let raw = positive(*rule_identity_id, "BuildPatch.rule_identity_id")?;
                let rule = RuleBundleId::new(raw).expect("positive rule bundle ID");
                if combat
                    .rules
                    .binary_search_by_key(&raw, |entry| entry.id.get())
                    .is_err()
                {
                    return Err(domain_fail("build patch refers to a missing rule"));
                }
                Ok(DataBuildPatch::AddRule(rule))
            }
            crate::generated::build_patch::BuildPatch::RemoveRule { rule_identity_id } => {
                let raw = positive(*rule_identity_id, "BuildPatch.rule_identity_id")?;
                let rule = RuleBundleId::new(raw).expect("positive rule bundle ID");
                if combat
                    .rules
                    .binary_search_by_key(&raw, |entry| entry.id.get())
                    .is_err()
                {
                    return Err(domain_fail("build patch refers to a missing rule"));
                }
                Ok(DataBuildPatch::RemoveRule(rule))
            }
            crate::generated::build_patch::BuildPatch::AddModifier {
                modifier_identity_id,
            } => {
                let modifier = ModifierDefinitionId::new(positive(
                    *modifier_identity_id,
                    "BuildPatch.modifier_identity_id",
                )?)
                .expect("positive modifier ID");
                if combat.modifiers.definition(modifier).is_none() {
                    return Err(domain_fail("build patch refers to a missing modifier"));
                }
                Ok(DataBuildPatch::AddModifier(modifier))
            }
            crate::generated::build_patch::BuildPatch::AddAbility { ability_id } => {
                let ability = AbilityId::new(positive(*ability_id, "BuildPatch.ability_id")?)
                    .expect("positive ability ID");
                if combat.ability_level_cap(ability).is_none() {
                    return Err(domain_fail("build patch refers to a missing ability"));
                }
                Ok(DataBuildPatch::AddAbility(ability))
            }
            crate::generated::build_patch::BuildPatch::ReplaceAbility {
                old_ability_id,
                new_ability_id,
            } => {
                let old = AbilityId::new(positive(*old_ability_id, "BuildPatch.old_ability_id")?)
                    .expect("positive ability ID");
                let new = AbilityId::new(positive(*new_ability_id, "BuildPatch.new_ability_id")?)
                    .expect("positive ability ID");
                if old == new
                    || combat.ability_level_cap(old).is_none()
                    || combat.ability_level_cap(new).is_none()
                {
                    return Err(domain_fail("invalid replacement ability patch"));
                }
                Ok(DataBuildPatch::ReplaceAbility { old, new })
            }
            crate::generated::build_patch::BuildPatch::AdjustAbilityLevel {
                ability_id,
                bonus,
                cap_delta,
            } => {
                let ability = AbilityId::new(positive(*ability_id, "BuildPatch.ability_id")?)
                    .expect("positive ability ID");
                if combat.ability_level_cap(ability).is_none() {
                    return Err(domain_fail("build patch refers to a missing ability"));
                }
                let bonus = i8::try_from(*bonus)
                    .map_err(|_| domain_fail("ability-level bonus does not fit i8"))?;
                let cap_delta = i8::try_from(*cap_delta)
                    .map_err(|_| domain_fail("ability-level cap delta does not fit i8"))?;
                if bonus == 0 || cap_delta == 0 {
                    return Err(domain_fail("ability-level patch must change level and cap"));
                }
                Ok(DataBuildPatch::AdjustAbilityLevel {
                    ability,
                    bonus,
                    cap_delta,
                })
            }
            _ => Err(domain_fail(format!(
                "{label} uses a patch outside the executable build subset"
            ))),
        })
        .collect()
}

fn validate_trace_graph(traces: &[TraceDefinition]) -> Result<(), CatalogLoadError> {
    let mut visited = BTreeSet::new();
    while visited.len() < traces.len() {
        let before = visited.len();
        for trace in traces {
            if !visited.contains(&trace.id)
                && trace
                    .prerequisites
                    .iter()
                    .all(|prerequisite| visited.contains(prerequisite))
            {
                visited.insert(trace.id);
            }
        }
        if visited.len() == before {
            return Err(domain_fail("Trace graph contains a cycle"));
        }
    }
    Ok(())
}

fn trace_kind(value: crate::generated::trace_node_kind::TraceNodeKind) -> u8 {
    use crate::generated::trace_node_kind::TraceNodeKind as V;
    match value {
        V::MajorPassive => 0,
        V::MinorStat => 1,
        V::AbilityUnlock => 2,
        V::AbilityLevel => 3,
        V::BasicLevel => 4,
    }
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
