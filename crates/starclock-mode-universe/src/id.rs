//! Stable authored identities owned by the Universe domain.

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

id_type!(UniverseProfileId, "Stable Standard Universe profile ID.");
id_type!(WorldId, "Stable authored World ID.");
id_type!(DifficultyId, "Stable authored World difficulty ID.");
id_type!(DomainId, "Stable authored abstract domain ID.");
id_type!(TopologyNodeId, "Stable authored abstract topology-node ID.");
id_type!(
    TopologyId,
    "Stable topology ID derived from its canonical start-node row."
);
id_type!(RoomId, "Stable authored abstract room ID.");
id_type!(ActivityBindingId, "Stable Universe-to-Activity binding ID.");
