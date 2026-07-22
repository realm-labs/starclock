//! Domain digest wrappers and private canonical composition.

use sha2::{Digest, Sha256};

macro_rules! digest_type {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            #[must_use]
            pub const fn new(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

digest_type!(
    UniverseBundleDigest,
    "SHA-256 identity of the exact isolated Universe Sora bundle."
);
digest_type!(
    UniverseProfileDigest,
    "Canonical identity of the validated Standard Universe profile row."
);
digest_type!(
    ActivityConfigurationDigest,
    "Composed combat/build/Universe/profile Activity configuration identity."
);
digest_type!(
    UniverseDefinitionsDigest,
    "Canonical identity of lowered Standard Universe structural definitions."
);

pub(crate) fn bundle_digest(bytes: &[u8]) -> UniverseBundleDigest {
    UniverseBundleDigest::new(Sha256::digest(bytes).into())
}

pub(crate) struct Encoder(Sha256);

impl Encoder {
    pub(crate) fn new(domain: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        write_bytes(&mut hasher, domain);
        Self(hasher)
    }

    pub(crate) fn text(&mut self, value: &str) {
        write_bytes(&mut self.0, value.as_bytes());
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.0.update(value.to_le_bytes());
    }

    pub(crate) fn u8(&mut self, value: u8) {
        self.0.update([value]);
    }

    pub(crate) fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    pub(crate) fn optional_text(&mut self, value: Option<&str>) {
        match value {
            Some(value) => {
                self.bool(true);
                self.text(value);
            }
            None => self.bool(false),
        }
    }

    pub(crate) fn digest(&mut self, value: [u8; 32]) {
        self.0.update(value);
    }

    pub(crate) fn optional_digest(&mut self, value: Option<[u8; 32]>) {
        match value {
            Some(value) => {
                self.0.update([1]);
                self.digest(value);
            }
            None => self.0.update([0]),
        }
    }

    pub(crate) fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}

fn write_bytes(hasher: &mut Sha256, value: &[u8]) {
    let length = u32::try_from(value.len()).expect("canonical static/domain text fits u32");
    hasher.update(length.to_le_bytes());
    hasher.update(value);
}
