//! Source-owned final factors never participate in underlying stat queries.
use super::{
    FormulaInputs, action_modifier_context, break_modifier_context, formula_modifier,
    formula_source, numeric_fault,
};
use crate::{
    DamageAmount, Rounding, Scalar, UnitId,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    event::cause::Cause,
    formula::{model::CombatElement, sustain::DamageCalculation},
    modifier::{
        model::{FormulaPurpose, FormulaStage, FormulaSubject, ModifierQueryContext},
        resolve::StatResolver,
    },
    resolver::transaction::Transaction,
};

pub(in crate::resolver) struct FinalBreakDamage {
    pub(in crate::resolver) target: UnitId,
    pub(in crate::resolver) element: CombatElement,
    pub(in crate::resolver) purpose: FormulaPurpose,
    pub(in crate::resolver) raw: Scalar,
}

impl FormulaInputs {
    pub(in crate::resolver) fn final_break_damage(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        input: FinalBreakDamage,
    ) -> Result<DamageCalculation, BattleFault> {
        let source = formula_source(txn, cause, input.purpose)?;
        let context = action_modifier_context(
            catalog,
            cause,
            break_modifier_context(txn, source, input.target, input.element, input.purpose)?,
        )
        .with_formula_subject(FormulaSubject::Source);
        let factor = multiplier(&self.resolver(catalog), source, input.purpose, &context)?;
        apply(input.raw, factor)
    }
}

pub(super) fn multiplier(
    resolver: &StatResolver<'_>,
    source: UnitId,
    purpose: FormulaPurpose,
    context: &ModifierQueryContext,
) -> Result<Scalar, BattleFault> {
    formula_modifier(
        resolver,
        source,
        FormulaStage::DamageFinalMultiply,
        purpose,
        context,
    )
}

pub(super) fn apply(raw: Scalar, multiplier: Scalar) -> Result<DamageCalculation, BattleFault> {
    if multiplier.scaled() < 0 {
        return Err(numeric_fault(77, multiplier.scaled()));
    }
    let raw = raw
        .checked_mul(multiplier, Rounding::NearestTiesEven)
        .map_err(|_| numeric_fault(78, raw.scaled()))?;
    Ok(DamageCalculation {
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)
            .map_err(|_| numeric_fault(79, raw.scaled()))?,
    })
}

#[cfg(test)]
mod tests {
    use super::apply;
    use crate::Scalar;

    #[test]
    fn final_damage_uses_unfloored_raw_and_checked_nonnegative_factors() {
        for (raw, factor, expected_raw, expected_damage) in [
            (1_900_000, 1_500_000, 2_850_000, 2),
            (1_900_000, 1_000_000, 1_900_000, 1),
            (1_900_000, 0, 0, 0),
            (1, 1_500_000, 2, 0),
            (3, 1_500_000, 4, 0),
        ] {
            let result = apply(Scalar::from_scaled(raw), Scalar::from_scaled(factor)).unwrap();
            assert_eq!(result.raw.scaled(), expected_raw);
            assert_eq!(result.finalized.get(), expected_damage);
        }
        assert!(apply(Scalar::ONE, Scalar::from_scaled(-1)).is_err());
        assert!(
            apply(
                Scalar::from_scaled(i64::MAX),
                Scalar::from_scaled(2_000_000)
            )
            .is_err()
        );
    }
}
