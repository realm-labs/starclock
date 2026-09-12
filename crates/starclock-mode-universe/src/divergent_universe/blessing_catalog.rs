//! Exact Version 4.4 Divergent Blessing identities, levels, rewrites and groups.

use std::collections::BTreeSet;
use std::sync::Arc;

use starclock_data::divergent_universe_blessing_catalog::{
    DivergentUniverseBlessingCatalog, DivergentUniverseBlessingCategory,
    DivergentUniverseBlessingGroupDefinition, DivergentUniverseBlessingGroupId,
    DivergentUniverseBlessingId, DivergentUniverseBlessingLevelDefinition,
    DivergentUniverseBlessingLevelId, DivergentUniverseBlessingState,
};
use starclock_data::divergent_universe_equation_catalog::DivergentUniversePathType;

use crate::path::ExactParameter;
use crate::path_lowering::parse_decimal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBlessingAccuracy {
    ExactReleasedPathsIdentitiesLevelsEnhancementsAndClosedGroups,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingLevelRuntime {
    id: DivergentUniverseBlessingLevelId,
    blessing: DivergentUniverseBlessingId,
    level: u16,
    state: DivergentUniverseBlessingState,
    binding_key: Box<str>,
    binding_type: Box<str>,
    modifier_name: Box<str>,
    parameters: Box<[ExactParameter]>,
    path: DivergentUniversePathType,
    rogue_buff_tag: Box<str>,
    extra_effect_ids: Box<[Box<str>]>,
    equation_contribution_identity: Box<str>,
}

impl DivergentUniverseBlessingLevelRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseBlessingLevelId {
        &self.id
    }
    #[must_use]
    pub const fn blessing(&self) -> &DivergentUniverseBlessingId {
        &self.blessing
    }
    #[must_use]
    pub const fn level(&self) -> u16 {
        self.level
    }
    #[must_use]
    pub const fn state(&self) -> DivergentUniverseBlessingState {
        self.state
    }
    #[must_use]
    pub fn binding_key(&self) -> &str {
        &self.binding_key
    }
    #[must_use]
    pub fn binding_type(&self) -> &str {
        &self.binding_type
    }
    #[must_use]
    pub fn modifier_name(&self) -> &str {
        &self.modifier_name
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
    #[must_use]
    pub const fn path(&self) -> &DivergentUniversePathType {
        &self.path
    }
    #[must_use]
    pub fn rogue_buff_tag(&self) -> &str {
        &self.rogue_buff_tag
    }
    #[must_use]
    pub fn extra_effect_ids(&self) -> &[Box<str>] {
        &self.extra_effect_ids
    }
    #[must_use]
    pub fn equation_contribution_identity(&self) -> &str {
        &self.equation_contribution_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingRuntimeDefinition {
    id: DivergentUniverseBlessingId,
    state_key: u64,
    category: DivergentUniverseBlessingCategory,
    path: DivergentUniversePathType,
    effect_ids: Box<[Box<str>]>,
    levels: [DivergentUniverseBlessingLevelRuntime; 2],
}

impl DivergentUniverseBlessingRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseBlessingId {
        &self.id
    }
    #[must_use]
    pub const fn state_key(&self) -> u64 {
        self.state_key
    }
    #[must_use]
    pub const fn category(&self) -> DivergentUniverseBlessingCategory {
        self.category
    }
    #[must_use]
    pub const fn path(&self) -> &DivergentUniversePathType {
        &self.path
    }
    #[must_use]
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }
    #[must_use]
    pub const fn levels(&self) -> &[DivergentUniverseBlessingLevelRuntime; 2] {
        &self.levels
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingEnhancementRuntime {
    blessing: DivergentUniverseBlessingId,
    input: DivergentUniverseBlessingLevelId,
    output: DivergentUniverseBlessingLevelId,
}

impl DivergentUniverseBlessingEnhancementRuntime {
    #[must_use]
    pub const fn blessing(&self) -> &DivergentUniverseBlessingId {
        &self.blessing
    }
    #[must_use]
    pub const fn input(&self) -> &DivergentUniverseBlessingLevelId {
        &self.input
    }
    #[must_use]
    pub const fn output(&self) -> &DivergentUniverseBlessingLevelId {
        &self.output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingOfferCandidate {
    blessing: DivergentUniverseBlessingId,
    level: DivergentUniverseBlessingLevelId,
    state_key: u64,
    level_number: u16,
    state: DivergentUniverseBlessingState,
}

impl DivergentUniverseBlessingOfferCandidate {
    #[must_use]
    pub const fn blessing(&self) -> &DivergentUniverseBlessingId {
        &self.blessing
    }
    #[must_use]
    pub const fn level_id(&self) -> &DivergentUniverseBlessingLevelId {
        &self.level
    }
    #[must_use]
    pub const fn level(&self) -> u16 {
        self.level_number
    }
    #[must_use]
    pub const fn state(&self) -> DivergentUniverseBlessingState {
        self.state
    }
    pub(super) const fn state_key(&self) -> u64 {
        self.state_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingGroupRuntime {
    id: DivergentUniverseBlessingGroupId,
    state_key: u64,
    candidates: Box<[DivergentUniverseBlessingOfferCandidate]>,
}

impl DivergentUniverseBlessingGroupRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseBlessingGroupId {
        &self.id
    }
    #[must_use]
    pub fn candidates(&self) -> &[DivergentUniverseBlessingOfferCandidate] {
        &self.candidates
    }
    pub(super) const fn state_key(&self) -> u64 {
        self.state_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CompiledBlessingCatalog {
    pub(super) blessings: Arc<[DivergentUniverseBlessingRuntimeDefinition]>,
    pub(super) enhancements: Arc<[DivergentUniverseBlessingEnhancementRuntime]>,
    pub(super) groups: Arc<[DivergentUniverseBlessingGroupRuntime]>,
    pub(super) path_count: u16,
}

impl CompiledBlessingCatalog {
    pub(super) fn compile(
        catalog: &DivergentUniverseBlessingCatalog,
        stable_group_key: impl Fn(&str) -> u64,
    ) -> Result<Self, BlessingCatalogCompileError> {
        let blessings = catalog
            .blessings()
            .iter()
            .enumerate()
            .map(|(index, blessing)| {
                let mut levels = catalog
                    .levels()
                    .iter()
                    .filter(|level| level.blessing == blessing.id)
                    .map(compile_level)
                    .collect::<Result<Vec<_>, _>>()?;
                levels.sort_unstable_by_key(DivergentUniverseBlessingLevelRuntime::level);
                let levels: [DivergentUniverseBlessingLevelRuntime; 2] =
                    levels.try_into().map_err(|_| BlessingCatalogCompileError)?;
                if blessing.runtime_lowered
                    || !blessing.handbook_visible
                    || levels[0].level != 1
                    || levels[0].state != DivergentUniverseBlessingState::Base
                    || levels[1].level != 2
                    || levels[1].state != DivergentUniverseBlessingState::Enhanced
                    || levels.iter().any(|level| {
                        level.path != blessing.path
                            || level.blessing != blessing.id
                            || level.equation_contribution_identity.as_ref()
                                != blessing.id.as_str().rsplit('.').next().unwrap_or_default()
                    })
                {
                    return Err(BlessingCatalogCompileError);
                }
                Ok(DivergentUniverseBlessingRuntimeDefinition {
                    id: blessing.id.clone(),
                    state_key: ordinal(index)?,
                    category: blessing.category,
                    path: blessing.path.clone(),
                    effect_ids: blessing.effect_ids.clone(),
                    levels,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let enhancements = catalog
            .rewrites()
            .iter()
            .filter(|rewrite| rewrite.timing.as_ref() == "AcceptedEnhanceOperation")
            .map(|rewrite| {
                let blessing = rewrite
                    .input
                    .as_ref()
                    .filter(|input| rewrite.output.as_ref() == Some(*input))
                    .ok_or(BlessingCatalogCompileError)?;
                let definition = blessings
                    .binary_search_by(|candidate| candidate.id.cmp(blessing))
                    .ok()
                    .map(|index| &blessings[index])
                    .ok_or(BlessingCatalogCompileError)?;
                if rewrite.candidate_policy.as_ref() != "ExactOwnedBlessing"
                    || rewrite.input_state.as_ref() != "Base"
                    || rewrite.output_state.as_ref() != "Enhanced"
                    || !rewrite.equation_identity_preserved
                    || rewrite.no_legal_candidate.as_ref() != "RejectWithoutMutation"
                    || rewrite.runtime_lowered
                {
                    return Err(BlessingCatalogCompileError);
                }
                Ok(DivergentUniverseBlessingEnhancementRuntime {
                    blessing: blessing.clone(),
                    input: definition.levels[0].id.clone(),
                    output: definition.levels[1].id.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let groups = catalog
            .groups()
            .iter()
            .map(|group| {
                let mut stack = Vec::new();
                let candidates = flatten_group(catalog, &blessings, group, &mut stack)?;
                let unique = candidates
                    .iter()
                    .map(|candidate| candidate.state_key)
                    .collect::<BTreeSet<_>>();
                if candidates.is_empty()
                    || candidates.len() > 144
                    || unique.len() != candidates.len()
                {
                    return Err(BlessingCatalogCompileError);
                }
                Ok(DivergentUniverseBlessingGroupRuntime {
                    id: group.id.clone(),
                    state_key: stable_group_key(group.id.as_str()),
                    candidates: candidates.into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let group_keys = groups
            .iter()
            .map(DivergentUniverseBlessingGroupRuntime::state_key)
            .collect::<BTreeSet<_>>();
        if catalog.paths().len() != 8
            || blessings.len() != 414
            || enhancements.len() != 414
            || groups.len() != 118
            || catalog.levels().len() != 828
            || catalog.rewrites().len() != 416
            || group_keys.len() != groups.len()
        {
            return Err(BlessingCatalogCompileError);
        }
        Ok(Self {
            blessings: blessings.into(),
            enhancements: enhancements.into(),
            groups: groups.into(),
            path_count: 8,
        })
    }
}

fn compile_level(
    level: &DivergentUniverseBlessingLevelDefinition,
) -> Result<DivergentUniverseBlessingLevelRuntime, BlessingCatalogCompileError> {
    if level.runtime_lowered
        || level.binding_key.is_empty()
        || level.binding_type.as_ref() != "StageAbilityBeforeCharacterBorn"
        || level.modifier_name.is_empty()
        || level.rogue_buff_tag.is_empty()
        || level.equation_contribution_identity.is_empty()
    {
        return Err(BlessingCatalogCompileError);
    }
    let parameters = level
        .parameters
        .iter()
        .map(|value| parse_decimal(value).map_err(|_| BlessingCatalogCompileError))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DivergentUniverseBlessingLevelRuntime {
        id: level.id.clone(),
        blessing: level.blessing.clone(),
        level: level.level,
        state: level.state,
        binding_key: level.binding_key.clone(),
        binding_type: level.binding_type.clone(),
        modifier_name: level.modifier_name.clone(),
        parameters: parameters.into_boxed_slice(),
        path: level.path.clone(),
        rogue_buff_tag: level.rogue_buff_tag.clone(),
        extra_effect_ids: level.extra_effect_ids.clone(),
        equation_contribution_identity: level.equation_contribution_identity.clone(),
    })
}

fn flatten_group(
    catalog: &DivergentUniverseBlessingCatalog,
    blessings: &[DivergentUniverseBlessingRuntimeDefinition],
    group: &DivergentUniverseBlessingGroupDefinition,
    stack: &mut Vec<DivergentUniverseBlessingGroupId>,
) -> Result<Vec<DivergentUniverseBlessingOfferCandidate>, BlessingCatalogCompileError> {
    if stack.contains(&group.id)
        || !matches!(
            group.membership_resolution.as_ref(),
            "ClosedModeOwned" | "ClosedModeOwnedOrNested"
        )
        || group.selection_policy.as_ref() != "OrderedSourceCandidates"
        || group.weight_program.as_ref() != "Unspecified"
        || !group.unresolved_source_ids.is_empty()
    {
        return Err(BlessingCatalogCompileError);
    }
    stack.push(group.id.clone());
    let mut direct = Vec::new();
    let mut subgroups = Vec::new();
    let mut candidates = Vec::new();
    for source in &group.source_candidate_ids {
        if let Some(level) = catalog
            .levels()
            .iter()
            .find(|level| level.rogue_buff_tag.as_ref() == source.as_ref())
        {
            direct.push(level.id.clone());
            candidates.push(candidate(blessings, level)?);
            continue;
        }
        let subgroup = catalog
            .groups()
            .iter()
            .find(|candidate| {
                candidate.id.as_str().rsplit('.').next().unwrap_or_default() == source.as_ref()
            })
            .ok_or(BlessingCatalogCompileError)?;
        subgroups.push(subgroup.id.clone());
        candidates.extend(flatten_group(catalog, blessings, subgroup, stack)?);
    }
    stack.pop();
    if direct.as_slice() != group.resolved_levels.as_ref()
        || subgroups.as_slice() != group.resolved_subgroups.as_ref()
    {
        return Err(BlessingCatalogCompileError);
    }
    Ok(candidates)
}

fn candidate(
    blessings: &[DivergentUniverseBlessingRuntimeDefinition],
    level: &DivergentUniverseBlessingLevelDefinition,
) -> Result<DivergentUniverseBlessingOfferCandidate, BlessingCatalogCompileError> {
    let definition = blessings
        .binary_search_by(|candidate| candidate.id.cmp(&level.blessing))
        .ok()
        .map(|index| &blessings[index])
        .ok_or(BlessingCatalogCompileError)?;
    Ok(DivergentUniverseBlessingOfferCandidate {
        blessing: level.blessing.clone(),
        level: level.id.clone(),
        state_key: definition.state_key,
        level_number: level.level,
        state: level.state,
    })
}

fn ordinal(index: usize) -> Result<u64, BlessingCatalogCompileError> {
    u64::try_from(index + 1)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(BlessingCatalogCompileError)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BlessingCatalogCompileError;
