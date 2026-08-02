//! Component-addressed Swarm Disaster replay over real complete-run execution.

use core::fmt;

use starclock_activity::{ActivityDefinitionIdentity, ActivityInstanceId, ActivityTerminalOutcome};
use starclock_replay::{
    battle_event::BattleEventPayloadError,
    codec::CodecError,
    component::{ComponentIdentityError, ConfigurationComponentKind, ConfigurationComponentSet},
    digest::{
        BuildCatalogDigest, CombatantBuildDigest, DefinitionDigest, EntrySpecDigest, StateDigest,
    },
    entry::{BuildBindings, ReplayEntry},
    format::{ReplayEnvironment, ReplayError, ReplayHeader, decode_replay, encode_replay},
    nested_battle::{
        NestedBattleCommandPayload, NestedBattlePayloadError, encode_nested_battle_command_payload,
    },
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

use crate::battle_materialization::UniverseBattleRoster;

use super::{
    SwarmDisasterRuntimeInstance,
    incremental_run::SwarmDisasterIncrementalRun,
    replay_action::{ActionPayloadError, decode_action, encode_action},
    replay_battle::{compare_battle, encode_nested_state},
    seeded_run::{SwarmRecordedExecution, SwarmSeededRunError, SwarmSeededRunRequest},
};

/// Profile identifier carried by every Swarm Disaster replay entry.
pub const SWARM_DISASTER_REPLAY_PROFILE: &str = "swarm-disaster-real-battle-replay";

/// First authoritative replay boundary that differs during fresh verification.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SwarmReplayDivergenceKind {
    /// The exact ordered consumed-component set differs.
    Component,
    /// Replay environment, entry identity, build binding or policy revision differs.
    Catalog,
    /// The nested-battle handoff identity differs.
    Assembly,
    /// The accepted Activity action payload differs.
    ActivityCommand,
    /// An accepted combat command or its owner differs.
    BattleCommand,
    /// The complete emitted combat event payload differs.
    Event,
    /// The post-command battle state hash differs.
    BattleState,
    /// The freshly produced projected battle result differs.
    BattleResult,
    /// The post-action Activity state hash differs.
    ActivityState,
}

struct RecordedSwarmRun {
    request: SwarmSeededRunRequest,
    execution: SwarmRecordedExecution,
}

/// Successful fresh-verification receipt for one complete Swarm replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwarmReplayReport {
    action_count: u32,
    battle_count: u32,
    battle_command_count: u32,
    final_state_hash: StateDigest,
    terminal: ActivityTerminalOutcome,
}

impl SwarmReplayReport {
    /// Number of accepted Activity actions compared.
    pub const fn action_count(self) -> u32 {
        self.action_count
    }
    /// Number of real nested battles re-executed.
    pub const fn battle_count(self) -> u32 {
        self.battle_count
    }
    /// Number of accepted nested-battle commands compared.
    pub const fn battle_command_count(self) -> u32 {
        self.battle_command_count
    }
    /// Final freshly computed Activity state hash.
    pub const fn final_state_hash(self) -> StateDigest {
        self.final_state_hash
    }
    /// Terminal outcome reached by fresh execution.
    pub const fn terminal(self) -> ActivityTerminalOutcome {
        self.terminal
    }
}

/// Environment identity frozen for the Swarm Disaster replay envelope.
pub fn swarm_replay_environment() -> Result<ReplayEnvironment, ReplayError> {
    ReplayEnvironment::new("4.4")
}

/// Executes one complete baseline run and encodes its canonical real-battle replay.
pub fn encode_complete_swarm_replay(
    instance: &SwarmDisasterRuntimeInstance,
    seed: u64,
    identity: ActivityDefinitionIdentity,
    activity_instance: ActivityInstanceId,
    roster: &UniverseBattleRoster,
    components: ConfigurationComponentSet,
) -> Result<Vec<u8>, SwarmReplayError> {
    let request = baseline_request(seed, identity, activity_instance);
    let recorded = record_swarm_run(instance, request, roster)?;
    let header = swarm_replay_header(components, request, roster)?;
    encode_swarm_replay(&header, &recorded)
}

/// Encodes one terminal incremental session through the canonical Swarm
/// replay encoder. Incomplete sessions fail closed.
pub fn encode_incremental_swarm_replay(
    instance: &SwarmDisasterRuntimeInstance,
    run: &SwarmDisasterIncrementalRun,
    roster: &UniverseBattleRoster,
    components: ConfigurationComponentSet,
) -> Result<Vec<u8>, SwarmReplayError> {
    let recorded = RecordedSwarmRun {
        request: run.request(),
        execution: run.recorded_execution(instance)?,
    };
    let header = swarm_replay_header(components, recorded.request, roster)?;
    encode_swarm_replay(&header, &recorded)
}

/// Re-executes one canonical replay from fresh local state without mutating a live session.
pub fn verify_complete_swarm_replay(
    bytes: &[u8],
    instance: &SwarmDisasterRuntimeInstance,
    seed: u64,
    identity: ActivityDefinitionIdentity,
    activity_instance: ActivityInstanceId,
    roster: &UniverseBattleRoster,
    components: &ConfigurationComponentSet,
) -> Result<SwarmReplayReport, SwarmReplayError> {
    verify_swarm_replay(
        bytes,
        instance,
        baseline_request(seed, identity, activity_instance),
        roster,
        components,
    )
}

fn baseline_request(
    seed: u64,
    identity: ActivityDefinitionIdentity,
    activity_instance: ActivityInstanceId,
) -> SwarmSeededRunRequest {
    SwarmSeededRunRequest {
        seed,
        identity,
        activity_instance,
        config_digest: identity.config_digest(),
        boundary: super::seeded_run::SwarmSeededBoundary::Baseline,
    }
}

fn record_swarm_run(
    instance: &SwarmDisasterRuntimeInstance,
    request: SwarmSeededRunRequest,
    roster: &UniverseBattleRoster,
) -> Result<RecordedSwarmRun, SwarmReplayError> {
    Ok(RecordedSwarmRun {
        request,
        execution: instance.execute_seeded_run_recorded(request, roster)?,
    })
}

fn swarm_replay_header(
    components: ConfigurationComponentSet,
    request: SwarmSeededRunRequest,
    roster: &UniverseBattleRoster,
) -> Result<ReplayHeader, SwarmReplayError> {
    let entry = replay_entry(&components, request, roster)?;
    Ok(ReplayHeader::new(
        swarm_replay_environment()?,
        components,
        request.seed,
        entry,
        0,
    )?)
}

fn encode_swarm_replay(
    header_template: &ReplayHeader,
    recorded: &RecordedSwarmRun,
) -> Result<Vec<u8>, SwarmReplayError> {
    validate_header(
        header_template,
        &header_template.components().clone(),
        recorded.request,
        None,
    )?;
    let count = record_count(&recorded.execution)?;
    let header = ReplayHeader::new(
        header_template.environment().clone(),
        header_template.components().clone(),
        header_template.master_seed(),
        header_template.entry().clone(),
        count,
    )?;
    let mut payloads = Vec::with_capacity(count as usize);
    for step in &recorded.execution.replay {
        if let Some(battle) = &step.battle {
            payloads.push((
                RecordKind::NestedBattleStart,
                starclock_replay::activity::encode_nested_battle_start_payload(
                    battle.start_identity,
                ),
            ));
        }
        payloads.push((
            RecordKind::AcceptedActivityCommand,
            encode_action(&step.action)?,
        ));
        if let Some(battle) = &step.battle {
            for nested in battle.report.trace() {
                payloads.push((
                    RecordKind::AcceptedBattleCommand,
                    encode_nested_battle_command_payload(&NestedBattleCommandPayload::new(
                        nested.controller() as u8,
                        nested.command().clone(),
                    ))?,
                ));
                payloads.push((
                    RecordKind::ExpectedBattleState,
                    encode_nested_state(nested.state_hash(), nested.events())?,
                ));
            }
            payloads.push((
                RecordKind::NestedBattleEnd,
                starclock_replay::activity::encode_nested_battle_end_payload(
                    battle.result.actual_digest(),
                ),
            ));
        }
        payloads.push((
            RecordKind::ExpectedActivityState,
            step.state_hash.bytes().to_vec(),
        ));
    }
    let records = payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| RecordRef::new(*kind, index as u64, payload))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(encode_replay(&header, &records, Vec::new())?)
}

/// Re-executes from fresh state. Recorded battle results are never submitted.
fn verify_swarm_replay(
    bytes: &[u8],
    instance: &SwarmDisasterRuntimeInstance,
    request: SwarmSeededRunRequest,
    roster: &UniverseBattleRoster,
    actual_components: &ConfigurationComponentSet,
) -> Result<SwarmReplayReport, SwarmReplayError> {
    let replay = decode_replay(bytes)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(|_| SwarmReplayError::ComponentDivergence)?;
    validate_header(replay.header(), actual_components, request, Some(roster))?;
    let actual = instance.execute_seeded_run_recorded(request, roster)?;
    compare_records(replay.records(), &actual)
}

fn replay_entry(
    components: &ConfigurationComponentSet,
    request: SwarmSeededRunRequest,
    roster: &UniverseBattleRoster,
) -> Result<ReplayEntry, SwarmReplayError> {
    let build = components
        .components()
        .iter()
        .find(|component| component.kind() == ConfigurationComponentKind::BuildCatalog)
        .ok_or(SwarmReplayError::MissingBuildComponent)?;
    let builds = BuildBindings::new(
        BuildCatalogDigest::new(build.digest().bytes()),
        roster
            .entries()
            .iter()
            .map(|entry| CombatantBuildDigest::new(entry.build_digest().bytes()))
            .collect(),
    )?;
    Ok(ReplayEntry::Activity {
        profile_id: SWARM_DISASTER_REPLAY_PROFILE.into(),
        definition_id: request.identity.id().get(),
        definition_digest: DefinitionDigest::new(request.identity.definition_digest().bytes()),
        spec_digest: EntrySpecDigest::new(request.config_digest.bytes()),
        builds: Some(builds),
    })
}

fn validate_header(
    header: &ReplayHeader,
    components: &ConfigurationComponentSet,
    request: SwarmSeededRunRequest,
    roster: Option<&UniverseBattleRoster>,
) -> Result<(), SwarmReplayError> {
    if header.master_seed() != request.seed || header.environment() != &swarm_replay_environment()?
    {
        return Err(divergence(SwarmReplayDivergenceKind::Catalog, 0, 0, 0));
    }
    let identity_matches = matches!(
        header.entry(),
        ReplayEntry::Activity {
            profile_id,
            definition_id,
            definition_digest,
            spec_digest,
            ..
        } if profile_id.as_ref() == SWARM_DISASTER_REPLAY_PROFILE
            && *definition_id == request.identity.id().get()
            && definition_digest.bytes() == request.identity.definition_digest().bytes()
            && spec_digest.bytes() == request.config_digest.bytes()
    );
    let builds_match = match roster {
        Some(roster) => header.entry() == &replay_entry(components, request, roster)?,
        None => true,
    };
    if !identity_matches || !builds_match {
        return Err(divergence(SwarmReplayDivergenceKind::Catalog, 0, 0, 0));
    }
    Ok(())
}

fn record_count(execution: &SwarmRecordedExecution) -> Result<u32, SwarmReplayError> {
    let mut count = 0_u32;
    for step in &execution.replay {
        count = checked_add(count, 2)?;
        if let Some(battle) = &step.battle {
            count = checked_add(count, 2)?;
            count = checked_add(
                count,
                u32::try_from(battle.report.trace().len())
                    .map_err(|_| SwarmReplayError::TooManyRecords)?
                    .checked_mul(2)
                    .ok_or(SwarmReplayError::TooManyRecords)?,
            )?;
        }
    }
    if count > MAX_REPLAY_RECORDS {
        Err(SwarmReplayError::TooManyRecords)
    } else {
        Ok(count)
    }
}

fn compare_records(
    records: &[RecordRef<'_>],
    actual: &SwarmRecordedExecution,
) -> Result<SwarmReplayReport, SwarmReplayError> {
    let mut cursor = 0_usize;
    let mut battle_index = 0_u32;
    let mut battle_command_count = 0_u32;
    for (action_index, step) in actual.replay.iter().enumerate() {
        let action_index =
            u32::try_from(action_index).map_err(|_| SwarmReplayError::TooManyRecords)?;
        let recorded_start = records
            .get(cursor)
            .filter(|record| record.kind() == RecordKind::NestedBattleStart);
        if let Some(battle) = &step.battle {
            let start = recorded_start.ok_or_else(|| {
                divergence(
                    SwarmReplayDivergenceKind::Assembly,
                    action_index,
                    battle_index,
                    0,
                )
            })?;
            let identity =
                starclock_replay::activity::decode_nested_battle_start_payload(start.payload())
                    .map_err(|_| {
                        divergence(
                            SwarmReplayDivergenceKind::Assembly,
                            action_index,
                            battle_index,
                            0,
                        )
                    })?;
            if identity != battle.start_identity {
                return Err(divergence(
                    SwarmReplayDivergenceKind::Assembly,
                    action_index,
                    battle_index,
                    0,
                ));
            }
            cursor += 1;
        } else if recorded_start.is_some() {
            return Err(divergence(
                SwarmReplayDivergenceKind::Assembly,
                action_index,
                battle_index,
                0,
            ));
        }

        let action = records
            .get(cursor)
            .filter(|record| record.kind() == RecordKind::AcceptedActivityCommand)
            .ok_or_else(|| {
                divergence(
                    SwarmReplayDivergenceKind::ActivityCommand,
                    action_index,
                    battle_index,
                    0,
                )
            })?;
        let decoded = match decode_action(action.payload()) {
            Ok(action) => action,
            Err(ActionPayloadError::PolicyRevision) => {
                return Err(divergence(
                    SwarmReplayDivergenceKind::Catalog,
                    action_index,
                    battle_index,
                    0,
                ));
            }
            Err(_) => {
                return Err(divergence(
                    SwarmReplayDivergenceKind::ActivityCommand,
                    action_index,
                    battle_index,
                    0,
                ));
            }
        };
        if decoded != step.action {
            return Err(divergence(
                SwarmReplayDivergenceKind::ActivityCommand,
                action_index,
                battle_index,
                0,
            ));
        }
        cursor += 1;

        if let Some(battle) = &step.battle {
            compare_battle(
                records,
                &mut cursor,
                action_index,
                battle_index,
                battle,
                &mut battle_command_count,
            )?;
            battle_index = checked_add(battle_index, 1)?;
        }
        let state = records
            .get(cursor)
            .filter(|record| record.kind() == RecordKind::ExpectedActivityState)
            .ok_or_else(|| {
                divergence(
                    SwarmReplayDivergenceKind::ActivityState,
                    action_index,
                    battle_index,
                    0,
                )
            })?;
        if state.payload() != step.state_hash.bytes() {
            return Err(divergence(
                SwarmReplayDivergenceKind::ActivityState,
                action_index,
                battle_index,
                0,
            ));
        }
        cursor += 1;
    }
    if cursor != records.len() {
        return Err(divergence(
            SwarmReplayDivergenceKind::ActivityState,
            u32::try_from(actual.replay.len()).unwrap_or(u32::MAX),
            battle_index,
            0,
        ));
    }
    Ok(SwarmReplayReport {
        action_count: u32::try_from(actual.replay.len())
            .map_err(|_| SwarmReplayError::TooManyRecords)?,
        battle_count: battle_index,
        battle_command_count,
        final_state_hash: StateDigest::new(actual.report.final_state_hash.bytes()),
        terminal: actual.report.terminal,
    })
}

fn checked_add(left: u32, right: u32) -> Result<u32, SwarmReplayError> {
    left.checked_add(right)
        .ok_or(SwarmReplayError::TooManyRecords)
}

fn divergence(
    kind: SwarmReplayDivergenceKind,
    action_index: u32,
    battle_index: u32,
    command_index: u32,
) -> SwarmReplayError {
    SwarmReplayError::FirstDivergence {
        kind,
        action_index,
        battle_index,
        command_index,
    }
}

/// Typed failures from canonical encoding or fresh replay verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwarmReplayError {
    /// The replay envelope or environment identity is invalid.
    Envelope,
    /// The ordered record stream is malformed.
    Format,
    /// A canonical primitive payload cannot be encoded or decoded.
    Codec,
    /// A nested-battle command or state payload is malformed.
    NestedPayload,
    /// A complete combat-event payload is malformed.
    EventPayload,
    /// The supplied component set violates component identity invariants.
    ComponentIdentity,
    /// The replay and verifier consumed different ordered component sets.
    ComponentDivergence,
    /// Fresh deterministic Swarm execution failed.
    Execution,
    /// The roster has no matching build component binding.
    MissingBuildComponent,
    /// A canonical count exceeds the frozen replay bound.
    TooManyRecords,
    /// Fresh execution first differed at one authoritative boundary.
    FirstDivergence {
        /// Boundary category, ordered by the frozen comparison contract.
        kind: SwarmReplayDivergenceKind,
        /// Zero-based Activity action index.
        action_index: u32,
        /// Zero-based nested battle index.
        battle_index: u32,
        /// Zero-based accepted combat command index within the battle.
        command_index: u32,
    },
}

impl SwarmReplayError {
    /// Returns the first authoritative divergence category, when verification reached one.
    pub const fn first_divergence(&self) -> Option<SwarmReplayDivergenceKind> {
        match self {
            Self::FirstDivergence { kind, .. } => Some(*kind),
            Self::ComponentDivergence => Some(SwarmReplayDivergenceKind::Component),
            _ => None,
        }
    }
}

impl From<ReplayError> for SwarmReplayError {
    fn from(_: ReplayError) -> Self {
        Self::Envelope
    }
}
impl From<ReplayFormatError> for SwarmReplayError {
    fn from(_: ReplayFormatError) -> Self {
        Self::Format
    }
}
impl From<CodecError> for SwarmReplayError {
    fn from(_: CodecError) -> Self {
        Self::Codec
    }
}
impl From<NestedBattlePayloadError> for SwarmReplayError {
    fn from(_: NestedBattlePayloadError) -> Self {
        Self::NestedPayload
    }
}
impl From<BattleEventPayloadError> for SwarmReplayError {
    fn from(_: BattleEventPayloadError) -> Self {
        Self::EventPayload
    }
}
impl From<ComponentIdentityError> for SwarmReplayError {
    fn from(_: ComponentIdentityError) -> Self {
        Self::ComponentIdentity
    }
}
impl From<SwarmSeededRunError> for SwarmReplayError {
    fn from(_: SwarmSeededRunError) -> Self {
        Self::Execution
    }
}
impl From<ActionPayloadError> for SwarmReplayError {
    fn from(value: ActionPayloadError) -> Self {
        match value {
            ActionPayloadError::Codec(_) => Self::Codec,
            ActionPayloadError::Version
            | ActionPayloadError::Kind
            | ActionPayloadError::InvalidId
            | ActionPayloadError::PolicyRevision => Self::FirstDivergence {
                kind: SwarmReplayDivergenceKind::ActivityCommand,
                action_index: 0,
                battle_index: 0,
                command_index: 0,
            },
        }
    }
}

impl fmt::Display for SwarmReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Swarm Disaster replay error: {self:?}")
    }
}

impl std::error::Error for SwarmReplayError {}
