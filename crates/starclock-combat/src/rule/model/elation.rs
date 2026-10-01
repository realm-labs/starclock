//! Explicit expression inputs for the shared dedicated damage operation.
use super::ValueExpr;

/// Caller-resolved Elation operands, with no implicit level or meter conversion.
///
/// Every operand must evaluate to Scalar at the program input boundary. Ratio
/// inputs retain their exact millionths; negative merrymaking and RES values
/// are legal only within the dedicated definition's declared domain. The
/// evaluator validates the entire definition without mutation or RNG.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ElationDamageExpressions {
    pub base_damage: ValueExpr,
    pub original_multiplier: ValueExpr,
    pub meter_multiplier: ValueExpr,
    pub merrymaking: ValueExpr,
    pub target_resistance: ValueExpr,
    pub penetration: ValueExpr,
    pub resistance_minimum: ValueExpr,
    pub resistance_maximum: ValueExpr,
    pub unbroken_multiplier: ValueExpr,
}
impl ElationDamageExpressions {
    /// Borrows operands in declaration order for validation/dependency analysis.
    /// This neither evaluates expressions nor reads or mutates battle state.
    #[must_use]
    pub fn expressions(&self) -> [&ValueExpr; 9] {
        [
            &self.base_damage,
            &self.original_multiplier,
            &self.meter_multiplier,
            &self.merrymaking,
            &self.target_resistance,
            &self.penetration,
            &self.resistance_minimum,
            &self.resistance_maximum,
            &self.unbroken_multiplier,
        ]
    }
}
