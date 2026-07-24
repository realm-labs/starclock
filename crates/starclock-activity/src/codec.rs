use sha2::{Digest, Sha256};

macro_rules! digest_type {
    ($name:ident, $description:literal, $checked:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Creates a digest from canonical SHA-256 bytes.
            #[must_use]
            pub const fn new(bytes: [u8; 32]) -> Option<Self> {
                if $checked && all_zero(&bytes) {
                    None
                } else {
                    Some(Self(bytes))
                }
            }

            /// Returns the exact canonical bytes.
            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

const fn all_zero(bytes: &[u8; 32]) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0 {
            return false;
        }
        index += 1;
    }
    true
}

digest_type!(
    ActivityDefinitionDigest,
    "Digest of the immutable activity definition.",
    true
);

pub const ACTIVITY_STATE_CODEC_REVISION: &str = "starclock-activity-state-v3";
pub const ACTIVITY_STATE_HASH_REVISION: &str = "sha256-v5";
digest_type!(
    ActivityGraphDigest,
    "Digest of one validated immutable Activity graph.",
    true
);
digest_type!(
    ActivityConfigDigest,
    "Digest of the complete validated activity configuration.",
    true
);
digest_type!(
    ParticipantLockDigest,
    "Digest of the canonical participant/loadout lock.",
    true
);
digest_type!(
    BuildDigest,
    "Opaque digest of an upstream selected combatant build.",
    true
);
digest_type!(
    EventDigest,
    "Digest of the complete ordered battle-event stream.",
    true
);
digest_type!(
    BattleResultDigest,
    "Digest of one returned battle-result envelope.",
    true
);
digest_type!(
    ActivityStateHash,
    "SHA-256 digest of canonical activity state at a command boundary.",
    false
);
digest_type!(
    EncounterPreparationDigest,
    "Digest of one validated immutable encounter-preparation definition.",
    true
);
digest_type!(
    TechniqueContributionDigest,
    "Opaque digest of the battle contributions compiled for one technique sequence.",
    true
);
digest_type!(
    BattleProjectionDigest,
    "Digest of one declared battle-result projection and its exact ordered fields.",
    true
);
digest_type!(
    BattleSettlementContractDigest,
    "Digest of one battle-result projection plus its carry and metric settlement policy.",
    true
);

pub(crate) struct CanonicalWriter(Sha256);

impl CanonicalWriter {
    pub(crate) fn new(domain: &[u8]) -> Self {
        let mut writer = Self(Sha256::new());
        writer.bytes(domain);
        writer
    }

    pub(crate) fn byte(&mut self, value: u8) {
        self.0.update([value]);
    }

    pub(crate) fn bool(&mut self, value: bool) {
        self.byte(u8::from(value));
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.0.update(value.to_be_bytes());
    }

    pub(crate) fn u64(&mut self, value: u64) {
        self.0.update(value.to_be_bytes());
    }

    pub(crate) fn i64(&mut self, value: i64) {
        self.0.update(value.to_be_bytes());
    }

    pub(crate) fn bytes(&mut self, value: &[u8]) {
        self.u64(value.len() as u64);
        self.0.update(value);
    }

    pub(crate) fn text(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }

    pub(crate) fn digest(&mut self, value: [u8; 32]) {
        self.0.update(value);
    }

    pub(crate) fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}

/// Goal 04 definition/state primitives. Kept separate from the legacy writer
/// so old one-battle bytes cannot be silently relabeled as Activity v2 bytes.
pub(crate) struct ActivityV2Writer(Sha256);

impl ActivityV2Writer {
    pub(crate) fn new(magic: [u8; 4], version: u32, domain: &[u8]) -> Self {
        let mut hash = Sha256::new();
        hash.update(magic);
        hash.update(version.to_le_bytes());
        hash.update((domain.len() as u32).to_le_bytes());
        hash.update(domain);
        Self(hash)
    }

    pub(crate) fn byte(&mut self, value: u8) {
        self.0.update([value]);
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.0.update(value.to_le_bytes());
    }

    pub(crate) fn i32(&mut self, value: i32) {
        self.0.update(value.to_le_bytes());
    }

    pub(crate) fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}

pub(crate) struct ActivityStateEncoder(Vec<u8>);

impl ActivityStateEncoder {
    pub(crate) fn new() -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"SCAS");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        Self(bytes)
    }
    pub(crate) fn byte(&mut self, value: u8) {
        self.0.push(value);
    }
    pub(crate) fn bool(&mut self, value: bool) {
        self.byte(u8::from(value));
    }
    pub(crate) fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    pub(crate) fn u64(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    pub(crate) fn i32(&mut self, value: i32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    pub(crate) fn i64(&mut self, value: i64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    pub(crate) fn digest(&mut self, value: [u8; 32]) {
        self.0.extend_from_slice(&value);
    }
    pub(crate) fn text(&mut self, value: &str) {
        self.u32(value.len() as u32);
        self.0.extend_from_slice(value.as_bytes());
    }
    pub(crate) fn finish(self) -> Box<[u8]> {
        self.0.into_boxed_slice()
    }
}

/// Canonical little-endian writer for immutable Activity extension identities.
pub(crate) struct ActivityRegistryWriter(Sha256);

impl ActivityRegistryWriter {
    pub(crate) fn new(domain: &[u8]) -> Self {
        let mut hash = Sha256::new();
        hash.update(b"SCAR");
        hash.update(1_u32.to_le_bytes());
        hash.update(
            u32::try_from(domain.len())
                .expect("static registry domain length fits u32")
                .to_le_bytes(),
        );
        hash.update(domain);
        Self(hash)
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.0.update(value.to_le_bytes());
    }

    pub(crate) fn text(&mut self, value: &str) {
        self.u32(u32::try_from(value.len()).expect("validated registry text length fits u32"));
        self.0.update(value.as_bytes());
    }

    pub(crate) fn digest(&mut self, value: [u8; 32]) {
        self.0.update(value);
    }

    pub(crate) fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}
