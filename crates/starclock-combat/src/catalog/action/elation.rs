//! Explicit native Elation inputs; production lowering owns their provenance.
use crate::{
    NumericError, Ratio, Rounding, Scalar,
    formula::model::{CombatElement, ResistanceInput},
};

/// One dedicated Elation hit, with a caller-resolved level base and meter factor.
///
/// The resolver reads live Elation, CRIT and target DEF, and target broken state.
/// This definition neither queries a build catalog nor guesses a level curve,
/// meter conversion, elemental RES baseline or RES bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElationDamageDefinition {
    pub(crate) base_damage: Scalar,
    pub(crate) original_multiplier: Ratio,
    pub(crate) meter_multiplier: Ratio,
    pub(crate) merrymaking: Ratio,
    pub(crate) resistance: ResistanceInput,
    pub(crate) unbroken_multiplier: Ratio,
    element: CombatElement,
}

impl ElationDamageDefinition {
    /// Constructs immutable explicit inputs. Invalid bases/factors/RES bounds
    /// return a typed error; live numeric failures use battle fault handling.
    pub fn new(
        base_damage: Scalar,
        original_multiplier: Ratio,
        meter_multiplier: Ratio,
        merrymaking: Ratio,
        resistance: ResistanceInput,
        unbroken_multiplier: Ratio,
        element: CombatElement,
    ) -> Result<Self, NumericError> {
        if base_damage.scaled() < 0
            || original_multiplier.scaled() < 0
            || meter_multiplier.scaled() < 0
            || unbroken_multiplier.scaled() < 0
            || Ratio::ONE.checked_add(merrymaking)?.scaled() < 0
            || resistance.minimum > resistance.maximum
            || resistance.maximum > Ratio::ONE
        {
            return Err(NumericError::OutOfDomain);
        }
        Ok(Self {
            base_damage,
            original_multiplier,
            meter_multiplier,
            merrymaking,
            resistance,
            unbroken_multiplier,
            element,
        })
    }
    #[must_use]
    pub const fn element(self) -> CombatElement {
        self.element
    }
    /// Applies one explicit hit share before formula finalization, without
    /// rounding damage to an integer or consuming RNG.
    pub(crate) fn with_share(mut self, share: Ratio) -> Result<Self, NumericError> {
        if share.scaled() < 0 {
            return Err(NumericError::OutOfDomain);
        }
        self.original_multiplier = self
            .original_multiplier
            .checked_mul(share, Rounding::NearestTiesEven)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests;
