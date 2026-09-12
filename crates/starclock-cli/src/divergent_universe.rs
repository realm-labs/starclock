use std::{fmt, fs, path::PathBuf};

use starclock_activity::ActivityTerminalOutcome;
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_mode_universe::divergent_universe::{
    DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE, DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE,
    DivergentUniverseBaselineFixture, encode_divergent_universe_replay,
    record_divergent_universe_run, record_divergent_universe_run_with_tawot_service,
    verify_divergent_universe_replay,
};
use starclock_replay::{
    codec::CanonicalSink, digest::Sha256Sink, entry::ReplayEntry, format::decode_replay,
};

const MODE: &str = "divergent-universe";

pub fn requested(args: &[String]) -> bool {
    args.windows(2)
        .any(|pair| pair[0] == "--mode" && pair[1] == MODE)
}

pub fn config_validate(args: &[String]) -> Result<(), DivergentUniverseCliError> {
    let json = mode_json_only(args)?;
    let fixture = DivergentUniverseBaselineFixture::production()
        .map_err(|_| DivergentUniverseCliError::Configuration)?;
    let identity = fixture.factory().bundle_identity();
    let source_obligations = identity.source_obligation_count();
    if json {
        println!(
            "{{\"kind\":\"universe-config-validation\",\"mode\":\"{MODE}\",\"valid\":true,\"bundle_sha256\":\"{}\",\"tables\":{},\"rows\":{},\"source_rows\":{},\"source_obligations\":{source_obligations},\"mechanic_programs\":669,\"fixtures\":25,\"policy_sources\":54}}",
            hex(identity.configuration_digest().bytes()),
            identity.table_count(),
            identity.row_count(),
            identity.source_count(),
        );
    } else {
        println!(
            "universe config valid mode={MODE} bundle_sha256={} tables={} rows={} source_rows={} source_obligations={source_obligations} mechanic_programs=669 fixtures=25 policy_sources=54",
            hex(identity.configuration_digest().bytes()),
            identity.table_count(),
            identity.row_count(),
            identity.source_count(),
        );
    }
    Ok(())
}

pub fn coverage(args: &[String]) -> Result<(), DivergentUniverseCliError> {
    let json = mode_json_only(args)?;
    let fixture = DivergentUniverseBaselineFixture::production()
        .map_err(|_| DivergentUniverseCliError::Configuration)?;
    let source_obligations = fixture
        .factory()
        .bundle_identity()
        .source_obligation_count();
    if json {
        println!(
            "{{\"kind\":\"universe-coverage\",\"mode\":\"{MODE}\",\"runtime_release_ready\":false,\"coverage_status\":\"behavioral-audit-incomplete\",\"source_obligations\":{source_obligations},\"terminal\":null,\"pending\":null,\"mechanic_programs\":669,\"mechanic_executable\":null,\"mechanic_metadata\":3,\"mechanic_excluded\":3,\"fixtures\":25,\"research_gaps\":25,\"policy_sources\":54,\"native_handlers\":0}}"
        );
    } else {
        println!(
            "universe coverage mode={MODE} runtime_release_ready=false coverage_status=behavioral-audit-incomplete source_obligations={source_obligations} terminal=unverified pending=unverified mechanic_programs=669 mechanic_executable=unverified mechanic_metadata=3 mechanic_excluded=3 fixtures=25 research_gaps=25 policy_sources=54 native_handlers=0"
        );
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), DivergentUniverseCliError> {
    let options = RunOptions::parse(args)?;
    let fixture = DivergentUniverseBaselineFixture::production()
        .map_err(|_| DivergentUniverseCliError::Configuration)?;
    let recorded = if let Some(level) = options.tawot_forge_level {
        record_divergent_universe_run_with_tawot_service(
            &fixture,
            options.family,
            options.seed,
            level,
        )
    } else {
        record_divergent_universe_run(&fixture, options.family, options.seed)
    }
    .map_err(|_| DivergentUniverseCliError::Simulation)?;
    if recorded.report().terminal() != ActivityTerminalOutcome::Completed {
        return Err(DivergentUniverseCliError::Simulation);
    }
    let replay = encode_divergent_universe_replay(&recorded)
        .map_err(|_| DivergentUniverseCliError::Replay)?;
    if let Some(path) = &options.replay_out {
        fs::write(path, &replay).map_err(DivergentUniverseCliError::Io)?;
    }
    let replay_digest = digest(&replay);
    let family = family_name(options.family);
    let report = recorded.report();
    let tawot = options
        .tawot_forge_level
        .map_or_else(|| "null".to_owned(), |level| level.to_string());
    if options.json {
        println!(
            "{{\"kind\":\"universe-run\",\"mode\":\"{MODE}\",\"family\":\"{family}\",\"tawot_forge_level\":{tawot},\"seed\":{},\"profile\":\"{}\",\"controller\":\"baseline\",\"configuration_components\":9,\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\",\"replay_bytes\":{},\"replay_sha256\":\"{}\"}}",
            options.seed,
            profile(options.family),
            recorded.action_count(),
            report.completed_battles(),
            recorded.battle_command_count(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
            hex(replay_digest),
        );
    } else {
        println!(
            "universe completed mode={MODE} family={family} tawot_forge_level={tawot} seed={} profile={} controller=baseline configuration_components=9 actions={} nested_battles={} battle_commands={} terminal=completed hash={} replay_bytes={} replay_sha256={}",
            options.seed,
            profile(options.family),
            recorded.action_count(),
            report.completed_battles(),
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
            if matches!(profile_id.as_ref(), DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE
                | DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE))
    })
}

pub fn verify_replay(bytes: &[u8], json: bool) -> Result<(), DivergentUniverseCliError> {
    let fixture = DivergentUniverseBaselineFixture::production()
        .map_err(|_| DivergentUniverseCliError::Configuration)?;
    let report = verify_divergent_universe_replay(bytes, &fixture)
        .map_err(|_| DivergentUniverseCliError::Replay)?;
    if report.terminal() != ActivityTerminalOutcome::Completed {
        return Err(DivergentUniverseCliError::Replay);
    }
    let family = family_name(report.run_family());
    let tawot = report
        .tawot_forge_level()
        .map_or_else(|| "null".to_owned(), |level| level.to_string());
    if json {
        println!(
            "{{\"kind\":\"replay-verify\",\"entry\":\"divergent-universe\",\"family\":\"{family}\",\"tawot_forge_level\":{tawot},\"configuration_components\":9,\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\"}}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    } else {
        println!(
            "divergent-universe replay verified family={family} tawot_forge_level={tawot} configuration_components=9 actions={} nested_battles={} battle_commands={} terminal=completed hash={}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    }
    Ok(())
}

struct RunOptions {
    tawot_forge_level: Option<u16>,
    family: DivergentUniverseRunFamily,
    seed: u64,
    replay_out: Option<PathBuf>,
    json: bool,
}

impl RunOptions {
    fn parse(args: &[String]) -> Result<Self, DivergentUniverseCliError> {
        let mut mode = false;
        let mut tawot_forge_level = None;
        let mut family = None;
        let mut seed = None;
        let mut replay_out = None;
        let mut json = false;
        let mut index = 0;
        while index < args.len() {
            let value = |offset: usize| args.get(index + offset).map(String::as_str);
            match args[index].as_str() {
                "--tawot-forge-level" if tawot_forge_level.is_none() => {
                    let level = value(1)
                        .and_then(|value| value.parse::<u16>().ok())
                        .filter(|level| (2..=5).contains(level))
                        .ok_or(DivergentUniverseCliError::Usage)?;
                    tawot_forge_level = Some(level);
                    index += 1;
                }
                "--mode" if !mode && value(1) == Some(MODE) => {
                    mode = true;
                    index += 1;
                }
                "--family" if family.is_none() => {
                    family = Some(parse_family(value(1))?);
                    index += 1;
                }
                "--seed" if seed.is_none() => {
                    seed = Some(parse(value(1))?);
                    index += 1;
                }
                "--replay-out" if replay_out.is_none() => {
                    replay_out = Some(PathBuf::from(
                        value(1).ok_or(DivergentUniverseCliError::Usage)?,
                    ));
                    index += 1;
                }
                "--controller" if value(1) == Some("baseline") => index += 1,
                "--json" if !json => json = true,
                _ => return Err(DivergentUniverseCliError::Usage),
            }
            index += 1;
        }
        if !mode {
            return Err(DivergentUniverseCliError::Usage);
        }
        Ok(Self {
            tawot_forge_level,
            family: family.unwrap_or(DivergentUniverseRunFamily::Ordinary),
            seed: seed.ok_or(DivergentUniverseCliError::Usage)?,
            replay_out,
            json,
        })
    }
}

fn mode_json_only(args: &[String]) -> Result<bool, DivergentUniverseCliError> {
    match args {
        [flag, mode] if flag == "--mode" && mode == MODE => Ok(false),
        [flag, mode, json] if flag == "--mode" && mode == MODE && json == "--json" => Ok(true),
        [json, flag, mode] if json == "--json" && flag == "--mode" && mode == MODE => Ok(true),
        _ => Err(DivergentUniverseCliError::Usage),
    }
}

fn parse_family(
    value: Option<&str>,
) -> Result<DivergentUniverseRunFamily, DivergentUniverseCliError> {
    match value {
        Some("ordinary") => Ok(DivergentUniverseRunFamily::Ordinary),
        Some("cyclical") => Ok(DivergentUniverseRunFamily::Cyclical),
        _ => Err(DivergentUniverseCliError::Usage),
    }
}

fn family_name(family: DivergentUniverseRunFamily) -> &'static str {
    match family {
        DivergentUniverseRunFamily::Ordinary => "ordinary",
        DivergentUniverseRunFamily::Cyclical => "cyclical",
    }
}

fn profile(family: DivergentUniverseRunFamily) -> &'static str {
    match family {
        DivergentUniverseRunFamily::Ordinary => DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE,
        DivergentUniverseRunFamily::Cyclical => DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE,
    }
}

fn parse<T: core::str::FromStr>(value: Option<&str>) -> Result<T, DivergentUniverseCliError> {
    value
        .ok_or(DivergentUniverseCliError::Usage)?
        .parse()
        .map_err(|_| DivergentUniverseCliError::Usage)
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
pub enum DivergentUniverseCliError {
    Usage,
    Configuration,
    Simulation,
    Replay,
    Io(std::io::Error),
}

impl DivergentUniverseCliError {
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

impl fmt::Display for DivergentUniverseCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str("divergent-universe usage error: run --mode divergent-universe --seed U64 [--family ordinary|cyclical] [--tawot-forge-level 2|3|4|5] [--controller baseline] [--replay-out PATH] [--json]"),
            Self::Configuration => formatter.write_str("divergent-universe configuration error"),
            Self::Simulation => formatter.write_str("divergent-universe simulation error"),
            Self::Replay => formatter.write_str("divergent-universe replay error"),
            Self::Io(error) => write!(formatter, "divergent-universe I/O error: {error}"),
        }
    }
}
