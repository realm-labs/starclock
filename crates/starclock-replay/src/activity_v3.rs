//! Replay-v3 nested battle identity boundaries.

use core::fmt;

use starclock_activity::{BattleResultDigest, BattleResultIdentity};

use crate::{
    codec::{CodecError, Decoder, Encoder},
    component::MAX_COMPONENT_TEXT_BYTES,
    digest::ComponentRootDigest,
};

use super::{
    fixed_digest,
    identity::{decode_identity, encode_identity},
};

pub const NESTED_BATTLE_IDENTITY_PAYLOAD_VERSION_V3: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedBattleStartV3 {
    component_root: ComponentRootDigest,
    combat_input_codec_revision: Box<str>,
    handoff_identity: BattleResultIdentity,
}

impl NestedBattleStartV3 {
    pub fn new(
        component_root: ComponentRootDigest,
        combat_input_codec_revision: impl Into<Box<str>>,
        handoff_identity: BattleResultIdentity,
    ) -> Result<Self, NestedBattleV3PayloadError> {
        let revision = combat_input_codec_revision.into();
        validate_revision(&revision)?;
        Ok(Self {
            component_root,
            combat_input_codec_revision: revision,
            handoff_identity,
        })
    }

    #[must_use]
    pub const fn component_root(&self) -> ComponentRootDigest {
        self.component_root
    }
    #[must_use]
    pub fn combat_input_codec_revision(&self) -> &str {
        &self.combat_input_codec_revision
    }
    #[must_use]
    pub const fn handoff_identity(&self) -> BattleResultIdentity {
        self.handoff_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NestedBattleEndV3 {
    result_identity: BattleResultIdentity,
    result_digest: BattleResultDigest,
}

impl NestedBattleEndV3 {
    #[must_use]
    pub const fn new(
        result_identity: BattleResultIdentity,
        result_digest: BattleResultDigest,
    ) -> Self {
        Self {
            result_identity,
            result_digest,
        }
    }
    #[must_use]
    pub const fn result_identity(self) -> BattleResultIdentity {
        self.result_identity
    }
    #[must_use]
    pub const fn result_digest(self) -> BattleResultDigest {
        self.result_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NestedBattleIdentityDivergence {
    Component,
    Assembly,
    CombatInput,
    Handoff,
    Result,
}

pub fn compare_nested_start_v3(
    recorded: &NestedBattleStartV3,
    actual_component_root: ComponentRootDigest,
    actual_codec_revision: &str,
    actual_handoff: BattleResultIdentity,
) -> Result<(), NestedBattleIdentityDivergence> {
    if recorded.component_root != actual_component_root {
        return Err(NestedBattleIdentityDivergence::Component);
    }
    if recorded.handoff_identity.assembly_digest() != actual_handoff.assembly_digest() {
        return Err(NestedBattleIdentityDivergence::Assembly);
    }
    if recorded.combat_input_codec_revision.as_ref() != actual_codec_revision
        || recorded.handoff_identity.combat_input_digest() != actual_handoff.combat_input_digest()
    {
        return Err(NestedBattleIdentityDivergence::CombatInput);
    }
    if recorded.handoff_identity != actual_handoff {
        return Err(NestedBattleIdentityDivergence::Handoff);
    }
    Ok(())
}

pub fn compare_nested_end_v3(
    recorded: NestedBattleEndV3,
    actual_identity: BattleResultIdentity,
    actual_digest: BattleResultDigest,
) -> Result<(), NestedBattleIdentityDivergence> {
    if recorded.result_identity != actual_identity || recorded.result_digest != actual_digest {
        return Err(NestedBattleIdentityDivergence::Result);
    }
    Ok(())
}

pub fn encode_nested_battle_start_v3(
    value: &NestedBattleStartV3,
) -> Result<Vec<u8>, NestedBattleV3PayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_IDENTITY_PAYLOAD_VERSION_V3);
    encoder.raw(&value.component_root.bytes());
    encoder.string(&value.combat_input_codec_revision)?;
    encode_identity(value.handoff_identity, &mut encoder);
    Ok(encoder.into_inner())
}

pub fn decode_nested_battle_start_v3(
    bytes: &[u8],
) -> Result<NestedBattleStartV3, NestedBattleV3PayloadError> {
    let mut decoder = Decoder::new(bytes);
    validate_version(decoder.u16()?)?;
    let root = ComponentRootDigest::new(fixed_digest(&mut decoder)?);
    let revision = decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?;
    let identity =
        decode_identity(&mut decoder).map_err(NestedBattleV3PayloadError::ActivityIdentity)?;
    decoder.finish()?;
    NestedBattleStartV3::new(root, revision, identity)
}

pub fn encode_nested_battle_end_v3(value: NestedBattleEndV3) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_IDENTITY_PAYLOAD_VERSION_V3);
    encode_identity(value.result_identity, &mut encoder);
    encoder.raw(&value.result_digest.bytes());
    encoder.into_inner()
}

pub fn decode_nested_battle_end_v3(
    bytes: &[u8],
) -> Result<NestedBattleEndV3, NestedBattleV3PayloadError> {
    let mut decoder = Decoder::new(bytes);
    validate_version(decoder.u16()?)?;
    let identity =
        decode_identity(&mut decoder).map_err(NestedBattleV3PayloadError::ActivityIdentity)?;
    let digest = BattleResultDigest::new(fixed_digest(&mut decoder)?)
        .ok_or(NestedBattleV3PayloadError::InvalidDigest)?;
    decoder.finish()?;
    Ok(NestedBattleEndV3::new(identity, digest))
}

fn validate_version(version: u16) -> Result<(), NestedBattleV3PayloadError> {
    if version == NESTED_BATTLE_IDENTITY_PAYLOAD_VERSION_V3 {
        Ok(())
    } else {
        Err(NestedBattleV3PayloadError::UnsupportedVersion(version))
    }
}

fn validate_revision(value: &str) -> Result<(), NestedBattleV3PayloadError> {
    if value.is_empty()
        || value.len() > MAX_COMPONENT_TEXT_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        Err(NestedBattleV3PayloadError::InvalidRevision)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NestedBattleV3PayloadError {
    Codec(CodecError),
    ActivityIdentity(super::ActivityCommandPayloadError),
    UnsupportedVersion(u16),
    InvalidRevision,
    InvalidDigest,
}

impl From<CodecError> for NestedBattleV3PayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

impl fmt::Display for NestedBattleV3PayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nested battle replay v3 payload error: {self:?}")
    }
}

impl std::error::Error for NestedBattleV3PayloadError {}
