use core::fmt;
use core::num::{NonZeroU32, NonZeroU64};

/// Error returned when zero is supplied for a stable definition or runtime ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZeroIdError;

impl fmt::Display for ZeroIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("stable IDs must be non-zero")
    }
}

impl std::error::Error for ZeroIdError {}

macro_rules! definition_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            /// Creates an ID, returning `None` when `raw` is zero.
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            /// Returns the stable fixed-width integer representation.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }

        impl TryFrom<u32> for $name {
            type Error = ZeroIdError;

            fn try_from(raw: u32) -> Result<Self, Self::Error> {
                Self::new(raw).ok_or(ZeroIdError)
            }
        }

        impl From<$name> for u32 {
            fn from(id: $name) -> Self {
                id.get()
            }
        }
    };
}

macro_rules! runtime_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU64);

        impl $name {
            /// Creates an ID, returning `None` when `raw` is zero.
            #[must_use]
            pub const fn new(raw: u64) -> Option<Self> {
                match NonZeroU64::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            /// Returns the monotonic fixed-width integer representation.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0.get()
            }
        }

        impl TryFrom<u64> for $name {
            type Error = ZeroIdError;

            fn try_from(raw: u64) -> Result<Self, Self::Error> {
                Self::new(raw).ok_or(ZeroIdError)
            }
        }

        impl From<$name> for u64 {
            fn from(id: $name) -> Self {
                id.get()
            }
        }
    };
}

definition_id!(
    CombatantId,
    "Stable catalog identity of a resolved combatant definition."
);
definition_id!(
    AbilityId,
    "Stable catalog identity of an ability definition."
);
definition_id!(EffectId, "Stable catalog identity of an effect definition.");
definition_id!(
    RuleId,
    "Stable catalog identity of a typed rule definition."
);
definition_id!(
    EncounterId,
    "Stable catalog identity of an encounter definition."
);

runtime_id!(
    UnitId,
    "Battle-local monotonic identity of a targetable or linked unit."
);
runtime_id!(
    TimelineActorId,
    "Battle-local monotonic identity of an action-gauge actor."
);
runtime_id!(
    EffectInstanceId,
    "Battle-local monotonic identity of an applied effect instance."
);
runtime_id!(
    ActionId,
    "Battle-local monotonic identity of an action envelope."
);
runtime_id!(
    EventId,
    "Battle-local monotonic identity of an emitted event."
);

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn definition_ids_reject_zero_and_order_by_raw_value() {
        assert_eq!(AbilityId::new(0), None);
        assert_eq!(AbilityId::try_from(0), Err(ZeroIdError));
        let low = AbilityId::new(1).expect("one is non-zero");
        let high = AbilityId::new(u32::MAX).expect("maximum u32 is non-zero");
        assert!(low < high);
        assert_eq!(u32::from(high), u32::MAX);
        assert_eq!(size_of::<AbilityId>(), size_of::<u32>());
    }

    #[test]
    fn runtime_ids_reject_zero_and_preserve_fixed_width() {
        assert_eq!(EventId::new(0), None);
        let id = EventId::new(u64::MAX).expect("maximum u64 is non-zero");
        assert_eq!(id.get(), u64::MAX);
        assert_eq!(size_of::<EventId>(), size_of::<u64>());
    }
}
