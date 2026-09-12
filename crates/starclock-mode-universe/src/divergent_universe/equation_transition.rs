//! Executable VersionedProjectPolicy descriptors for Equation ownership transitions.

use std::sync::Arc;

use starclock_data::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCatalog, DivergentUniverseEquationTransitionId,
};

use super::DivergentUniverseRuntimeFactory;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationTransitionAccuracy {
    VersionedProjectPolicyExplicitStableIdAtomic,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseEquationTransitionKind {
    Acquire,
    Discard,
    OwnedBlessingRefresh,
    Replace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationTransitionPolicy {
    id: DivergentUniverseEquationTransitionId,
    kind: DivergentUniverseEquationTransitionKind,
    ordered_operations: Box<[Box<str>]>,
}

impl DivergentUniverseEquationTransitionPolicy {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseEquationTransitionId {
        &self.id
    }

    #[must_use]
    pub const fn kind(&self) -> DivergentUniverseEquationTransitionKind {
        self.kind
    }

    #[must_use]
    pub fn ordered_operations(&self) -> &[Box<str>] {
        &self.ordered_operations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationTransitionRuntime {
    policies: Arc<[DivergentUniverseEquationTransitionPolicy]>,
}

impl DivergentUniverseRuntimeFactory {
    pub fn equation_transition_runtime(
        &self,
    ) -> Result<DivergentUniverseEquationTransitionRuntime, DivergentUniverseEquationTransitionError>
    {
        DivergentUniverseEquationTransitionRuntime::compile(self.bundle.equation_catalog())
    }
}

impl DivergentUniverseEquationTransitionRuntime {
    pub(super) fn compile(
        catalog: &DivergentUniverseEquationCatalog,
    ) -> Result<Self, DivergentUniverseEquationTransitionError> {
        let policies = catalog
            .transitions()
            .iter()
            .map(|definition| {
                let (kind, expected) = expected_transition(definition.operation.as_ref())?;
                if definition.candidate_policy.as_ref() != "ExplicitStableIDSelection"
                    || definition.no_legal_candidate.as_ref() != "RejectWithoutMutation"
                    || definition.preserved_state.as_ref() != "AllAuthoritativeStateOnRejection"
                    || definition.runtime_lowered
                    || definition.ordered_operations.len() != expected.len()
                    || definition
                        .ordered_operations
                        .iter()
                        .zip(expected.iter())
                        .any(|(actual, expected)| actual.as_ref() != *expected)
                {
                    return Err(DivergentUniverseEquationTransitionError::InvalidCatalog);
                }
                Ok(DivergentUniverseEquationTransitionPolicy {
                    id: definition.id.clone(),
                    kind,
                    ordered_operations: definition.ordered_operations.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if policies.len() != 4
            || policies.windows(2).any(|pair| pair[0].id >= pair[1].id)
            || policies
                .iter()
                .map(|policy| policy.kind)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != 4
        {
            return Err(DivergentUniverseEquationTransitionError::InvalidCatalog);
        }
        Ok(Self {
            policies: policies.into(),
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseEquationTransitionAccuracy {
        DivergentUniverseEquationTransitionAccuracy::VersionedProjectPolicyExplicitStableIdAtomic
    }

    #[must_use]
    pub fn policies(&self) -> &[DivergentUniverseEquationTransitionPolicy] {
        &self.policies
    }
}

fn expected_transition(
    operation: &str,
) -> Result<
    (
        DivergentUniverseEquationTransitionKind,
        &'static [&'static str],
    ),
    DivergentUniverseEquationTransitionError,
> {
    match operation {
        "acquire" => Ok((
            DivergentUniverseEquationTransitionKind::Acquire,
            &[
                "AddUnexpandedEquation",
                "RecomputeRecipeProgress",
                "ActivateIfSatisfied",
            ],
        )),
        "discard" => Ok((
            DivergentUniverseEquationTransitionKind::Discard,
            &["ValidateOwnedInput", "RemoveEquation", "RemoveDerivedState"],
        )),
        "owned-blessing-refresh" => Ok((
            DivergentUniverseEquationTransitionKind::OwnedBlessingRefresh,
            &[
                "SortEquationsByStableId",
                "RecomputeRecipeProgress",
                "ApplyStateTransitions",
            ],
        )),
        "replace" => Ok((
            DivergentUniverseEquationTransitionKind::Replace,
            &[
                "ValidateOwnedInput",
                "ValidateOfferedOutput",
                "CommitReplacement",
            ],
        )),
        _ => Err(DivergentUniverseEquationTransitionError::InvalidCatalog),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEquationTransitionError {
    InvalidCatalog,
}

impl core::fmt::Display for DivergentUniverseEquationTransitionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Equation transition error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseEquationTransitionError {}
