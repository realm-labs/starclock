use sha2::{Digest, Sha256};

use crate::codec::CanonicalSink;

macro_rules! digest_type {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Creates a digest from exact SHA-256 bytes.
            #[must_use]
            pub const fn new(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }
            /// Returns exact digest bytes.
            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

digest_type!(
    ConfigBundleDigest,
    "Exact production configuration-bundle digest."
);
digest_type!(
    EntrySpecDigest,
    "Digest of the replay entry's resolved specification."
);
digest_type!(
    ControllerDigest,
    "Digest of deterministic controller configuration."
);
digest_type!(
    DefinitionDigest,
    "Digest of an activity or battle definition."
);
digest_type!(
    BuildCatalogDigest,
    "Digest of the build catalog used by a build-aware entry."
);
digest_type!(
    CombatantBuildDigest,
    "Digest of one ordered selected character build."
);
digest_type!(
    StateDigest,
    "Canonical authoritative state digest at one command boundary."
);
digest_type!(
    Sha256Digest,
    "SHA-256 digest of an explicitly selected canonical byte stream."
);

/// Streaming SHA-256 byte sink with no canonical-state byte allocation.
#[derive(Debug, Default)]
pub struct Sha256Sink(Sha256);

impl Sha256Sink {
    /// Creates an empty SHA-256 stream.
    #[must_use]
    pub fn new() -> Self {
        Self(Sha256::new())
    }
    /// Finalizes the exact bytes written so far.
    #[must_use]
    pub fn finalize(self) -> Sha256Digest {
        Sha256Digest::new(self.0.finalize().into())
    }
}

impl CanonicalSink for Sha256Sink {
    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}
