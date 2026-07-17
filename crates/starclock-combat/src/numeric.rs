use core::fmt;
use fixnum::{FixedPoint, typenum::U6};

type Repr = FixedPoint<i64, U6>;

/// Signed decimal fixed-point scalar with exactly six fractional digits.
///
/// The backing implementation is private. Construction and inspection use the
/// fixed-width scaled integer that later canonical codecs will encode directly.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Scalar(Repr);

impl Scalar {
    /// Number of fractional decimal digits in the canonical representation.
    pub const FRACTIONAL_DIGITS: u32 = 6;

    /// Additive identity.
    pub const ZERO: Self = Self::from_scaled(0);

    /// Creates a scalar from millionths without floating-point conversion.
    #[must_use]
    pub const fn from_scaled(raw: i64) -> Self {
        Self(Repr::from_bits(raw))
    }

    /// Returns the signed millionths used by canonical state.
    #[must_use]
    pub fn scaled(self) -> i64 {
        self.0.into_bits()
    }
}

impl fmt::Debug for Scalar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Scalar")
            .field(&self.0.into_bits())
            .finish()
    }
}

/// Dimensionless ratio represented as six-decimal fixed point.
///
/// Domain-specific bounds such as probability or non-negative modifier ranges
/// are intentionally owned by later wrapper types, not inferred here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ratio(Scalar);

impl Ratio {
    /// Multiplicative identity, exactly `1.000000`.
    pub const ONE: Self = Self(Scalar::from_scaled(1_000_000));

    /// Creates an unrestricted signed ratio from millionths.
    #[must_use]
    pub const fn from_scaled(raw: i64) -> Self {
        Self(Scalar::from_scaled(raw))
    }

    /// Returns the signed millionths used by canonical state.
    #[must_use]
    pub fn scaled(self) -> i64 {
        self.0.scaled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn wrapper_preserves_raw_fixed_point_bits() {
        let values = [i64::MIN, -1_000_001, -1, 0, 1, 1_000_000, i64::MAX];
        for raw in values {
            assert_eq!(Scalar::from_scaled(raw).scaled(), raw);
            assert_eq!(Ratio::from_scaled(raw).scaled(), raw);
        }
    }

    #[test]
    fn backend_does_not_add_layout_overhead() {
        assert_eq!(size_of::<Scalar>(), size_of::<i64>());
        assert_eq!(size_of::<Ratio>(), size_of::<i64>());
        assert_eq!(Ratio::ONE.scaled(), 1_000_000);
    }
}
