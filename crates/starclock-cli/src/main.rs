//! Headless Starclock command-line entry point.

#![forbid(unsafe_code)]

use std::{env, fmt, fs, path::PathBuf, process::ExitCode};

use starclock_combat::{BattlePhase, Command, DecisionKind};
use starclock_mode_standard::synthetic::{
    SYNTHETIC_STANDARD_CATALOG_REVISION, SYNTHETIC_STANDARD_CONFIG_DIGEST,
    SYNTHETIC_STANDARD_RULES_REVISION, SYNTHETIC_STANDARD_SCENARIO_ID, SyntheticStandardProfile,
};
use starclock_replay::{
    battle::{
        BattleReplayError, BattleTraceEntry, battle_record_count, encode_battle_trace,
        verify_battle_replay,
    },
    digest::{ConfigBundleDigest, ControllerDigest, EntrySpecDigest},
    format::{ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity, decode_replay},
};

const CONTROLLER_REVISION: &str = "synthetic-offered-first-v1";
const CONTROLLER_DIGEST: [u8; 32] = [0xc1; 32];
const MAX_SMOKE_COMMANDS: usize = 16;

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
        [group, command, rest @ ..] if group == "battle" && command == "run" => battle_run(rest),
        [group, command, file, rest @ ..] if group == "replay" && command == "verify" => {
            replay_verify(file, rest)
        }
        _ => Err(CliError::Usage(
            "usage: starclock battle run --scenario synthetic-standard-v1 --seed U64 [--replay-out PATH] [--json] | starclock replay verify FILE [--json]",
        )),
    }
}

fn battle_run(args: &[String]) -> Result<(), CliError> {
    let mut scenario = None;
    let mut seed = None;
    let mut replay_out = None;
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
            "--json" => json = true,
            _ => return Err(CliError::Usage("unknown battle run option")),
        }
        index += 1;
    }
    if scenario != Some(SYNTHETIC_STANDARD_SCENARIO_ID) {
        return Err(CliError::UnknownScenario);
    }
    let seed = seed.ok_or(CliError::Usage("battle run requires --seed"))?;
    let instantiated = SyntheticStandardProfile.instantiate(seed);
    let mut battle = instantiated
        .create_battle()
        .map_err(|_| CliError::Simulation("synthetic battle construction failed"))?;
    let mut trace = Vec::new();
    while !battle.view().phase().is_terminal() {
        if trace.len() == MAX_SMOKE_COMMANDS {
            return Err(CliError::Simulation("synthetic command budget exhausted"));
        }
        let command = select_smoke_command(&battle)?;
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
            "{{\"schema\":1,\"kind\":\"battle-run\",\"scenario\":\"{}\",\"seed\":{},\"commands\":{},\"phase\":\"won\",\"state_hash\":\"{}\",\"replay_bytes\":{}}}",
            SYNTHETIC_STANDARD_SCENARIO_ID,
            seed,
            trace.len(),
            hex(final_hash),
            replay.len()
        );
    } else {
        println!(
            "battle won scenario={} seed={} commands={} hash={} replay_bytes={}",
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
            && decoded.header().controller().digest()
                == ControllerDigest::new(CONTROLLER_DIGEST) => {}
        _ => return Err(CliError::UnknownScenario),
    }
    let instantiated = SyntheticStandardProfile.instantiate(seed);
    let battle = instantiated
        .create_battle()
        .map_err(|_| CliError::Simulation("synthetic replay battle construction failed"))?;
    let report = verify_battle_replay(&bytes, battle)?;
    if json {
        println!(
            "{{\"schema\":1,\"kind\":\"replay-verify\",\"commands\":{},\"phase\":\"{}\",\"state_hash\":\"{}\"}}",
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

fn select_smoke_command(battle: &starclock_combat::Battle) -> Result<Command, CliError> {
    let decision = battle
        .decision()
        .ok_or(CliError::Simulation("nonterminal battle has no decision"))?;
    let selected = match decision.kind() {
        DecisionKind::BattleStart => decision.legal_commands().first(),
        DecisionKind::InterruptWindow => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. })),
        DecisionKind::NormalAction => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::UseAbility { .. })),
        DecisionKind::BattleChoice => None,
    };
    selected.cloned().ok_or(CliError::Simulation(
        "synthetic controller found no supported offered command",
    ))
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
    let controller = ControllerIdentity::new(
        CONTROLLER_REVISION,
        ControllerDigest::new(CONTROLLER_DIGEST),
    )?;
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

#[derive(Debug)]
enum CliError {
    Usage(&'static str),
    UnknownScenario,
    Simulation(&'static str),
    Io(std::io::Error),
    Replay(BattleReplayError),
}

impl CliError {
    const fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => 2,
            Self::UnknownScenario => 3,
            Self::Replay(_) => 4,
            Self::Simulation(_) => 5,
            Self::Io(_) => 6,
        }
    }
}

impl From<BattleReplayError> for CliError {
    fn from(value: BattleReplayError) -> Self {
        Self::Replay(value)
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
            Self::UnknownScenario => formatter.write_str("unknown or incompatible scenario"),
            Self::Simulation(message) => write!(formatter, "simulation error: {message}"),
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Replay(error) => error.fmt(formatter),
        }
    }
}
