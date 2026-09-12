//! Cross-runtime offer hardening and explicit empty-candidate service policy.

use std::sync::Arc;

use starclock_activity::{ActivityStateHash, GraphActivity, GraphActivityCommandError};

use super::{
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseEquationOfferRuntime, DivergentUniverseEquationRuntimeError,
    DivergentUniverseRuntimeFactory,
};

const CURSE_CHEST_POLICY_ID: &str = "divergent-universe.service-offer.curse-chest.1001";
const WORKBENCH_POLICY_ID: &str = "divergent-universe.service-rule.workbench.1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseOfferHardeningAccuracy {
    ExactEquationAndBlessingCapsOrderingAndRngIsolation,
    VersionedProjectPolicyFailClosedNoLegalCandidate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEmptyCandidatePolicyKind {
    CurseChestRandomPool,
    WorkbenchBlessingEnhance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEmptyCandidatePolicy {
    kind: DivergentUniverseEmptyCandidatePolicyKind,
    source_id: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseEmptyCandidatePolicy {
    #[must_use]
    pub const fn kind(&self) -> DivergentUniverseEmptyCandidatePolicyKind {
        self.kind
    }
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseOfferHardeningAccuracy {
        DivergentUniverseOfferHardeningAccuracy::VersionedProjectPolicyFailClosedNoLegalCandidate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOfferHardeningRuntime {
    equations: Arc<DivergentUniverseEquationOfferRuntime>,
    blessings: Arc<DivergentUniverseBlessingRuntime>,
    empty_candidate_policies: [DivergentUniverseEmptyCandidatePolicy; 2],
}

impl DivergentUniverseRuntimeFactory {
    pub fn offer_hardening_runtime(
        &self,
    ) -> Result<DivergentUniverseOfferHardeningRuntime, DivergentUniverseOfferHardeningError> {
        let service = self.bundle.service_catalog();
        let curse_chest = service
            .offers()
            .iter()
            .find(|value| value.id.as_str() == CURSE_CHEST_POLICY_ID)
            .filter(|value| {
                value.candidates.is_empty()
                    && value.weights.is_empty()
                    && value.refresh_rule.as_ref() == "OneAcceptedChoice"
                    && value.fallback.as_ref() == "LeaveWithoutMutation"
                    && !value.runtime_lowered
            })
            .ok_or(DivergentUniverseOfferHardeningError::InvalidCatalog)?;
        let workbench = service
            .service_rules()
            .iter()
            .find(|value| value.id.as_str() == WORKBENCH_POLICY_ID)
            .filter(|value| {
                value.service_kind.as_ref() == "BuffEnhance"
                    && value.price.as_ref() == "UnspecifiedAmount"
                    && value.ordered_operations.iter().map(AsRef::as_ref).eq([
                        "Consume:OwnedBaseBlessing",
                        "Produce:SameIdentityEnhancedBlessing",
                    ])
                    && value.fallback.as_ref() == "RejectWithoutMutation"
                    && !value.runtime_lowered
            })
            .ok_or(DivergentUniverseOfferHardeningError::InvalidCatalog)?;
        let runtime = DivergentUniverseOfferHardeningRuntime {
            equations: Arc::new(self.equation_offer_runtime()?),
            blessings: Arc::new(self.blessing_runtime()?),
            empty_candidate_policies: [
                DivergentUniverseEmptyCandidatePolicy {
                    kind: DivergentUniverseEmptyCandidatePolicyKind::CurseChestRandomPool,
                    source_id: curse_chest.id.as_str().into(),
                    fallback: curse_chest.fallback.clone(),
                },
                DivergentUniverseEmptyCandidatePolicy {
                    kind: DivergentUniverseEmptyCandidatePolicyKind::WorkbenchBlessingEnhance,
                    source_id: workbench.id.as_str().into(),
                    fallback: workbench.fallback.clone(),
                },
            ],
        };
        runtime.validate_caps_and_order()?;
        Ok(runtime)
    }
}

impl DivergentUniverseOfferHardeningRuntime {
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseOfferHardeningAccuracy {
        DivergentUniverseOfferHardeningAccuracy::ExactEquationAndBlessingCapsOrderingAndRngIsolation
    }
    #[must_use]
    pub const fn equation_identity_cap(&self) -> u16 {
        80
    }
    #[must_use]
    pub const fn blessing_identity_cap(&self) -> u16 {
        414
    }
    #[must_use]
    pub const fn blessing_group_candidate_cap(&self) -> u16 {
        144
    }
    #[must_use]
    pub const fn empty_candidate_policies(&self) -> &[DivergentUniverseEmptyCandidatePolicy; 2] {
        &self.empty_candidate_policies
    }

    pub fn reject_empty_candidate_policy(
        &self,
        activity: &GraphActivity,
        expected_state_hash: ActivityStateHash,
        kind: DivergentUniverseEmptyCandidatePolicyKind,
    ) -> Result<(), DivergentUniverseOfferHardeningError> {
        if activity.state_hash() != expected_state_hash {
            return Err(DivergentUniverseOfferHardeningError::Activity(
                GraphActivityCommandError::StaleStateHash,
            ));
        }
        if self
            .empty_candidate_policies
            .iter()
            .all(|policy| policy.kind != kind)
        {
            return Err(DivergentUniverseOfferHardeningError::InvalidCatalog);
        }
        Err(DivergentUniverseOfferHardeningError::NoLegalCandidate)
    }

    fn validate_caps_and_order(&self) -> Result<(), DivergentUniverseOfferHardeningError> {
        if self.equations.candidate_identity_count() != 80
            || self.blessings.blessings().len() != 414
            || self
                .blessings
                .groups()
                .iter()
                .map(|group| group.candidates().len())
                .max()
                != Some(144)
            || self
                .blessings
                .blessings()
                .windows(2)
                .any(|pair| pair[0].id() >= pair[1].id())
        {
            Err(DivergentUniverseOfferHardeningError::InvalidCatalog)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseOfferHardeningError {
    InvalidCatalog,
    NoLegalCandidate,
    Equation(DivergentUniverseEquationRuntimeError),
    Blessing(DivergentUniverseBlessingRuntimeError),
    Activity(GraphActivityCommandError),
}

impl From<DivergentUniverseEquationRuntimeError> for DivergentUniverseOfferHardeningError {
    fn from(value: DivergentUniverseEquationRuntimeError) -> Self {
        Self::Equation(value)
    }
}

impl From<DivergentUniverseBlessingRuntimeError> for DivergentUniverseOfferHardeningError {
    fn from(value: DivergentUniverseBlessingRuntimeError) -> Self {
        Self::Blessing(value)
    }
}

impl core::fmt::Display for DivergentUniverseOfferHardeningError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe offer hardening error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseOfferHardeningError {}
