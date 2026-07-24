//! Current dual-digest and historical single-digest Activity identities.

use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityInstanceId, AttemptId,
    BattleResultConfiguration, BattleResultIdentity, BattleSequence, ScopeIdentity, SectionId,
};
use starclock_combat::{AssemblyDigest, BattleSeed, BattleSpecDigest, CombatInputDigest};

use crate::codec::{Decoder, Encoder};

use super::{ActivityCommandPayloadError, fixed_digest};

pub(super) fn encode_identity(identity: BattleResultIdentity, encoder: &mut Encoder<Vec<u8>>) {
    encoder.u64(identity.activity().get());
    encoder.u32(identity.scope().section().get());
    encoder.u32(identity.scope().node().get());
    encoder.u32(identity.scope().attempt().get());
    encoder.u32(identity.battle_sequence().get());
    encoder.raw(&identity.definition_digest().bytes());
    encoder.raw(&identity.config_digest().bytes());
    encoder.raw(&identity.participant_lock_digest().bytes());
    encoder.raw(&identity.combat_input_digest().bytes());
    encoder.raw(&identity.assembly_digest().bytes());
    encoder.raw(&identity.seed().bytes());
}

pub(super) fn decode_identity(
    decoder: &mut Decoder<'_>,
) -> Result<BattleResultIdentity, ActivityCommandPayloadError> {
    let (scope, sequence, configuration) = decode_prefix(decoder)?;
    let combat_input = CombatInputDigest::new(fixed_digest(decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    let assembly = AssemblyDigest::new(fixed_digest(decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    let seed = BattleSeed::new(fixed_digest(decoder)?);
    Ok(BattleResultIdentity::new(
        scope,
        sequence,
        configuration,
        combat_input,
        assembly,
        seed,
    ))
}

pub(super) fn decode_identity_legacy(
    decoder: &mut Decoder<'_>,
) -> Result<BattleResultIdentity, ActivityCommandPayloadError> {
    let (scope, sequence, configuration) = decode_prefix(decoder)?;
    let spec = BattleSpecDigest::new(fixed_digest(decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    let seed = BattleSeed::new(fixed_digest(decoder)?);
    Ok(BattleResultIdentity::new_legacy(
        scope,
        sequence,
        configuration,
        spec,
        seed,
    ))
}

fn decode_prefix(
    decoder: &mut Decoder<'_>,
) -> Result<(ScopeIdentity, BattleSequence, BattleResultConfiguration), ActivityCommandPayloadError>
{
    let activity =
        ActivityInstanceId::new(decoder.u64()?).ok_or(ActivityCommandPayloadError::InvalidId)?;
    let section = SectionId::new(decoder.u32()?).ok_or(ActivityCommandPayloadError::InvalidId)?;
    let node = starclock_activity::NodeId::new(decoder.u32()?)
        .ok_or(ActivityCommandPayloadError::InvalidId)?;
    let attempt = AttemptId::new(decoder.u32()?).ok_or(ActivityCommandPayloadError::InvalidId)?;
    let sequence =
        BattleSequence::new(decoder.u32()?).ok_or(ActivityCommandPayloadError::InvalidId)?;
    let definition = ActivityDefinitionDigest::new(fixed_digest(decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    let config = ActivityConfigDigest::new(fixed_digest(decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    let lock = starclock_activity::ParticipantLockDigest::new(fixed_digest(decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    Ok((
        ScopeIdentity::new(activity, section, node, attempt),
        sequence,
        BattleResultConfiguration::new(definition, config, lock),
    ))
}
