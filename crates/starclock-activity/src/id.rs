use core::num::{NonZeroU32, NonZeroU64};

macro_rules! id_type {
    ($name:ident, $raw:ty, $nonzero:ty, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name($nonzero);

        impl $name {
            /// Creates an identity from a non-zero authored value.
            #[must_use]
            pub const fn new(raw: $raw) -> Option<Self> {
                match <$nonzero>::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            /// Returns the stable integer representation.
            #[must_use]
            pub const fn get(self) -> $raw {
                self.0.get()
            }
        }
    };
}

id_type!(
    ActivityDefinitionId,
    u32,
    NonZeroU32,
    "Stable authored activity definition ID."
);
id_type!(
    ActivityInstanceId,
    u64,
    NonZeroU64,
    "Stable identity of one activity execution."
);
id_type!(
    SectionId,
    u32,
    NonZeroU32,
    "Stable generic Section definition ID."
);
id_type!(
    NodeId,
    u32,
    NonZeroU32,
    "Stable generic Node definition ID."
);
id_type!(
    ActivityEdgeId,
    u32,
    NonZeroU32,
    "Stable authored edge identity within an Activity graph."
);
id_type!(
    AttemptId,
    u32,
    NonZeroU32,
    "Stable attempt identity within one node visit."
);
id_type!(
    BattleSequence,
    u32,
    NonZeroU32,
    "One-based battle sequence within an activity."
);
id_type!(
    ActivitySlotId,
    u32,
    NonZeroU32,
    "Stable typed activity-slot ID."
);
id_type!(
    ParticipantId,
    u32,
    NonZeroU32,
    "Stable activity participant identity."
);
id_type!(
    ProjectionId,
    u32,
    NonZeroU32,
    "Stable battle-result projection ID."
);
