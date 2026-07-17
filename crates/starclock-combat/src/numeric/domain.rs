use super::{
    rounding::{NumericError, Rounding, rounded_quotient},
    scalar::{Ratio, Scalar},
};

/// Non-negative fixed-point stat value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StatValue(Scalar);

impl StatValue {
    /// Creates a stat value from canonical non-negative millionths.
    pub fn from_scaled(raw: i64) -> Result<Self, NumericError> {
        non_negative_scalar(raw).map(Self)
    }

    /// Returns canonical millionths.
    #[must_use]
    pub fn scaled(self) -> i64 {
        self.0.scaled()
    }

    /// Applies a signed fixed-point delta while preserving non-negativity.
    pub fn checked_add_delta(self, delta: Scalar) -> Result<Self, NumericError> {
        self.0.checked_add(delta).and_then(Self::from_scalar)
    }

    /// Applies a ratio using an explicit rounding policy.
    pub fn checked_scale(self, ratio: Ratio, rounding: Rounding) -> Result<Self, NumericError> {
        ratio
            .checked_apply(self.0, rounding)
            .and_then(Self::from_scalar)
    }

    fn from_scalar(value: Scalar) -> Result<Self, NumericError> {
        if value.scaled() < 0 {
            Err(NumericError::OutOfDomain)
        } else {
            Ok(Self(value))
        }
    }
}

/// Strictly positive fixed-point Speed value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Speed(Scalar);

impl Speed {
    /// Creates Speed from canonical positive millionths.
    pub fn from_scaled(raw: i64) -> Result<Self, NumericError> {
        if raw <= 0 {
            Err(NumericError::OutOfDomain)
        } else {
            Ok(Self(Scalar::from_scaled(raw)))
        }
    }

    /// Returns canonical millionths.
    #[must_use]
    pub fn scaled(self) -> i64 {
        self.0.scaled()
    }
}

/// Non-negative fixed-point Action Gauge value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionGauge(Scalar);

impl ActionGauge {
    /// Creates Action Gauge from canonical non-negative millionths.
    pub fn from_scaled(raw: i64) -> Result<Self, NumericError> {
        non_negative_scalar(raw).map(Self)
    }

    /// Returns canonical millionths.
    #[must_use]
    pub fn scaled(self) -> i64 {
        self.0.scaled()
    }

    /// Applies a signed advance/delay delta while preserving non-negativity.
    pub fn checked_add_delta(self, delta: Scalar) -> Result<Self, NumericError> {
        self.0
            .checked_add(delta)
            .and_then(|value| Self::from_scaled(value.scaled()))
    }

    /// Advances this actor by the exact time needed for a selected actor.
    ///
    /// The elapsed distance is floored to the six-decimal gauge so no actor is
    /// advanced beyond the exact rational boundary. The selected actor is set
    /// to zero explicitly by the scheduler.
    pub(crate) fn checked_advance_for_selection(
        self,
        speed: Speed,
        selected_gauge: Self,
        selected_speed: Speed,
    ) -> Result<Self, NumericError> {
        let elapsed = rounded_quotient(
            i128::from(speed.scaled()) * i128::from(selected_gauge.scaled()),
            i128::from(selected_speed.scaled()),
            Rounding::Floor,
        )?;
        self.0
            .checked_sub(Scalar::from_scaled(elapsed))
            .and_then(|value| Self::from_scaled(value.scaled()))
    }
}

/// Integer probability threshold in inclusive millionths `[0, 1_000_000]`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Probability(u32);

impl Probability {
    /// Probability that never succeeds.
    pub const ZERO: Self = Self(0);
    /// Probability that always succeeds.
    pub const ONE: Self = Self(1_000_000);

    /// Creates a checked probability threshold.
    pub const fn from_millionths(raw: u32) -> Result<Self, NumericError> {
        if raw <= 1_000_000 {
            Ok(Self(raw))
        } else {
            Err(NumericError::OutOfDomain)
        }
    }

    /// Converts an unrestricted ratio after checking its signed representation.
    pub fn from_ratio(value: Ratio) -> Result<Self, NumericError> {
        let raw = u32::try_from(value.scaled()).map_err(|_| NumericError::InvalidConversion)?;
        Self::from_millionths(raw)
    }

    /// Returns the inclusive integer threshold.
    #[must_use]
    pub const fn millionths(self) -> u32 {
        self.0
    }
}

macro_rules! integral_amount {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(i64);

        impl $name {
            /// Creates a checked non-negative integral amount.
            pub const fn new(raw: i64) -> Result<Self, NumericError> {
                if raw < 0 {
                    Err(NumericError::OutOfDomain)
                } else {
                    Ok(Self(raw))
                }
            }

            /// Finalizes a non-negative scalar with explicit rounding.
            pub fn from_scalar(value: Scalar, rounding: Rounding) -> Result<Self, NumericError> {
                let raw = value.rounded_integer(rounding)?;
                Self::new(raw)
            }

            /// Returns the authoritative integral representation.
            #[must_use]
            pub const fn get(self) -> i64 {
                self.0
            }
        }
    };
}

integral_amount!(Hp, "Non-negative integral current or maximum HP value.");
integral_amount!(
    ShieldAmount,
    "Non-negative integral shield capacity after formula finalization."
);
integral_amount!(
    DamageAmount,
    "Non-negative integral applied damage after formula finalization."
);
integral_amount!(
    HealingAmount,
    "Non-negative integral applied healing after formula finalization."
);
integral_amount!(
    RawToughness,
    "Non-negative integral raw Toughness unit used by authored mechanics."
);

fn non_negative_scalar(raw: i64) -> Result<Scalar, NumericError> {
    if raw < 0 {
        Err(NumericError::OutOfDomain)
    } else {
        Ok(Scalar::from_scaled(raw))
    }
}
