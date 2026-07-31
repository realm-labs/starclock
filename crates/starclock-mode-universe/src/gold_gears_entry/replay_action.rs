//! Canonical Gold complete-run action payload and policy identities.

use starclock_replay::{
    codec::{CodecError, Decoder, Encoder},
    component::MAX_COMPONENT_TEXT_BYTES,
};

use super::{
    GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION, GOLD_AND_GEARS_BATTLE_MATERIALIZATION_REVISION,
    GOLD_AND_GEARS_ENCOUNTER_DIFFICULTY_REVISION, GOLD_AND_GEARS_ENCOUNTER_SELECTION_REVISION,
    GOLD_AND_GEARS_PLANE_COMPLETION_REVISION, GOLD_AND_GEARS_TOPOLOGY_REVISION,
    GoldAndGearsEncounterRole, GoldAndGearsSeededRunAction,
};

/// Canonical Gold action payload revision.
pub const GOLD_AND_GEARS_REPLAY_ACTION_VERSION: u16 = 1;

pub(super) fn encode_action(
    action: &GoldAndGearsSeededRunAction,
) -> Result<Vec<u8>, ActionPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(GOLD_AND_GEARS_REPLAY_ACTION_VERSION);
    match action {
        GoldAndGearsSeededRunAction::PlaneCreation { source_node, plane } => {
            encoder.u8(0);
            encoder.string(GOLD_AND_GEARS_TOPOLOGY_REVISION)?;
            encoder.u32(source_node.get());
            encoder.u8(*plane);
        }
        GoldAndGearsSeededRunAction::BossSelection {
            source_node,
            plane,
            boss,
        } => {
            encoder.u8(1);
            encoder.string(GOLD_AND_GEARS_PLANE_COMPLETION_REVISION)?;
            encoder.u32(source_node.get());
            encoder.u8(*plane);
            encoder.string(boss)?;
        }
        GoldAndGearsSeededRunAction::Traverse { source_node, edge } => {
            encoder.u8(2);
            encoder.string(GOLD_AND_GEARS_TOPOLOGY_REVISION)?;
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
            for revision in battle_policy_revisions() {
                encoder.string(revision)?;
            }
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
    if decoder.u16()? != GOLD_AND_GEARS_REPLAY_ACTION_VERSION {
        return Err(ActionPayloadError::Version);
    }
    let kind = decoder.u8()?;
    let action = match kind {
        0 => {
            expect_revision(&mut decoder, GOLD_AND_GEARS_TOPOLOGY_REVISION)?;
            GoldAndGearsSeededRunAction::PlaneCreation {
                source_node: node(&mut decoder)?,
                plane: decoder.u8()?,
            }
        }
        1 => {
            expect_revision(&mut decoder, GOLD_AND_GEARS_PLANE_COMPLETION_REVISION)?;
            GoldAndGearsSeededRunAction::BossSelection {
                source_node: node(&mut decoder)?,
                plane: decoder.u8()?,
                boss: decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?.into(),
            }
        }
        2 => {
            expect_revision(&mut decoder, GOLD_AND_GEARS_TOPOLOGY_REVISION)?;
            GoldAndGearsSeededRunAction::Traverse {
                source_node: node(&mut decoder)?,
                edge: starclock_activity::ActivityEdgeId::new(decoder.u32()?)
                    .ok_or(ActionPayloadError::InvalidId)?,
            }
        }
        3 => {
            for revision in battle_policy_revisions() {
                expect_revision(&mut decoder, revision)?;
            }
            GoldAndGearsSeededRunAction::Battle {
                source_node: node(&mut decoder)?,
                role: decode_role(decoder.u8()?)?,
                group: decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?.into(),
                member: decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)?.into(),
                effective_level: decoder.u16()?,
            }
        }
        _ => return Err(ActionPayloadError::Kind),
    };
    decoder.finish()?;
    Ok(action)
}

fn battle_policy_revisions() -> [&'static str; 4] {
    [
        GOLD_AND_GEARS_ENCOUNTER_SELECTION_REVISION,
        GOLD_AND_GEARS_ENCOUNTER_DIFFICULTY_REVISION,
        GOLD_AND_GEARS_BATTLE_MATERIALIZATION_REVISION,
        GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION,
    ]
}

fn expect_revision(decoder: &mut Decoder<'_>, expected: &str) -> Result<(), ActionPayloadError> {
    if decoder.string(MAX_COMPONENT_TEXT_BYTES as u32)? == expected {
        Ok(())
    } else {
        Err(ActionPayloadError::PolicyRevision)
    }
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
    Version,
    Kind,
    InvalidId,
    PolicyRevision,
}

impl From<CodecError> for ActionPayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}
