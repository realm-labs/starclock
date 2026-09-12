//! Blessing-level battle inputs and explicit fail-closed interaction policies.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{ActivityStateHash, GraphActivity, GraphActivityCommandError};
use starclock_data::divergent_universe_blessing_catalog::{
    DivergentUniverseBlessingId, DivergentUniverseBlessingLevelId, DivergentUniverseBlessingState,
};
use starclock_data::divergent_universe_curio_catalog::{
    DivergentUniverseCurioId, DivergentUniverseCurioLifecycleId,
};
use starclock_data::divergent_universe_equation_catalog::DivergentUniversePathType;

use crate::path::ExactParameter;

use super::{
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseEquationBattleError, DivergentUniverseEquationBattleRuntime,
    DivergentUniverseEquationBattleSnapshot, DivergentUniverseRuntimeFactory,
};

const CURIO_FIXTURE_SOURCE: &str = "divergent-universe.curio-lifecycle.9001";
const TITAN_FIXTURE_SOURCE: &str = "divergent-universe.titan-contribution.boon.10101";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBlessingInteractionAccuracy {
    ExactBlessingLevelsAndEquationContributions,
    VersionedProjectPolicyExplicitAcceptedStableId,
    VersionedProjectPolicyStableIdAscendingSamePhase,
    VersionedProjectPolicyRejectUnprovenCurioLifecycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingBattleContribution {
    level: DivergentUniverseBlessingLevelId,
    blessing: DivergentUniverseBlessingId,
    level_number: u16,
    state: DivergentUniverseBlessingState,
    binding_key: Box<str>,
    binding_type: Box<str>,
    modifier_name: Box<str>,
    parameters: Box<[ExactParameter]>,
    path: DivergentUniversePathType,
    effect_ids: Box<[Box<str>]>,
    equation_contribution_identity: Box<str>,
}

impl DivergentUniverseBlessingBattleContribution {
    #[must_use]
    pub const fn level(&self) -> &DivergentUniverseBlessingLevelId {
        &self.level
    }
    #[must_use]
    pub const fn blessing(&self) -> &DivergentUniverseBlessingId {
        &self.blessing
    }
    #[must_use]
    pub const fn level_number(&self) -> u16 {
        self.level_number
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
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }
    #[must_use]
    pub fn equation_contribution_identity(&self) -> &str {
        &self.equation_contribution_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseBlessingInteractionSnapshotDigest([u8; 32]);

impl DivergentUniverseBlessingInteractionSnapshotDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingInteractionSnapshot {
    source_state_hash: ActivityStateHash,
    blessings: Box<[DivergentUniverseBlessingBattleContribution]>,
    equations: DivergentUniverseEquationBattleSnapshot,
    digest: DivergentUniverseBlessingInteractionSnapshotDigest,
}

impl DivergentUniverseBlessingInteractionSnapshot {
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }
    #[must_use]
    pub fn blessings(&self) -> &[DivergentUniverseBlessingBattleContribution] {
        &self.blessings
    }
    #[must_use]
    pub const fn equations(&self) -> &DivergentUniverseEquationBattleSnapshot {
        &self.equations
    }
    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseBlessingInteractionSnapshotDigest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingServiceRewritePolicy {
    source_id: Box<str>,
    timing: Box<str>,
    equation_identity_preserved: bool,
}

impl DivergentUniverseBlessingServiceRewritePolicy {
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub fn timing(&self) -> &str {
        &self.timing
    }
    #[must_use]
    pub const fn equation_identity_preserved(&self) -> bool {
        self.equation_identity_preserved
    }
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseBlessingInteractionAccuracy {
        DivergentUniverseBlessingInteractionAccuracy::VersionedProjectPolicyExplicitAcceptedStableId
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioLifecyclePolicy {
    id: DivergentUniverseCurioLifecycleId,
    curio: DivergentUniverseCurioId,
    fallback: Box<str>,
}

impl DivergentUniverseCurioLifecyclePolicy {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseCurioLifecycleId {
        &self.id
    }
    #[must_use]
    pub const fn curio(&self) -> &DivergentUniverseCurioId {
        &self.curio
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseBlessingInteractionAccuracy {
        DivergentUniverseBlessingInteractionAccuracy::VersionedProjectPolicyRejectUnprovenCurioLifecycle
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurioLifecycleTransition {
    Activate,
    ConsumeCharge,
    Destroy,
    Repair,
    Replace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseSimultaneousContribution {
    source_id: Box<str>,
    stable_priority: u16,
}

impl DivergentUniverseSimultaneousContribution {
    #[must_use]
    pub fn new(source_id: impl Into<Box<str>>, stable_priority: u16) -> Self {
        Self {
            source_id: source_id.into(),
            stable_priority,
        }
    }
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub const fn stable_priority(&self) -> u16 {
        self.stable_priority
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseSimultaneousOrderPolicy {
    StableIdAscendingUnlessExactAuthoredOrder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingInteractionRuntime {
    blessings: Arc<DivergentUniverseBlessingRuntime>,
    equations: Arc<DivergentUniverseEquationBattleRuntime>,
    contributions: Arc<[DivergentUniverseBlessingBattleContribution]>,
    rewrite_policies: Arc<[DivergentUniverseBlessingServiceRewritePolicy]>,
    curio_lifecycle: Arc<[DivergentUniverseCurioLifecyclePolicy]>,
    component_digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn blessing_interaction_runtime(
        &self,
    ) -> Result<
        DivergentUniverseBlessingInteractionRuntime,
        DivergentUniverseBlessingInteractionError,
    > {
        DivergentUniverseBlessingInteractionRuntime::compile(
            Arc::new(self.blessing_runtime()?),
            Arc::new(self.equation_battle_runtime()?),
            self.bundle.blessing_catalog(),
            self.bundle.curio_catalog(),
            self.bundle.titan_catalog(),
            self.bundle.identity().component_digest().bytes(),
        )
    }
}

impl DivergentUniverseBlessingInteractionRuntime {
    fn compile(
        blessings: Arc<DivergentUniverseBlessingRuntime>,
        equations: Arc<DivergentUniverseEquationBattleRuntime>,
        blessing_catalog: &starclock_data::divergent_universe_blessing_catalog::DivergentUniverseBlessingCatalog,
        curio_catalog: &starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioCatalog,
        titan_catalog: &starclock_data::divergent_universe_titan_catalog::DivergentUniverseTitanCatalog,
        component_digest: [u8; 32],
    ) -> Result<Self, DivergentUniverseBlessingInteractionError> {
        let contributions = blessings
            .blessings()
            .iter()
            .flat_map(|blessing| blessing.levels())
            .map(|level| DivergentUniverseBlessingBattleContribution {
                level: level.id().clone(),
                blessing: level.blessing().clone(),
                level_number: level.level(),
                state: level.state(),
                binding_key: level.binding_key().into(),
                binding_type: level.binding_type().into(),
                modifier_name: level.modifier_name().into(),
                parameters: level.parameters().into(),
                path: level.path().clone(),
                effect_ids: level.extra_effect_ids().into(),
                equation_contribution_identity: level.equation_contribution_identity().into(),
            })
            .collect::<Vec<_>>();
        let rewrite_policies = blessing_catalog
            .rewrites()
            .iter()
            .filter(|rewrite| rewrite.timing.as_ref() == "AcceptedServiceOperation")
            .map(|rewrite| {
                if rewrite.candidate_policy.as_ref() != "ExplicitStableIDSelection"
                    || rewrite.input.is_some()
                    || rewrite.output.is_some()
                    || rewrite.input_state.as_ref() != "Owned"
                    || rewrite.output_state.as_ref() != "AcceptedOutput"
                    || rewrite.no_legal_candidate.as_ref() != "RejectWithoutMutation"
                    || rewrite.runtime_lowered
                {
                    return Err(DivergentUniverseBlessingInteractionError::InvalidCatalog);
                }
                Ok(DivergentUniverseBlessingServiceRewritePolicy {
                    source_id: rewrite.id.as_str().into(),
                    timing: rewrite.timing.clone(),
                    equation_identity_preserved: rewrite.equation_identity_preserved,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let curio_lifecycle = curio_catalog
            .lifecycle()
            .iter()
            .map(|lifecycle| {
                if [
                    lifecycle.activation.as_ref(),
                    lifecycle.charges.as_ref(),
                    lifecycle.destruction.as_ref(),
                    lifecycle.repair.as_ref(),
                    lifecycle.replacement.as_ref(),
                    lifecycle.simultaneous_trigger_order.as_ref(),
                ]
                .iter()
                .any(|value| *value != "Unspecified")
                    || lifecycle.fallback.as_ref() != "RejectWithoutMutation"
                    || lifecycle.runtime_lowered
                {
                    return Err(DivergentUniverseBlessingInteractionError::InvalidCatalog);
                }
                Ok(DivergentUniverseCurioLifecyclePolicy {
                    id: lifecycle.id.clone(),
                    curio: lifecycle.curio.clone(),
                    fallback: lifecycle.fallback.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let titan = titan_catalog
            .contributions()
            .iter()
            .find(|value| value.id.as_str() == TITAN_FIXTURE_SOURCE)
            .ok_or(DivergentUniverseBlessingInteractionError::InvalidCatalog)?;
        if contributions.len() != 828
            || rewrite_policies.len() != 2
            || curio_lifecycle.len() != 179
            || !curio_lifecycle
                .iter()
                .any(|value| value.id.as_str() == CURIO_FIXTURE_SOURCE)
            || titan.activation.as_ref() != "AcceptedGoldenBloodBoon"
            || titan.scope.as_ref() != "Battle"
            || titan.teardown.as_ref() != "BattleEnd"
            || titan.ordered_effects.len() != 1
            || titan.ordered_effects[0].operation.as_ref()
                != "InstallStageAbilityBeforeCharacterBorn"
            || titan.ordered_effects[0].binding_key.as_deref() != Some("StageAbility_634020")
        {
            return Err(DivergentUniverseBlessingInteractionError::InvalidCatalog);
        }
        Ok(Self {
            blessings,
            equations,
            contributions: contributions.into(),
            rewrite_policies: rewrite_policies.into(),
            curio_lifecycle: curio_lifecycle.into(),
            component_digest,
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseBlessingInteractionAccuracy {
        DivergentUniverseBlessingInteractionAccuracy::ExactBlessingLevelsAndEquationContributions
    }
    #[must_use]
    pub fn contributions(&self) -> &[DivergentUniverseBlessingBattleContribution] {
        &self.contributions
    }
    #[must_use]
    pub fn rewrite_policies(&self) -> &[DivergentUniverseBlessingServiceRewritePolicy] {
        &self.rewrite_policies
    }
    #[must_use]
    pub fn curio_lifecycle_policies(&self) -> &[DivergentUniverseCurioLifecyclePolicy] {
        &self.curio_lifecycle
    }
    #[must_use]
    pub const fn simultaneous_order_policy(&self) -> DivergentUniverseSimultaneousOrderPolicy {
        DivergentUniverseSimultaneousOrderPolicy::StableIdAscendingUnlessExactAuthoredOrder
    }

    pub fn snapshot(
        &self,
        activity: &GraphActivity,
    ) -> Result<
        DivergentUniverseBlessingInteractionSnapshot,
        DivergentUniverseBlessingInteractionError,
    > {
        let owned = self.blessings.owned(activity)?;
        let selected = owned
            .iter()
            .map(|owned| {
                self.contributions
                    .binary_search_by(|candidate| {
                        candidate
                            .blessing
                            .cmp(owned.blessing())
                            .then(candidate.level_number.cmp(&owned.level()))
                    })
                    .ok()
                    .and_then(|index| self.contributions.get(index))
                    .cloned()
                    .ok_or(DivergentUniverseBlessingInteractionError::InvalidCatalog)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let equations = self.equations.snapshot(activity)?;
        let state_hash = activity.state_hash();
        let digest = interaction_digest(
            state_hash,
            self.component_digest,
            &selected,
            equations.digest().bytes(),
        )?;
        Ok(DivergentUniverseBlessingInteractionSnapshot {
            source_state_hash: state_hash,
            blessings: selected.into_boxed_slice(),
            equations,
            digest,
        })
    }

    pub fn order_simultaneous(
        &self,
        contributions: &[DivergentUniverseSimultaneousContribution],
    ) -> Result<
        Box<[DivergentUniverseSimultaneousContribution]>,
        DivergentUniverseBlessingInteractionError,
    > {
        if contributions.is_empty()
            || contributions.iter().any(|value| {
                !matches!(
                    value.source_id(),
                    CURIO_FIXTURE_SOURCE | TITAN_FIXTURE_SOURCE
                )
            })
            || contributions
                .iter()
                .any(|value| value.stable_priority != contributions[0].stable_priority)
            || contributions
                .iter()
                .map(DivergentUniverseSimultaneousContribution::source_id)
                .collect::<BTreeSet<_>>()
                .len()
                != contributions.len()
        {
            return Err(DivergentUniverseBlessingInteractionError::InvalidSimultaneousSet);
        }
        let mut ordered = contributions.to_vec();
        ordered.sort_unstable_by(|left, right| left.source_id.cmp(&right.source_id));
        Ok(ordered.into_boxed_slice())
    }

    pub fn reject_unproven_curio_lifecycle(
        &self,
        activity: &GraphActivity,
        expected_state_hash: ActivityStateHash,
        lifecycle: &DivergentUniverseCurioLifecycleId,
        _transition: DivergentUniverseCurioLifecycleTransition,
    ) -> Result<(), DivergentUniverseBlessingInteractionError> {
        if activity.state_hash() != expected_state_hash {
            return Err(DivergentUniverseBlessingInteractionError::Activity(
                GraphActivityCommandError::StaleStateHash,
            ));
        }
        if self
            .curio_lifecycle
            .binary_search_by(|candidate| candidate.id.cmp(lifecycle))
            .is_err()
        {
            return Err(DivergentUniverseBlessingInteractionError::UnknownCurioLifecycle);
        }
        Err(DivergentUniverseBlessingInteractionError::UnprovenCurioLifecycle)
    }
}

fn interaction_digest(
    state_hash: ActivityStateHash,
    component_digest: [u8; 32],
    blessings: &[DivergentUniverseBlessingBattleContribution],
    equation_digest: [u8; 32],
) -> Result<
    DivergentUniverseBlessingInteractionSnapshotDigest,
    DivergentUniverseBlessingInteractionError,
> {
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.divergent-universe.blessing-interaction-snapshot.v1");
    hash.update(component_digest);
    hash.update(state_hash.bytes());
    hash.update(equation_digest);
    push_len(&mut hash, blessings.len())?;
    for contribution in blessings {
        push_text(&mut hash, contribution.level.as_str())?;
        push_text(&mut hash, contribution.blessing.as_str())?;
        hash.update(contribution.level_number.to_le_bytes());
        push_text(&mut hash, contribution.binding_key())?;
        push_text(&mut hash, contribution.binding_type())?;
        push_text(&mut hash, contribution.modifier_name())?;
        push_text(&mut hash, contribution.path.as_str())?;
        push_text(&mut hash, contribution.equation_contribution_identity())?;
        push_len(&mut hash, contribution.parameters.len())?;
        for parameter in &contribution.parameters {
            hash.update(parameter.coefficient().to_le_bytes());
            hash.update([parameter.scale()]);
        }
        push_len(&mut hash, contribution.effect_ids.len())?;
        for effect in &contribution.effect_ids {
            push_text(&mut hash, effect)?;
        }
    }
    Ok(DivergentUniverseBlessingInteractionSnapshotDigest(
        hash.finalize(),
    ))
}

fn push_text(
    hash: &mut CanonicalDigestBuilder,
    value: &str,
) -> Result<(), DivergentUniverseBlessingInteractionError> {
    push_len(hash, value.len())?;
    hash.update(value.as_bytes());
    Ok(())
}

fn push_len(
    hash: &mut CanonicalDigestBuilder,
    value: usize,
) -> Result<(), DivergentUniverseBlessingInteractionError> {
    hash.update(
        u64::try_from(value)
            .map_err(|_| DivergentUniverseBlessingInteractionError::InvalidCatalog)?
            .to_le_bytes(),
    );
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBlessingInteractionError {
    InvalidCatalog,
    InvalidSimultaneousSet,
    UnknownCurioLifecycle,
    UnprovenCurioLifecycle,
    Blessing(DivergentUniverseBlessingRuntimeError),
    Equation(DivergentUniverseEquationBattleError),
    Activity(GraphActivityCommandError),
}

impl From<DivergentUniverseBlessingRuntimeError> for DivergentUniverseBlessingInteractionError {
    fn from(value: DivergentUniverseBlessingRuntimeError) -> Self {
        Self::Blessing(value)
    }
}

impl From<DivergentUniverseEquationBattleError> for DivergentUniverseBlessingInteractionError {
    fn from(value: DivergentUniverseEquationBattleError) -> Self {
        Self::Equation(value)
    }
}

impl core::fmt::Display for DivergentUniverseBlessingInteractionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Blessing interaction error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseBlessingInteractionError {}
