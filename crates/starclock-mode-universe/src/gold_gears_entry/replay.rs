//! Component-addressed Gold and Gears replay over the real seeded executor.

use core::fmt;

use starclock_activity::ActivityTerminalOutcome;
use starclock_combat::{NUMERIC_POLICY_REVISION, rng::RNG_ALGORITHM_REVISION};
use starclock_replay::{
    battle_event::{
        BATTLE_EVENT_PAYLOAD_VERSION_V5, BattleEventPayloadError,
        encode_battle_event_payload_for_version,
    },
    codec::{CodecError, Encoder},
    component::{
        ComponentIdentityError, ConfigurationComponentDivergence, ConfigurationComponentKind,
        ConfigurationComponentSet,
    },
    digest::{
        BuildCatalogDigest, CombatantBuildDigest, DefinitionDigest, EntrySpecDigest, StateDigest,
    },
    format::{BuildBindings, ReplayEntry},
    format_v2::{
        ReplayCompatibilityV2, ReplayHeaderV2, ReplayV2Error, decode_replay_v2, encode_replay_v2,
    },
    nested_battle::{
        MAX_NESTED_BATTLE_EVENTS_PER_COMMAND, NESTED_BATTLE_STATE_PAYLOAD_VERSION,
        NestedBattleCommandPayload, NestedBattlePayloadError, decode_nested_battle_command_payload,
        decode_nested_battle_state_payload, encode_nested_battle_command_payload,
    },
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

use crate::battle_materialization::UniverseBattleRoster;

use super::{
    GoldAndGearsRuntimeInstance, GoldAndGearsSeededRunError, GoldAndGearsSeededRunReport,
    GoldAndGearsSeededRunRequest,
    replay_action::{ActionPayloadError, decode_action, encode_action},
    seeded_run::{GoldAndGearsRecordedExecution, GoldAndGearsSeededBattleRecord},
};

/// Frozen mode replay revision from the Goal 14 Phase 0 contract.
pub const GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION: &str = "gold-and-gears-real-battle-replay-v1";
/// Frozen nested event payload revision for this replay contract.
pub const GOLD_AND_GEARS_REPLAY_EVENT_PAYLOAD_VERSION: u16 = BATTLE_EVENT_PAYLOAD_VERSION_V5;

/// First authoritative boundary which differs during fresh verification.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsReplayDivergenceKind {
    Component,
    Catalog,
    Assembly,
    ActivityCommand,
    BattleCommand,
    Event,
    BattleState,
    BattleResult,
    ActivityState,
}

/// One freshly executed run together with the replay-only nested trace.
pub struct RecordedGoldAndGearsRun {
    request: GoldAndGearsSeededRunRequest,
    execution: GoldAndGearsRecordedExecution,
}

impl RecordedGoldAndGearsRun {
    #[must_use]
    pub const fn report(&self) -> &GoldAndGearsSeededRunReport {
        &self.execution.report
    }

    #[must_use]
    pub fn action_count(&self) -> usize {
        self.execution.replay.len()
    }

    #[must_use]
    pub fn battle_command_count(&self) -> usize {
        self.execution
            .replay
            .iter()
            .filter_map(|step| step.battle.as_ref())
            .map(|battle| battle.report.trace().len())
            .sum()
    }
}

/// Successful verification receipt for one complete replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsReplayReport {
    action_count: u32,
    battle_count: u32,
    battle_command_count: u32,
    final_state_hash: StateDigest,
    terminal: ActivityTerminalOutcome,
}

impl GoldAndGearsReplayReport {
    #[must_use]
    pub const fn action_count(self) -> u32 {
        self.action_count
    }
    #[must_use]
    pub const fn battle_count(self) -> u32 {
        self.battle_count
    }
    #[must_use]
    pub const fn battle_command_count(self) -> u32 {
        self.battle_command_count
    }
    #[must_use]
    pub const fn final_state_hash(self) -> StateDigest {
        self.final_state_hash
    }
    #[must_use]
    pub const fn terminal(self) -> ActivityTerminalOutcome {
        self.terminal
    }
}

/// Runs the real complete-run executor and retains every nested command/event.
pub fn record_gold_and_gears_run(
    instance: &GoldAndGearsRuntimeInstance,
    request: GoldAndGearsSeededRunRequest,
    roster: &UniverseBattleRoster,
) -> Result<RecordedGoldAndGearsRun, GoldAndGearsReplayError> {
    Ok(RecordedGoldAndGearsRun {
        request,
        execution: instance.execute_seeded_run_recorded(request, roster)?,
    })
}

/// Current compatibility identity for Gold replay-v2 envelopes.
pub fn gold_and_gears_replay_compatibility()
-> Result<ReplayCompatibilityV2, GoldAndGearsReplayError> {
    Ok(ReplayCompatibilityV2::new(
        "4.4",
        NUMERIC_POLICY_REVISION,
        RNG_ALGORITHM_REVISION,
        starclock_activity::ACTIVITY_STATE_HASH_REVISION,
    )?)
}

/// Creates a zero-record, build-aware header for one exact run entry.
pub fn gold_and_gears_header_v2(
    components: ConfigurationComponentSet,
    request: GoldAndGearsSeededRunRequest,
    roster: &UniverseBattleRoster,
) -> Result<ReplayHeaderV2, GoldAndGearsReplayError> {
    let entry = replay_entry(&components, request, roster)?;
    Ok(ReplayHeaderV2::new(
        gold_and_gears_replay_compatibility()?,
        components,
        request.seed(),
        entry,
        0,
    )?)
}

/// Encodes a complete real-battle trace under the frozen ReplayV2 envelope.
pub fn encode_gold_and_gears_replay(
    header_template: &ReplayHeaderV2,
    recorded: &RecordedGoldAndGearsRun,
) -> Result<Vec<u8>, GoldAndGearsReplayError> {
    validate_header(
        header_template,
        &header_template.components().clone(),
        recorded.request,
        None,
    )?;
    let count = record_count(&recorded.execution)?;
    let header = ReplayHeaderV2::new(
        header_template.compatibility().clone(),
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
                    encode_nested_state_v5(nested.state_hash(), nested.events())?,
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
    Ok(encode_replay_v2(&header, &records, Vec::new())?)
}

/// Re-executes the complete run from fresh state and reports the first typed
/// authoritative divergence; recorded battle results are never submitted.
pub fn verify_gold_and_gears_replay(
    bytes: &[u8],
    instance: &GoldAndGearsRuntimeInstance,
    request: GoldAndGearsSeededRunRequest,
    roster: &UniverseBattleRoster,
    actual_components: &ConfigurationComponentSet,
) -> Result<GoldAndGearsReplayReport, GoldAndGearsReplayError> {
    let replay = decode_replay_v2(bytes)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(GoldAndGearsReplayError::ComponentDivergence)?;
    validate_header(replay.header(), actual_components, request, Some(roster))?;
    let actual = instance.execute_seeded_run_recorded(request, roster)?;
    compare_records(replay.records(), &actual)
}

fn replay_entry(
    components: &ConfigurationComponentSet,
    request: GoldAndGearsSeededRunRequest,
    roster: &UniverseBattleRoster,
) -> Result<ReplayEntry, GoldAndGearsReplayError> {
    let build = components
        .components()
        .iter()
        .find(|component| component.kind() == ConfigurationComponentKind::BuildCatalog)
        .ok_or(GoldAndGearsReplayError::MissingBuildComponent)?;
    let builds = BuildBindings::new(
        build.revision(),
        BuildCatalogDigest::new(build.digest().bytes()),
        roster
            .entries()
            .iter()
            .map(|entry| CombatantBuildDigest::new(entry.build_digest().bytes()))
            .collect(),
    )?;
    let identity = request.identity();
    Ok(ReplayEntry::Activity {
        profile_id: GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION.into(),
        definition_id: identity.id().get(),
        definition_digest: DefinitionDigest::new(identity.definition_digest().bytes()),
        spec_digest: EntrySpecDigest::new(identity.config_digest().bytes()),
        builds: Some(builds),
    })
}

fn validate_header(
    header: &ReplayHeaderV2,
    components: &ConfigurationComponentSet,
    request: GoldAndGearsSeededRunRequest,
    roster: Option<&UniverseBattleRoster>,
) -> Result<(), GoldAndGearsReplayError> {
    if header.master_seed() != request.seed()
        || header.compatibility() != &gold_and_gears_replay_compatibility()?
    {
        return Err(divergence(
            GoldAndGearsReplayDivergenceKind::Catalog,
            0,
            0,
            0,
        ));
    }
    let identity = request.identity();
    let identity_matches = matches!(
        header.entry(),
        ReplayEntry::Activity {
            profile_id,
            definition_id,
            definition_digest,
            spec_digest,
            ..
        } if profile_id.as_ref() == GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION
            && *definition_id == identity.id().get()
            && definition_digest.bytes() == identity.definition_digest().bytes()
            && spec_digest.bytes() == identity.config_digest().bytes()
    );
    let builds_match = match roster {
        Some(roster) => header.entry() == &replay_entry(components, request, roster)?,
        None => true,
    };
    if !identity_matches || !builds_match {
        return Err(divergence(
            GoldAndGearsReplayDivergenceKind::Catalog,
            0,
            0,
            0,
        ));
    }
    Ok(())
}

fn record_count(execution: &GoldAndGearsRecordedExecution) -> Result<u32, GoldAndGearsReplayError> {
    let mut count = 0_u32;
    for step in &execution.replay {
        count = checked_add(count, 2)?;
        if let Some(battle) = &step.battle {
            count = checked_add(count, 2)?;
            count = checked_add(
                count,
                u32::try_from(battle.report.trace().len())
                    .map_err(|_| GoldAndGearsReplayError::TooManyRecords)?
                    .checked_mul(2)
                    .ok_or(GoldAndGearsReplayError::TooManyRecords)?,
            )?;
        }
    }
    if count > MAX_REPLAY_RECORDS {
        Err(GoldAndGearsReplayError::TooManyRecords)
    } else {
        Ok(count)
    }
}

fn compare_records(
    records: &[RecordRef<'_>],
    actual: &GoldAndGearsRecordedExecution,
) -> Result<GoldAndGearsReplayReport, GoldAndGearsReplayError> {
    let mut cursor = 0_usize;
    let mut battle_index = 0_u32;
    let mut battle_command_count = 0_u32;
    for (action_index, step) in actual.replay.iter().enumerate() {
        let action_index =
            u32::try_from(action_index).map_err(|_| GoldAndGearsReplayError::TooManyRecords)?;
        let recorded_start = records
            .get(cursor)
            .filter(|record| record.kind() == RecordKind::NestedBattleStart);
        if let Some(battle) = &step.battle {
            let start = recorded_start.ok_or_else(|| {
                divergence(
                    GoldAndGearsReplayDivergenceKind::Assembly,
                    action_index,
                    battle_index,
                    0,
                )
            })?;
            let identity =
                starclock_replay::activity::decode_nested_battle_start_payload(start.payload())
                    .map_err(|_| {
                        divergence(
                            GoldAndGearsReplayDivergenceKind::Assembly,
                            action_index,
                            battle_index,
                            0,
                        )
                    })?;
            if identity != battle.start_identity {
                return Err(divergence(
                    GoldAndGearsReplayDivergenceKind::Assembly,
                    action_index,
                    battle_index,
                    0,
                ));
            }
            cursor += 1;
        } else if recorded_start.is_some() {
            return Err(divergence(
                GoldAndGearsReplayDivergenceKind::Assembly,
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
                    GoldAndGearsReplayDivergenceKind::ActivityCommand,
                    action_index,
                    battle_index,
                    0,
                )
            })?;
        let decoded = match decode_action(action.payload()) {
            Ok(action) => action,
            Err(ActionPayloadError::PolicyRevision) => {
                return Err(divergence(
                    GoldAndGearsReplayDivergenceKind::Catalog,
                    action_index,
                    battle_index,
                    0,
                ));
            }
            Err(_) => {
                return Err(divergence(
                    GoldAndGearsReplayDivergenceKind::ActivityCommand,
                    action_index,
                    battle_index,
                    0,
                ));
            }
        };
        if decoded != step.action {
            return Err(divergence(
                GoldAndGearsReplayDivergenceKind::ActivityCommand,
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
                    GoldAndGearsReplayDivergenceKind::ActivityState,
                    action_index,
                    battle_index,
                    0,
                )
            })?;
        if state.payload() != step.state_hash.bytes() {
            return Err(divergence(
                GoldAndGearsReplayDivergenceKind::ActivityState,
                action_index,
                battle_index,
                0,
            ));
        }
        cursor += 1;
    }
    if cursor != records.len() {
        return Err(divergence(
            GoldAndGearsReplayDivergenceKind::ActivityState,
            u32::try_from(actual.replay.len()).unwrap_or(u32::MAX),
            battle_index,
            0,
        ));
    }
    Ok(GoldAndGearsReplayReport {
        action_count: u32::try_from(actual.replay.len())
            .map_err(|_| GoldAndGearsReplayError::TooManyRecords)?,
        battle_count: battle_index,
        battle_command_count,
        final_state_hash: StateDigest::new(actual.report.final_state_hash().bytes()),
        terminal: actual.report.terminal(),
    })
}

fn compare_battle(
    records: &[RecordRef<'_>],
    cursor: &mut usize,
    action_index: u32,
    battle_index: u32,
    actual: &GoldAndGearsSeededBattleRecord,
    total_commands: &mut u32,
) -> Result<(), GoldAndGearsReplayError> {
    for (command_index, step) in actual.report.trace().iter().enumerate() {
        let command_index =
            u32::try_from(command_index).map_err(|_| GoldAndGearsReplayError::TooManyRecords)?;
        let record = records
            .get(*cursor)
            .filter(|record| record.kind() == RecordKind::AcceptedBattleCommand)
            .ok_or_else(|| {
                divergence(
                    GoldAndGearsReplayDivergenceKind::BattleCommand,
                    action_index,
                    battle_index,
                    command_index,
                )
            })?;
        let command = decode_nested_battle_command_payload(record.payload()).map_err(|_| {
            divergence(
                GoldAndGearsReplayDivergenceKind::BattleCommand,
                action_index,
                battle_index,
                command_index,
            )
        })?;
        if command.controller() != step.controller() as u8 || command.command() != step.command() {
            return Err(divergence(
                GoldAndGearsReplayDivergenceKind::BattleCommand,
                action_index,
                battle_index,
                command_index,
            ));
        }
        *cursor += 1;
        let state = records
            .get(*cursor)
            .filter(|record| record.kind() == RecordKind::ExpectedBattleState)
            .ok_or_else(|| {
                divergence(
                    GoldAndGearsReplayDivergenceKind::BattleState,
                    action_index,
                    battle_index,
                    command_index,
                )
            })?;
        let decoded = decode_nested_battle_state_payload(state.payload()).map_err(|_| {
            divergence(
                GoldAndGearsReplayDivergenceKind::BattleState,
                action_index,
                battle_index,
                command_index,
            )
        })?;
        compare_events(
            decoded.event_payloads(),
            step.events(),
            action_index,
            battle_index,
            command_index,
        )?;
        if decoded.state_hash().bytes() != step.state_hash().bytes() {
            return Err(divergence(
                GoldAndGearsReplayDivergenceKind::BattleState,
                action_index,
                battle_index,
                command_index,
            ));
        }
        *cursor += 1;
        *total_commands = checked_add(*total_commands, 1)?;
    }
    if records
        .get(*cursor)
        .is_some_and(|record| record.kind() == RecordKind::AcceptedBattleCommand)
    {
        return Err(divergence(
            GoldAndGearsReplayDivergenceKind::BattleCommand,
            action_index,
            battle_index,
            u32::try_from(actual.report.trace().len()).unwrap_or(u32::MAX),
        ));
    }
    let end = records
        .get(*cursor)
        .filter(|record| record.kind() == RecordKind::NestedBattleEnd)
        .ok_or_else(|| {
            divergence(
                GoldAndGearsReplayDivergenceKind::BattleResult,
                action_index,
                battle_index,
                u32::try_from(actual.report.trace().len()).unwrap_or(u32::MAX),
            )
        })?;
    let digest = starclock_replay::activity::decode_nested_battle_end_payload(end.payload())
        .map_err(|_| {
            divergence(
                GoldAndGearsReplayDivergenceKind::BattleResult,
                action_index,
                battle_index,
                u32::try_from(actual.report.trace().len()).unwrap_or(u32::MAX),
            )
        })?;
    if digest != actual.result.actual_digest() {
        return Err(divergence(
            GoldAndGearsReplayDivergenceKind::BattleResult,
            action_index,
            battle_index,
            u32::try_from(actual.report.trace().len()).unwrap_or(u32::MAX),
        ));
    }
    *cursor += 1;
    Ok(())
}

fn compare_events(
    expected: &[&[u8]],
    actual: &[starclock_combat::BattleEvent],
    action_index: u32,
    battle_index: u32,
    command_index: u32,
) -> Result<(), GoldAndGearsReplayError> {
    if expected.len() != actual.len() {
        return Err(divergence(
            GoldAndGearsReplayDivergenceKind::Event,
            action_index,
            battle_index,
            command_index,
        ));
    }
    for (payload, event) in expected.iter().zip(actual) {
        let version = payload
            .get(..2)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u16::from_le_bytes);
        if version != Some(GOLD_AND_GEARS_REPLAY_EVENT_PAYLOAD_VERSION)
            || payload
                != &encode_battle_event_payload_for_version(
                    event,
                    GOLD_AND_GEARS_REPLAY_EVENT_PAYLOAD_VERSION,
                )?
                .as_slice()
        {
            return Err(divergence(
                GoldAndGearsReplayDivergenceKind::Event,
                action_index,
                battle_index,
                command_index,
            ));
        }
    }
    Ok(())
}

fn encode_nested_state_v5(
    state_hash: starclock_combat::BattleStateHash,
    events: &[starclock_combat::BattleEvent],
) -> Result<Vec<u8>, GoldAndGearsReplayError> {
    if events.len() > MAX_NESTED_BATTLE_EVENTS_PER_COMMAND as usize {
        return Err(GoldAndGearsReplayError::TooManyRecords);
    }
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_STATE_PAYLOAD_VERSION);
    encoder.raw(&state_hash.bytes());
    encoder.u32(u32::try_from(events.len()).map_err(|_| CodecError::LengthOverflow)?);
    for event in events {
        encoder.bytes(&encode_battle_event_payload_for_version(
            event,
            GOLD_AND_GEARS_REPLAY_EVENT_PAYLOAD_VERSION,
        )?)?;
    }
    Ok(encoder.into_inner())
}

fn checked_add(left: u32, right: u32) -> Result<u32, GoldAndGearsReplayError> {
    left.checked_add(right)
        .ok_or(GoldAndGearsReplayError::TooManyRecords)
}

fn divergence(
    kind: GoldAndGearsReplayDivergenceKind,
    action_index: u32,
    battle_index: u32,
    command_index: u32,
) -> GoldAndGearsReplayError {
    GoldAndGearsReplayError::FirstDivergence {
        kind,
        action_index,
        battle_index,
        command_index,
    }
}

#[derive(Debug)]
pub enum GoldAndGearsReplayError {
    Envelope(ReplayV2Error),
    Format(ReplayFormatError),
    Codec(CodecError),
    NestedPayload(NestedBattlePayloadError),
    EventPayload(BattleEventPayloadError),
    ComponentIdentity(ComponentIdentityError),
    ComponentDivergence(Box<ConfigurationComponentDivergence>),
    Execution(GoldAndGearsSeededRunError),
    MissingBuildComponent,
    TooManyRecords,
    FirstDivergence {
        kind: GoldAndGearsReplayDivergenceKind,
        action_index: u32,
        battle_index: u32,
        command_index: u32,
    },
}

impl GoldAndGearsReplayError {
    #[must_use]
    pub const fn first_divergence(&self) -> Option<GoldAndGearsReplayDivergenceKind> {
        match self {
            Self::FirstDivergence { kind, .. } => Some(*kind),
            Self::ComponentDivergence(_) => Some(GoldAndGearsReplayDivergenceKind::Component),
            _ => None,
        }
    }
}

impl From<ReplayV2Error> for GoldAndGearsReplayError {
    fn from(value: ReplayV2Error) -> Self {
        Self::Envelope(value)
    }
}
impl From<ReplayFormatError> for GoldAndGearsReplayError {
    fn from(value: ReplayFormatError) -> Self {
        Self::Format(value)
    }
}
impl From<CodecError> for GoldAndGearsReplayError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}
impl From<NestedBattlePayloadError> for GoldAndGearsReplayError {
    fn from(value: NestedBattlePayloadError) -> Self {
        Self::NestedPayload(value)
    }
}
impl From<BattleEventPayloadError> for GoldAndGearsReplayError {
    fn from(value: BattleEventPayloadError) -> Self {
        Self::EventPayload(value)
    }
}
impl From<ComponentIdentityError> for GoldAndGearsReplayError {
    fn from(value: ComponentIdentityError) -> Self {
        Self::ComponentIdentity(value)
    }
}
impl From<GoldAndGearsSeededRunError> for GoldAndGearsReplayError {
    fn from(value: GoldAndGearsSeededRunError) -> Self {
        Self::Execution(value)
    }
}
impl From<ActionPayloadError> for GoldAndGearsReplayError {
    fn from(value: ActionPayloadError) -> Self {
        match value {
            ActionPayloadError::Codec(error) => Self::Codec(error),
            ActionPayloadError::Version
            | ActionPayloadError::Kind
            | ActionPayloadError::InvalidId
            | ActionPayloadError::PolicyRevision => Self::FirstDivergence {
                kind: GoldAndGearsReplayDivergenceKind::ActivityCommand,
                action_index: 0,
                battle_index: 0,
                command_index: 0,
            },
        }
    }
}

impl fmt::Display for GoldAndGearsReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Gold and Gears replay error: {self:?}")
    }
}

impl std::error::Error for GoldAndGearsReplayError {}
