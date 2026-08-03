//! Nested-battle identity boundaries for component-addressed replays.

use core::fmt;

use starclock_activity::{BattleResultDigest, BattleResultIdentity};

use crate::{
    codec::{CodecError, Decoder, Encoder},
    digest::ComponentRootDigest,
};

use super::ActivityCommandPayloadError;
use super::{
    fixed_digest,
    identity::{decode_identity, encode_identity},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedBattleStart {
    component_root: ComponentRootDigest,
    handoff_identity: BattleResultIdentity,
}

impl NestedBattleStart {
    #[must_use]
    pub const fn new(
        component_root: ComponentRootDigest,
        handoff_identity: BattleResultIdentity,
    ) -> Self {
        Self {
            component_root,
            handoff_identity,
        }
    }

    #[must_use]
    pub const fn component_root(&self) -> ComponentRootDigest {
        self.component_root
    }
    #[must_use]
    pub const fn handoff_identity(&self) -> BattleResultIdentity {
        self.handoff_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NestedBattleEnd {
    result_identity: BattleResultIdentity,
    result_digest: BattleResultDigest,
}

impl NestedBattleEnd {
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

pub fn compare_nested_start(
    recorded: &NestedBattleStart,
    actual_component_root: ComponentRootDigest,
    actual_handoff: BattleResultIdentity,
) -> Result<(), NestedBattleIdentityDivergence> {
    if recorded.component_root != actual_component_root {
        return Err(NestedBattleIdentityDivergence::Component);
    }
    if recorded.handoff_identity.assembly_digest() != actual_handoff.assembly_digest() {
        return Err(NestedBattleIdentityDivergence::Assembly);
    }
    if recorded.handoff_identity.combat_input_digest() != actual_handoff.combat_input_digest() {
        return Err(NestedBattleIdentityDivergence::CombatInput);
    }
    if recorded.handoff_identity != actual_handoff {
        return Err(NestedBattleIdentityDivergence::Handoff);
    }
    Ok(())
}

pub fn compare_nested_end(
    recorded: NestedBattleEnd,
    actual_identity: BattleResultIdentity,
    actual_digest: BattleResultDigest,
) -> Result<(), NestedBattleIdentityDivergence> {
    if recorded.result_identity != actual_identity || recorded.result_digest != actual_digest {
        return Err(NestedBattleIdentityDivergence::Result);
    }
    Ok(())
}

pub fn encode_nested_battle_start(
    value: &NestedBattleStart,
) -> Result<Vec<u8>, NestedBattleIdentityPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.raw(&value.component_root.bytes());
    encode_identity(value.handoff_identity, &mut encoder);
    Ok(encoder.into_inner())
}

pub fn decode_nested_battle_start(
    bytes: &[u8],
) -> Result<NestedBattleStart, NestedBattleIdentityPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let root = ComponentRootDigest::new(fixed_digest(&mut decoder)?);
    let identity = decode_identity(&mut decoder)
        .map_err(NestedBattleIdentityPayloadError::ActivityIdentity)?;
    decoder.finish()?;
    Ok(NestedBattleStart::new(root, identity))
}

pub fn encode_nested_battle_end(value: NestedBattleEnd) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encode_identity(value.result_identity, &mut encoder);
    encoder.raw(&value.result_digest.bytes());
    encoder.into_inner()
}

pub fn decode_nested_battle_end(
    bytes: &[u8],
) -> Result<NestedBattleEnd, NestedBattleIdentityPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let identity = decode_identity(&mut decoder)
        .map_err(NestedBattleIdentityPayloadError::ActivityIdentity)?;
    let digest = BattleResultDigest::new(fixed_digest(&mut decoder)?)
        .ok_or(NestedBattleIdentityPayloadError::InvalidDigest)?;
    decoder.finish()?;
    Ok(NestedBattleEnd::new(identity, digest))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NestedBattleIdentityPayloadError {
    Codec(CodecError),
    ActivityIdentity(ActivityCommandPayloadError),
    InvalidDigest,
}

impl From<CodecError> for NestedBattleIdentityPayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

impl fmt::Display for NestedBattleIdentityPayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nested battle identity payload error: {self:?}")
    }
}

impl std::error::Error for NestedBattleIdentityPayloadError {}
