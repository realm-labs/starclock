//! Component-addressed Standard Universe replay with real nested battle proof.

use std::sync::Arc;

use starclock_activity::{
    ACTIVITY_STATE_HASH_REVISION, ActivityDecisionKind, ActivityExternalOutcomeId,
    ActivityStateHash, ActivityTerminalOutcome, BattleResultIdentity,
};
use starclock_combat::{
    CommandErrorKind, DecisionOwner, NUMERIC_POLICY_REVISION, TeamSide, catalog::CombatCatalog,
    rng::RNG_ALGORITHM_REVISION,
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
    digest::{ComponentDigest, StateDigest},
    format::ReplayEntry,
    format_v2::{
        ReplayCompatibilityV2, ReplayHeaderV2, ReplayV2Error, decode_replay_v2, encode_replay_v2,
    },
    nested_battle::{
        NestedBattleCommandPayload, NestedBattlePayloadError, decode_nested_battle_command_payload,
        decode_nested_battle_state_payload, encode_nested_battle_command_payload,
        encode_nested_battle_state_payload,
    },
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

use crate::{
    baseline_runner::{
        NestedBattleExecutionError, StandardUniverseBaselinePolicy, StandardUniverseBaselineReport,
    },
    battle_materialization::UniverseBattleMaterialization,
    catalog::UniverseCatalog,
    entry::CompiledActivity,
    handler_bundle::activity_handler_registry,
    nested_battle_executor::{
        EventCommitment, NestedBattleExecutionReport, UniverseNestedBattleExecutor,
        create_nested_battle, project_result,
    },
    runtime::StandardUniverseActivity,
    universe_replay::{
        StandardUniverseReplayAction, StandardUniverseReplayError, StandardUniverseTraceEntry,
        decode_action, encode_action, record_baseline_run, replay_entry_for,
    },
};

pub const STANDARD_UNIVERSE_REAL_BATTLE_REPLAY_REVISION: &str =
    "standard-universe-real-battle-replay-v1";

/// Builds the exact ordered component manifest consumed by a materialized
/// Standard Universe activity and its selected controller.
pub fn standard_universe_component_set(
    catalog: &UniverseCatalog,
    compiled: &CompiledActivity,
    materialized: &UniverseBattleMaterialization,
    controller_id: &str,
    controller_revision: &str,
    controller_digest: [u8; 32],
) -> Result<ConfigurationComponentSet, ComponentIdentityError> {
    let identity = catalog.identity();
    let activity = compiled.runtime_definition().identity();
    let handlers = activity_handler_registry();
    ConfigurationComponentSet::new(vec![
        component(
            ConfigurationComponentKind::CombatCatalog,
            "combat-catalog",
            materialized.combat_catalog().revision().as_str(),
            materialized.combat_catalog().digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::BuildCatalog,
            "build-catalog",
            identity.core_data_revision(),
            identity.build_catalog_digest(),
        )?,
        component(
            ConfigurationComponentKind::ActivityCore,
            "standard-universe-activity",
            ACTIVITY_STATE_HASH_REVISION,
            activity.definition_digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::ModeProfile,
            "standard-universe-profile",
            identity.profile_revision(),
            identity.profile_digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "standard-universe-content",
            identity.catalog_revision(),
            identity.universe_bundle_digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::ActivityHandlerRegistry,
            "activity-handlers",
            starclock_activity::ACTIVITY_HANDLER_REGISTRY_REVISION,
            handlers.digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::CombatRuleRegistry,
            "universe-combat-rules",
            "standard-universe-rule-registry-v1",
            materialized.digest(),
        )?,
        component(
            ConfigurationComponentKind::EncounterOverlay,
            "universe-encounter-overlay",
            "standard-universe-encounter-overlay-v1",
            materialized.overlay().digest().bytes(),
        )?,
        component(
            ConfigurationComponentKind::Controller,
            controller_id,
            controller_revision,
            controller_digest,
        )?,
    ])
}

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    revision: &str,
    digest: [u8; 32],
) -> Result<ConfigurationComponentIdentity, ComponentIdentityError> {
    ConfigurationComponentIdentity::new(kind, id, revision, ComponentDigest::new(digest))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedStandardUniverseRunV2 {
    report: StandardUniverseBaselineReport,
    trace: Box<[StandardUniverseTraceEntry]>,
    battles: Box<[NestedBattleExecutionReport]>,
}

impl RecordedStandardUniverseRunV2 {
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

/// Drives the production nested executor and retains every accepted command
/// plus complete emitted events. Fake/atomic battle executors cannot enter v2.
pub fn record_baseline_run_v2(
    activity: &mut StandardUniverseActivity,
    policy: &StandardUniverseBaselinePolicy,
    executor: &mut UniverseNestedBattleExecutor,
) -> Result<RecordedStandardUniverseRunV2, StandardUniverseReplayV2Error> {
    let first_report = executor.reports().len();
    let recorded = record_baseline_run(activity, policy, executor)?;
    let battles = executor.reports()[first_report..].to_vec();
    let expected_battles = recorded
        .trace()
        .iter()
        .filter(|entry| matches!(entry.action(), StandardUniverseReplayAction::Battle { .. }))
        .count();
    if battles.len() != expected_battles {
        return Err(StandardUniverseReplayV2Error::CapturedBattleMismatch);
    }
    Ok(RecordedStandardUniverseRunV2 {
        report: recorded.report().clone(),
        trace: recorded.trace().to_vec().into_boxed_slice(),
        battles: battles.into_boxed_slice(),
    })
}

pub fn standard_universe_record_count_v2(
    recorded: &RecordedStandardUniverseRunV2,
) -> Result<u32, StandardUniverseReplayV2Error> {
    standard_universe_record_count_parts_v2(recorded.trace(), recorded.battles())
}

fn standard_universe_record_count_parts_v2(
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<u32, StandardUniverseReplayV2Error> {
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
                .ok_or(StandardUniverseReplayV2Error::CapturedBattleMismatch)?;
            battle_index += 1;
            count = checked_add(count, 2)?;
            count = checked_add(
                count,
                u32::try_from(report.trace().len())
                    .map_err(|_| StandardUniverseReplayV2Error::TooManyRecords)?
                    .checked_mul(2)
                    .ok_or(StandardUniverseReplayV2Error::TooManyRecords)?,
            )?;
        }
    }
    if battle_index != battles.len() || count > MAX_REPLAY_RECORDS {
        Err(StandardUniverseReplayV2Error::TooManyRecords)
    } else {
        Ok(count)
    }
}

pub fn encode_standard_universe_trace_v2(
    header_template: &ReplayHeaderV2,
    recorded: &RecordedStandardUniverseRunV2,
) -> Result<Vec<u8>, StandardUniverseReplayV2Error> {
    encode_standard_universe_trace_parts_v2(header_template, recorded.trace(), recorded.battles())
}

/// Encodes an incremental externally controlled Activity session. Battle
/// reports must correspond one-to-one with Battle actions in trace order.
pub fn encode_standard_universe_trace_parts_v2(
    header_template: &ReplayHeaderV2,
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<Vec<u8>, StandardUniverseReplayV2Error> {
    let count = standard_universe_record_count_parts_v2(trace, battles)?;
    let header = ReplayHeaderV2::new(
        header_template.compatibility().clone(),
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
                    .ok_or(StandardUniverseReplayV2Error::CapturedBattleMismatch)?,
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
        return Err(StandardUniverseReplayV2Error::CapturedBattleMismatch);
    }
    let records = payloads
        .iter()
        .enumerate()
        .map(|(sequence, (kind, payload))| RecordRef::new(*kind, sequence as u64, payload))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(encode_replay_v2(&header, &records, Vec::new())?)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardUniverseReplayReportV2 {
    action_count: u32,
    battle_count: u32,
    battle_command_count: u32,
    final_state_hash: StateDigest,
    terminal: ActivityTerminalOutcome,
}

impl StandardUniverseReplayReportV2 {
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
pub fn verify_standard_universe_replay_v2(
    bytes: &[u8],
    mut activity: StandardUniverseActivity,
    catalog: Arc<CombatCatalog>,
    actual_components: &ConfigurationComponentSet,
    actual_compatibility: &ReplayCompatibilityV2,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReportV2, StandardUniverseReplayV2Error> {
    let replay = decode_replay_v2(bytes)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(StandardUniverseReplayV2Error::ComponentDivergence)?;
    validate_compatibility(replay.header().compatibility(), actual_compatibility)?;
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
                        .map_err(|_| StandardUniverseReplayV2Error::ActivityCommandRejected {
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
                        .map_err(|_| StandardUniverseReplayV2Error::ActivityCommandRejected {
                            action_index,
                        })?,
                    _ => activity
                        .choose_option(hash, *decision, *option)
                        .map(|_| ())
                        .map_err(|_| StandardUniverseReplayV2Error::ActivityCommandRejected {
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
                    .map_err(|_| StandardUniverseReplayV2Error::ActivityCommandRejected {
                        action_index,
                    })?;
            }
            StandardUniverseReplayAction::Battle {
                result: recorded_result,
            } => {
                if diagnostic.is_some() {
                    return Err(layout(cursor));
                }
                let start = nested_start
                    .ok_or(StandardUniverseReplayV2Error::MissingNestedBoundary { action_index })?;
                let handoff = activity
                    .start_pending_battle(activity.view().state_hash())
                    .map_err(|_| StandardUniverseReplayV2Error::ActivityCommandRejected {
                        action_index,
                    })?;
                if start != handoff.identity() || recorded_result.identity() != start {
                    return Err(StandardUniverseReplayV2Error::NestedStartDivergence {
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
                    Arc::clone(&catalog),
                    recorded_result,
                )?;
                battle_command_count = checked_add(battle_command_count, commands)?;
                battle_index += 1;
                activity
                    .submit_pending_battle_result(activity.view().state_hash(), actual_result)
                    .map_err(|_| StandardUniverseReplayV2Error::ActivityCommandRejected {
                        action_index,
                    })?;
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
        .ok_or(StandardUniverseReplayV2Error::IncompleteActivity)?;
    Ok(StandardUniverseReplayReportV2 {
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
) -> Result<(u32, starclock_activity::BattleResult), StandardUniverseReplayV2Error> {
    let mut battle = create_nested_battle(Arc::clone(&catalog), handoff)?;
    let mut commitment = EventCommitment::new(&catalog, handoff);
    let mut command_index = 0_u32;
    while records
        .get(*cursor)
        .is_some_and(|record| record.kind() == RecordKind::AcceptedBattleCommand)
    {
        let command_payload = decode_nested_battle_command_payload(records[*cursor].payload())?;
        validate_controller(
            command_payload.controller(),
            battle.decision().map(|d| d.owner()),
        )
        .map_err(
            |actual| StandardUniverseReplayV2Error::ControllerDivergence {
                battle_index,
                command_index,
                recorded: command_payload.controller(),
                actual,
            },
        )?;
        *cursor += 1;
        let expected_record = expect_record(records, *cursor, RecordKind::ExpectedBattleState)?;
        let expected = decode_nested_battle_state_payload(expected_record.payload())?;
        *cursor += 1;
        let command = command_payload.command().clone();
        let resolution = battle.apply(command.clone()).map_err(|error| {
            StandardUniverseReplayV2Error::BattleCommandRejected {
                battle_index,
                command_index,
                kind: error.kind(),
            }
        })?;
        if expected.state_hash().bytes() != resolution.state_hash().bytes() {
            return Err(StandardUniverseReplayV2Error::BattleStateDivergence {
                battle_index,
                command_index,
                expected: expected.state_hash(),
                actual: StateDigest::new(resolution.state_hash().bytes()),
            });
        }
        compare_events(
            battle_index,
            command_index,
            expected.event_payloads(),
            resolution.events(),
        )?;
        commitment.push(&command, &resolution);
        command_index += 1;
    }
    if !battle.view().phase().is_terminal() {
        return Err(StandardUniverseReplayV2Error::NestedBattleIncomplete {
            battle_index,
            command_index,
        });
    }
    let end = expect_record(records, *cursor, RecordKind::NestedBattleEnd)?;
    let expected_end = decode_nested_battle_end_payload(end.payload())?;
    *cursor += 1;
    let actual_result = project_result(&battle, handoff, commitment.finish())?;
    if expected_end != actual_result.actual_digest() || recorded_result != &actual_result {
        return Err(StandardUniverseReplayV2Error::NestedResultDivergence {
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
) -> Result<(), StandardUniverseReplayV2Error> {
    let shared = expected.len().min(actual.len());
    for event_index in 0..shared {
        let payload = encode_battle_event_payload(&actual[event_index])?;
        if expected[event_index] != payload {
            return Err(StandardUniverseReplayV2Error::BattleEventDivergence {
                battle_index,
                command_index,
                event_index: event_index as u32,
                expected_count: expected.len() as u32,
                actual_count: actual.len() as u32,
            });
        }
    }
    if expected.len() != actual.len() {
        return Err(StandardUniverseReplayV2Error::BattleEventDivergence {
            battle_index,
            command_index,
            event_index: shared as u32,
            expected_count: expected.len() as u32,
            actual_count: actual.len() as u32,
        });
    }
    Ok(())
}

fn validate_controller(recorded: u8, owner: Option<DecisionOwner>) -> Result<(), u8> {
    let actual = match owner {
        Some(DecisionOwner::System) => 0,
        Some(DecisionOwner::Team(TeamSide::Player)) => 1,
        Some(DecisionOwner::Team(TeamSide::Enemy)) => 2,
        None => u8::MAX,
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
) -> Result<Option<ControllerDiagnostic>, StandardUniverseReplayV2Error> {
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
) -> Result<Option<BattleResultIdentity>, StandardUniverseReplayV2Error> {
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
) -> Result<(), StandardUniverseReplayV2Error> {
    let view = activity.view();
    let offered = view
        .decision()
        .filter(|value| value.id() == decision && value.kind() == kind)
        .ok_or(StandardUniverseReplayV2Error::DecisionDivergence { action_index })?;
    let diagnostic =
        diagnostic.ok_or(StandardUniverseReplayV2Error::DecisionDivergence { action_index })?;
    let selected = diagnostic.selected_ordinal() as usize;
    if diagnostic.kind() != ControllerDecisionKind::Activity
        || diagnostic.decision_sequence() != u64::from(action_index)
        || diagnostic.scores().len() != offered.options().len()
    {
        return Err(StandardUniverseReplayV2Error::DecisionDivergence { action_index });
    }
    if offered.options().get(selected).map(|value| value.id()) != Some(option) {
        return Err(StandardUniverseReplayV2Error::DecisionDivergence { action_index });
    }
    Ok(())
}

fn compare_activity_state(
    records: &[RecordRef<'_>],
    cursor: &mut usize,
    action_index: u32,
    actual: ActivityStateHash,
    final_hash: &mut StateDigest,
) -> Result<(), StandardUniverseReplayV2Error> {
    let record = expect_record(records, *cursor, RecordKind::ExpectedActivityState)?;
    let expected: [u8; 32] = record
        .payload()
        .try_into()
        .map_err(|_| StandardUniverseReplayV2Error::InvalidActivityStatePayload)?;
    if expected != actual.bytes() {
        return Err(StandardUniverseReplayV2Error::ActivityStateDivergence {
            action_index,
            expected: StateDigest::new(expected),
            actual: StateDigest::new(actual.bytes()),
        });
    }
    *final_hash = StateDigest::new(actual.bytes());
    *cursor += 1;
    Ok(())
}

fn validate_compatibility(
    expected: &ReplayCompatibilityV2,
    actual: &ReplayCompatibilityV2,
) -> Result<(), StandardUniverseReplayV2Error> {
    if expected != actual
        || actual.numeric_policy_revision() != NUMERIC_POLICY_REVISION
        || actual.rng_algorithm_revision() != RNG_ALGORITHM_REVISION
        || actual.state_hash_revision() != ACTIVITY_STATE_HASH_REVISION
    {
        return Err(StandardUniverseReplayV2Error::CompatibilityMismatch);
    }
    Ok(())
}

fn validate_entry(
    entry: &ReplayEntry,
    activity: &StandardUniverseActivity,
    expected_profile_id: &str,
) -> Result<(), StandardUniverseReplayV2Error> {
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
        _ => Err(StandardUniverseReplayV2Error::EntryMismatch),
    }
}

fn expect_record<'a>(
    records: &'a [RecordRef<'a>],
    cursor: usize,
    kind: RecordKind,
) -> Result<&'a RecordRef<'a>, StandardUniverseReplayV2Error> {
    records
        .get(cursor)
        .filter(|record| record.kind() == kind)
        .ok_or_else(|| layout(cursor))
}

fn checked_add(left: u32, right: u32) -> Result<u32, StandardUniverseReplayV2Error> {
    left.checked_add(right)
        .ok_or(StandardUniverseReplayV2Error::TooManyRecords)
}

fn layout(record_index: usize) -> StandardUniverseReplayV2Error {
    StandardUniverseReplayV2Error::InvalidRecordLayout {
        record_index: record_index as u32,
    }
}

#[derive(Debug)]
pub enum StandardUniverseReplayV2Error {
    Format(ReplayFormatError),
    Envelope(ReplayV2Error),
    Payload(ActivityCommandPayloadError),
    Legacy(StandardUniverseReplayError),
    NestedPayload(NestedBattlePayloadError),
    EventPayload(BattleEventPayloadError),
    Execution(NestedBattleExecutionError),
    ComponentIdentity(ComponentIdentityError),
    ComponentDivergence(Box<ConfigurationComponentDivergence>),
    CapturedBattleMismatch,
    TooManyRecords,
    CompatibilityMismatch,
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

impl From<ReplayFormatError> for StandardUniverseReplayV2Error {
    fn from(value: ReplayFormatError) -> Self {
        Self::Format(value)
    }
}
impl From<ReplayV2Error> for StandardUniverseReplayV2Error {
    fn from(value: ReplayV2Error) -> Self {
        Self::Envelope(value)
    }
}
impl From<ActivityCommandPayloadError> for StandardUniverseReplayV2Error {
    fn from(value: ActivityCommandPayloadError) -> Self {
        Self::Payload(value)
    }
}
impl From<StandardUniverseReplayError> for StandardUniverseReplayV2Error {
    fn from(value: StandardUniverseReplayError) -> Self {
        Self::Legacy(value)
    }
}
impl From<NestedBattlePayloadError> for StandardUniverseReplayV2Error {
    fn from(value: NestedBattlePayloadError) -> Self {
        Self::NestedPayload(value)
    }
}
impl From<BattleEventPayloadError> for StandardUniverseReplayV2Error {
    fn from(value: BattleEventPayloadError) -> Self {
        Self::EventPayload(value)
    }
}
impl From<NestedBattleExecutionError> for StandardUniverseReplayV2Error {
    fn from(value: NestedBattleExecutionError) -> Self {
        Self::Execution(value)
    }
}
impl From<ComponentIdentityError> for StandardUniverseReplayV2Error {
    fn from(value: ComponentIdentityError) -> Self {
        Self::ComponentIdentity(value)
    }
}

/// Convenience constructor for a zero-record production header template.
pub fn standard_universe_header_v2(
    compatibility: ReplayCompatibilityV2,
    components: ConfigurationComponentSet,
    master_seed: u64,
    activity: &StandardUniverseActivity,
    profile_id: &str,
) -> Result<ReplayHeaderV2, StandardUniverseReplayV2Error> {
    Ok(ReplayHeaderV2::new(
        compatibility,
        components,
        master_seed,
        replay_entry_for(activity, profile_id),
        0,
    )?)
}
