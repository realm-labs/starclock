use core::fmt;

/// Typed deterministic failure from authoritative numeric work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericError {
    /// A result cannot fit the documented fixed-width representation.
    Overflow,
    /// A divisor is zero.
    DivisionByZero,
    /// A source representation cannot be converted exactly as requested.
    InvalidConversion,
    /// A value violates a domain invariant such as non-negativity.
    OutOfDomain,
}

impl fmt::Display for NumericError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Overflow => "numeric overflow",
            Self::DivisionByZero => "division by zero",
            Self::InvalidConversion => "invalid numeric conversion",
            Self::OutOfDomain => "numeric value is outside its domain",
        })
    }
}

impl std::error::Error for NumericError {}

/// Explicit policy used whenever authoritative precision can be discarded.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rounding {
    /// Round toward negative infinity.
    Floor,
    /// Round toward positive infinity.
    Ceil,
    /// Round toward zero.
    TowardZero,
    /// Round away from zero.
    AwayFromZero,
    /// Round to nearest; exact halves move away from zero.
    NearestTiesAway,
    /// Round to nearest; exact halves select the even result.
    NearestTiesEven,
}

pub(super) fn rounded_quotient(
    numerator: i128,
    denominator: i128,
    rounding: Rounding,
) -> Result<i64, NumericError> {
    if denominator == 0 {
        return Err(NumericError::DivisionByZero);
    }
    let quotient = numerator
        .checked_div(denominator)
        .ok_or(NumericError::Overflow)?;
    let remainder = numerator
        .checked_rem(denominator)
        .ok_or(NumericError::Overflow)?;
    if remainder == 0 {
        return i64::try_from(quotient).map_err(|_| NumericError::Overflow);
    }

    let direction = if (numerator < 0) == (denominator < 0) {
        1_i128
    } else {
        -1_i128
    };
    let doubled_remainder = remainder.unsigned_abs() * 2;
    let denominator_magnitude = denominator.unsigned_abs();
    let adjustment = match rounding {
        Rounding::Floor if direction < 0 => direction,
        Rounding::Ceil if direction > 0 => direction,
        Rounding::TowardZero => 0,
        Rounding::AwayFromZero => direction,
        Rounding::NearestTiesAway if doubled_remainder >= denominator_magnitude => direction,
        Rounding::NearestTiesEven
            if doubled_remainder > denominator_magnitude
                || (doubled_remainder == denominator_magnitude && quotient % 2 != 0) =>
        {
            direction
        }
        _ => 0,
    };
    let result = quotient
        .checked_add(adjustment)
        .ok_or(NumericError::Overflow)?;
    i64::try_from(result).map_err(|_| NumericError::Overflow)
}
