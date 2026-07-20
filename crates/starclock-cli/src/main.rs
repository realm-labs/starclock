//! Headless Starclock command-line entry point.

#![forbid(unsafe_code)]

use std::{env, fmt, fs, path::PathBuf, process::ExitCode};

use starclock_ai::baseline::{
    BaselineAbilityClass, BaselineAbilityHint, BaselineController, BaselineHints,
    BaselineScoreComponents, BaselineTargetHint,
};
use starclock_combat::{AbilityId, BattlePhase, UnitId};
use starclock_data::{
    catalog::{CatalogLoadError, SimulationCatalog},
    coverage::{GoalCoverageCategory, GoalCoverageCategorySummary},
};
use starclock_mode_standard::synthetic::{
    SYNTHETIC_STANDARD_CATALOG_REVISION, SYNTHETIC_STANDARD_CONFIG_DIGEST,
    SYNTHETIC_STANDARD_RULES_REVISION, SYNTHETIC_STANDARD_SCENARIO_ID, SyntheticStandardProfile,
};
use starclock_replay::{
    battle::{
        BattleReplayError, BattleTraceEntry, battle_record_count, encode_battle_trace,
        verify_battle_replay,
    },
    codec::CanonicalSink,
    digest::{ConfigBundleDigest, ControllerDigest, EntrySpecDigest, Sha256Sink},
    format::{ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity, decode_replay},
};

const CONTROLLER_REVISION: &str = BaselineController::REVISION;
const CONTROLLER_DESCRIPTOR: &[u8] = b"baseline-battle-controller-v1\0synthetic-standard-v1\0ability:1:basic:0:0:0:0:0:false\0target:2:0";
const MAX_SMOKE_COMMANDS: usize = 16;
const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const CLI_SCHEMA_REVISION: &str = "starclock-cli-v1";

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    match args.as_slice() {
        [group, command, rest @ ..] if group == "config" && command == "validate" => {
            config_validate(rest)
        }
        [group, command, rest @ ..] if group == "catalog" && command == "coverage" => {
            catalog_coverage(rest)
        }
        [group, command, rest @ ..] if group == "battle" && command == "run" => battle_run(rest),
        [group, command, file, rest @ ..] if group == "replay" && command == "verify" => {
            replay_verify(file, rest)
        }
        _ => Err(CliError::Usage(
            "starclock config validate [--bundle PATH] [--json] | catalog coverage [--goal core-combat-v1] [--category NAME] [--json] | battle run --scenario ID --seed U64 [--controller baseline|replay] [--replay-out PATH] [--json] | replay verify FILE [--json]",
        )),
    }
}

fn config_validate(args: &[String]) -> Result<(), CliError> {
    let mut bundle_path = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--bundle" => {
                bundle_path = Some(PathBuf::from(value_after(args, &mut index, "--bundle")?));
            }
            "--json" => json = true,
            _ => return Err(CliError::Usage("unknown config validate option")),
        }
        index += 1;
    }
    let owned;
    let bytes = if let Some(path) = &bundle_path {
        owned = fs::read(path).map_err(CliError::ConfigurationIo)?;
        owned.as_slice()
    } else {
        PRODUCTION_BUNDLE
    };
    let catalog = starclock_data::catalog::load(bytes)?;
    let summary = catalog.summary();
    let mut digest = Sha256Sink::new();
    digest.write(bytes);
    let bundle_digest = hex(digest.finalize().bytes());
    if json {
        println!(
            "{{\"schema_revision\":\"{}\",\"kind\":\"config-validation\",\"valid\":true,\"game_version\":\"{}\",\"data_revision\":\"{}\",\"bundle_sha256\":\"{}\",\"identities\":{},\"enabled\":{}}}",
            CLI_SCHEMA_REVISION,
            json_escape(&catalog.manifest().game_version),
            json_escape(&catalog.manifest().data_revision),
            bundle_digest,
            summary.identity_count,
            summary.enabled_identity_count,
        );
    } else {
        println!(
            "config valid game_version={} data_revision={} bundle_sha256={} identities={} enabled={}",
            catalog.manifest().game_version,
            catalog.manifest().data_revision,
            bundle_digest,
            summary.identity_count,
            summary.enabled_identity_count,
        );
    }
    Ok(())
}

fn catalog_coverage(args: &[String]) -> Result<(), CliError> {
    let mut category = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--goal" => {
                if value_after(args, &mut index, "--goal")? != "core-combat-v1" {
                    return Err(CliError::Usage("unknown coverage goal"));
                }
            }
            "--category" => {
                let value = value_after(args, &mut index, "--category")?;
                category = Some(
                    GoalCoverageCategory::parse(value)
                        .ok_or(CliError::Usage("unknown coverage category"))?,
                );
            }
            "--json" => json = true,
            _ => return Err(CliError::Usage("unknown catalog coverage option")),
        }
        index += 1;
    }
    let catalog = starclock_data::catalog::load(PRODUCTION_BUNDLE)?;
    write_coverage(&catalog, category, json);
    Ok(())
}

fn write_coverage(catalog: &SimulationCatalog, selected: Option<GoalCoverageCategory>, json: bool) {
    let report = catalog.goal_coverage();
    let categories = report
        .categories()
        .iter()
        .copied()
        .filter(|row| selected.is_none_or(|category| row.category() == category))
        .collect::<Vec<_>>();
    if json {
        let rows = categories
            .iter()
            .map(coverage_json)
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "{{\"schema_revision\":\"{}\",\"kind\":\"catalog-coverage\",\"goal_id\":\"core-combat-v1\",\"manifest_sha256\":\"{}\",\"required\":{},\"enabled\":{},\"data_ready\":{},\"golden_verified\":{},\"categories\":[{}]}}",
            CLI_SCHEMA_REVISION,
            report.manifest_digest(),
            categories.iter().map(|row| row.required()).sum::<usize>(),
            categories.iter().map(|row| row.enabled()).sum::<usize>(),
            categories.iter().map(|row| row.data_ready()).sum::<usize>(),
            categories
                .iter()
                .map(|row| row.golden_verified())
                .sum::<usize>(),
            rows,
        );
    } else {
        println!(
            "catalog coverage goal=core-combat-v1 manifest={} required={} enabled={} data_ready={} golden_verified={}",
            report.manifest_digest(),
            categories.iter().map(|row| row.required()).sum::<usize>(),
            categories.iter().map(|row| row.enabled()).sum::<usize>(),
            categories.iter().map(|row| row.data_ready()).sum::<usize>(),
            categories
                .iter()
                .map(|row| row.golden_verified())
                .sum::<usize>(),
        );
        for row in categories {
            println!(
                "{} required={} enabled={} data_ready={} golden_verified={}",
                row.category().name(),
                row.required(),
                row.enabled(),
                row.data_ready(),
                row.golden_verified(),
            );
        }
    }
}

fn coverage_json(row: &GoalCoverageCategorySummary) -> String {
    format!(
        "{{\"category\":\"{}\",\"required\":{},\"enabled\":{},\"data_ready\":{},\"golden_verified\":{}}}",
        row.category().name(),
        row.required(),
        row.enabled(),
        row.data_ready(),
        row.golden_verified(),
    )
}

fn battle_run(args: &[String]) -> Result<(), CliError> {
    let mut scenario = None;
    let mut seed = None;
    let mut replay_out = None;
    let mut controller = "baseline";
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--scenario" => {
                scenario = Some(value_after(args, &mut index, "--scenario")?);
            }
            "--seed" => {
                seed = Some(
                    value_after(args, &mut index, "--seed")?
                        .parse::<u64>()
                        .map_err(|_| {
                            CliError::Usage("--seed requires an unsigned 64-bit integer")
                        })?,
                );
            }
            "--replay-out" => {
                replay_out = Some(PathBuf::from(value_after(
                    args,
                    &mut index,
                    "--replay-out",
                )?));
            }
            "--controller" => {
                controller = value_after(args, &mut index, "--controller")?;
            }
            "--json" => json = true,
            _ => return Err(CliError::Usage("unknown battle run option")),
        }
        index += 1;
    }
    if scenario != Some(SYNTHETIC_STANDARD_SCENARIO_ID) {
        return Err(CliError::UnknownScenario);
    }
    let seed = seed.ok_or(CliError::Usage("battle run requires --seed"))?;
    if controller == "replay" {
        return Err(CliError::Usage(
            "battle run replay control requires an accepted stream; use replay verify FILE",
        ));
    }
    if controller != "baseline" {
        return Err(CliError::Usage("unknown battle controller"));
    }
    let instantiated = SyntheticStandardProfile.instantiate(seed);
    let mut battle = instantiated
        .create_battle()
        .map_err(|_| CliError::Simulation("synthetic battle construction failed"))?;
    let mut trace = Vec::new();
    let hints = synthetic_baseline_hints()?;
    while !battle.view().phase().is_terminal() {
        if trace.len() == MAX_SMOKE_COMMANDS {
            return Err(CliError::Simulation("synthetic command budget exhausted"));
        }
        let decision = battle
            .decision()
            .ok_or(CliError::Simulation("nonterminal battle has no decision"))?;
        let command = BaselineController
            .decide(battle.view(), decision, &hints)
            .map_err(|_| CliError::Simulation("baseline controller rejected authored hints"))?
            .command()
            .clone();
        let resolution = battle
            .apply(command.clone())
            .map_err(|_| CliError::Simulation("offered command was rejected"))?;
        trace.push(BattleTraceEntry::new(command, resolution.state_hash()));
    }
    if battle.view().phase() != BattlePhase::Won {
        return Err(CliError::Simulation(
            "synthetic Standard battle did not win",
        ));
    }
    let header = replay_header(&instantiated, trace.len())?;
    let replay = encode_battle_trace(&header, &trace)?;
    if let Some(path) = &replay_out {
        fs::write(path, &replay).map_err(CliError::Io)?;
    }
    let final_hash = battle.state_hash().bytes();
    if json {
        println!(
            "{{\"schema_revision\":\"{}\",\"kind\":\"battle-run\",\"scenario\":\"{}\",\"seed\":{},\"controller\":\"baseline\",\"commands\":{},\"phase\":\"won\",\"state_hash\":\"{}\",\"replay_bytes\":{}}}",
            CLI_SCHEMA_REVISION,
            SYNTHETIC_STANDARD_SCENARIO_ID,
            seed,
            trace.len(),
            hex(final_hash),
            replay.len()
        );
    } else {
        println!(
            "battle won scenario={} seed={} controller=baseline commands={} hash={} replay_bytes={}",
            SYNTHETIC_STANDARD_SCENARIO_ID,
            seed,
            trace.len(),
            hex(final_hash),
            replay.len()
        );
    }
    Ok(())
}

fn replay_verify(file: &str, args: &[String]) -> Result<(), CliError> {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => {
            return Err(CliError::Usage(
                "replay verify accepts only optional --json",
            ));
        }
    };
    let bytes = fs::read(file).map_err(CliError::Io)?;
    let decoded = decode_replay(&bytes).map_err(BattleReplayError::from)?;
    let seed = decoded.header().master_seed();
    match decoded.header().entry() {
        ReplayEntry::Battle {
            definition_id: 1, ..
        } if decoded.header().identity().config_bundle()
            == ConfigBundleDigest::new(SYNTHETIC_STANDARD_CONFIG_DIGEST)
            && decoded.header().identity().game_version() == "synthetic-v1"
            && decoded.header().identity().rules_revision()
                == SYNTHETIC_STANDARD_RULES_REVISION
            && decoded.header().identity().data_revision()
                == SYNTHETIC_STANDARD_CATALOG_REVISION
            && decoded.header().controller().revision() == CONTROLLER_REVISION
            && decoded.header().controller().digest() == controller_digest() => {}
        _ => return Err(CliError::UnknownScenario),
    }
    let instantiated = SyntheticStandardProfile.instantiate(seed);
    let battle = instantiated
        .create_battle()
        .map_err(|_| CliError::Simulation("synthetic replay battle construction failed"))?;
    let report = verify_battle_replay(&bytes, battle)?;
    if json {
        println!(
            "{{\"schema_revision\":\"{}\",\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":{},\"phase\":\"{}\",\"state_hash\":\"{}\"}}",
            CLI_SCHEMA_REVISION,
            report.command_count(),
            phase_name(report.phase()),
            hex(report.final_hash().bytes())
        );
    } else {
        println!(
            "replay verified commands={} phase={} hash={}",
            report.command_count(),
            phase_name(report.phase()),
            hex(report.final_hash().bytes())
        );
    }
    Ok(())
}

fn synthetic_baseline_hints() -> Result<BaselineHints, CliError> {
    let components =
        BaselineScoreComponents::new(0, 0, 0, 0, 0, false).expect("zero components are bounded");
    BaselineHints::new(
        vec![BaselineAbilityHint::new(
            AbilityId::new(1).expect("synthetic ability ID"),
            BaselineAbilityClass::Basic,
            components,
        )],
        vec![
            BaselineTargetHint::new(UnitId::try_from(2).expect("synthetic runtime ID"), 0)
                .expect("zero target score is bounded"),
        ],
    )
    .map_err(|_| CliError::Simulation("synthetic baseline hints are invalid"))
}

fn replay_header(
    scenario: &starclock_mode_standard::synthetic::SyntheticStandardBattle,
    command_count: usize,
) -> Result<ReplayHeader, CliError> {
    let identity = ReplayIdentity::new(
        "synthetic-v1",
        SYNTHETIC_STANDARD_RULES_REVISION,
        scenario.catalog_revision(),
        ConfigBundleDigest::new(scenario.config_digest()),
        starclock_combat::NUMERIC_POLICY_REVISION,
        starclock_combat::rng::RNG_ALGORITHM_REVISION,
        starclock_combat::STATE_HASH_REVISION,
    )?;
    let controller = ControllerIdentity::new(CONTROLLER_REVISION, controller_digest())?;
    let entry = ReplayEntry::Battle {
        definition_id: scenario.encounter().get(),
        spec_digest: EntrySpecDigest::new(scenario.spec_digest().bytes()),
    };
    ReplayHeader::new(
        identity,
        controller,
        scenario.master_seed(),
        entry,
        battle_record_count(command_count)?,
    )
    .map_err(Into::into)
}

fn value_after<'a>(
    args: &'a [String],
    index: &mut usize,
    name: &'static str,
) -> Result<&'a str, CliError> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or(CliError::Usage(name))
}

fn phase_name(phase: BattlePhase) -> &'static str {
    match phase {
        BattlePhase::Initializing => "initializing",
        BattlePhase::AwaitingCommand => "awaiting-command",
        BattlePhase::Resolving => "resolving",
        BattlePhase::Won => "won",
        BattlePhase::Lost => "lost",
        BattlePhase::Faulted => "faulted",
    }
}

fn hex(bytes: [u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(64);
    for byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    value
}

fn controller_digest() -> ControllerDigest {
    let mut digest = Sha256Sink::new();
    digest.write(CONTROLLER_DESCRIPTOR);
    ControllerDigest::new(digest.finalize().bytes())
}

fn json_escape(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if control <= '\u{1f}' => {
                use std::fmt::Write as _;
                write!(output, "\\u{:04x}", u32::from(control))
                    .expect("writing to a String cannot fail");
            }
            other => output.push(other),
        }
    }
    output
}

#[derive(Debug)]
enum CliError {
    Usage(&'static str),
    Configuration(CatalogLoadError),
    ConfigurationIo(std::io::Error),
    UnknownScenario,
    Simulation(&'static str),
    Io(std::io::Error),
    Replay(BattleReplayError),
}

impl CliError {
    const fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => 2,
            Self::Configuration(_) | Self::ConfigurationIo(_) => 3,
            Self::Replay(_) => 4,
            Self::UnknownScenario => 5,
            Self::Simulation(_) => 6,
            Self::Io(_) => 7,
        }
    }
}

impl From<BattleReplayError> for CliError {
    fn from(value: BattleReplayError) -> Self {
        Self::Replay(value)
    }
}

impl From<CatalogLoadError> for CliError {
    fn from(value: CatalogLoadError) -> Self {
        Self::Configuration(value)
    }
}

impl From<starclock_replay::record::ReplayFormatError> for CliError {
    fn from(value: starclock_replay::record::ReplayFormatError) -> Self {
        Self::Replay(value.into())
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "usage error: {message}"),
            Self::Configuration(error) => write!(formatter, "configuration error: {error}"),
            Self::ConfigurationIo(error) => write!(formatter, "configuration I/O error: {error}"),
            Self::UnknownScenario => formatter.write_str("unknown or incompatible scenario"),
            Self::Simulation(message) => write!(formatter, "simulation error: {message}"),
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Replay(error) => error.fmt(formatter),
        }
    }
}
