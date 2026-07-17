use sha2::{Digest, Sha256};

use super::{
    RNG_ALGORITHM_REVISION,
    types::{RngError, RngSeed},
};

const DOMAIN: &[u8] = b"starclock-rng-stream-v1\0";
const MAX_TEXT_BYTES: usize = 128;

/// Validated canonical path for one independent deterministic RNG stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamPath {
    activity_profile_id: Box<str>,
    activity_instance_id: u64,
    section: u32,
    node: u32,
    attempt: u32,
    battle_sequence: u32,
    label: Box<str>,
}

impl StreamPath {
    /// Creates a stream path. Text identities must be non-empty printable ASCII.
    pub fn new(
        activity_profile_id: impl Into<Box<str>>,
        activity_instance_id: u64,
        section: u32,
        node: u32,
        attempt: u32,
        battle_sequence: u32,
        label: impl Into<Box<str>>,
    ) -> Result<Self, RngError> {
        let activity_profile_id = activity_profile_id.into();
        let label = label.into();
        validate_text(&activity_profile_id)?;
        validate_text(&label)?;
        Ok(Self {
            activity_profile_id,
            activity_instance_id,
            section,
            node,
            attempt,
            battle_sequence,
            label,
        })
    }

    /// Derives the 32-byte ChaCha8 seed from a master activity seed.
    #[must_use]
    pub fn derive_seed(&self, master_seed: u64) -> RngSeed {
        let mut hasher = Sha256::new();
        hasher.update(DOMAIN);
        write_text(&mut hasher, RNG_ALGORITHM_REVISION);
        hasher.update(master_seed.to_be_bytes());
        write_text(&mut hasher, &self.activity_profile_id);
        hasher.update(self.activity_instance_id.to_be_bytes());
        hasher.update(self.section.to_be_bytes());
        hasher.update(self.node.to_be_bytes());
        hasher.update(self.attempt.to_be_bytes());
        hasher.update(self.battle_sequence.to_be_bytes());
        write_text(&mut hasher, &self.label);
        RngSeed::new(hasher.finalize().into())
    }

    /// Returns the ASCII stream label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

fn validate_text(value: &str) -> Result<(), RngError> {
    if value.is_empty()
        || value.len() > MAX_TEXT_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(RngError::InvalidStreamIdentity);
    }
    Ok(())
}

fn write_text(hasher: &mut Sha256, value: &str) {
    let length = u16::try_from(value.len()).expect("validated stream text fits u16");
    hasher.update(length.to_be_bytes());
    hasher.update(value.as_bytes());
}
