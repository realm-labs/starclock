use core::fmt;

/// Exact 32-byte seed accepted by the pinned ChaCha8 generator.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RngSeed([u8; 32]);

impl RngSeed {
    /// Wraps canonical seed bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the canonical seed bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Stable purpose code attached to every consumed raw word.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DrawPurpose(u16);

impl DrawPurpose {
    /// Per-hit critical check.
    pub const CRIT: Self = Self(1);
    /// Effect/debuff application check.
    pub const EFFECT_CHANCE: Self = Self(2);
    /// Bounce target selection.
    pub const BOUNCE_TARGET: Self = Self(3);
    /// Aggro-weighted target selection.
    pub const AGGRO_TARGET: Self = Self(4);
    /// Authored enemy behavior choice.
    pub const BEHAVIOR_CHOICE: Self = Self(5);

    /// Creates a stable non-zero extension purpose code.
    #[must_use]
    pub const fn new(code: u16) -> Option<Self> {
        if code == 0 { None } else { Some(Self(code)) }
    }

    /// Returns the canonical purpose code.
    #[must_use]
    pub const fn code(self) -> u16 {
        self.0
    }
}

/// One raw generator word and its monotonic identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawSample {
    index: u64,
    purpose: DrawPurpose,
    raw: u64,
}

impl DrawSample {
    pub(super) const fn new(index: u64, purpose: DrawPurpose, raw: u64) -> Self {
        Self {
            index,
            purpose,
            raw,
        }
    }

    /// Returns the zero-based raw-draw index.
    #[must_use]
    pub const fn index(self) -> u64 {
        self.index
    }
    /// Returns the stable purpose tag.
    #[must_use]
    pub const fn purpose(self) -> DrawPurpose {
        self.purpose
    }
    /// Returns the raw `u64` sample before project mapping.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.raw
    }
}

/// Accepted result of unbiased mapping into `[0, upper)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeSelection {
    pub(super) sample: DrawSample,
    pub(super) upper: u64,
    pub(super) value: u64,
    pub(super) rejected_draws: u32,
}

impl RangeSelection {
    /// Returns the accepted raw sample.
    #[must_use]
    pub const fn sample(self) -> DrawSample {
        self.sample
    }
    /// Returns the exclusive upper bound.
    #[must_use]
    pub const fn upper(self) -> u64 {
        self.upper
    }
    /// Returns the mapped value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
    /// Returns how many preceding raw samples were rejected.
    #[must_use]
    pub const fn rejected_draws(self) -> u32 {
        self.rejected_draws
    }
}

/// Accepted weighted candidate and its underlying range selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeightedSelection {
    pub(super) range: RangeSelection,
    pub(super) index: u32,
}

impl WeightedSelection {
    /// Returns the authored candidate index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }
    /// Returns the integer range selection over the total weight.
    #[must_use]
    pub const fn range(self) -> RangeSelection {
        self.range
    }
}

/// Stable deterministic RNG validation or exhaustion failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RngError {
    /// A stream text identity is empty, oversized or not printable ASCII.
    InvalidStreamIdentity,
    /// A direct range request has an exclusive upper bound of zero.
    EmptyRange,
    /// The monotonic raw-draw counter cannot advance.
    DrawCounterExhausted,
    /// More rejections occurred than the diagnostic counter can represent.
    RejectionBudgetExhausted,
    /// Integer candidate weights overflowed their `u64` total.
    WeightTotalOverflow,
    /// A candidate slice cannot be represented by canonical `u32` indexes.
    TooManyCandidates,
    /// Validated weights and mapping did not resolve a candidate.
    MappingInvariant,
}

impl fmt::Display for RngError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidStreamIdentity => "invalid RNG stream identity",
            Self::EmptyRange => "RNG range must be non-empty",
            Self::DrawCounterExhausted => "RNG draw counter exhausted",
            Self::RejectionBudgetExhausted => "RNG rejection counter exhausted",
            Self::WeightTotalOverflow => "RNG weight total overflow",
            Self::TooManyCandidates => "too many RNG candidates",
            Self::MappingInvariant => "RNG mapping invariant failed",
        })
    }
}

impl std::error::Error for RngError {}
