//! Canonical replay envelope for deterministic complete baseline runs.

use starclock_activity::{ActivityInstanceId, ActivityMasterSeed, ActivityTerminalOutcome};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::{
    component::{
        ConfigurationComponentDivergence, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::{DefinitionDigest, EntrySpecDigest, StateDigest},
    entry::ReplayEntry,
    format::{ReplayEnvironment, ReplayError, ReplayHeader, decode_replay, encode_replay},
    nested_battle::{
        NestedBattleCommandPayload, NestedBattlePayloadError, encode_nested_battle_command_payload,
        encode_nested_battle_state_payload,
    },
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

use super::baseline_runtime::completed_report;
use super::{
    DivergentUniverseBaselineError, DivergentUniverseBaselineFixture,
    DivergentUniverseBaselineFixtureError, DivergentUniverseBaselineReport,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep,
};

#[path = "baseline_replay_commands.rs"]
mod commands;
#[path = "baseline_replay_entry.rs"]
mod entry;
use entry::EntryInputs;

pub const DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE: &str =
    "divergent-universe-ordinary-real-battle-replay";
pub const DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE: &str =
    "divergent-universe-cyclical-real-battle-replay";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseRecordedRun {
    entry: EntryInputs,
    initial_state: [u8; 32],
    seed: u64,
    components: ConfigurationComponentSet,
    definition_id: u32,
    definition_digest: [u8; 32],
    config_digest: [u8; 32],
    report: DivergentUniverseBaselineReport,
}

impl DivergentUniverseRecordedRun {
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }
    #[must_use]
    pub const fn components(&self) -> &ConfigurationComponentSet {
        &self.components
    }
    #[must_use]
    pub const fn report(&self) -> &DivergentUniverseBaselineReport {
        &self.report
    }
    #[must_use]
    pub fn action_count(&self) -> usize {
        self.report.steps().len()
    }
    #[must_use]
    pub fn battle_command_count(&self) -> usize {
        self.report
            .steps()
            .iter()
            .filter_map(|step| match step {
                DivergentUniverseBaselineStep::Battle { execution, .. } => {
                    Some(execution.trace().len())
                }
                DivergentUniverseBaselineStep::ActivityDecision { .. } => None,
            })
            .sum()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DivergentUniverseReplayReport {
    run_family: DivergentUniverseRunFamily,
    tawot_forge_level: Option<u16>,
    action_count: u32,
    battle_count: u32,
    battle_command_count: u32,
    final_state_hash: StateDigest,
    terminal: ActivityTerminalOutcome,
}

impl DivergentUniverseReplayReport {
    /// Caller-selected entry configuration, checked against the rebuilt graph.
    #[must_use]
    pub const fn tawot_forge_level(self) -> Option<u16> {
        self.tawot_forge_level
    }
    #[must_use]
    pub const fn run_family(self) -> DivergentUniverseRunFamily {
        self.run_family
    }
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

pub fn record_divergent_universe_run(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    seed: u64,
) -> Result<DivergentUniverseRecordedRun, DivergentUniverseReplayError> {
    let (area, difficulty) = default_selection(family);
    record_divergent_universe_selected_run(fixture, family, area, difficulty, seed)
}

pub fn record_divergent_universe_selected_run(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    area: &str,
    difficulty: &str,
    seed: u64,
) -> Result<DivergentUniverseRecordedRun, DivergentUniverseReplayError> {
    let flow = fixture
        .flow_for_selection(family, area, difficulty)
        .map_err(DivergentUniverseReplayError::Fixture)?;
    record_flow(fixture, flow, seed)
}

/// Records the same baseline with an explicitly admitted, policy-bound service.
pub fn record_divergent_universe_run_with_tawot_service(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    seed: u64,
    forge_level: u16,
) -> Result<DivergentUniverseRecordedRun, DivergentUniverseReplayError> {
    let flow = fixture
        .flow_with_tawot_service(family, forge_level)
        .map_err(DivergentUniverseReplayError::Fixture)?;
    record_flow(fixture, flow, seed)
}

fn record_flow(
    fixture: &DivergentUniverseBaselineFixture,
    flow: super::DivergentUniverseFlowInstance,
    seed: u64,
) -> Result<DivergentUniverseRecordedRun, DivergentUniverseReplayError> {
    let identity = flow.definition().identity();
    let components = fixture
        .components(&flow)
        .map_err(DivergentUniverseReplayError::Fixture)?;
    let mut activity = flow
        .start(
            ActivityInstanceId::new(1).expect("fixed replay instance is non-zero"),
            ActivityMasterSeed::from_u64(seed),
        )
        .map_err(|_| DivergentUniverseReplayError::ActivityStart)?
        .into_activity();
    let initial_state = activity.state_hash().bytes();
    let report = DivergentUniverseBaselineRunner::default()
        .run_to_terminal(
            fixture.factory(),
            &flow,
            &mut activity,
            fixture.core(),
            &fixture
                .policy()
                .map_err(DivergentUniverseReplayError::Fixture)?,
        )
        .map_err(DivergentUniverseReplayError::Baseline)?;
    Ok(DivergentUniverseRecordedRun {
        entry: EntryInputs::from_flow(&flow),
        initial_state,
        seed,
        components,
        definition_id: identity.id().get(),
        definition_digest: identity.definition_digest().bytes(),
        config_digest: identity.config_digest().bytes(),
        report,
    })
}

/// Seals an externally driven terminal transcript using the same canonical
/// component and replay identity as the deterministic baseline runner.
pub fn record_divergent_universe_transcript(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &super::DivergentUniverseFlowInstance,
    activity: &starclock_activity::GraphActivity,
    seed: u64,
    steps: Vec<DivergentUniverseBaselineStep>,
) -> Result<DivergentUniverseRecordedRun, DivergentUniverseReplayError> {
    let identity = flow.definition().identity();
    let components = fixture
        .components(flow)
        .map_err(DivergentUniverseReplayError::Fixture)?;
    let report =
        completed_report(flow, activity, steps).map_err(DivergentUniverseReplayError::Baseline)?;
    Ok(DivergentUniverseRecordedRun {
        entry: EntryInputs::from_flow(flow),
        initial_state: flow
            .start(
                ActivityInstanceId::new(1).expect("fixed replay instance is nonzero"),
                ActivityMasterSeed::from_u64(seed),
            )
            .map_err(|_| DivergentUniverseReplayError::ActivityStart)?
            .into_activity()
            .state_hash()
            .bytes(),
        seed,
        components,
        definition_id: identity.id().get(),
        definition_digest: identity.definition_digest().bytes(),
        config_digest: identity.config_digest().bytes(),
        report,
    })
}

pub fn encode_divergent_universe_replay(
    recorded: &DivergentUniverseRecordedRun,
) -> Result<Vec<u8>, DivergentUniverseReplayError> {
    let mut payloads = vec![
        (RecordKind::AcceptedActivityCommand, recorded.entry.encode()),
        (
            RecordKind::ExpectedActivityState,
            recorded.initial_state.to_vec(),
        ),
    ];
    payloads.extend(replay_payloads(recorded.report().steps())?);
    let record_count =
        u32::try_from(payloads.len()).map_err(|_| DivergentUniverseReplayError::TooManyRecords)?;
    if record_count > MAX_REPLAY_RECORDS {
        return Err(DivergentUniverseReplayError::TooManyRecords);
    }
    let header = ReplayHeader::new(
        ReplayEnvironment::new("4.4").map_err(DivergentUniverseReplayError::Replay)?,
        recorded.components.clone(),
        recorded.seed,
        ReplayEntry::Activity {
            profile_id: profile(recorded.report.run_family()).into(),
            definition_id: recorded.definition_id,
            definition_digest: DefinitionDigest::new(recorded.definition_digest),
            spec_digest: EntrySpecDigest::new(recorded.config_digest),
            builds: None,
        },
        record_count,
    )
    .map_err(DivergentUniverseReplayError::Replay)?;
    let records = payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| RecordRef::new(*kind, index as u64, payload))
        .collect::<Result<Vec<_>, _>>()
        .map_err(DivergentUniverseReplayError::Format)?;
    encode_replay(&header, &records, Vec::new()).map_err(DivergentUniverseReplayError::Replay)
}

/// Reconstructs the exact fixture, flow, Activity and nested battle from the
/// replay seed. Recorded battle outcomes are never trusted or resubmitted.
pub fn verify_divergent_universe_replay(
    bytes: &[u8],
    fixture: &DivergentUniverseBaselineFixture,
) -> Result<DivergentUniverseReplayReport, DivergentUniverseReplayError> {
    let decoded = decode_replay(bytes).map_err(DivergentUniverseReplayError::Replay)?;
    let family = family(decoded.header().entry())?;
    let inputs = EntryInputs::decode(&decoded)?;
    verify_divergent_universe_selected_replay(
        bytes,
        fixture,
        family,
        &inputs.area,
        &inputs.difficulty,
    )
}

/// Reconstructs one explicit legal area/difficulty selection from fresh
/// production inputs. The caller-owned selection is checked against the replay
/// family and canonical entry/component identities before records are compared.
pub fn verify_divergent_universe_selected_replay(
    bytes: &[u8],
    fixture: &DivergentUniverseBaselineFixture,
    selected_family: DivergentUniverseRunFamily,
    area: &str,
    difficulty: &str,
) -> Result<DivergentUniverseReplayReport, DivergentUniverseReplayError> {
    let decoded = decode_replay(bytes).map_err(DivergentUniverseReplayError::Replay)?;
    let family = family(decoded.header().entry())?;
    if family != selected_family {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::Activity,
            0,
        ));
    }
    let inputs = EntryInputs::decode(&decoded)?;
    if inputs.area.as_ref() != area || inputs.difficulty.as_ref() != difficulty {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::Activity,
            0,
        ));
    }
    let flow = fixture
        .flow_for_entry_configuration(
            family,
            area,
            difficulty,
            inputs.tawot,
            inputs.source_deck_selection,
        )
        .map_err(DivergentUniverseReplayError::Fixture)?;
    let components = fixture
        .components(&flow)
        .map_err(DivergentUniverseReplayError::Fixture)?;
    decoded
        .header()
        .components()
        .verify_exact(&components)
        .map_err(component_divergence)?;
    validate_entry(decoded.header().entry(), &flow)?;
    let actual = commands::reconstruct(&decoded, fixture, &flow)?;
    let reconstructed = encode_divergent_universe_replay(&actual)?;
    compare_records(
        &decoded,
        &decode_replay(&reconstructed).map_err(DivergentUniverseReplayError::Replay)?,
    )?;
    Ok(DivergentUniverseReplayReport {
        run_family: family,
        tawot_forge_level: inputs.tawot,
        action_count: u32::try_from(actual.action_count())
            .map_err(|_| DivergentUniverseReplayError::TooManyRecords)?,
        battle_count: actual.report.completed_battles(),
        battle_command_count: u32::try_from(actual.battle_command_count())
            .map_err(|_| DivergentUniverseReplayError::TooManyRecords)?,
        final_state_hash: StateDigest::new(actual.report.final_state_hash().bytes()),
        terminal: actual.report.terminal(),
    })
}

const fn default_selection(family: DivergentUniverseRunFamily) -> (&'static str, &'static str) {
    match family {
        DivergentUniverseRunFamily::Ordinary => (
            "divergent-universe.area.401",
            "divergent-universe.difficulty.3011",
        ),
        DivergentUniverseRunFamily::Cyclical => (
            "divergent-universe.area.20401",
            "divergent-universe.difficulty.3011",
        ),
    }
}

fn replay_payloads(
    steps: &[DivergentUniverseBaselineStep],
) -> Result<Vec<(RecordKind, Vec<u8>)>, DivergentUniverseReplayError> {
    let mut payloads = Vec::new();
    for step in steps {
        let decision = match step {
            DivergentUniverseBaselineStep::ActivityDecision { decision, .. }
            | DivergentUniverseBaselineStep::Battle { decision, .. } => decision,
        };
        payloads.push((
            RecordKind::AcceptedActivityCommand,
            decision_payload(decision),
        ));
        if let DivergentUniverseBaselineStep::Battle {
            identity,
            result_digest,
            execution,
            ..
        } = step
        {
            payloads.push((
                RecordKind::NestedBattleStart,
                starclock_replay::activity::encode_nested_battle_start_payload(**identity),
            ));
            for nested in execution.trace() {
                payloads.push((
                    RecordKind::AcceptedBattleCommand,
                    encode_nested_battle_command_payload(&NestedBattleCommandPayload::new(
                        nested.controller() as u8,
                        nested.command().clone(),
                    ))
                    .map_err(DivergentUniverseReplayError::NestedPayload)?,
                ));
                payloads.push((
                    RecordKind::ExpectedBattleState,
                    encode_nested_battle_state_payload(nested.state_hash(), nested.events())
                        .map_err(DivergentUniverseReplayError::NestedPayload)?,
                ));
            }
            payloads.push((
                RecordKind::NestedBattleEnd,
                starclock_replay::activity::encode_nested_battle_end_payload(*result_digest),
            ));
        }
        payloads.push((
            RecordKind::ExpectedActivityState,
            step.state_hash().bytes().to_vec(),
        ));
    }
    Ok(payloads)
}

fn decision_payload(decision: &crate::baseline_controller::ActivityBaselineDecision) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"DUA1");
    payload.push(decision_kind(decision.kind()));
    payload.extend_from_slice(&decision.decision().get().to_le_bytes());
    payload.extend_from_slice(&decision.option().get().to_le_bytes());
    payload.extend_from_slice(
        &u32::try_from(decision.scores().len())
            .expect("Activity offer is bounded")
            .to_le_bytes(),
    );
    for score in decision.scores() {
        payload.extend_from_slice(&score.option().get().to_le_bytes());
        payload.extend_from_slice(&score.authored_priority().to_le_bytes());
        payload.extend_from_slice(&score.hint_total().to_le_bytes());
        payload.extend_from_slice(&score.total().to_le_bytes());
    }
    payload
}

const fn decision_kind(kind: starclock_activity::ActivityDecisionKind) -> u8 {
    match kind {
        starclock_activity::ActivityDecisionKind::Choice => 0,
        starclock_activity::ActivityDecisionKind::Route => 1,
        starclock_activity::ActivityDecisionKind::Encounter => 2,
        starclock_activity::ActivityDecisionKind::Preparation => 3,
        starclock_activity::ActivityDecisionKind::Reward => 4,
        starclock_activity::ActivityDecisionKind::Shop => 5,
        starclock_activity::ActivityDecisionKind::Service => 6,
        starclock_activity::ActivityDecisionKind::Roster => 7,
        starclock_activity::ActivityDecisionKind::ExternalOutcome => 8,
        starclock_activity::ActivityDecisionKind::BattleReady => 9,
        starclock_activity::ActivityDecisionKind::Checkpoint => 10,
        starclock_activity::ActivityDecisionKind::Abandon => 11,
    }
}

fn profile(family: DivergentUniverseRunFamily) -> &'static str {
    match family {
        DivergentUniverseRunFamily::Ordinary => DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE,
        DivergentUniverseRunFamily::Cyclical => DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE,
    }
}

fn family(entry: &ReplayEntry) -> Result<DivergentUniverseRunFamily, DivergentUniverseReplayError> {
    let ReplayEntry::Activity { profile_id, .. } = entry else {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::Activity,
            0,
        ));
    };
    match profile_id.as_ref() {
        DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE => Ok(DivergentUniverseRunFamily::Ordinary),
        DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE => Ok(DivergentUniverseRunFamily::Cyclical),
        _ => Err(divergence(
            DivergentUniverseReplayDivergenceKind::Activity,
            0,
        )),
    }
}

fn validate_entry(
    entry: &ReplayEntry,
    flow: &super::DivergentUniverseFlowInstance,
) -> Result<(), DivergentUniverseReplayError> {
    let identity = flow.definition().identity();
    let expected = ReplayEntry::Activity {
        profile_id: profile(flow.run_family()).into(),
        definition_id: identity.id().get(),
        definition_digest: DefinitionDigest::new(identity.definition_digest().bytes()),
        spec_digest: EntrySpecDigest::new(identity.config_digest().bytes()),
        builds: None,
    };
    let ReplayEntry::Activity {
        profile_id,
        definition_id,
        definition_digest,
        spec_digest,
        builds,
    } = entry
    else {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::Activity,
            0,
        ));
    };
    let ReplayEntry::Activity {
        profile_id: expected_profile,
        definition_id: expected_id,
        definition_digest: expected_definition,
        spec_digest: expected_spec,
        builds: expected_builds,
    } = expected
    else {
        unreachable!("expected Divergent Universe replay entry is Activity")
    };
    if profile_id != &expected_profile
        || definition_id != &expected_id
        || definition_digest != &expected_definition
        || builds != &expected_builds
    {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::Activity,
            0,
        ));
    }
    if spec_digest != &expected_spec {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::Mapping,
            0,
        ));
    }
    Ok(())
}

fn compare_records(
    expected: &starclock_replay::format::DecodedReplay<'_>,
    actual: &starclock_replay::format::DecodedReplay<'_>,
) -> Result<(), DivergentUniverseReplayError> {
    let shared = expected.records().len().min(actual.records().len());
    for index in 0..shared {
        let left = &expected.records()[index];
        let right = &actual.records()[index];
        if left.kind() != right.kind() || left.payload() != right.payload() {
            return Err(divergence(
                record_divergence_kind(left.kind(), left.payload(), right.payload()),
                u32::try_from(index).unwrap_or(u32::MAX),
            ));
        }
    }
    if expected.records().len() != actual.records().len() {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::RecordLayout,
            u32::try_from(shared).unwrap_or(u32::MAX),
        ));
    }
    Ok(())
}

fn record_divergence_kind(
    kind: RecordKind,
    expected: &[u8],
    actual: &[u8],
) -> DivergentUniverseReplayDivergenceKind {
    match kind {
        RecordKind::AcceptedActivityCommand => {
            DivergentUniverseReplayDivergenceKind::ActivityCommand
        }
        RecordKind::NestedBattleStart => nested_start_divergence(expected, actual),
        RecordKind::AcceptedBattleCommand => DivergentUniverseReplayDivergenceKind::BattleCommand,
        RecordKind::ExpectedBattleState => {
            let expected =
                starclock_replay::nested_battle::decode_nested_battle_state_payload(expected);
            let actual =
                starclock_replay::nested_battle::decode_nested_battle_state_payload(actual);
            match (expected, actual) {
                (Ok(left), Ok(right)) if left.state_hash() == right.state_hash() => {
                    DivergentUniverseReplayDivergenceKind::BattleEvent
                }
                _ => DivergentUniverseReplayDivergenceKind::BattleState,
            }
        }
        RecordKind::NestedBattleEnd => DivergentUniverseReplayDivergenceKind::Settlement,
        RecordKind::ExpectedActivityState => DivergentUniverseReplayDivergenceKind::ActivityState,
        _ => DivergentUniverseReplayDivergenceKind::RecordLayout,
    }
}

fn nested_start_divergence(
    expected: &[u8],
    actual: &[u8],
) -> DivergentUniverseReplayDivergenceKind {
    let expected = starclock_replay::activity::decode_nested_battle_start_payload(expected);
    let actual = starclock_replay::activity::decode_nested_battle_start_payload(actual);
    match (expected, actual) {
        (Ok(left), Ok(right))
            if left.participant_lock_digest() != right.participant_lock_digest() =>
        {
            DivergentUniverseReplayDivergenceKind::Mapping
        }
        (Ok(left), Ok(right)) if left.combat_input_digest() != right.combat_input_digest() => {
            DivergentUniverseReplayDivergenceKind::ContributionSnapshot
        }
        _ => DivergentUniverseReplayDivergenceKind::BattleAssembly,
    }
}

fn component_divergence(
    value: Box<ConfigurationComponentDivergence>,
) -> DivergentUniverseReplayError {
    let kind = value.expected.as_ref().or(value.actual.as_ref()).map_or(
        DivergentUniverseReplayDivergenceKind::Catalog,
        |component| match component.kind() {
            ConfigurationComponentKind::CombatCatalog
            | ConfigurationComponentKind::BuildCatalog
            | ConfigurationComponentKind::ModeContent
            | ConfigurationComponentKind::CombatRuleRegistry => {
                DivergentUniverseReplayDivergenceKind::Catalog
            }
            ConfigurationComponentKind::ModeProfile => {
                DivergentUniverseReplayDivergenceKind::Mapping
            }
            ConfigurationComponentKind::EncounterOverlay => {
                DivergentUniverseReplayDivergenceKind::BattleAssembly
            }
            ConfigurationComponentKind::Controller => {
                DivergentUniverseReplayDivergenceKind::ActivityCommand
            }
            ConfigurationComponentKind::ActivityCore
            | ConfigurationComponentKind::ActivityHandlerRegistry => {
                DivergentUniverseReplayDivergenceKind::Activity
            }
        },
    );
    DivergentUniverseReplayError::FirstDivergence {
        kind,
        record_index: 0,
        component: Some(value),
    }
}

const fn divergence(
    kind: DivergentUniverseReplayDivergenceKind,
    record_index: u32,
) -> DivergentUniverseReplayError {
    DivergentUniverseReplayError::FirstDivergence {
        kind,
        record_index,
        component: None,
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseReplayDivergenceKind {
    Catalog,
    Activity,
    Mapping,
    ContributionSnapshot,
    BattleAssembly,
    ActivityCommand,
    BattleCommand,
    BattleEvent,
    BattleState,
    Settlement,
    ActivityState,
    RecordLayout,
}

#[derive(Debug)]
pub enum DivergentUniverseReplayError {
    Fixture(DivergentUniverseBaselineFixtureError),
    ActivityStart,
    Baseline(DivergentUniverseBaselineError),
    Replay(ReplayError),
    Format(ReplayFormatError),
    NestedPayload(NestedBattlePayloadError),
    TooManyRecords,
    FirstDivergence {
        kind: DivergentUniverseReplayDivergenceKind,
        record_index: u32,
        component: Option<Box<ConfigurationComponentDivergence>>,
    },
}

impl DivergentUniverseReplayError {
    #[must_use]
    pub const fn first_divergence(&self) -> Option<DivergentUniverseReplayDivergenceKind> {
        match self {
            Self::FirstDivergence { kind, .. } => Some(*kind),
            _ => None,
        }
    }

    #[must_use]
    pub const fn record_index(&self) -> Option<u32> {
        match self {
            Self::FirstDivergence { record_index, .. } => Some(*record_index),
            _ => None,
        }
    }

    #[must_use]
    pub fn component_divergence(&self) -> Option<&ConfigurationComponentDivergence> {
        match self {
            Self::FirstDivergence { component, .. } => component.as_deref(),
            _ => None,
        }
    }
}
