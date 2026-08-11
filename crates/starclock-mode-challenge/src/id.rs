use core::num::NonZeroU32;

macro_rules! id_type {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

id_type!(ChallengeProfileId, "Stable challenge profile identity.");
id_type!(ChallengeStageId, "Stable challenge stage identity.");
id_type!(ChallengeNodeId, "Stable challenge node identity.");
id_type!(ObjectiveId, "Stable authored objective identity.");
id_type!(
    MemoryEnemyBindingId,
    "Stable Memory of Chaos enemy behavior binding identity."
);
id_type!(
    ApocalypticEnemyBindingId,
    "Stable Apocalyptic Shadow enemy behavior binding identity."
);
id_type!(
    PureFictionEnemyBindingId,
    "Stable Pure Fiction enemy behavior binding identity."
);
id_type!(
    AnomalyQuadrantId,
    "Stable Anomaly Arbitration Quadrant option identity."
);
