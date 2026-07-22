//! Owned, bounded and visibility-controlled battle observations.

use serde::{Deserialize, Serialize};

use crate::schema::{AgentSInt, AgentUInt};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "owned visibility-controlled projections";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityPolicy {
    PlayerVisible,
    OmniscientDebug,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentBattleStatus {
    AwaitingPlayer,
    Won,
    Lost,
    Faulted,
    Closed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentBattlePhase {
    AwaitingCommand,
    Won,
    Lost,
    Faulted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTeamSide {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentLifeState {
    Alive,
    Defeated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentPresenceState {
    Present,
    Untargetable,
    Linked,
    Reserved,
    Departed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEffectCategory {
    Buff,
    Debuff,
    Control,
    Dot,
    Mark,
    Field,
    Shield,
    NeutralState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentWaveView {
    pub number: AgentUInt,
    pub total: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentTeamView {
    pub side: AgentTeamSide,
    pub skill_points: AgentUInt,
    pub maximum_skill_points: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentUnitView {
    pub unit_id: AgentUInt,
    pub side: AgentTeamSide,
    pub formation: AgentUInt,
    pub life: AgentLifeState,
    pub presence: AgentPresenceState,
    pub current_hp: AgentUInt,
    pub maximum_hp: AgentUInt,
    pub current_energy_scaled: AgentSInt,
    pub maximum_energy_scaled: AgentSInt,
    pub weakness_broken: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_intent: Option<Box<str>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentEffectView {
    pub effect_id: AgentUInt,
    pub target_unit_id: AgentUInt,
    pub category: AgentEffectCategory,
    pub stacks: AgentUInt,
    pub remaining: Option<AgentUInt>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentTimelineView {
    pub actor_id: AgentUInt,
    pub owner_unit_id: AgentUInt,
    pub active: bool,
    pub action_gauge_scaled: AgentSInt,
    pub speed_scaled: AgentSInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentBattleView {
    pub phase: AgentBattlePhase,
    pub committed_revision: AgentUInt,
    pub rng_draw_count: AgentUInt,
    pub wave: AgentWaveView,
    pub teams: Box<[AgentTeamView]>,
    pub units: Box<[AgentUnitView]>,
    pub effects: Box<[AgentEffectView]>,
    pub timeline: Box<[AgentTimelineView]>,
}
