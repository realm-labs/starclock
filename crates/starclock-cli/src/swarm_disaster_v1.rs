use std::{fmt, fs, path::PathBuf};

use starclock_activity::{ActivityInstanceId, ActivityTerminalOutcome};
use starclock_mode_universe::{
    swarm_disaster_catalog::validate_swarm_disaster_bundle,
    swarm_disaster_entry::{
        SwarmDisasterRuntimeFactory,
        baseline_fixture::{
            SWARM_DISASTER_BASELINE_BATTLE_EXECUTION_REVISION,
            SWARM_DISASTER_BASELINE_FIXTURE_ACCURACY, SWARM_DISASTER_BASELINE_PROFILE,
            SwarmDisasterBaselineFixture,
        },
        replay::{
            SWARM_DISASTER_REAL_BATTLE_REPLAY_REVISION, encode_complete_swarm_replay_v2,
            verify_complete_swarm_replay_v2,
        },
    },
};
use starclock_replay::{
    codec::CanonicalSink, digest::Sha256Sink, format::ReplayEntry, format_v2::decode_replay_v2,
};

const SWARM_BUNDLE: &[u8] = include_bytes!("../../../config/swarm-disaster-generated/config.sora");
const CLI_REVISION: &str = "starclock-cli-swarm-disaster-v1";
const MODE: &str = "swarm-disaster";
const COVERAGE_SHA256: &str = "8aeb60d2c1b322f9dcf8f84bc45dc1901276633398cdb60a984ccc4846f0bff4";

pub fn requested(args: &[String]) -> bool {
    args.windows(2)
        .any(|pair| pair[0] == "--mode" && pair[1] == MODE)
}

pub fn config_validate(args: &[String]) -> Result<(), SwarmDisasterCliError> {
    let json = mode_json_only(args)?;
    validate_swarm_disaster_bundle(SWARM_BUNDLE)
        .map_err(|_| SwarmDisasterCliError::Configuration)?;
    SwarmDisasterRuntimeFactory::load_candidate(SWARM_BUNDLE)
        .map_err(|_| SwarmDisasterCliError::Configuration)?;
    let bundle_digest = hex(digest(SWARM_BUNDLE));
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-config-validation\",\"mode\":\"{MODE}\",\"valid\":true,\"bundle_sha256\":\"{bundle_digest}\",\"tables\":65,\"rows\":33380,\"source_obligations\":6963,\"mechanic_rules\":23,\"fixtures\":23,\"policy_boundaries\":31}}"
        );
    } else {
        println!(
            "universe config valid mode={MODE} bundle_sha256={bundle_digest} tables=65 rows=33380 source_obligations=6963 rules=23 fixtures=23 policies=31"
        );
    }
    Ok(())
}

pub fn coverage(args: &[String]) -> Result<(), SwarmDisasterCliError> {
    let json = mode_json_only(args)?;
    let factory = SwarmDisasterRuntimeFactory::load_candidate(SWARM_BUNDLE)
        .map_err(|_| SwarmDisasterCliError::Configuration)?;
    if hex(factory.runtime_coverage_digest()) != COVERAGE_SHA256 {
        return Err(SwarmDisasterCliError::Configuration);
    }
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-coverage\",\"mode\":\"{MODE}\",\"goal_id\":\"swarm-disaster-runtime-v1\",\"source_categories\":42,\"runtime_slices\":42,\"source_obligations\":6963,\"integrated\":6282,\"shared_integrated\":652,\"external_outcomes\":6,\"metadata\":23,\"mechanic_rules\":23,\"fixtures\":23,\"native_handlers\":0,\"coverage_digest\":\"{COVERAGE_SHA256}\"}}"
        );
    } else {
        println!(
            "universe coverage mode={MODE} goal=swarm-disaster-runtime-v1 categories=42 slices=42 source_obligations=6963 integrated=6282 shared_integrated=652 external_outcomes=6 metadata=23 rules=23 fixtures=23 native_handlers=0 digest={COVERAGE_SHA256}"
        );
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), SwarmDisasterCliError> {
    let options = RunOptions::parse(args)?;
    let fixture = fixture()?;
    let activity_instance = activity_instance()?;
    let replay = encode_complete_swarm_replay_v2(
        fixture.instance(),
        options.seed,
        fixture.activity_identity(),
        activity_instance,
        fixture.roster(),
        fixture.components().clone(),
    )
    .map_err(|_| SwarmDisasterCliError::Simulation)?;
    let report = verify_complete_swarm_replay_v2(
        &replay,
        fixture.instance(),
        options.seed,
        fixture.activity_identity(),
        activity_instance,
        fixture.roster(),
        fixture.components(),
    )
    .map_err(|_| SwarmDisasterCliError::Replay)?;
    if report.terminal() != ActivityTerminalOutcome::Completed {
        return Err(SwarmDisasterCliError::Simulation);
    }
    let replay_digest = digest(&replay);
    if let Some(path) = &options.replay_out {
        fs::write(path, &replay).map_err(SwarmDisasterCliError::Io)?;
    }
    if options.json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-run\",\"mode\":\"{MODE}\",\"seed\":{},\"profile\":\"{SWARM_DISASTER_BASELINE_PROFILE}\",\"area\":\"{}\",\"path\":\"{}\",\"audience_die\":\"{}\",\"controller\":\"baseline\",\"battle_executor\":\"{SWARM_DISASTER_BASELINE_BATTLE_EXECUTION_REVISION}\",\"fixture_accuracy\":\"{SWARM_DISASTER_BASELINE_FIXTURE_ACCURACY}\",\"component_root\":\"{}\",\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\",\"replay_bytes\":{},\"replay_sha256\":\"{}\"}}",
            options.seed,
            fixture.area(),
            fixture.path(),
            fixture.audience_die(),
            hex(fixture.components().root().bytes()),
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
            hex(replay_digest),
        );
    } else {
        println!(
            "universe completed mode={MODE} seed={} profile={SWARM_DISASTER_BASELINE_PROFILE} controller=baseline battle_executor={SWARM_DISASTER_BASELINE_BATTLE_EXECUTION_REVISION} fixture_accuracy={SWARM_DISASTER_BASELINE_FIXTURE_ACCURACY} component_root={} actions={} nested_battles={} battle_commands={} hash={} replay_bytes={} replay_sha256={}",
            options.seed,
            hex(fixture.components().root().bytes()),
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
            hex(replay_digest),
        );
    }
    Ok(())
}

pub fn is_replay(bytes: &[u8]) -> bool {
    decode_replay_v2(bytes).is_ok_and(|replay| {
        matches!(replay.header().entry(), ReplayEntry::Activity { profile_id, .. }
            if profile_id.as_ref() == SWARM_DISASTER_REAL_BATTLE_REPLAY_REVISION)
    })
}

pub fn verify_replay(bytes: &[u8], json: bool) -> Result<(), SwarmDisasterCliError> {
    let replay = decode_replay_v2(bytes).map_err(|_| SwarmDisasterCliError::Replay)?;
    let fixture = fixture()?;
    let report = verify_complete_swarm_replay_v2(
        bytes,
        fixture.instance(),
        replay.header().master_seed(),
        fixture.activity_identity(),
        activity_instance()?,
        fixture.roster(),
        fixture.components(),
    )
    .map_err(|_| SwarmDisasterCliError::Replay)?;
    if report.terminal() != ActivityTerminalOutcome::Completed {
        return Err(SwarmDisasterCliError::Replay);
    }
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"replay-verify\",\"entry\":\"swarm-disaster\",\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\"}}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    } else {
        println!(
            "swarm-disaster replay verified actions={} nested_battles={} battle_commands={} terminal=completed hash={}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    }
    Ok(())
}

fn fixture() -> Result<SwarmDisasterBaselineFixture, SwarmDisasterCliError> {
    SwarmDisasterRuntimeFactory::load_candidate(SWARM_BUNDLE)
        .map_err(|_| SwarmDisasterCliError::Configuration)?
        .compile_synthetic_baseline_fixture()
        .map_err(|_| SwarmDisasterCliError::Configuration)
}

fn activity_instance() -> Result<ActivityInstanceId, SwarmDisasterCliError> {
    ActivityInstanceId::new(1).ok_or(SwarmDisasterCliError::Usage)
}

struct RunOptions {
    seed: u64,
    replay_out: Option<PathBuf>,
    json: bool,
}

impl RunOptions {
    fn parse(args: &[String]) -> Result<Self, SwarmDisasterCliError> {
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
                    replay_out = Some(PathBuf::from(value(1).ok_or(SwarmDisasterCliError::Usage)?));
                    index += 1;
                }
                "--controller" if value(1) == Some("baseline") => index += 1,
                "--json" if !json => json = true,
                _ => return Err(SwarmDisasterCliError::Usage),
            }
            index += 1;
        }
        if !mode {
            return Err(SwarmDisasterCliError::Usage);
        }
        Ok(Self {
            seed: seed.ok_or(SwarmDisasterCliError::Usage)?,
            replay_out,
            json,
        })
    }
}

fn mode_json_only(args: &[String]) -> Result<bool, SwarmDisasterCliError> {
    match args {
        [flag, mode] if flag == "--mode" && mode == MODE => Ok(false),
        [flag, mode, json] if flag == "--mode" && mode == MODE && json == "--json" => Ok(true),
        [json, flag, mode] if json == "--json" && flag == "--mode" && mode == MODE => Ok(true),
        _ => Err(SwarmDisasterCliError::Usage),
    }
}

fn parse<T: core::str::FromStr>(value: Option<&str>) -> Result<T, SwarmDisasterCliError> {
    value
        .ok_or(SwarmDisasterCliError::Usage)?
        .parse()
        .map_err(|_| SwarmDisasterCliError::Usage)
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    let mut digest = Sha256Sink::new();
    digest.write(bytes);
    digest.finalize().bytes()
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug)]
pub enum SwarmDisasterCliError {
    Usage,
    Configuration,
    Simulation,
    Replay,
    Io(std::io::Error),
}

impl SwarmDisasterCliError {
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

impl fmt::Display for SwarmDisasterCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str("swarm-disaster usage error"),
            Self::Configuration => formatter.write_str("swarm-disaster configuration error"),
            Self::Simulation => formatter.write_str("swarm-disaster simulation error"),
            Self::Replay => formatter.write_str("swarm-disaster replay error"),
            Self::Io(error) => write!(formatter, "swarm-disaster I/O error: {error}"),
        }
    }
}
