use core::fmt;

use fixnum::{FixedPoint, ops::*, typenum::U6};

use super::rounding::{NumericError, Rounding, rounded_quotient};

type Repr = FixedPoint<i64, U6>;
const SCALE: i128 = 1_000_000;

/// Signed decimal fixed-point scalar with exactly six fractional digits.
///
/// The backing implementation is private. Construction and inspection use the
/// fixed-width signed millionths encoded by canonical state.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Scalar(Repr);

impl Scalar {
    /// Number of fractional decimal digits in the canonical representation.
    pub const FRACTIONAL_DIGITS: u32 = 6;
    /// Smallest representable scalar.
    pub const MIN: Self = Self::from_scaled(i64::MIN);
    /// Largest representable scalar.
    pub const MAX: Self = Self::from_scaled(i64::MAX);
    /// Additive identity.
    pub const ZERO: Self = Self::from_scaled(0);
    /// Multiplicative identity.
    pub const ONE: Self = Self::from_scaled(1_000_000);

    /// Creates a scalar from canonical signed millionths.
    #[must_use]
    pub const fn from_scaled(raw: i64) -> Self {
        Self(Repr::from_bits(raw))
    }

    /// Converts an integral value without losing precision.
    pub fn checked_from_integer(value: i64) -> Result<Self, NumericError> {
        value
            .checked_mul(1_000_000)
            .map(Self::from_scaled)
            .ok_or(NumericError::Overflow)
    }

    /// Returns canonical signed millionths.
    #[must_use]
    pub fn scaled(self) -> i64 {
        self.0.into_bits()
    }

    /// Checked addition of values with the same unit.
    pub fn checked_add(self, rhs: Self) -> Result<Self, NumericError> {
        self.0
            .cadd(rhs.0)
            .map(Self)
            .map_err(|_| NumericError::Overflow)
    }

    /// Checked subtraction of values with the same unit.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, NumericError> {
        self.0
            .csub(rhs.0)
            .map(Self)
            .map_err(|_| NumericError::Overflow)
    }

    /// Checked negation.
    pub fn checked_neg(self) -> Result<Self, NumericError> {
        self.0.cneg().map(Self).map_err(|_| NumericError::Overflow)
    }

    /// Checked fixed-point multiplication with explicit result rounding.
    pub fn checked_mul(self, rhs: Self, rounding: Rounding) -> Result<Self, NumericError> {
        let numerator = i128::from(self.scaled()) * i128::from(rhs.scaled());
        rounded_quotient(numerator, SCALE, rounding).map(Self::from_scaled)
    }

    /// Checked fixed-point division with explicit result rounding.
    pub fn checked_div(self, rhs: Self, rounding: Rounding) -> Result<Self, NumericError> {
        if rhs == Self::ZERO {
            return Err(NumericError::DivisionByZero);
        }
        let numerator = i128::from(self.scaled()) * SCALE;
        rounded_quotient(numerator, i128::from(rhs.scaled()), rounding).map(Self::from_scaled)
    }

    /// Checked multiplication by an exact integer.
    pub fn checked_mul_integer(self, rhs: i64) -> Result<Self, NumericError> {
        self.0
            .cmul(rhs)
            .map(Self)
            .map_err(|_| NumericError::Overflow)
    }

    /// Checked division by an integer with explicit fixed-point rounding.
    pub fn checked_div_integer(self, rhs: i64, rounding: Rounding) -> Result<Self, NumericError> {
        rounded_quotient(i128::from(self.scaled()), i128::from(rhs), rounding)
            .map(Self::from_scaled)
    }

    /// Finalizes a fixed-point intermediate to an integral state amount.
    pub fn rounded_integer(self, rounding: Rounding) -> Result<i64, NumericError> {
        rounded_quotient(i128::from(self.scaled()), SCALE, rounding)
    }
}

impl fmt::Debug for Scalar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Scalar")
            .field(&self.scaled())
            .finish()
    }
}

/// Dimensionless six-decimal fixed-point ratio.
///
/// This base ratio is intentionally signed and unrestricted; probability and
/// other bounded concepts use separate domain types.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ratio(Scalar);

impl Ratio {
    /// Additive identity.
    pub const ZERO: Self = Self(Scalar::ZERO);
    /// Multiplicative identity.
    pub const ONE: Self = Self(Scalar::ONE);

    /// Creates an unrestricted signed ratio from canonical millionths.
    #[must_use]
    pub const fn from_scaled(raw: i64) -> Self {
        Self(Scalar::from_scaled(raw))
    }

    /// Returns canonical signed millionths.
    #[must_use]
    pub fn scaled(self) -> i64 {
        self.0.scaled()
    }

    /// Checked ratio addition.
    pub fn checked_add(self, rhs: Self) -> Result<Self, NumericError> {
        self.0.checked_add(rhs.0).map(Self)
    }

    /// Checked ratio subtraction.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, NumericError> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    /// Checked ratio multiplication with explicit rounding.
    pub fn checked_mul(self, rhs: Self, rounding: Rounding) -> Result<Self, NumericError> {
        self.0.checked_mul(rhs.0, rounding).map(Self)
    }

    /// Checked ratio division with explicit rounding.
    pub fn checked_div(self, rhs: Self, rounding: Rounding) -> Result<Self, NumericError> {
        self.0.checked_div(rhs.0, rounding).map(Self)
    }

    /// Applies this ratio to a scalar with explicit rounding.
    pub fn checked_apply(self, value: Scalar, rounding: Rounding) -> Result<Scalar, NumericError> {
        value.checked_mul(self.0, rounding)
    }
}
