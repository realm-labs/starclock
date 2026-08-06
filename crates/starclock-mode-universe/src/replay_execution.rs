//! Component-addressed Standard Universe replay with real nested battle proof.

use std::sync::Arc;

use starclock_activity::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityStateHash, ActivityTerminalOutcome,
    BattleResultIdentity,
};
use starclock_combat::{
    BattlePhase, CommandErrorKind, DecisionOwner, TeamSide, catalog::CombatCatalog,
};
use starclock_replay::{
    activity::{
        ActivityCommandPayloadError, ControllerDecisionKind, ControllerDiagnostic,
        decode_controller_diagnostic_payload, decode_nested_battle_end_payload,
        decode_nested_battle_start_payload, encode_controller_diagnostic_payload,
        encode_nested_battle_end_payload, encode_nested_battle_start_payload,
    },
    battle_event::{BattleEventPayloadError, encode_battle_event_payload},
    component::{
        ComponentIdentityError, ConfigurationComponentDivergence, ConfigurationComponentIdentity,
        ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::{ComponentDigest, DefinitionDigest, EntrySpecDigest, StateDigest},
    entry::ReplayEntry,
    format::{ReplayEnvironment, ReplayError, ReplayHeader, decode_replay, encode_replay},
    nested_battle::{
        NestedBattleCommandPayload, NestedBattlePayloadError, decode_nested_battle_command_payload,
        decode_nested_battle_state_payload, encode_nested_battle_command_payload,
        encode_nested_battle_state_payload,
    },
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

use crate::{
    baseline_runner::{NestedBattleExecutionError, StandardUniverseBaselineReport},
    battle_materialization::UniverseBattleMaterialization,
    catalog::UniverseCatalog,
    dynamic_battle_assembler::StandardUniverseBattleAssembler,
    entry::CompiledActivity,
    handler_bundle::activity_handler_registry,
    nested_battle_executor::{
        EventCommitment, NestedBattleExecutionReport, create_nested_battle, project_result,
    },
    replay_trace::{
        StandardUniverseReplayAction, StandardUniverseReplayError as TraceReplayError,
        StandardUniverseTraceEntry, decode_action, encode_action,
    },
    runtime::StandardUniverseActivity,
};

/// Builds the exact ordered component manifest consumed by a materialized
/// Standard Universe activity and its selected controller.
pub fn standard_universe_component_set(
    catalog: &UniverseCatalog,
    compiled: &CompiledActivity,
    materialized: &UniverseBattleMaterialization,
    controller_id: &str,
    controller_digest: [u8; 32],
) -> Result<ConfigurationComponentSet, ComponentIdentityError> {
    let identity = catalog.identity();
    let activity = compiled.runtime_definition().identity();
    let handlers = activity_handler_registry();
    ConfigurationComponentSet::new(vec![
        component(
            ConfigurationComponentKind::CombatCatalog,
            "combat-catalog",
            materialized.combat_catalog().digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::BuildCatalog,
            "build-catalog",
            identity.build_catalog_digest(),
        )?,
        component(
            ConfigurationComponentKind::ActivityCore,
            "standard-universe-activity",
            activity.definition_digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::ModeProfile,
            "standard-universe-profile",
            identity.profile_digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "standard-universe-content",
            identity.universe_bundle_digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::ActivityHandlerRegistry,
            "activity-handlers",
            handlers.digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::CombatRuleRegistry,
            "universe-combat-rules",
            materialized.digest(),
        )?,
        component(
            ConfigurationComponentKind::EncounterOverlay,
            "universe-encounter-overlay",
            materialized.overlay().digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::Controller,
            controller_id,
            controller_digest,
        )?,
    ])
}

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    digest: [u8; 32],
) -> Result<ConfigurationComponentIdentity, ComponentIdentityError> {
    ConfigurationComponentIdentity::new(kind, id, ComponentDigest::new(digest))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedStandardUniverseRun {
    report: StandardUniverseBaselineReport,
    trace: Box<[StandardUniverseTraceEntry]>,
    battles: Box<[NestedBattleExecutionReport]>,
}

impl RecordedStandardUniverseRun {
    pub(crate) fn new(
        report: StandardUniverseBaselineReport,
        trace: Box<[StandardUniverseTraceEntry]>,
        battles: Box<[NestedBattleExecutionReport]>,
    ) -> Self {
        Self {
            report,
            trace,
            battles,
        }
    }

    #[must_use]
    pub const fn report(&self) -> &StandardUniverseBaselineReport {
        &self.report
    }
    #[must_use]
    pub fn trace(&self) -> &[StandardUniverseTraceEntry] {
        &self.trace
    }
    #[must_use]
    pub fn battles(&self) -> &[NestedBattleExecutionReport] {
        &self.battles
    }
}

fn standard_universe_record_count_parts(
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<u32, ReplayExecutionError> {
    let mut count = 0_u32;
    let mut battle_index = 0_usize;
    for entry in trace {
        count = checked_add(count, 2)?;
        if entry.diagnostic().is_some() {
            count = checked_add(count, 1)?;
        }
        if matches!(entry.action(), StandardUniverseReplayAction::Battle { .. }) {
            let report = battles
                .get(battle_index)
                .ok_or(ReplayExecutionError::CapturedBattleMismatch)?;
            battle_index += 1;
            count = checked_add(count, 2)?;
            count = checked_add(
                count,
                u32::try_from(report.trace().len())
                    .map_err(|_| ReplayExecutionError::TooManyRecords)?
                    .checked_mul(2)
                    .ok_or(ReplayExecutionError::TooManyRecords)?,
            )?;
        }
    }
    if battle_index != battles.len() || count > MAX_REPLAY_RECORDS {
        Err(ReplayExecutionError::TooManyRecords)
    } else {
        Ok(count)
    }
}

pub(crate) fn encode_trace_for_verification(
    header_template: &ReplayHeader,
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<Vec<u8>, ReplayExecutionError> {
    encode_standard_universe_trace_parts(header_template, trace, battles)
}

fn encode_standard_universe_trace_parts(
    header_template: &ReplayHeader,
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<Vec<u8>, ReplayExecutionError> {
    let count = standard_universe_record_count_parts(trace, battles)?;
    let header = ReplayHeader::new(
        header_template.environment().clone(),
        header_template.components().clone(),
        header_template.master_seed(),
        header_template.entry().clone(),
        count,
    )?;
    let mut payloads = Vec::<(RecordKind, Vec<u8>)>::with_capacity(count as usize);
    let mut battles = battles.iter();
    for entry in trace {
        if let Some(diagnostic) = entry.diagnostic() {
            payloads.push((
                RecordKind::ControllerDiagnostic,
                encode_controller_diagnostic_payload(diagnostic)?,
            ));
        }
        let battle_report = if let StandardUniverseReplayAction::Battle { result } = entry.action()
        {
            payloads.push((
                RecordKind::NestedBattleStart,
                encode_nested_battle_start_payload(result.identity()),
            ));
            Some(
                battles
                    .next()
                    .ok_or(ReplayExecutionError::CapturedBattleMismatch)?,
            )
        } else {
            None
        };
        payloads.push((
            RecordKind::AcceptedActivityCommand,
            encode_action(entry.action())?,
        ));
        if let Some(report) = battle_report {
            for step in report.trace() {
                payloads.push((
                    RecordKind::AcceptedBattleCommand,
                    encode_nested_battle_command_payload(&NestedBattleCommandPayload::new(
                        step.controller() as u8,
                        step.command().clone(),
                    ))?,
                ));
                payloads.push((
                    RecordKind::ExpectedBattleState,
                    encode_nested_battle_state_payload(step.state_hash(), step.events())?,
                ));
            }
            let result = match entry.action() {
                StandardUniverseReplayAction::Battle { result } => result,
                _ => unreachable!("battle report has a battle action"),
            };
            payloads.push((
                RecordKind::NestedBattleEnd,
                encode_nested_battle_end_payload(result.actual_digest()),
            ));
        }
        payloads.push((
            RecordKind::ExpectedActivityState,
            entry.state_hash().bytes().to_vec(),
        ));
    }
    if battles.next().is_some() {
        return Err(ReplayExecutionError::CapturedBattleMismatch);
    }
    let records = payloads
        .iter()
        .enumerate()
        .map(|(sequence, (kind, payload))| RecordRef::new(*kind, sequence as u64, payload))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(encode_replay(&header, &records, Vec::new())?)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardUniverseReplayReport {
    action_count: u32,
    battle_count: u32,
    battle_command_count: u32,
    final_state_hash: StateDigest,
    terminal: ActivityTerminalOutcome,
}

impl StandardUniverseReplayReport {
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

#[allow(clippy::too_many_arguments)]
pub fn execute_standard_universe_replay(
    bytes: &[u8],
    activity: StandardUniverseActivity,
    catalog: Arc<CombatCatalog>,
    actual_components: &ConfigurationComponentSet,
    actual_environment: &ReplayEnvironment,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, ReplayExecutionError> {
    execute_standard_universe_replay_with_source(
        bytes,
        activity,
        BattleCatalogSource::Static(catalog),
        actual_components,
        actual_environment,
        expected_profile_id,
    )
}

pub(crate) fn execute_standard_universe_replay_dynamic(
    bytes: &[u8],
    activity: StandardUniverseActivity,
    assembler: &StandardUniverseBattleAssembler,
    actual_components: &ConfigurationComponentSet,
    actual_environment: &ReplayEnvironment,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, ReplayExecutionError> {
    execute_standard_universe_replay_with_source(
        bytes,
        activity,
        BattleCatalogSource::Dynamic(assembler),
        actual_components,
        actual_environment,
        expected_profile_id,
    )
}

enum BattleCatalogSource<'a> {
    Static(Arc<CombatCatalog>),
    Dynamic(&'a StandardUniverseBattleAssembler),
}

#[allow(clippy::too_many_arguments)]
fn execute_standard_universe_replay_with_source(
    bytes: &[u8],
    mut activity: StandardUniverseActivity,
    source: BattleCatalogSource<'_>,
    actual_components: &ConfigurationComponentSet,
    actual_environment: &ReplayEnvironment,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, ReplayExecutionError> {
    let replay = decode_replay(bytes)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(ReplayExecutionError::ComponentDivergence)?;
    validate_environment(replay.header().environment(), actual_environment)?;
    validate_entry(replay.header().entry(), &activity, expected_profile_id)?;

    let records = replay.records();
    let mut cursor = 0_usize;
    let mut action_index = 0_u32;
    let mut battle_index = 0_u32;
    let mut battle_command_count = 0_u32;
    let mut final_state_hash = StateDigest::new(activity.view().state_hash().bytes());
    while cursor < records.len() {
        let diagnostic = take_diagnostic(records, &mut cursor)?;
        let nested_start = take_nested_start(records, &mut cursor)?;
        let action_record = expect_record(records, cursor, RecordKind::AcceptedActivityCommand)?;
        let action = decode_action(action_record.payload())?;
        cursor += 1;
        match &action {
            StandardUniverseReplayAction::Decision {
                decision,
                kind,
                option,
                technique_points,
            } => {
                if nested_start.is_some() {
                    return Err(layout(cursor));
                }
                validate_diagnostic(
                    &activity,
                    *decision,
                    *kind,
                    *option,
                    diagnostic.as_ref(),
                    action_index,
                )?;
                let hash = activity.view().state_hash();
                match kind {
                    ActivityDecisionKind::Encounter => activity
                        .engage_encounter(hash, *decision, *option, *technique_points)
                        .map(|_| ())
                        .map_err(|_| ReplayExecutionError::ActivityCommandRejected {
                            action_index,
                        })?,
                    ActivityDecisionKind::ExternalOutcome => activity
                        .submit_external_outcome(
                            hash,
                            *decision,
                            ActivityExternalOutcomeId::new(option.get())
                                .expect("offered option ID is non-zero"),
                        )
                        .map(|_| ())
                        .map_err(|_| ReplayExecutionError::ActivityCommandRejected {
                            action_index,
                        })?,
                    _ => activity
                        .choose_option(hash, *decision, *option)
                        .map(|_| ())
                        .map_err(|_| ReplayExecutionError::ActivityCommandRejected {
                            action_index,
                        })?,
                }
            }
            StandardUniverseReplayAction::Preparation { option } => {
                if nested_start.is_some() || diagnostic.is_some() {
                    return Err(layout(cursor));
                }
                activity
                    .choose_preparation_option(activity.view().state_hash(), *option)
                    .map_err(|_| ReplayExecutionError::ActivityCommandRejected { action_index })?;
            }
            StandardUniverseReplayAction::Battle {
                result: recorded_result,
            } => {
                if diagnostic.is_some() {
                    return Err(layout(cursor));
                }
                let start = nested_start
                    .ok_or(ReplayExecutionError::MissingNestedBoundary { action_index })?;
                let (handoff, catalog) = match &source {
                    BattleCatalogSource::Static(catalog) => (
                        activity
                            .start_pending_battle(activity.view().state_hash())
                            .map_err(|_| ReplayExecutionError::ActivityCommandRejected {
                                action_index,
                            })?,
                        Arc::clone(catalog),
                    ),
                    BattleCatalogSource::Dynamic(assembler) => {
                        let start =
                            assembler.start_pending_battle(&mut activity).map_err(|_| {
                                ReplayExecutionError::ActivityCommandRejected { action_index }
                            })?;
                        (start.handoff().clone(), Arc::clone(start.combat_catalog()))
                    }
                };
                if start != handoff.identity() || recorded_result.identity() != start {
                    return Err(ReplayExecutionError::NestedStartDivergence {
                        action_index,
                        expected: Box::new(handoff.identity()),
                        actual: Box::new(start),
                    });
                }
                let (commands, actual_result) = verify_nested_battle(
                    records,
                    &mut cursor,
                    action_index,
                    battle_index,
                    &handoff,
                    catalog,
                    recorded_result,
                )?;
                battle_command_count = checked_add(battle_command_count, commands)?;
                battle_index += 1;
                activity
                    .submit_pending_battle_result(activity.view().state_hash(), actual_result)
                    .map_err(|_| ReplayExecutionError::ActivityCommandRejected { action_index })?;
            }
        }
        compare_activity_state(
            records,
            &mut cursor,
            action_index,
            activity.view().state_hash(),
            &mut final_state_hash,
        )?;
        action_index += 1;
    }
    let terminal = activity
        .view()
        .terminal()
        .ok_or(ReplayExecutionError::IncompleteActivity)?;
    Ok(StandardUniverseReplayReport {
        action_count: action_index,
        battle_count: battle_index,
        battle_command_count,
        final_state_hash,
        terminal,
    })
}

#[allow(clippy::too_many_arguments)]
fn verify_nested_battle(
    records: &[RecordRef<'_>],
    cursor: &mut usize,
    action_index: u32,
    battle_index: u32,
    handoff: &starclock_activity::ActivityBattleHandoff,
    catalog: Arc<CombatCatalog>,
    recorded_result: &starclock_activity::BattleResult,
) -> Result<(u32, starclock_activity::BattleResult), ReplayExecutionError> {
    let mut battle = create_nested_battle(Arc::clone(&catalog), handoff)?;
    let mut commitment = EventCommitment::new(&catalog, handoff);
    let mut command_index = 0_u32;
    let mut timeline_elapsed_scaled = 0_i64;
    while records
        .get(*cursor)
        .is_some_and(|record| record.kind() == RecordKind::AcceptedBattleCommand)
    {
        let command_payload = decode_nested_battle_command_payload(records[*cursor].payload())?;
        validate_controller(
            command_payload.controller(),
            battle.decision().map(|d| d.owner()),
            battle.view().phase(),
        )
        .map_err(|actual| ReplayExecutionError::ControllerDivergence {
            battle_index,
            command_index,
            recorded: command_payload.controller(),
            actual,
        })?;
        *cursor += 1;
        let expected_record = expect_record(records, *cursor, RecordKind::ExpectedBattleState)?;
        let expected = decode_nested_battle_state_payload(expected_record.payload())?;
        *cursor += 1;
        let command = command_payload.command().clone();
        let resolution = battle.apply(command.clone()).map_err(|error| {
            ReplayExecutionError::BattleCommandRejected {
                battle_index,
                command_index,
                kind: error.kind(),
            }
        })?;
        timeline_elapsed_scaled = timeline_elapsed_scaled
            .checked_add(resolution.timeline_elapsed_scaled())
            .ok_or(ReplayExecutionError::NestedBattleIncomplete {
                battle_index,
                command_index,
            })?;
        compare_events(
            battle_index,
            command_index,
            expected.event_payloads(),
            resolution.events(),
        )?;
        if expected.state_hash().bytes() != resolution.state_hash().bytes() {
            return Err(ReplayExecutionError::BattleStateDivergence {
                battle_index,
                command_index,
                expected: expected.state_hash(),
                actual: StateDigest::new(resolution.state_hash().bytes()),
            });
        }
        commitment.push(&command, &resolution);
        command_index += 1;
    }
    if !battle.view().phase().is_terminal() {
        return Err(ReplayExecutionError::NestedBattleIncomplete {
            battle_index,
            command_index,
        });
    }
    let end = expect_record(records, *cursor, RecordKind::NestedBattleEnd)?;
    let expected_end = decode_nested_battle_end_payload(end.payload())?;
    *cursor += 1;
    let actual_result = project_result(
        &battle,
        handoff,
        commitment.finish(),
        timeline_elapsed_scaled,
    )?;
    if expected_end != actual_result.actual_digest() || recorded_result != &actual_result {
        return Err(ReplayExecutionError::NestedResultDivergence {
            action_index,
            expected: recorded_result.actual_digest(),
            actual: actual_result.actual_digest(),
        });
    }
    Ok((command_index, actual_result))
}

fn compare_events(
    battle_index: u32,
    command_index: u32,
    expected: &[&[u8]],
    actual: &[starclock_combat::BattleEvent],
) -> Result<(), ReplayExecutionError> {
    let shared = expected.len().min(actual.len());
    for event_index in 0..shared {
        let payload = encode_battle_event_payload(&actual[event_index])?;
        if expected[event_index] != payload {
            return Err(ReplayExecutionError::BattleEventDivergence {
                battle_index,
                command_index,
                event_index: event_index as u32,
                expected_count: expected.len() as u32,
                actual_count: actual.len() as u32,
            });
        }
    }
    if expected.len() != actual.len() {
        return Err(ReplayExecutionError::BattleEventDivergence {
            battle_index,
            command_index,
            event_index: shared as u32,
            expected_count: expected.len() as u32,
            actual_count: actual.len() as u32,
        });
    }
    Ok(())
}

fn validate_controller(
    recorded: u8,
    owner: Option<DecisionOwner>,
    phase: BattlePhase,
) -> Result<(), u8> {
    let actual = match (owner, phase) {
        (None, BattlePhase::ReadyToAdvance) => 0,
        (Some(DecisionOwner::System), _) => 0,
        (Some(DecisionOwner::Team(TeamSide::Player)), _) => 1,
        (Some(DecisionOwner::Team(TeamSide::Enemy)), _) => 2,
        (None, _) => u8::MAX,
    };
    if recorded == actual {
        Ok(())
    } else {
        Err(actual)
    }
}

fn take_diagnostic(
    records: &[RecordRef<'_>],
    cursor: &mut usize,
) -> Result<Option<ControllerDiagnostic>, ReplayExecutionError> {
    if records
        .get(*cursor)
        .is_some_and(|record| record.kind() == RecordKind::ControllerDiagnostic)
    {
        let value = decode_controller_diagnostic_payload(records[*cursor].payload())?;
        *cursor += 1;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn take_nested_start(
    records: &[RecordRef<'_>],
    cursor: &mut usize,
) -> Result<Option<BattleResultIdentity>, ReplayExecutionError> {
    if records
        .get(*cursor)
        .is_some_and(|record| record.kind() == RecordKind::NestedBattleStart)
    {
        let value = decode_nested_battle_start_payload(records[*cursor].payload())?;
        *cursor += 1;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn validate_diagnostic(
    activity: &StandardUniverseActivity,
    decision: starclock_activity::ActivityDecisionId,
    kind: ActivityDecisionKind,
    option: starclock_activity::ActivityOptionId,
    diagnostic: Option<&ControllerDiagnostic>,
    action_index: u32,
) -> Result<(), ReplayExecutionError> {
    let view = activity.view();
    let offered = view
        .decision()
        .filter(|value| value.id() == decision && value.kind() == kind)
        .ok_or(ReplayExecutionError::DecisionDivergence { action_index })?;
    let diagnostic = diagnostic.ok_or(ReplayExecutionError::DecisionDivergence { action_index })?;
    let selected = diagnostic.selected_ordinal() as usize;
    if diagnostic.kind() != ControllerDecisionKind::Activity
        || diagnostic.decision_sequence() != u64::from(action_index)
        || diagnostic.scores().len() != offered.options().len()
    {
        return Err(ReplayExecutionError::DecisionDivergence { action_index });
    }
    if offered.options().get(selected).map(|value| value.id()) != Some(option) {
        return Err(ReplayExecutionError::DecisionDivergence { action_index });
    }
    Ok(())
}

fn compare_activity_state(
    records: &[RecordRef<'_>],
    cursor: &mut usize,
    action_index: u32,
    actual: ActivityStateHash,
    final_hash: &mut StateDigest,
) -> Result<(), ReplayExecutionError> {
    let record = expect_record(records, *cursor, RecordKind::ExpectedActivityState)?;
    let expected: [u8; 32] = record
        .payload()
        .try_into()
        .map_err(|_| ReplayExecutionError::InvalidActivityStatePayload)?;
    if expected != actual.bytes() {
        return Err(ReplayExecutionError::ActivityStateDivergence {
            action_index,
            expected: StateDigest::new(expected),
            actual: StateDigest::new(actual.bytes()),
        });
    }
    *final_hash = StateDigest::new(actual.bytes());
    *cursor += 1;
    Ok(())
}

fn validate_environment(
    expected: &ReplayEnvironment,
    actual: &ReplayEnvironment,
) -> Result<(), ReplayExecutionError> {
    if expected != actual {
        return Err(ReplayExecutionError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_entry(
    entry: &ReplayEntry,
    activity: &StandardUniverseActivity,
    expected_profile_id: &str,
) -> Result<(), ReplayExecutionError> {
    let identity = activity.graph().definition().identity();
    match entry {
        ReplayEntry::Activity {
            profile_id,
            definition_id,
            definition_digest,
            spec_digest,
            ..
        } if profile_id.as_ref() == expected_profile_id
            && *definition_id == identity.id().get()
            && definition_digest.bytes() == identity.definition_digest().bytes()
            && spec_digest.bytes() == identity.config_digest().bytes() =>
        {
            Ok(())
        }
        _ => Err(ReplayExecutionError::EntryMismatch),
    }
}

fn expect_record<'a>(
    records: &'a [RecordRef<'a>],
    cursor: usize,
    kind: RecordKind,
) -> Result<&'a RecordRef<'a>, ReplayExecutionError> {
    records
        .get(cursor)
        .filter(|record| record.kind() == kind)
        .ok_or_else(|| layout(cursor))
}

fn checked_add(left: u32, right: u32) -> Result<u32, ReplayExecutionError> {
    left.checked_add(right)
        .ok_or(ReplayExecutionError::TooManyRecords)
}

fn layout(record_index: usize) -> ReplayExecutionError {
    ReplayExecutionError::InvalidRecordLayout {
        record_index: record_index as u32,
    }
}

#[derive(Debug)]
pub enum ReplayExecutionError {
    Format(ReplayFormatError),
    Envelope(ReplayError),
    Payload(ActivityCommandPayloadError),
    Trace(TraceReplayError),
    NestedPayload(NestedBattlePayloadError),
    EventPayload(BattleEventPayloadError),
    Execution(NestedBattleExecutionError),
    ComponentIdentity(ComponentIdentityError),
    ComponentDivergence(Box<ConfigurationComponentDivergence>),
    CapturedBattleMismatch,
    TooManyRecords,
    EnvironmentMismatch,
    EntryMismatch,
    InvalidRecordLayout {
        record_index: u32,
    },
    InvalidActivityStatePayload,
    MissingNestedBoundary {
        action_index: u32,
    },
    DecisionDivergence {
        action_index: u32,
    },
    ActivityCommandRejected {
        action_index: u32,
    },
    NestedStartDivergence {
        action_index: u32,
        expected: Box<BattleResultIdentity>,
        actual: Box<BattleResultIdentity>,
    },
    ControllerDivergence {
        battle_index: u32,
        command_index: u32,
        recorded: u8,
        actual: u8,
    },
    BattleCommandRejected {
        battle_index: u32,
        command_index: u32,
        kind: CommandErrorKind,
    },
    BattleStateDivergence {
        battle_index: u32,
        command_index: u32,
        expected: StateDigest,
        actual: StateDigest,
    },
    BattleEventDivergence {
        battle_index: u32,
        command_index: u32,
        event_index: u32,
        expected_count: u32,
        actual_count: u32,
    },
    NestedBattleIncomplete {
        battle_index: u32,
        command_index: u32,
    },
    NestedResultDivergence {
        action_index: u32,
        expected: starclock_activity::BattleResultDigest,
        actual: starclock_activity::BattleResultDigest,
    },
    ActivityStateDivergence {
        action_index: u32,
        expected: StateDigest,
        actual: StateDigest,
    },
    IncompleteActivity,
}

impl From<ReplayFormatError> for ReplayExecutionError {
    fn from(value: ReplayFormatError) -> Self {
        Self::Format(value)
    }
}
impl From<ReplayError> for ReplayExecutionError {
    fn from(value: ReplayError) -> Self {
        Self::Envelope(value)
    }
}
impl From<ActivityCommandPayloadError> for ReplayExecutionError {
    fn from(value: ActivityCommandPayloadError) -> Self {
        Self::Payload(value)
    }
}
impl From<TraceReplayError> for ReplayExecutionError {
    fn from(value: TraceReplayError) -> Self {
        Self::Trace(value)
    }
}
impl From<NestedBattlePayloadError> for ReplayExecutionError {
    fn from(value: NestedBattlePayloadError) -> Self {
        Self::NestedPayload(value)
    }
}
impl From<BattleEventPayloadError> for ReplayExecutionError {
    fn from(value: BattleEventPayloadError) -> Self {
        Self::EventPayload(value)
    }
}
impl From<NestedBattleExecutionError> for ReplayExecutionError {
    fn from(value: NestedBattleExecutionError) -> Self {
        Self::Execution(value)
    }
}
impl From<ComponentIdentityError> for ReplayExecutionError {
    fn from(value: ComponentIdentityError) -> Self {
        Self::ComponentIdentity(value)
    }
}

/// Convenience constructor for a zero-record production header template.
pub fn standard_universe_header(
    environment: ReplayEnvironment,
    components: ConfigurationComponentSet,
    master_seed: u64,
    activity: &StandardUniverseActivity,
    profile_id: &str,
) -> Result<ReplayHeader, ReplayExecutionError> {
    Ok(ReplayHeader::new(
        environment,
        components,
        master_seed,
        replay_entry(activity, profile_id),
        0,
    )?)
}

fn replay_entry(activity: &StandardUniverseActivity, profile_id: &str) -> ReplayEntry {
    let identity = activity.graph().definition().identity();
    ReplayEntry::Activity {
        profile_id: profile_id.into(),
        definition_id: identity.id().get(),
        definition_digest: DefinitionDigest::new(identity.definition_digest().bytes()),
        spec_digest: EntrySpecDigest::new(identity.config_digest().bytes()),
        builds: None,
    }
}
