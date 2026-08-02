use std::{fmt, fs, path::PathBuf};

use starclock_activity::{ActivityInstanceId, ActivityTerminalOutcome};
use starclock_mode_universe::{
    gold_gears_catalog::validate_gold_and_gears_bundle,
    gold_gears_entry::{
        GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION, GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION,
        GoldAndGearsRuntimeFactory, GoldAndGearsSeededRunRequest,
        baseline_fixture::{GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY, GoldAndGearsBaselineFixture},
        encode_gold_and_gears_replay, gold_and_gears_replay_header, record_gold_and_gears_run,
        verify_gold_and_gears_replay,
    },
    gold_gears_identity::GoldAndGearsCatalogIdentity,
};
use starclock_replay::{
    codec::CanonicalSink, current::decode_replay, digest::Sha256Sink, format::ReplayEntry,
};

const GOLD_BUNDLE: &[u8] = include_bytes!("../../../config/gold-and-gears-generated/config.sora");
const CLI_REVISION: &str = "starclock-cli-gold-and-gears-v1";
const MODE: &str = "gold-and-gears";

pub fn requested(args: &[String]) -> bool {
    args.windows(2)
        .any(|pair| pair[0] == "--mode" && pair[1] == MODE)
}

pub fn config_validate(args: &[String]) -> Result<(), GoldAndGearsCliError> {
    let json = mode_json_only(args)?;
    let summary = validate_gold_and_gears_bundle(GOLD_BUNDLE)
        .map_err(|_| GoldAndGearsCliError::Configuration)?;
    GoldAndGearsRuntimeFactory::load_candidate(GOLD_BUNDLE)
        .map_err(|_| GoldAndGearsCliError::Configuration)?;
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-config-validation\",\"mode\":\"{MODE}\",\"valid\":true,\"bundle_sha256\":\"{}\",\"tables\":{},\"rows\":{},\"source_obligations\":{},\"mechanic_rules\":{},\"fixtures\":{},\"policy_boundaries\":{}}}",
            hex(summary.bundle_digest()),
            summary.table_count(),
            summary.row_count(),
            summary.source_obligations(),
            summary.mechanic_rules(),
            summary.semantic_fixtures(),
            summary.policy_boundaries(),
        );
    } else {
        println!(
            "universe config valid mode={MODE} bundle_sha256={} tables={} rows={} source_obligations={} rules={} fixtures={} policies={}",
            hex(summary.bundle_digest()),
            summary.table_count(),
            summary.row_count(),
            summary.source_obligations(),
            summary.mechanic_rules(),
            summary.semantic_fixtures(),
            summary.policy_boundaries(),
        );
    }
    Ok(())
}

pub fn coverage(args: &[String]) -> Result<(), GoldAndGearsCliError> {
    let json = mode_json_only(args)?;
    let factory = GoldAndGearsRuntimeFactory::load_candidate(GOLD_BUNDLE)
        .map_err(|_| GoldAndGearsCliError::Configuration)?;
    let coverage = factory.runtime_coverage_summary();
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-coverage\",\"mode\":\"{MODE}\",\"goal_id\":\"gold-and-gears-runtime-v1\",\"source_categories\":{},\"runtime_slices\":{},\"source_obligations\":{},\"integrated\":{},\"shared_integrated\":{},\"external_outcomes\":{},\"metadata\":{},\"mechanic_rules\":{},\"fixtures\":{},\"native_handlers\":{},\"coverage_digest\":\"{}\"}}",
            coverage.source_categories(),
            coverage.source_runtime_slices(),
            coverage.source_obligations(),
            coverage.integrated_obligations(),
            coverage.shared_integrated_obligations(),
            coverage.external_outcomes(),
            coverage.metadata_obligations(),
            coverage.mechanic_rules(),
            coverage.semantic_fixtures(),
            coverage.native_handlers(),
            hex(coverage.digest()),
        );
    } else {
        println!(
            "universe coverage mode={MODE} goal=gold-and-gears-runtime-v1 categories={} slices={} source_obligations={} integrated={} shared_integrated={} external_outcomes={} metadata={} rules={} fixtures={} native_handlers={} digest={}",
            coverage.source_categories(),
            coverage.source_runtime_slices(),
            coverage.source_obligations(),
            coverage.integrated_obligations(),
            coverage.shared_integrated_obligations(),
            coverage.external_outcomes(),
            coverage.metadata_obligations(),
            coverage.mechanic_rules(),
            coverage.semantic_fixtures(),
            coverage.native_handlers(),
            hex(coverage.digest()),
        );
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), GoldAndGearsCliError> {
    let options = RunOptions::parse(args)?;
    let fixture = fixture()?;
    let request = request(options.seed, &fixture)?;
    let recorded = record_gold_and_gears_run(fixture.instance(), request, fixture.roster())
        .map_err(|_| GoldAndGearsCliError::Simulation)?;
    let header =
        gold_and_gears_replay_header(fixture.components().clone(), request, fixture.roster())
            .map_err(|_| GoldAndGearsCliError::Replay)?;
    let replay = encode_gold_and_gears_replay(&header, &recorded)
        .map_err(|_| GoldAndGearsCliError::Replay)?;
    let mut replay_digest = Sha256Sink::new();
    replay_digest.write(&replay);
    let replay_digest = replay_digest.finalize().bytes();
    if let Some(path) = &options.replay_out {
        fs::write(path, &replay).map_err(GoldAndGearsCliError::Io)?;
    }
    let report = recorded.report();
    if options.json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-run\",\"mode\":\"{MODE}\",\"seed\":{},\"profile\":\"gold-gears.profile.v1\",\"area\":\"{}\",\"path\":\"{}\",\"custom_dice\":\"{}\",\"controller\":\"baseline\",\"battle_executor\":\"{GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION}\",\"fixture_accuracy\":\"{GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY}\",\"component_root\":\"{}\",\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\",\"replay_bytes\":{},\"replay_sha256\":\"{}\"}}",
            options.seed,
            fixture.area(),
            fixture.path(),
            fixture.custom_dice(),
            hex(fixture.components().root().bytes()),
            recorded.action_count(),
            report.battle_count(),
            recorded.battle_command_count(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
            hex(replay_digest),
        );
    } else {
        println!(
            "universe completed mode={MODE} seed={} profile=gold-gears.profile.v1 controller=baseline battle_executor={GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION} fixture_accuracy={GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY} component_root={} actions={} nested_battles={} battle_commands={} hash={} replay_bytes={} replay_sha256={}",
            options.seed,
            hex(fixture.components().root().bytes()),
            recorded.action_count(),
            report.battle_count(),
            recorded.battle_command_count(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
            hex(replay_digest),
        );
    }
    Ok(())
}

pub fn is_replay(bytes: &[u8]) -> bool {
    decode_replay(bytes).is_ok_and(|replay| {
        matches!(replay.header().entry(), ReplayEntry::Activity { profile_id, .. }
            if profile_id.as_ref() == GOLD_AND_GEARS_REAL_BATTLE_REPLAY_REVISION)
    })
}

pub fn verify_replay(bytes: &[u8], json: bool) -> Result<(), GoldAndGearsCliError> {
    let replay = decode_replay(bytes).map_err(|_| GoldAndGearsCliError::Replay)?;
    let fixture = fixture()?;
    let request = request(replay.header().master_seed(), &fixture)?;
    let report = verify_gold_and_gears_replay(
        bytes,
        fixture.instance(),
        request,
        fixture.roster(),
        fixture.components(),
    )
    .map_err(|_| GoldAndGearsCliError::Replay)?;
    if report.terminal() != ActivityTerminalOutcome::Completed {
        return Err(GoldAndGearsCliError::Replay);
    }
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"replay-verify\",\"entry\":\"gold-and-gears\",\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\"}}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    } else {
        println!(
            "gold-and-gears replay verified actions={} nested_battles={} battle_commands={} terminal=completed hash={}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    }
    Ok(())
}

fn fixture() -> Result<GoldAndGearsBaselineFixture, GoldAndGearsCliError> {
    let identity = GoldAndGearsCatalogIdentity::load(GOLD_BUNDLE)
        .map_err(|_| GoldAndGearsCliError::Configuration)?;
    GoldAndGearsRuntimeFactory::load_candidate(GOLD_BUNDLE)
        .map_err(|_| GoldAndGearsCliError::Configuration)?
        .compile_synthetic_baseline_fixture(&identity)
        .map_err(|_| GoldAndGearsCliError::Configuration)
}

fn request(
    seed: u64,
    fixture: &GoldAndGearsBaselineFixture,
) -> Result<GoldAndGearsSeededRunRequest, GoldAndGearsCliError> {
    Ok(GoldAndGearsSeededRunRequest::new(
        seed,
        fixture.activity_identity(),
        ActivityInstanceId::new(1).ok_or(GoldAndGearsCliError::Usage)?,
    ))
}

struct RunOptions {
    seed: u64,
    replay_out: Option<PathBuf>,
    json: bool,
}

impl RunOptions {
    fn parse(args: &[String]) -> Result<Self, GoldAndGearsCliError> {
        let mut mode = false;
        let mut seed = None;
        let mut replay_out = None;
        let mut json = false;
        let mut index = 0;
        while index < args.len() {
            let value = |offset: usize| args.get(index + offset).map(String::as_str);
            match args[index].as_str() {
                "--mode" if !mode && value(1) == Some(MODE) => {
                    mode = true;
                    index += 1;
                }
                "--seed" if seed.is_none() => {
                    seed = Some(parse(value(1))?);
                    index += 1;
                }
                "--replay-out" if replay_out.is_none() => {
                    replay_out = Some(PathBuf::from(value(1).ok_or(GoldAndGearsCliError::Usage)?));
                    index += 1;
                }
                "--controller" if value(1) == Some("baseline") => index += 1,
                "--json" if !json => json = true,
                _ => return Err(GoldAndGearsCliError::Usage),
            }
            index += 1;
        }
        if !mode {
            return Err(GoldAndGearsCliError::Usage);
        }
        Ok(Self {
            seed: seed.ok_or(GoldAndGearsCliError::Usage)?,
            replay_out,
            json,
        })
    }
}

fn mode_json_only(args: &[String]) -> Result<bool, GoldAndGearsCliError> {
    match args {
        [flag, mode] if flag == "--mode" && mode == MODE => Ok(false),
        [flag, mode, json] if flag == "--mode" && mode == MODE && json == "--json" => Ok(true),
        [json, flag, mode] if json == "--json" && flag == "--mode" && mode == MODE => Ok(true),
        _ => Err(GoldAndGearsCliError::Usage),
    }
}

fn parse<T: core::str::FromStr>(value: Option<&str>) -> Result<T, GoldAndGearsCliError> {
    value
        .ok_or(GoldAndGearsCliError::Usage)?
        .parse()
        .map_err(|_| GoldAndGearsCliError::Usage)
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug)]
pub enum GoldAndGearsCliError {
    Usage,
    Configuration,
    Simulation,
    Replay,
    Io(std::io::Error),
}

impl GoldAndGearsCliError {
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Usage => 2,
            Self::Configuration => 3,
            Self::Replay => 4,
            Self::Simulation => 6,
            Self::Io(_) => 7,
        }
    }
}

impl fmt::Display for GoldAndGearsCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str("gold-and-gears usage error"),
            Self::Configuration => formatter.write_str("gold-and-gears configuration error"),
            Self::Simulation => formatter.write_str("gold-and-gears simulation error"),
            Self::Replay => formatter.write_str("gold-and-gears replay error"),
            Self::Io(error) => write!(formatter, "gold-and-gears I/O error: {error}"),
        }
    }
}
