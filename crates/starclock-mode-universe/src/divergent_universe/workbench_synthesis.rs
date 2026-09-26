//! Exact active function-4 admission for the trusted owning-service settlement.

use super::{
    DivergentUniverseWorkbenchCurseError, DivergentUniverseWorkbenchCurseRuntime,
    DivergentUniverseWorkbenchFunctionKind, function_receipt_key, validate_hash,
};
use starclock_activity::{ActivityStateHash, GraphActivity};
use starclock_data::divergent_universe_service_catalog::DivergentUniverseWorkbenchId;

impl DivergentUniverseWorkbenchCurseRuntime {
    pub(in crate::divergent_universe) fn synthesis_receipt(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        id: &DivergentUniverseWorkbenchId,
    ) -> Result<u64, DivergentUniverseWorkbenchCurseError> {
        validate_hash(activity, expected)?;
        self.require_current_workbench(activity, id)?;
        self.synthesis_service_keys(id).map(|keys| keys.1)
    }

    pub(in crate::divergent_universe) fn synthesis_service_keys(
        &self,
        id: &DivergentUniverseWorkbenchId,
    ) -> Result<(u64, u64), DivergentUniverseWorkbenchCurseError> {
        let workbench = self.workbench(id)?;
        let function = self
            .functions
            .iter()
            .find(|function| function.kind == DivergentUniverseWorkbenchFunctionKind::CurioCompose)
            .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)?;
        if !workbench.functions.contains(&function.id) {
            return Err(DivergentUniverseWorkbenchCurseError::FunctionUnavailable);
        }
        if function.input_policy.as_ref() != "EqualRarityCurios"
            || function.output_policy.as_ref() != "RandomCurioOfSameOrHigherRarity"
            || function.price_currency.as_ref() != "InputCurios"
            || function.price_formula.as_ref() != "UnspecifiedInputCount"
            || function.price_reset.as_ref() != "NotApplicable"
        {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
        }
        Ok((workbench.state_key, function_receipt_key(function)?))
    }
}
