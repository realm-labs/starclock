use core::fmt;

use starclock_replay::{
    battle::encode_battle_command_payload,
    battle_event::encode_battle_event_payload,
    codec::{CanonicalSink, Encoder},
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::{ComponentDigest, DefinitionDigest, EntrySpecDigest, Sha256Sink},
    entry::ReplayEntry,
    format::{DecodedReplay, ReplayEnvironment, ReplayHeader, decode_replay, encode_replay},
    record::{RecordKind, RecordRef},
};

use super::{
    CurrencyWarsBaselineBattleReport, CurrencyWarsBaselineController, CurrencyWarsBaselineRunReport,
};

const ENVIRONMENT: &str = "4.4";
const PROFILE_PREFIX: &str = "currency-wars/route-";
const RECORD_VERSION: u8 = 2;
const DEFINITION_ID: u32 = 31;
const INITIAL_ROLES: [u32; 4] = [1301, 1306, 1014, 1015];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurrencyWarsReplayGambit {
    Standard = 1,
    Overclock = 2,
}

impl CurrencyWarsReplayGambit {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Overclock => "overclock",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsReplayRequest {
    route_id: u32,
    difficulty_id: u32,
    gambit: CurrencyWarsReplayGambit,
    seed: u64,
}

impl CurrencyWarsReplayRequest {
    #[must_use]
    pub const fn new(
        route_id: u32,
        difficulty_id: u32,
        gambit: CurrencyWarsReplayGambit,
        seed: u64,
    ) -> Self {
        Self {
            route_id,
            difficulty_id,
            gambit,
            seed,
        }
    }

    #[must_use]
    pub const fn route_id(self) -> u32 {
        self.route_id
    }
    #[must_use]
    pub const fn difficulty_id(self) -> u32 {
        self.difficulty_id
    }
    #[must_use]
    pub const fn gambit(self) -> CurrencyWarsReplayGambit {
        self.gambit
    }
    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsReplayIdentity {
    schema_digest: [u8; 32],
    configuration_digest: [u8; 32],
    content_digest: [u8; 32],
    battle_resources_digest: [u8; 32],
    base_combat_catalog_digest: [u8; 32],
}

impl CurrencyWarsReplayIdentity {
    #[must_use]
    pub const fn new(
        schema_digest: [u8; 32],
        configuration_digest: [u8; 32],
        content_digest: [u8; 32],
        battle_resources_digest: [u8; 32],
        base_combat_catalog_digest: [u8; 32],
    ) -> Self {
        Self {
            schema_digest,
            configuration_digest,
            content_digest,
            battle_resources_digest,
            base_combat_catalog_digest,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyWarsReplayDivergenceKind {
    Catalog,
    Activity,
    BattleAssembly,
    BattleCommand,
    Settlement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsReplayDivergence {
    kind: CurrencyWarsReplayDivergenceKind,
    sequence: Option<u64>,
    battle_index: Option<u32>,
    battle_command_index: Option<u32>,
}

impl CurrencyWarsReplayDivergence {
    #[must_use]
    pub const fn kind(self) -> CurrencyWarsReplayDivergenceKind {
        self.kind
    }
    #[must_use]
    pub const fn sequence(self) -> Option<u64> {
        self.sequence
    }
    #[must_use]
    pub const fn battle_index(self) -> Option<u32> {
        self.battle_index
    }
    #[must_use]
    pub const fn battle_command_index(self) -> Option<u32> {
        self.battle_command_index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsReplayError {
    InvalidReplay(Box<str>),
    Diverged(CurrencyWarsReplayDivergence),
}

impl fmt::Display for CurrencyWarsReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReplay(message) => {
                write!(formatter, "invalid Currency Wars replay: {message}")
            }
            Self::Diverged(divergence) => write!(
                formatter,
                "Currency Wars replay first diverged at {:?} (record={:?}, battle={:?}, command={:?})",
                divergence.kind,
                divergence.sequence,
                divergence.battle_index,
                divergence.battle_command_index,
            ),
        }
    }
}

impl std::error::Error for CurrencyWarsReplayError {}

pub fn encode_currency_wars_replay(
    request: CurrencyWarsReplayRequest,
    identity: CurrencyWarsReplayIdentity,
    report: &CurrencyWarsBaselineRunReport,
) -> Result<Vec<u8>, CurrencyWarsReplayError> {
    let payloads = records(report)?;
    let header = ReplayHeader::new(
        ReplayEnvironment::new(ENVIRONMENT).map_err(invalid)?,
        components(request, identity, report)?,
        request.seed,
        ReplayEntry::Activity {
            profile_id: profile_id(request).into_boxed_str(),
            definition_id: DEFINITION_ID,
            definition_digest: DefinitionDigest::new(identity.content_digest),
            spec_digest: entry_spec_digest(request, identity),
            builds: None,
        },
        u32::try_from(payloads.len()).map_err(invalid)?,
    )
    .map_err(invalid)?;
    let records = payloads
        .iter()
        .enumerate()
        .map(|(sequence, (kind, payload))| {
            RecordRef::new(*kind, u64::try_from(sequence).map_err(invalid)?, payload)
                .map_err(invalid)
        })
        .collect::<Result<Vec<_>, _>>()?;
    encode_replay(&header, &records, Vec::new()).map_err(invalid)
}

pub fn decode_currency_wars_replay_request(
    bytes: &[u8],
) -> Result<CurrencyWarsReplayRequest, CurrencyWarsReplayError> {
    let decoded = decode_replay(bytes).map_err(invalid)?;
    parse_request(decoded.header())
}

pub fn verify_currency_wars_replay(
    bytes: &[u8],
    request: CurrencyWarsReplayRequest,
    identity: CurrencyWarsReplayIdentity,
    report: &CurrencyWarsBaselineRunReport,
) -> Result<(), CurrencyWarsReplayError> {
    let actual = decode_replay(bytes).map_err(invalid)?;
    let expected_bytes = encode_currency_wars_replay(request, identity, report)?;
    let expected = decode_replay(&expected_bytes).map_err(invalid)?;
    if actual.header().environment() != expected.header().environment()
        || actual.header().master_seed() != expected.header().master_seed()
        || actual.header().entry() != expected.header().entry()
    {
        return Err(diverged(
            CurrencyWarsReplayDivergenceKind::Activity,
            None,
            None,
            None,
        ));
    }
    if let Err(mismatch) = expected
        .header()
        .components()
        .verify_exact(actual.header().components())
    {
        let component = mismatch
            .expected
            .as_ref()
            .or(mismatch.actual.as_ref())
            .map(ConfigurationComponentIdentity::kind);
        return Err(diverged(
            component.map_or(
                CurrencyWarsReplayDivergenceKind::Catalog,
                component_divergence_kind,
            ),
            None,
            None,
            None,
        ));
    }
    compare_records(&expected, &actual)
}

fn compare_records(
    expected: &DecodedReplay<'_>,
    actual: &DecodedReplay<'_>,
) -> Result<(), CurrencyWarsReplayError> {
    let shared = expected.records().len().min(actual.records().len());
    let mut battle_index = None;
    let mut battle_command_index = None;
    for index in 0..shared {
        let expected_record = expected.records()[index];
        if expected_record.kind() == RecordKind::NestedBattleStart {
            battle_index = payload_index(expected_record.payload());
            battle_command_index = Some(0);
        }
        if expected_record != actual.records()[index] {
            return Err(diverged(
                record_divergence_kind(expected_record.kind()),
                Some(expected_record.sequence()),
                battle_index,
                battle_command_index,
            ));
        }
        if expected_record.kind() == RecordKind::ExpectedBattleState {
            battle_command_index = battle_command_index.and_then(|value| value.checked_add(1));
        }
        if expected_record.kind() == RecordKind::NestedBattleEnd {
            battle_index = None;
            battle_command_index = None;
        }
    }
    if expected.records().len() != actual.records().len() {
        let record = expected
            .records()
            .get(shared)
            .or(actual.records().get(shared));
        return Err(diverged(
            record.map_or(CurrencyWarsReplayDivergenceKind::Settlement, |value| {
                record_divergence_kind(value.kind())
            }),
            record.map(|value| value.sequence()),
            battle_index,
            battle_command_index,
        ));
    }
    Ok(())
}

fn records(
    report: &CurrencyWarsBaselineRunReport,
) -> Result<Vec<(RecordKind, Vec<u8>)>, CurrencyWarsReplayError> {
    let mut records = Vec::new();
    for activity in report.activity_trace() {
        if let Some(index) = activity.battle_index() {
            let battle = report
                .battles()
                .get(
                    usize::try_from(
                        index
                            .checked_sub(1)
                            .ok_or_else(|| invalid("zero battle index"))?,
                    )
                    .map_err(invalid)?,
                )
                .ok_or_else(|| invalid("battle trace index is out of range"))?;
            records.push((
                RecordKind::NestedBattleStart,
                encode_battle_start(index, battle),
            ));
            encode_battle_records(battle, &mut records)?;
            records.push((
                RecordKind::NestedBattleEnd,
                encode_battle_end(index, battle),
            ));
        }
        let mut command = Encoder::new(Vec::new());
        command.u8(RECORD_VERSION);
        command.u8(activity.action() as u8);
        command.u32(activity.battle_index().unwrap_or(0));
        records.push((RecordKind::AcceptedActivityCommand, command.into_inner()));
        let mut state = Encoder::new(Vec::new());
        state.u8(RECORD_VERSION);
        state.raw(&activity.state_hash().bytes());
        records.push((RecordKind::ExpectedActivityState, state.into_inner()));
    }
    let mut summary = Encoder::new(Vec::new());
    summary.u8(RECORD_VERSION);
    summary.u8(terminal_tag(report.terminal()));
    summary.u32(report.activity_steps());
    summary.u32(report.supply_decisions());
    summary.u32(report.route_decisions());
    summary.u32(u32::try_from(report.battles().len()).map_err(invalid)?);
    summary.raw(&report.final_state_hash().bytes());
    records.push((RecordKind::ControllerDiagnostic, summary.into_inner()));
    Ok(records)
}

fn encode_battle_records(
    battle: &CurrencyWarsBaselineBattleReport,
    records: &mut Vec<(RecordKind, Vec<u8>)>,
) -> Result<(), CurrencyWarsReplayError> {
    for entry in battle.trace() {
        let encoded = encode_battle_command_payload(entry.command()).map_err(invalid)?;
        let mut command = Encoder::new(Vec::new());
        command.u8(RECORD_VERSION);
        command.u8(entry.controller() as u8);
        command.bytes(&encoded).map_err(invalid)?;
        records.push((RecordKind::AcceptedBattleCommand, command.into_inner()));
        let mut state = Encoder::new(Vec::new());
        state.u8(RECORD_VERSION);
        state.raw(&entry.state_hash().bytes());
        state.u32(u32::try_from(entry.events().len()).map_err(invalid)?);
        for event in entry.events() {
            state
                .bytes(&encode_battle_event_payload(event).map_err(invalid)?)
                .map_err(invalid)?;
        }
        records.push((RecordKind::ExpectedBattleState, state.into_inner()));
    }
    Ok(())
}

fn encode_battle_start(index: u32, battle: &CurrencyWarsBaselineBattleReport) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u8(RECORD_VERSION);
    encoder.u32(index);
    encoder.raw(&battle.catalog_digest());
    encoder.raw(&battle.combat_input_digest());
    encoder.raw(&battle.assembly_digest());
    encoder.into_inner()
}

fn encode_battle_end(index: u32, battle: &CurrencyWarsBaselineBattleReport) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u8(RECORD_VERSION);
    encoder.u32(index);
    encoder.u8(outcome_tag(battle.outcome()));
    encoder.raw(&battle.final_state_hash().bytes());
    encoder.raw(&battle.event_digest().bytes());
    encoder.i64(battle.progress().scaled());
    encoder.i64(battle.remaining_action_value().scaled());
    encoder.into_inner()
}

fn components(
    request: CurrencyWarsReplayRequest,
    identity: CurrencyWarsReplayIdentity,
    report: &CurrencyWarsBaselineRunReport,
) -> Result<ConfigurationComponentSet, CurrencyWarsReplayError> {
    let spec = entry_spec_digest(request, identity).bytes();
    ConfigurationComponentSet::new(vec![
        component(
            ConfigurationComponentKind::CombatCatalog,
            "combat-catalog",
            aggregate_battle_digest(
                b"catalog",
                report,
                CurrencyWarsBaselineBattleReport::catalog_digest,
            ),
        )?,
        component(
            ConfigurationComponentKind::BuildCatalog,
            "build-catalog",
            aggregate_battle_digest(
                b"build",
                report,
                CurrencyWarsBaselineBattleReport::combat_input_digest,
            ),
        )?,
        component(
            ConfigurationComponentKind::ActivityCore,
            "currency-wars-activity",
            spec,
        )?,
        component(
            ConfigurationComponentKind::ModeProfile,
            "currency-wars-profile",
            profile_digest(request, identity),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "currency-wars-content",
            identity.configuration_digest,
        )?,
        component(
            ConfigurationComponentKind::ActivityHandlerRegistry,
            "currency-wars-activity-handlers",
            fixed_digest(b"starclock.currency-wars.activity-handlers.empty.v1"),
        )?,
        component(
            ConfigurationComponentKind::CombatRuleRegistry,
            "currency-wars-combat-rules",
            identity.base_combat_catalog_digest,
        )?,
        component(
            ConfigurationComponentKind::EncounterOverlay,
            "currency-wars-encounter-overlay",
            identity.battle_resources_digest,
        )?,
        component(
            ConfigurationComponentKind::Controller,
            "currency-wars-baseline-controller",
            CurrencyWarsBaselineController::identity_digest(),
        )?,
    ])
    .map_err(invalid)
}

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    digest: [u8; 32],
) -> Result<ConfigurationComponentIdentity, CurrencyWarsReplayError> {
    ConfigurationComponentIdentity::new(kind, id, ComponentDigest::new(digest)).map_err(invalid)
}

fn aggregate_battle_digest(
    domain: &[u8],
    report: &CurrencyWarsBaselineRunReport,
    digest: fn(&CurrencyWarsBaselineBattleReport) -> [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256Sink::new();
    hash.write(b"starclock.currency-wars.replay-component.v1");
    hash.write(domain);
    hash.write(
        &u32::try_from(report.battles().len())
            .unwrap_or(u32::MAX)
            .to_le_bytes(),
    );
    for battle in report.battles() {
        hash.write(&digest(battle));
    }
    hash.finalize().bytes()
}

fn profile_digest(
    request: CurrencyWarsReplayRequest,
    identity: CurrencyWarsReplayIdentity,
) -> [u8; 32] {
    let mut hash = Sha256Sink::new();
    hash.write(b"starclock.currency-wars.profile.v1");
    hash.write(&identity.schema_digest);
    hash.write(&identity.content_digest);
    hash.write(&request.route_id.to_le_bytes());
    hash.write(&request.difficulty_id.to_le_bytes());
    hash.write(&[request.gambit as u8]);
    hash.finalize().bytes()
}

fn fixed_digest(domain: &[u8]) -> [u8; 32] {
    let mut hash = Sha256Sink::new();
    hash.write(domain);
    hash.finalize().bytes()
}

fn entry_spec_digest(
    request: CurrencyWarsReplayRequest,
    identity: CurrencyWarsReplayIdentity,
) -> EntrySpecDigest {
    let mut hash = Sha256Sink::new();
    hash.write(b"starclock.currency-wars.replay-entry.v2");
    hash.write(&identity.schema_digest);
    hash.write(&request.route_id.to_le_bytes());
    hash.write(&request.difficulty_id.to_le_bytes());
    hash.write(&[request.gambit as u8]);
    for role in INITIAL_ROLES {
        hash.write(&role.to_le_bytes());
    }
    EntrySpecDigest::new(hash.finalize().bytes())
}

fn profile_id(request: CurrencyWarsReplayRequest) -> String {
    format!(
        "{PROFILE_PREFIX}{}/difficulty-{}/gambit-{}",
        request.route_id,
        request.difficulty_id,
        request.gambit.name()
    )
}

fn parse_request(
    header: &ReplayHeader,
) -> Result<CurrencyWarsReplayRequest, CurrencyWarsReplayError> {
    if header.environment().game_version() != ENVIRONMENT {
        return Err(invalid("unsupported environment"));
    }
    let ReplayEntry::Activity {
        profile_id,
        definition_id,
        ..
    } = header.entry()
    else {
        return Err(invalid("entry is not an Activity"));
    };
    if *definition_id != DEFINITION_ID {
        return Err(invalid("definition identity is invalid"));
    }
    let suffix = profile_id
        .strip_prefix(PROFILE_PREFIX)
        .ok_or_else(|| invalid("profile is invalid"))?;
    let (route, suffix) = suffix
        .split_once("/difficulty-")
        .ok_or_else(|| invalid("route is invalid"))?;
    let (difficulty, gambit) = suffix
        .split_once("/gambit-")
        .ok_or_else(|| invalid("difficulty is invalid"))?;
    let gambit = match gambit {
        "standard" => CurrencyWarsReplayGambit::Standard,
        "overclock" => CurrencyWarsReplayGambit::Overclock,
        _ => return Err(invalid("Gambit is invalid")),
    };
    Ok(CurrencyWarsReplayRequest::new(
        route.parse().map_err(invalid)?,
        difficulty.parse().map_err(invalid)?,
        gambit,
        header.master_seed(),
    ))
}

fn payload_index(payload: &[u8]) -> Option<u32> {
    let bytes: [u8; 4] = payload.get(1..5)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

const fn component_divergence_kind(
    kind: ConfigurationComponentKind,
) -> CurrencyWarsReplayDivergenceKind {
    match kind {
        ConfigurationComponentKind::EncounterOverlay => {
            CurrencyWarsReplayDivergenceKind::BattleAssembly
        }
        ConfigurationComponentKind::ActivityCore
        | ConfigurationComponentKind::ModeProfile
        | ConfigurationComponentKind::ActivityHandlerRegistry
        | ConfigurationComponentKind::Controller => CurrencyWarsReplayDivergenceKind::Activity,
        ConfigurationComponentKind::CombatCatalog
        | ConfigurationComponentKind::BuildCatalog
        | ConfigurationComponentKind::ModeContent
        | ConfigurationComponentKind::CombatRuleRegistry => {
            CurrencyWarsReplayDivergenceKind::Catalog
        }
    }
}

const fn record_divergence_kind(kind: RecordKind) -> CurrencyWarsReplayDivergenceKind {
    match kind {
        RecordKind::NestedBattleStart => CurrencyWarsReplayDivergenceKind::BattleAssembly,
        RecordKind::AcceptedBattleCommand | RecordKind::ExpectedBattleState => {
            CurrencyWarsReplayDivergenceKind::BattleCommand
        }
        RecordKind::NestedBattleEnd | RecordKind::ControllerDiagnostic => {
            CurrencyWarsReplayDivergenceKind::Settlement
        }
        RecordKind::AcceptedActivityCommand | RecordKind::ExpectedActivityState => {
            CurrencyWarsReplayDivergenceKind::Activity
        }
    }
}

fn diverged(
    kind: CurrencyWarsReplayDivergenceKind,
    sequence: Option<u64>,
    battle_index: Option<u32>,
    battle_command_index: Option<u32>,
) -> CurrencyWarsReplayError {
    CurrencyWarsReplayError::Diverged(CurrencyWarsReplayDivergence {
        kind,
        sequence,
        battle_index,
        battle_command_index,
    })
}

fn invalid(error: impl fmt::Debug) -> CurrencyWarsReplayError {
    CurrencyWarsReplayError::InvalidReplay(format!("{error:?}").into_boxed_str())
}

const fn terminal_tag(terminal: starclock_activity::ActivityTerminalOutcome) -> u8 {
    match terminal {
        starclock_activity::ActivityTerminalOutcome::Completed => 1,
        starclock_activity::ActivityTerminalOutcome::Failed => 2,
        starclock_activity::ActivityTerminalOutcome::Abandoned => 3,
        starclock_activity::ActivityTerminalOutcome::Faulted => 4,
    }
}

const fn outcome_tag(outcome: starclock_activity::BattleOutcome) -> u8 {
    match outcome {
        starclock_activity::BattleOutcome::Won => 1,
        starclock_activity::BattleOutcome::Lost => 2,
        starclock_activity::BattleOutcome::Faulted => 3,
        starclock_activity::BattleOutcome::Finalized => 4,
    }
}
