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

definition_id!(UnitDefinitionId, "Stable catalog identity of a unit form.");
definition_id!(
    AbilityId,
    "Stable catalog identity of an ability definition."
);
definition_id!(
    EffectDefinitionId,
    "Stable catalog identity of an effect definition."
);
definition_id!(
    RuleId,
    "Stable catalog identity of a typed rule definition."
);
definition_id!(ProgramId, "Stable catalog identity of a typed program.");
definition_id!(SelectorId, "Stable catalog identity of a typed selector.");
definition_id!(
    RuleBundleId,
    "Stable catalog identity of an ordered rule bundle."
);
definition_id!(
    ModifierDefinitionId,
    "Stable catalog identity of a modifier definition."
);
definition_id!(
    EnemyDefinitionId,
    "Stable catalog identity of an enemy definition."
);
definition_id!(
    EncounterId,
    "Stable catalog identity of an encounter definition."
);
definition_id!(
    NativeHandlerId,
    "Stable catalog identity of a validated static native handler."
);
definition_id!(
    StateSlotDefinitionId,
    "Stable catalog identity of a typed rule-state slot."
);
definition_id!(TriggerId, "Stable catalog identity of a rule trigger.");
definition_id!(
    HitPlanDefinitionId,
    "Stable catalog identity of an ordered hit-plan definition."
);
definition_id!(
    SourceDefinitionId,
    "Stable catalog identity used for generic rule-source attribution."
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
    ShieldInstanceId,
    "Battle-local monotonic identity of a shield instance."
);
runtime_id!(
    RuleInstanceId,
    "Battle-local monotonic identity of a bound rule instance."
);
runtime_id!(
    ModifierInstanceId,
    "Battle-local monotonic identity of an active modifier instance."
);
runtime_id!(
    ActionId,
    "Battle-local monotonic identity of an action envelope."
);
runtime_id!(
    PhaseId,
    "Battle-local monotonic identity of an action phase."
);
runtime_id!(HitId, "Battle-local monotonic identity of an authored hit.");
runtime_id!(
    OperationId,
    "Battle-local monotonic identity of a requested operation."
);
runtime_id!(
    EventId,
    "Battle-local monotonic identity of an emitted event."
);
runtime_id!(
    DecisionId,
    "Battle-local monotonic identity of an externally visible decision point."
);
runtime_id!(
    CommandId,
    "Battle-local monotonic identity of one accepted external command."
);
runtime_id!(
    WaveInstanceId,
    "Battle-local monotonic identity of an encounter wave instance."
);
runtime_id!(
    SpawnSequence,
    "Battle-local monotonic spawn sequence used as a final stable tie-breaker."
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
        assert_ne!(
            core::any::TypeId::of::<UnitDefinitionId>(),
            core::any::TypeId::of::<EnemyDefinitionId>()
        );
    }

    #[test]
    fn runtime_ids_reject_zero_and_preserve_fixed_width() {
        assert_eq!(EventId::new(0), None);
        let id = EventId::new(u64::MAX).expect("maximum u64 is non-zero");
        assert_eq!(id.get(), u64::MAX);
        assert_eq!(size_of::<EventId>(), size_of::<u64>());
        assert_ne!(
            core::any::TypeId::of::<EffectInstanceId>(),
            core::any::TypeId::of::<ShieldInstanceId>()
        );
    }
}
