//! Immutable Equation keyword programs and expanded battle contributions.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{ActivityStateHash, GraphActivity};
use starclock_data::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCatalog, DivergentUniverseEquationEffectId,
    DivergentUniverseEquationId, DivergentUniversePathType,
};

use crate::{path::ExactParameter, path_lowering::parse_decimal};

use super::{
    DivergentUniverseRuntimeFactory,
    equation_progress::{
        DivergentUniverseEquationExpansionState, DivergentUniverseEquationProgressError,
        DivergentUniverseEquationProgressRuntime,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationBattleAccuracy {
    ExactCurrentPathKeywordBindingsAndExpandedEquationContributions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationKeywordProgram {
    id: DivergentUniverseEquationEffectId,
    keyword_id: Box<str>,
    path: DivergentUniversePathType,
    maze_buff_id: Box<str>,
    maze_buff_ids: Box<[Box<str>]>,
    extra_effect_id: Box<str>,
    keyword_extra_effect_id: Box<str>,
    formula_source_ids: Box<[Box<str>]>,
    parameters: Box<[ExactParameter]>,
}

impl DivergentUniverseEquationKeywordProgram {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseEquationEffectId {
        &self.id
    }

    #[must_use]
    pub fn keyword_id(&self) -> &str {
        &self.keyword_id
    }

    #[must_use]
    pub const fn path(&self) -> &DivergentUniversePathType {
        &self.path
    }

    #[must_use]
    pub fn maze_buff_id(&self) -> &str {
        &self.maze_buff_id
    }

    #[must_use]
    pub fn maze_buff_ids(&self) -> &[Box<str>] {
        &self.maze_buff_ids
    }

    #[must_use]
    pub fn extra_effect_id(&self) -> &str {
        &self.extra_effect_id
    }

    #[must_use]
    pub fn keyword_extra_effect_id(&self) -> &str {
        &self.keyword_extra_effect_id
    }

    #[must_use]
    pub fn formula_source_ids(&self) -> &[Box<str>] {
        &self.formula_source_ids
    }

    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseExpandedEquationContribution {
    equation: DivergentUniverseEquationId,
    maze_buff_id: Box<str>,
    effect_ids: Box<[DivergentUniverseEquationEffectId]>,
}

impl DivergentUniverseExpandedEquationContribution {
    #[must_use]
    pub const fn equation(&self) -> &DivergentUniverseEquationId {
        &self.equation
    }

    #[must_use]
    pub fn maze_buff_id(&self) -> &str {
        &self.maze_buff_id
    }

    #[must_use]
    pub fn effect_ids(&self) -> &[DivergentUniverseEquationEffectId] {
        &self.effect_ids
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseEquationBattleSnapshotDigest([u8; 32]);

impl DivergentUniverseEquationBattleSnapshotDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationBattleSnapshot {
    source_state_hash: ActivityStateHash,
    keyword_programs: Arc<[DivergentUniverseEquationKeywordProgram]>,
    expanded_equations: Box<[DivergentUniverseExpandedEquationContribution]>,
    digest: DivergentUniverseEquationBattleSnapshotDigest,
}

impl DivergentUniverseEquationBattleSnapshot {
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }

    /// Complete exact current-Path keyword program catalog bound into this
    /// immutable battle input. Program presence does not by itself claim that
    /// every keyword fires for every expanded Equation.
    #[must_use]
    pub fn keyword_programs(&self) -> &[DivergentUniverseEquationKeywordProgram] {
        &self.keyword_programs
    }

    #[must_use]
    pub fn expanded_equations(&self) -> &[DivergentUniverseExpandedEquationContribution] {
        &self.expanded_equations
    }

    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseEquationBattleSnapshotDigest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EquationProjection {
    id: DivergentUniverseEquationId,
    maze_buff_id: Box<str>,
    effect_ids: Box<[DivergentUniverseEquationEffectId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationBattleRuntime {
    equations: Arc<[EquationProjection]>,
    keyword_programs: Arc<[DivergentUniverseEquationKeywordProgram]>,
    excluded_keyword_programs: u16,
    progress: Arc<DivergentUniverseEquationProgressRuntime>,
    component_digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn equation_battle_runtime(
        &self,
    ) -> Result<DivergentUniverseEquationBattleRuntime, DivergentUniverseEquationBattleError> {
        let progress = DivergentUniverseEquationProgressRuntime::compile(
            self.bundle.equation_catalog(),
            self.bundle.blessing_catalog(),
        )
        .map_err(DivergentUniverseEquationBattleError::Progress)?;
        DivergentUniverseEquationBattleRuntime::compile(
            self.bundle.equation_catalog(),
            Arc::new(progress),
            self.bundle.identity().component_digest().bytes(),
        )
    }
}

impl DivergentUniverseEquationBattleRuntime {
    fn compile(
        catalog: &DivergentUniverseEquationCatalog,
        progress: Arc<DivergentUniverseEquationProgressRuntime>,
        component_digest: [u8; 32],
    ) -> Result<Self, DivergentUniverseEquationBattleError> {
        let equations = catalog
            .equations()
            .iter()
            .map(|equation| {
                if equation.maze_buff_id.is_empty()
                    || equation.effect_ids.len() != 1
                    || equation.effect_ids[0].as_str()
                        != format!(
                            "divergent-universe.equation-effect.binding.{}",
                            equation.id.as_str().rsplit('.').next().unwrap_or_default()
                        )
                {
                    return Err(DivergentUniverseEquationBattleError::InvalidCatalog);
                }
                Ok(EquationProjection {
                    id: equation.id.clone(),
                    maze_buff_id: equation.maze_buff_id.clone(),
                    effect_ids: equation.effect_ids.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut excluded = 0_u16;
        let keyword_programs = catalog
            .effects()
            .iter()
            .filter_map(|effect| {
                if effect.runtime_lowered || !effect.rule_contribution_ids.is_empty() {
                    return Some(Err(DivergentUniverseEquationBattleError::InvalidCatalog));
                }
                if !effect.current_path {
                    excluded = excluded.saturating_add(1);
                    return None;
                }
                Some(compile_keyword(effect))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if equations.len() != 80
            || equations.windows(2).any(|pair| pair[0].id >= pair[1].id)
            || keyword_programs.len() != 23
            || excluded != 2
            || keyword_programs
                .windows(2)
                .any(|pair| pair[0].id >= pair[1].id)
            || keyword_programs
                .iter()
                .map(|program| program.parameters.len())
                .sum::<usize>()
                != 23
        {
            return Err(DivergentUniverseEquationBattleError::InvalidCatalog);
        }
        Ok(Self {
            equations: equations.into(),
            keyword_programs: keyword_programs.into(),
            excluded_keyword_programs: excluded,
            progress,
            component_digest,
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseEquationBattleAccuracy {
        DivergentUniverseEquationBattleAccuracy::ExactCurrentPathKeywordBindingsAndExpandedEquationContributions
    }

    #[must_use]
    pub fn keyword_programs(&self) -> &[DivergentUniverseEquationKeywordProgram] {
        &self.keyword_programs
    }

    #[must_use]
    pub const fn excluded_keyword_programs(&self) -> u16 {
        self.excluded_keyword_programs
    }

    pub fn snapshot(
        &self,
        activity: &GraphActivity,
    ) -> Result<DivergentUniverseEquationBattleSnapshot, DivergentUniverseEquationBattleError> {
        let progress = self
            .progress
            .observations(activity)
            .map_err(DivergentUniverseEquationBattleError::Progress)?;
        let expanded_equations = progress
            .iter()
            .filter(|observation| {
                observation.state() == DivergentUniverseEquationExpansionState::Expanded
            })
            .map(|observation| {
                let equation = self
                    .equations
                    .binary_search_by(|equation| equation.id.cmp(observation.equation()))
                    .ok()
                    .and_then(|index| self.equations.get(index))
                    .ok_or(DivergentUniverseEquationBattleError::InvalidCatalog)?;
                Ok(DivergentUniverseExpandedEquationContribution {
                    equation: equation.id.clone(),
                    maze_buff_id: equation.maze_buff_id.clone(),
                    effect_ids: equation.effect_ids.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let state_hash = activity.state_hash();
        let digest = snapshot_digest(
            state_hash,
            self.component_digest,
            &self.keyword_programs,
            &expanded_equations,
        )?;
        Ok(DivergentUniverseEquationBattleSnapshot {
            source_state_hash: state_hash,
            keyword_programs: Arc::clone(&self.keyword_programs),
            expanded_equations,
            digest,
        })
    }
}

fn compile_keyword(
    effect: &starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationEffectDefinition,
) -> Result<DivergentUniverseEquationKeywordProgram, DivergentUniverseEquationBattleError> {
    let path = DivergentUniversePathType::new(effect.path_type_id.clone())
        .map_err(|_| DivergentUniverseEquationBattleError::InvalidCatalog)?;
    if effect.keyword_id.is_empty()
        || effect.maze_buff_id.is_empty()
        || effect.extra_effect_id.is_empty()
        || effect.keyword_extra_effect_id.is_empty()
        || effect.formula_source_ids.is_empty()
        || effect
            .maze_buff_ids
            .iter()
            .chain(effect.formula_source_ids.iter())
            .any(|value| value.is_empty())
    {
        return Err(DivergentUniverseEquationBattleError::InvalidCatalog);
    }
    let parameters = effect
        .parameters
        .iter()
        .map(|parameter| {
            parse_decimal(parameter)
                .map_err(|_| DivergentUniverseEquationBattleError::InvalidCatalog)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DivergentUniverseEquationKeywordProgram {
        id: effect.id.clone(),
        keyword_id: effect.keyword_id.clone(),
        path,
        maze_buff_id: effect.maze_buff_id.clone(),
        maze_buff_ids: effect.maze_buff_ids.clone(),
        extra_effect_id: effect.extra_effect_id.clone(),
        keyword_extra_effect_id: effect.keyword_extra_effect_id.clone(),
        formula_source_ids: effect.formula_source_ids.clone(),
        parameters: parameters.into_boxed_slice(),
    })
}

fn snapshot_digest(
    state_hash: ActivityStateHash,
    component_digest: [u8; 32],
    programs: &[DivergentUniverseEquationKeywordProgram],
    expanded: &[DivergentUniverseExpandedEquationContribution],
) -> Result<DivergentUniverseEquationBattleSnapshotDigest, DivergentUniverseEquationBattleError> {
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.divergent-universe.equation-battle-snapshot.v1");
    hash.update(component_digest);
    hash.update(state_hash.bytes());
    push_len(&mut hash, programs.len())?;
    for program in programs {
        push_text(&mut hash, program.id.as_str())?;
        push_text(&mut hash, &program.keyword_id)?;
        push_text(&mut hash, program.path.as_str())?;
        push_text(&mut hash, &program.maze_buff_id)?;
        push_texts(&mut hash, &program.maze_buff_ids)?;
        push_text(&mut hash, &program.extra_effect_id)?;
        push_text(&mut hash, &program.keyword_extra_effect_id)?;
        push_texts(&mut hash, &program.formula_source_ids)?;
        push_len(&mut hash, program.parameters.len())?;
        for parameter in &program.parameters {
            hash.update(parameter.coefficient().to_le_bytes());
            hash.update([parameter.scale()]);
        }
    }
    push_len(&mut hash, expanded.len())?;
    for contribution in expanded {
        push_text(&mut hash, contribution.equation.as_str())?;
        push_text(&mut hash, &contribution.maze_buff_id)?;
        push_len(&mut hash, contribution.effect_ids.len())?;
        for effect in &contribution.effect_ids {
            push_text(&mut hash, effect.as_str())?;
        }
    }
    Ok(DivergentUniverseEquationBattleSnapshotDigest(
        hash.finalize(),
    ))
}

fn push_texts(
    hash: &mut CanonicalDigestBuilder,
    values: &[Box<str>],
) -> Result<(), DivergentUniverseEquationBattleError> {
    push_len(hash, values.len())?;
    for value in values {
        push_text(hash, value)?;
    }
    Ok(())
}

fn push_text(
    hash: &mut CanonicalDigestBuilder,
    value: &str,
) -> Result<(), DivergentUniverseEquationBattleError> {
    push_len(hash, value.len())?;
    hash.update(value.as_bytes());
    Ok(())
}

fn push_len(
    hash: &mut CanonicalDigestBuilder,
    value: usize,
) -> Result<(), DivergentUniverseEquationBattleError> {
    let value =
        u64::try_from(value).map_err(|_| DivergentUniverseEquationBattleError::InvalidCatalog)?;
    hash.update(value.to_le_bytes());
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationBattleError {
    InvalidCatalog,
    Progress(DivergentUniverseEquationProgressError),
}

impl core::fmt::Display for DivergentUniverseEquationBattleError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Equation battle snapshot error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseEquationBattleError {}
