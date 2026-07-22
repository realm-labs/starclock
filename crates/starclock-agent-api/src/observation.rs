//! Owned, bounded and visibility-controlled battle observations.

use core::fmt;

use serde::{Deserialize, Serialize};
use starclock_combat::{
    BattleEvent, BattleEventKind, BattlePhase, BattleView, EffectCategory, LifeState,
    PresenceState, TeamSide,
};

use crate::schema::{AgentSInt, AgentUInt, EventCursor};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "owned visibility-controlled projections";
pub const MAX_UNITS: usize = 128;
pub const MAX_EFFECTS: usize = 2_048;
pub const MAX_TIMELINE_ENTRIES: usize = 256;
pub const MAX_EVENTS_PER_PAGE: usize = 256;

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

/// Explicit in-process acknowledgement required before requesting debug mode.
///
/// This is a capability boundary, not a remote authorization credential. MCP
/// adapters must grant it only after their independent debug-scope check.
pub struct OmniscientDebugCapability {
    private: (),
}

impl OmniscientDebugCapability {
    #[must_use]
    pub const fn acknowledge_trusted_debug_access() -> Self {
        Self { private: () }
    }
}

impl fmt::Debug for OmniscientDebugCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OmniscientDebugCapability([explicit])")
    }
}

/// Separately typed and visibly marked debug-mode projection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AgentDebugProjection {
    visibility_policy: VisibilityPolicy,
    debug_authorized: bool,
    battle: AgentBattleView,
}

impl AgentDebugProjection {
    #[must_use]
    pub const fn visibility_policy(&self) -> VisibilityPolicy {
        self.visibility_policy
    }

    #[must_use]
    pub const fn debug_authorized(&self) -> bool {
        self.debug_authorized
    }

    #[must_use]
    pub const fn battle(&self) -> &AgentBattleView {
        &self.battle
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentEventSummary {
    pub event_id: AgentUInt,
    pub kind: Box<str>,
    pub summary: Box<str>,
    pub root_command_id: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentEventPage {
    pub events: Box<[AgentEventSummary]>,
    pub next_cursor: EventCursor,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionError {
    UnstableBoundary,
    DownedUnit,
    TooManyUnits,
    TooManyEffects,
    TooManyTimelineEntries,
    InvalidHealth,
    InvalidCursor,
    UnauthorizedDebug,
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "agent projection failed: {self:?}")
    }
}

impl std::error::Error for ProjectionError {}

/// Projects only fields approved by the frozen player-visible allowlist.
pub fn project_player_visible(view: BattleView<'_>) -> Result<AgentBattleView, ProjectionError> {
    let phase = match view.phase() {
        BattlePhase::AwaitingCommand => AgentBattlePhase::AwaitingCommand,
        BattlePhase::Won => AgentBattlePhase::Won,
        BattlePhase::Lost => AgentBattlePhase::Lost,
        BattlePhase::Faulted => AgentBattlePhase::Faulted,
        BattlePhase::Initializing | BattlePhase::Resolving => {
            return Err(ProjectionError::UnstableBoundary);
        }
    };
    let mut units = Vec::new();
    for unit in view.units_by_id() {
        if units.len() == MAX_UNITS {
            return Err(ProjectionError::TooManyUnits);
        }
        let life = match unit.life() {
            LifeState::Alive => AgentLifeState::Alive,
            LifeState::Defeated => AgentLifeState::Defeated,
            LifeState::Downed => return Err(ProjectionError::DownedUnit),
        };
        let presence = match unit.presence() {
            PresenceState::Present | PresenceState::Transformed => AgentPresenceState::Present,
            PresenceState::Untargetable => AgentPresenceState::Untargetable,
            PresenceState::Linked => AgentPresenceState::Linked,
            PresenceState::Reserved => AgentPresenceState::Reserved,
            PresenceState::Departed => AgentPresenceState::Departed,
        };
        units.push(AgentUnitView {
            unit_id: AgentUInt::from_u64(unit.id().get()),
            side: side(unit.side()),
            formation: AgentUInt::from_u64(u64::from(unit.formation().get())),
            life,
            presence,
            current_hp: AgentUInt::from_u64(
                u64::try_from(unit.current_hp().get())
                    .map_err(|_| ProjectionError::InvalidHealth)?,
            ),
            maximum_hp: AgentUInt::from_u64(
                u64::try_from(unit.maximum_hp().get())
                    .map_err(|_| ProjectionError::InvalidHealth)?,
            ),
            current_energy_scaled: AgentSInt::from_i64(unit.current_energy().scaled()),
            maximum_energy_scaled: AgentSInt::from_i64(unit.maximum_energy().scaled()),
            weakness_broken: unit.weakness_broken(),
            public_intent: None,
        });
    }
    let mut effects = Vec::new();
    for effect in view.effects_by_id() {
        if effects.len() == MAX_EFFECTS {
            return Err(ProjectionError::TooManyEffects);
        }
        effects.push(AgentEffectView {
            effect_id: AgentUInt::from_u64(effect.id().get()),
            target_unit_id: AgentUInt::from_u64(effect.target().get()),
            category: effect_category(effect.category()),
            stacks: AgentUInt::from_u64(u64::from(effect.stacks())),
            remaining: effect
                .remaining()
                .map(|value| AgentUInt::from_u64(u64::from(value))),
        });
    }
    let mut timeline = Vec::new();
    for actor in view.timeline_actors() {
        if timeline.len() == MAX_TIMELINE_ENTRIES {
            return Err(ProjectionError::TooManyTimelineEntries);
        }
        timeline.push(AgentTimelineView {
            actor_id: AgentUInt::from_u64(actor.id().get()),
            owner_unit_id: AgentUInt::from_u64(actor.owner().get()),
            active: actor.is_active(),
            action_gauge_scaled: AgentSInt::from_i64(actor.action_gauge().scaled()),
            speed_scaled: AgentSInt::from_i64(actor.speed().scaled()),
        });
    }
    let encounter = view.encounter();
    Ok(AgentBattleView {
        phase,
        committed_revision: AgentUInt::from_u64(view.committed_revision()),
        rng_draw_count: AgentUInt::from_u64(view.rng_draw_count()),
        wave: AgentWaveView {
            number: AgentUInt::from_u64(u64::from(encounter.number())),
            total: AgentUInt::from_u64(u64::from(encounter.total_waves())),
        },
        teams: [team(view, TeamSide::Player), team(view, TeamSide::Enemy)].into(),
        units: units.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        timeline: timeline.into_boxed_slice(),
    })
}

/// Projects the frozen debug mode only after an explicit capability grant.
///
/// Version one intentionally adds no hidden payload fields beyond the frozen
/// battle schema; the separate type and required markers prevent accidental
/// default-path exposure and leave future expansion revisioned.
pub fn project_omniscient_debug(
    view: BattleView<'_>,
    capability: Option<&OmniscientDebugCapability>,
) -> Result<AgentDebugProjection, ProjectionError> {
    let capability = capability.ok_or(ProjectionError::UnauthorizedDebug)?;
    let () = capability.private;
    Ok(AgentDebugProjection {
        visibility_policy: VisibilityPolicy::OmniscientDebug,
        debug_authorized: true,
        battle: project_player_visible(view)?,
    })
}

/// Summarizes a committed event slice without serializing private typed payloads.
pub fn project_event_page(events: &[BattleEvent]) -> Result<AgentEventPage, ProjectionError> {
    let truncated = events.len() > MAX_EVENTS_PER_PAGE;
    let visible = &events[..events.len().min(MAX_EVENTS_PER_PAGE)];
    let summaries = visible
        .iter()
        .map(|event| {
            let (kind, summary) = event_label(event.kind());
            AgentEventSummary {
                event_id: AgentUInt::from_u64(event.id().get()),
                kind: kind.into(),
                summary: summary.into(),
                root_command_id: AgentUInt::from_u64(event.cause().root_command().get()),
            }
        })
        .collect::<Vec<_>>();
    let cursor_id = visible.last().map_or(0, |event| event.id().get());
    let next_cursor = EventCursor::parse(&format!("event_{cursor_id}"))
        .map_err(|_| ProjectionError::InvalidCursor)?;
    Ok(AgentEventPage {
        events: summaries.into_boxed_slice(),
        next_cursor,
        truncated,
    })
}

fn team(view: BattleView<'_>, requested: TeamSide) -> AgentTeamView {
    let team = view.team(requested);
    AgentTeamView {
        side: side(requested),
        skill_points: AgentUInt::from_u64(u64::from(team.skill_points())),
        maximum_skill_points: AgentUInt::from_u64(u64::from(team.maximum_skill_points())),
    }
}

const fn side(value: TeamSide) -> AgentTeamSide {
    match value {
        TeamSide::Player => AgentTeamSide::Player,
        TeamSide::Enemy => AgentTeamSide::Enemy,
    }
}

const fn effect_category(value: EffectCategory) -> AgentEffectCategory {
    match value {
        EffectCategory::Buff => AgentEffectCategory::Buff,
        EffectCategory::Debuff => AgentEffectCategory::Debuff,
        EffectCategory::Control => AgentEffectCategory::Control,
        EffectCategory::Dot => AgentEffectCategory::Dot,
        EffectCategory::Mark => AgentEffectCategory::Mark,
        EffectCategory::Field => AgentEffectCategory::Field,
        EffectCategory::Shield => AgentEffectCategory::Shield,
        EffectCategory::NeutralState => AgentEffectCategory::NeutralState,
    }
}

fn event_label(kind: &BattleEventKind) -> (&'static str, &'static str) {
    match kind {
        BattleEventKind::Battle(_) => ("battle", "Battle lifecycle changed."),
        BattleEventKind::Decision(_) => ("decision", "A decision boundary changed."),
        BattleEventKind::Turn(_) => ("turn", "A timeline turn boundary changed."),
        BattleEventKind::Action(_) => ("action", "An action boundary changed."),
        BattleEventKind::Phase(_) => ("phase", "An action phase changed."),
        BattleEventKind::Hit(_) => ("hit", "A hit boundary changed."),
        BattleEventKind::Damage(_) => ("damage", "Damage changed public HP."),
        BattleEventKind::Heal(_) => ("heal", "Healing changed public HP."),
        BattleEventKind::HpConsumption(_) => ("hp_consumption", "HP was consumed."),
        BattleEventKind::Shield(_) => ("shield", "A shield changed."),
        BattleEventKind::Toughness(_) => ("toughness", "Toughness state changed."),
        BattleEventKind::BreakDamage(_) => ("break_damage", "Break damage changed public HP."),
        BattleEventKind::Unit(_) => ("unit", "A unit lifecycle changed."),
        BattleEventKind::Wave(_) => ("wave", "The encounter wave changed."),
        BattleEventKind::EnemyPhase(_) => ("enemy_phase", "A public enemy phase changed."),
        BattleEventKind::Resource(_) => ("resource", "A public resource changed."),
        BattleEventKind::Effect(_) => ("effect", "A public effect changed."),
        BattleEventKind::RuleState(_) | BattleEventKind::RuleSignal(_) => {
            ("rule", "A public rule fact occurred.")
        }
        BattleEventKind::Fault(_) => ("fault", "Battle resolution faulted."),
        _ => ("event", "A battle fact occurred."),
    }
}
