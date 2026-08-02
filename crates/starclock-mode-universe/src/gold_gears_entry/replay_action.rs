//! Canonical Gold complete-run action payload and policy identities.

use starclock_replay::{
    codec::{CodecError, Decoder, Encoder},
    component::MAX_COMPONENT_TEXT_BYTES,
};

use super::{GoldAndGearsEncounterRole, GoldAndGearsSeededRunAction};

pub(super) fn encode_action(
    action: &GoldAndGearsSeededRunAction,
) -> Result<Vec<u8>, ActionPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    match action {
        GoldAndGearsSeededRunAction::PlaneCreation { source_node, plane } => {
            encoder.u8(0);
            encoder.u32(source_node.get());
            encoder.u8(*plane);
        }
        GoldAndGearsSeededRunAction::BossSelection {
            source_node,
            plane,
            boss,
        } => {
            encoder.u8(1);
            encoder.u32(source_node.get());
            encoder.u8(*plane);
            encoder.string(boss)?;
        }
        GoldAndGearsSeededRunAction::Traverse { source_node, edge } => {
            encoder.u8(2);
            encoder.u32(source_node.get());
            encoder.u32(edge.get());
        }
        GoldAndGearsSeededRunAction::Battle {
            source_node,
            role,
            group,
            member,
            effective_level,
        } => {
            encoder.u8(3);
            encoder.u32(source_node.get());
            encoder.u8(role_code(*role));
            encoder.string(group)?;
            encoder.string(member)?;
            encoder.u16(*effective_level);
        }
    }
    Ok(encoder.into_inner())
}

pub(super) fn decode_action(
    bytes: &[u8],
) -> Result<GoldAndGearsSeededRunAction, ActionPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let kind = decoder.u8()?;
    let action = match kind {
        0 => GoldAndGearsSeededRunAction::PlaneCreation {
            source_node: node(&mut decoder)?,
            plane: decoder.u8()?,
        },
        1 => GoldAndGearsSeededRunAction::BossSelection {
            source_node: node(&mut decoder)?,
            plane: decoder.u8()?,
            boss: decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?.into(),
        },
        2 => GoldAndGearsSeededRunAction::Traverse {
            source_node: node(&mut decoder)?,
            edge: starclock_activity::ActivityEdgeId::new(decoder.u32()?)
                .ok_or(ActionPayloadError::InvalidId)?,
        },
        3 => GoldAndGearsSeededRunAction::Battle {
            source_node: node(&mut decoder)?,
            role: decode_role(decoder.u8()?)?,
            group: decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?.into(),
            member: decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?.into(),
            effective_level: decoder.u16()?,
        },
        _ => return Err(ActionPayloadError::Kind),
    };
    decoder.finish()?;
    Ok(action)
}

fn node(decoder: &mut Decoder<'_>) -> Result<starclock_activity::NodeId, ActionPayloadError> {
    starclock_activity::NodeId::new(decoder.u32()?).ok_or(ActionPayloadError::InvalidId)
}

fn role_code(role: GoldAndGearsEncounterRole) -> u8 {
    match role {
        GoldAndGearsEncounterRole::Combat => 0,
        GoldAndGearsEncounterRole::Elite => 1,
        GoldAndGearsEncounterRole::FirstPlaneBoss => 2,
        GoldAndGearsEncounterRole::SecondPlaneBoss => 3,
        GoldAndGearsEncounterRole::FinalBoss => 4,
    }
}

fn decode_role(value: u8) -> Result<GoldAndGearsEncounterRole, ActionPayloadError> {
    match value {
        0 => Ok(GoldAndGearsEncounterRole::Combat),
        1 => Ok(GoldAndGearsEncounterRole::Elite),
        2 => Ok(GoldAndGearsEncounterRole::FirstPlaneBoss),
        3 => Ok(GoldAndGearsEncounterRole::SecondPlaneBoss),
        4 => Ok(GoldAndGearsEncounterRole::FinalBoss),
        _ => Err(ActionPayloadError::Kind),
    }
}

#[derive(Debug)]
pub(super) enum ActionPayloadError {
    Codec(CodecError),
    Kind,
    InvalidId,
}

impl From<CodecError> for ActionPayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}
