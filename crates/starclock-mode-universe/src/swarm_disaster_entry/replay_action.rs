//! Canonical Swarm complete-run actions and policy identities.

use starclock_activity::{ActivityEdgeId, NodeId};
use starclock_replay::{
    codec::{CodecError, Decoder, Encoder},
    component::MAX_COMPONENT_TEXT_BYTES,
};

use super::encounter_runtime::EncounterRole;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SwarmSeededRunAction {
    ProfileEntry {
        source_node: NodeId,
    },
    AudienceInitialization {
        source_node: NodeId,
    },
    TrailRunStart {
        source_node: NodeId,
    },
    CountdownSetup {
        source_node: NodeId,
        delta: i64,
    },
    PlaneCreation {
        source_node: NodeId,
        plane: u8,
    },
    DiceRoll {
        source_node: NodeId,
    },
    Traverse {
        source_node: NodeId,
        edge: ActivityEdgeId,
    },
    BossDecaySelection {
        source_node: NodeId,
        plane: u8,
        decay: Box<str>,
    },
    BossSelection {
        source_node: NodeId,
        plane: u8,
        boss: Box<str>,
    },
    Battle {
        source_node: NodeId,
        role: EncounterRole,
        group: Box<str>,
        member: Box<str>,
        effective_level: u16,
    },
}

pub(super) fn encode_action(action: &SwarmSeededRunAction) -> Result<Vec<u8>, ActionPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    match action {
        SwarmSeededRunAction::ProfileEntry { source_node } => {
            encoder.u8(0);
            encoder.u32(source_node.get());
        }
        SwarmSeededRunAction::AudienceInitialization { source_node } => {
            encoder.u8(1);
            encoder.u32(source_node.get());
        }
        SwarmSeededRunAction::TrailRunStart { source_node } => {
            encoder.u8(2);
            encoder.u32(source_node.get());
        }
        SwarmSeededRunAction::CountdownSetup { source_node, delta } => {
            encoder.u8(3);
            encoder.u32(source_node.get());
            encoder.i64(*delta);
        }
        SwarmSeededRunAction::PlaneCreation { source_node, plane } => {
            encoder.u8(4);
            encoder.u32(source_node.get());
            encoder.u8(*plane);
        }
        SwarmSeededRunAction::DiceRoll { source_node } => {
            encoder.u8(5);
            encoder.u32(source_node.get());
        }
        SwarmSeededRunAction::Traverse { source_node, edge } => {
            encoder.u8(6);
            encoder.u32(source_node.get());
            encoder.u32(edge.get());
        }
        SwarmSeededRunAction::BossDecaySelection {
            source_node,
            plane,
            decay,
        } => {
            encoder.u8(7);
            encoder.u32(source_node.get());
            encoder.u8(*plane);
            encoder.string(decay)?;
        }
        SwarmSeededRunAction::BossSelection {
            source_node,
            plane,
            boss,
        } => {
            encoder.u8(8);
            encoder.u32(source_node.get());
            encoder.u8(*plane);
            encoder.string(boss)?;
        }
        SwarmSeededRunAction::Battle {
            source_node,
            role,
            group,
            member,
            effective_level,
        } => {
            encoder.u8(9);
            encoder.u32(source_node.get());
            encoder.u8(role_code(*role));
            encoder.string(group)?;
            encoder.string(member)?;
            encoder.u16(*effective_level);
        }
    }
    Ok(encoder.into_inner())
}

pub(super) fn decode_action(bytes: &[u8]) -> Result<SwarmSeededRunAction, ActionPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let action = match decoder.u8()? {
        0 => SwarmSeededRunAction::ProfileEntry {
            source_node: node(&mut decoder)?,
        },
        1 => SwarmSeededRunAction::AudienceInitialization {
            source_node: node(&mut decoder)?,
        },
        2 => SwarmSeededRunAction::TrailRunStart {
            source_node: node(&mut decoder)?,
        },
        3 => SwarmSeededRunAction::CountdownSetup {
            source_node: node(&mut decoder)?,
            delta: decoder.i64()?,
        },
        4 => SwarmSeededRunAction::PlaneCreation {
            source_node: node(&mut decoder)?,
            plane: decoder.u8()?,
        },
        5 => SwarmSeededRunAction::DiceRoll {
            source_node: node(&mut decoder)?,
        },
        6 => SwarmSeededRunAction::Traverse {
            source_node: node(&mut decoder)?,
            edge: ActivityEdgeId::new(decoder.u32()?).ok_or(ActionPayloadError::InvalidId)?,
        },
        7 => SwarmSeededRunAction::BossDecaySelection {
            source_node: node(&mut decoder)?,
            plane: decoder.u8()?,
            decay: text(&mut decoder)?,
        },
        8 => SwarmSeededRunAction::BossSelection {
            source_node: node(&mut decoder)?,
            plane: decoder.u8()?,
            boss: text(&mut decoder)?,
        },
        9 => SwarmSeededRunAction::Battle {
            source_node: node(&mut decoder)?,
            role: decode_role(decoder.u8()?)?,
            group: text(&mut decoder)?,
            member: text(&mut decoder)?,
            effective_level: decoder.u16()?,
        },
        _ => return Err(ActionPayloadError::Kind),
    };
    decoder.finish()?;
    Ok(action)
}

fn node(decoder: &mut Decoder<'_>) -> Result<NodeId, ActionPayloadError> {
    NodeId::new(decoder.u32()?).ok_or(ActionPayloadError::InvalidId)
}

fn text(decoder: &mut Decoder<'_>) -> Result<Box<str>, ActionPayloadError> {
    Ok(decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?.into())
}

const fn role_code(role: EncounterRole) -> u8 {
    match role {
        EncounterRole::Combat => 0,
        EncounterRole::Elite => 1,
        EncounterRole::FirstPlaneBoss => 2,
        EncounterRole::SecondPlaneBoss => 3,
        EncounterRole::FinalBoss => 4,
    }
}

fn decode_role(value: u8) -> Result<EncounterRole, ActionPayloadError> {
    match value {
        0 => Ok(EncounterRole::Combat),
        1 => Ok(EncounterRole::Elite),
        2 => Ok(EncounterRole::FirstPlaneBoss),
        3 => Ok(EncounterRole::SecondPlaneBoss),
        4 => Ok(EncounterRole::FinalBoss),
        _ => Err(ActionPayloadError::Kind),
    }
}

pub(super) enum ActionPayloadError {
    Codec(CodecError),
    Kind,
    InvalidId,
}

impl core::fmt::Debug for ActionPayloadError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Codec(error) => formatter.debug_tuple("Codec").field(error).finish(),
            Self::Kind => formatter.write_str("Kind"),
            Self::InvalidId => formatter.write_str("InvalidId"),
        }
    }
}

impl From<CodecError> for ActionPayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}
