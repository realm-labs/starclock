//! Trusted function-4 settlement, not original random offers or NPC admission.

#[path = "curio_synthesis_offers.rs"]
pub mod offers;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioLifecycleState, DivergentUniverseCurioRuntime,
    DivergentUniverseCurioRuntimeError, DivergentUniverseRuntimeFactory,
    DivergentUniverseWorkbenchCurseError, state::SERVICE_RECEIPTS_SLOT,
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityPlayerView, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_data::{
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};
use std::slice::from_ref;

const PROGRAM: u32 = 22_563;

/// Authenticated owning-service selection, not an untrusted player action.
/// Two distinct input states and a distinct output; current ownership, quality
/// and active lifecycle are checked during settlement. Input order is canonical.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedCurioSynthesis {
    consumed: [DivergentUniverseCurioStateId; 2],
    acquired: DivergentUniverseCurioStateId,
}
impl AcceptedCurioSynthesis {
    pub fn new(
        mut consumed: [DivergentUniverseCurioStateId; 2],
        acquired: DivergentUniverseCurioStateId,
    ) -> Result<Self, CurioSynthesisError> {
        if consumed[0] == consumed[1] || consumed.contains(&acquired) {
            return Err(CurioSynthesisError::InvalidSelection);
        }
        consumed.sort_unstable();
        Ok(Self { consumed, acquired })
    }
    #[must_use]
    pub const fn consumed(&self) -> &[DivergentUniverseCurioStateId; 2] {
        &self.consumed
    }
    #[must_use]
    pub const fn acquired(&self) -> &DivergentUniverseCurioStateId {
        &self.acquired
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioSynthesisAccuracy {
    VersionedProjectPolicyAcceptedActiveCurioPairSameOrHigherQualityOriginalViewGrants,
}
/// Current acceptance/snapshot policy identity. An owning offered profile must
/// bind this alongside exact production/catalog/Workbench identity. It supplies
/// no selector membership, weights, NPC placement or per-Workbench numeric cap.
#[must_use]
pub fn configuration_digest() -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.du.accepted-curio-synthesis.two-active-distinct-inputs.distinct-owners.same-or-higher-quality.exclude-current-owners.ordinary-output.no-evolution-only-output.original-view-grants.input-curio-cost.run-function-4-receipt");
    hash.finalize()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioSynthesisResolution {
    acquired: DivergentUniverseCurioStateId,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}
impl CurioSynthesisResolution {
    #[must_use]
    pub const fn acquired(&self) -> &DivergentUniverseCurioStateId {
        &self.acquired
    }
    #[must_use]
    pub const fn accuracy(&self) -> CurioSynthesisAccuracy {
        CurioSynthesisAccuracy::VersionedProjectPolicyAcceptedActiveCurioPairSameOrHigherQualityOriginalViewGrants
    }
    #[must_use]
    pub const fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Debug)]
pub enum CurioSynthesisError {
    InvalidSelection,
    DifferentInputQuality,
    NegativeCurio,
    LowerOutputQuality,
    Workbench(DivergentUniverseWorkbenchCurseError),
    Curio(DivergentUniverseCurioRuntimeError),
    Activity(GraphActivityCommandError),
}
impl std::fmt::Display for CurioSynthesisError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Divergent Universe Curio synthesis: {self:?}")
    }
}
impl std::error::Error for CurioSynthesisError {}

impl DivergentUniverseRuntimeFactory {
    /// Settle a trusted selection on an active exact function-4 Workbench.
    /// Two active holdings of distinct owners become one different unowned state
    /// of same/higher quality. Negative Curios and evolution-only outputs reject.
    /// Consume states/charges/activations, acquire with mandatory current rewards,
    /// and increment the independent Run-wide function receipt in one generated
    /// transaction. No Fragment fee or Heat debit is added: cost is input Curios.
    /// Original offer/pool/weight/cancellation/limit parity remains unproven.
    /// Failure restores exact bytes/events/RNG; authored rewards are not skipped.
    pub fn settle_curio_synthesis_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        workbench: &DivergentUniverseWorkbenchId,
        selection: &AcceptedCurioSynthesis,
    ) -> Result<CurioSynthesisResolution, CurioSynthesisError> {
        let receipt = self
            .workbench_curse_runtime()
            .map_err(CurioSynthesisError::Workbench)?
            .synthesis_receipt(activity, expected, workbench)
            .map_err(CurioSynthesisError::Workbench)?;
        let curios = self.curio_runtime().map_err(CurioSynthesisError::Curio)?;
        let program =
            ActivityProgramId::new(PROGRAM).expect("nonzero mode-owned synthesis program");
        let mut generation_error = None;
        let view = activity.player_view();
        let result = activity
            .apply_generated_boundary(expected, program, |rng| {
                let operations = (|| {
                    validate_selection(&curios, &view, selection)?;
                    let mut operations = curios
                        .consumption_acquisition_operations(
                            &view,
                            &selection.consumed,
                            from_ref(&selection.acquired),
                            rng,
                        )
                        .map_err(CurioSynthesisError::Curio)?;
                    operations.push(ActivityOperation::AddCounter {
                        slot: SERVICE_RECEIPTS_SLOT,
                        key: receipt,
                        delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                    });
                    Ok::<_, CurioSynthesisError>(operations)
                })()
                .map_err(|error| {
                    generation_error = Some(error);
                    GraphActivityCommandError::Runtime(
                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                    )
                })?;
                Ok((operations, ()))
            })
            .map_err(|error| generation_error.unwrap_or(CurioSynthesisError::Activity(error)))?;
        Ok(CurioSynthesisResolution {
            acquired: selection.acquired.clone(),
            events: result.events().into(),
            state_hash: activity.state_hash(),
        })
    }
}

fn validate_selection(
    curios: &DivergentUniverseCurioRuntime,
    view: &ActivityPlayerView,
    selection: &AcceptedCurioSynthesis,
) -> Result<(), CurioSynthesisError> {
    let input_quality = validate_inputs(curios, view, &selection.consumed)?;
    let output = curios
        .state(&selection.acquired)
        .map_err(CurioSynthesisError::Curio)?;
    if quality(output.category())? < input_quality {
        return Err(CurioSynthesisError::LowerOutputQuality);
    }
    // The inventory planner also rejects missing/evolution-only output identity,
    // duplicate owners and consumed owners before constructing any reward draws.
    Ok(())
}

fn validate_inputs(
    curios: &DivergentUniverseCurioRuntime,
    view: &ActivityPlayerView,
    consumed: &[DivergentUniverseCurioStateId; 2],
) -> Result<u8, CurioSynthesisError> {
    if consumed[0] == consumed[1] {
        return Err(CurioSynthesisError::InvalidSelection);
    }
    let held = curios
        .owned_from_view(view)
        .map_err(CurioSynthesisError::Curio)?;
    let mut qualities = Vec::new();
    let mut owners = Vec::new();
    for id in consumed {
        let definition = curios.state(id).map_err(CurioSynthesisError::Curio)?;
        let owned =
            held.iter()
                .find(|owned| owned.state() == id)
                .ok_or(CurioSynthesisError::Curio(
                    DivergentUniverseCurioRuntimeError::NotOwned,
                ))?;
        if owned.lifecycle() != DivergentUniverseCurioLifecycleState::Active {
            return Err(CurioSynthesisError::InvalidSelection);
        }
        if owners.contains(owned.curio()) {
            return Err(CurioSynthesisError::InvalidSelection);
        }
        owners.push(owned.curio().clone());
        qualities.push(quality(definition.category())?);
    }
    if qualities[0] != qualities[1] {
        return Err(CurioSynthesisError::DifferentInputQuality);
    }
    Ok(qualities[0])
}
fn quality(category: DivergentUniverseCurioCategory) -> Result<u8, CurioSynthesisError> {
    match category {
        DivergentUniverseCurioCategory::Common => Ok(1),
        DivergentUniverseCurioCategory::Rare => Ok(2),
        DivergentUniverseCurioCategory::Legendary => Ok(3),
        DivergentUniverseCurioCategory::Negative => Err(CurioSynthesisError::NegativeCurio),
    }
}
