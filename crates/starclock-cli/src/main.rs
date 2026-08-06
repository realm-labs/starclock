//! Headless Starclock command-line entry point.

#![forbid(unsafe_code)]

mod gold_gears;
mod standard;
mod swarm_disaster;
mod universe;

use std::{env, fmt, fs, path::PathBuf, process::ExitCode};

use starclock_ai::baseline::{
    BaselineAbilityClass, BaselineAbilityHint, BaselineController, BaselineHints,
    BaselineScoreComponents, BaselineTargetHint,
};
use starclock_combat::{AbilityId, BattlePhase, Command, DecisionKind, UnitId};
use starclock_data::{
    catalog::{CatalogLoadError, SimulationCatalog},
    coverage::{GoalCoverageCategory, GoalCoverageCategorySummary},
};
use starclock_mode_standard::synthetic::{
    SYNTHETIC_STANDARD_CONFIG_DIGEST, SYNTHETIC_STANDARD_SCENARIO_ID, SyntheticStandardProfile,
};
use starclock_replay::{
    battle::{
        BattleReplayError, BattleTraceEntry, battle_record_count, encode_battle_trace,
        verify_battle_replay,
    },
    codec::CanonicalSink,
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::{ComponentDigest, EntrySpecDigest, Sha256Sink},
    entry::ReplayEntry,
    format::{ReplayEnvironment, ReplayError, ReplayHeader, decode_replay},
};

const CONTROLLER_DESCRIPTOR: &[u8] =
    b"baseline-battle-controller\0synthetic-standard\0ability:1:basic:0:0:0:0:0:false\0target:2:0";
const STANDARD_CONTROLLER_DESCRIPTOR: &[u8] =
    b"baseline-battle-controller\0standard\0first-canonical-supported-command";
const MAX_SMOKE_COMMANDS: usize = 16;
const MAX_STANDARD_COMMANDS: usize = 512;
const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");

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
        [group, command, rest @ ..]
            if group == "universe" && command == "run" && swarm_disaster::requested(rest) =>
        {
            swarm_disaster::run(rest).map_err(CliError::SwarmDisaster)
        }
        [group, command, rest @ ..]
            if group == "universe" && command == "run" && gold_gears::requested(rest) =>
        {
            gold_gears::run(rest).map_err(CliError::GoldAndGears)
        }
        [group, command, rest @ ..] if group == "universe" && command == "run" => {
            universe::run(rest).map_err(CliError::Universe)
        }
        [group, command, rest @ ..]
            if group == "universe" && command == "coverage" && swarm_disaster::requested(rest) =>
        {
            swarm_disaster::coverage(rest).map_err(CliError::SwarmDisaster)
        }
        [group, command, rest @ ..]
            if group == "universe" && command == "coverage" && gold_gears::requested(rest) =>
        {
            gold_gears::coverage(rest).map_err(CliError::GoldAndGears)
        }
        [group, command, rest @ ..] if group == "universe" && command == "coverage" => {
            universe::coverage(rest).map_err(CliError::Universe)
        }
        [group, scope, command, rest @ ..]
            if group == "universe"
                && scope == "config"
                && command == "validate"
                && swarm_disaster::requested(rest) =>
        {
            swarm_disaster::config_validate(rest).map_err(CliError::SwarmDisaster)
        }
        [group, scope, command, rest @ ..]
            if group == "universe"
                && scope == "config"
                && command == "validate"
                && gold_gears::requested(rest) =>
        {
            gold_gears::config_validate(rest).map_err(CliError::GoldAndGears)
        }
        [group, scope, command, rest @ ..]
            if group == "universe" && scope == "config" && command == "validate" =>
        {
            universe::config_validate(rest).map_err(CliError::Universe)
        }
        [group, command, rest @ ..] if group == "mcp" && command == "serve" => mcp_serve(rest),
        [group, command, file, rest @ ..] if group == "replay" && command == "verify" => {
            replay_verify(file, rest)
        }
        _ => Err(CliError::Usage(
            "starclock config validate [--bundle PATH] [--json] | catalog coverage [--goal core-combat-v1] [--category NAME] [--json] | battle run --scenario ID --seed U64 [--controller baseline|replay] [--replay-out PATH] [--json] | universe config validate [--mode gold-and-gears|swarm-disaster] [--json] | universe coverage [--mode gold-and-gears|swarm-disaster] [--json] | universe run (--world ID --difficulty-index N | --mode gold-and-gears|swarm-disaster) --seed U64 [--controller baseline] [--replay-out PATH] [--json] | replay verify FILE [--json] | mcp serve --transport stdio | mcp serve --transport streamable-http --development-loopback --bind IP:PORT --allow-origin ORIGIN",
        )),
    }
}

fn mcp_serve(args: &[String]) -> Result<(), CliError> {
    if matches!(args, [flag, transport] if flag == "--transport" && transport == "stdio") {
        return starclock_mcp::stdio::serve().map_err(CliError::Mcp);
    }
    let mut transport = None;
    let mut development_loopback = false;
    let mut bind = None;
    let mut allowed_origins = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--transport" if transport.is_none() => {
                transport = Some(value_after(args, &mut index, "--transport")?);
            }
            "--development-loopback" if !development_loopback => development_loopback = true,
            "--bind" if bind.is_none() => {
                bind = Some(
                    value_after(args, &mut index, "--bind")?
                        .parse::<std::net::SocketAddr>()
                        .map_err(|_| CliError::Usage("--bind requires an IP socket address"))?,
                );
            }
            "--allow-origin" => {
                allowed_origins.push(value_after(args, &mut index, "--allow-origin")?.to_owned());
            }
            _ => return Err(CliError::Usage("unknown or duplicate mcp serve option")),
        }
        index += 1;
    }
    if transport != Some("streamable-http") || !development_loopback {
        return Err(CliError::Usage(
            "HTTP requires --transport streamable-http --development-loopback",
        ));
    }
    let bind = bind.ok_or(CliError::Usage("HTTP requires --bind IP:PORT"))?;
    let config = starclock_mcp::http::LoopbackHttpConfig::new(bind, allowed_origins)
        .map_err(CliError::McpHttp)?;
    starclock_mcp::http::serve_loopback(config).map_err(CliError::McpHttp)
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
            "{{\"kind\":\"config-validation\",\"valid\":true,\"game_version\":\"{}\",\"bundle_sha256\":\"{}\",\"identities\":{},\"enabled\":{}}}",
            json_escape(&catalog.manifest().game_version),
            bundle_digest,
            summary.identity_count,
            summary.enabled_identity_count,
        );
    } else {
        println!(
            "config valid game_version={} bundle_sha256={} identities={} enabled={}",
            catalog.manifest().game_version,
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
            "{{\"kind\":\"catalog-coverage\",\"required\":{},\"enabled\":{},\"data_ready\":{},\"golden_verified\":{},\"categories\":[{}]}}",
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
            "catalog coverage required={} enabled={} data_ready={} golden_verified={}",
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
    let scenario = scenario.ok_or(CliError::UnknownScenario)?;
    let seed = seed.ok_or(CliError::Usage("battle run requires --seed"))?;
    if controller == "replay" {
        return Err(CliError::Usage(
            "battle run replay control requires an accepted stream; use replay verify FILE",
        ));
    }
    if controller != "baseline" {
        return Err(CliError::Usage("unknown battle controller"));
    }
    if scenario != SYNTHETIC_STANDARD_SCENARIO_ID {
        return standard_battle_run(scenario, seed, replay_out, json);
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
        let command = if battle.view().phase() == BattlePhase::ReadyToAdvance {
            battle
                .advance_command()
                .ok_or(CliError::Simulation("battle has no action boundary"))?
        } else {
            let decision = battle
                .decision()
                .ok_or(CliError::Simulation("nonterminal battle has no decision"))?;
            BaselineController
                .decide(battle.view(), decision, &hints)
                .map_err(|_| CliError::Simulation("baseline controller rejected authored hints"))?
                .command()
                .clone()
        };
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
            "{{\"kind\":\"battle-run\",\"scenario\":\"{}\",\"seed\":{},\"controller\":\"baseline\",\"commands\":{},\"phase\":\"won\",\"state_hash\":\"{}\",\"replay_bytes\":{}}}",
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

fn standard_battle_run(
    scenario: &str,
    seed: u64,
    replay_out: Option<PathBuf>,
    json: bool,
) -> Result<(), CliError> {
    let mut instantiated =
        standard::instantiate(scenario, Some(seed)).map_err(|_| CliError::UnknownScenario)?;
    let header_identity = (
        instantiated.encounter(),
        instantiated.assembly_digest(),
        instantiated.master_seed(),
    );
    let mut trace = Vec::new();
    let battle = instantiated.battle_mut();
    while !battle.view().phase().is_terminal() {
        if trace.len() == MAX_STANDARD_COMMANDS {
            return Err(CliError::Simulation("Standard command budget exhausted"));
        }
        let command = if battle.view().phase() == BattlePhase::ReadyToAdvance {
            battle
                .advance_command()
                .ok_or(CliError::Simulation("battle has no action boundary"))?
        } else {
            let decision = battle
                .decision()
                .ok_or(CliError::Simulation("nonterminal battle has no decision"))?;
            match decision.kind() {
                DecisionKind::BattleStart => decision.legal_commands().first(),
                DecisionKind::NormalAction => decision
                    .legal_commands()
                    .iter()
                    .find(|command| matches!(command, Command::UseAbility { .. })),
                DecisionKind::PreparedAction => decision
                    .legal_commands()
                    .iter()
                    .find(|command| matches!(command, Command::CommitPreparedAction { .. })),
                DecisionKind::BattleChoice => None,
            }
            .cloned()
            .ok_or(CliError::Simulation(
                "Standard decision has no supported command",
            ))?
        };
        let resolution = battle
            .apply(command.clone())
            .map_err(|_| CliError::Simulation("offered command was rejected"))?;
        trace.push(BattleTraceEntry::new(command, resolution.state_hash()));
    }
    if battle.view().phase() != BattlePhase::Won {
        return Err(CliError::Simulation("Standard battle did not win"));
    }
    let header = standard_replay_header(header_identity, trace.len())?;
    let replay = encode_battle_trace(&header, &trace)?;
    if let Some(path) = &replay_out {
        fs::write(path, &replay).map_err(CliError::Io)?;
    }
    let final_hash = battle.state_hash().bytes();
    if json {
        println!(
            "{{\"kind\":\"battle-run\",\"scenario\":\"{}\",\"seed\":{},\"controller\":\"baseline\",\"commands\":{},\"phase\":\"won\",\"state_hash\":\"{}\",\"replay_bytes\":{}}}",
            json_escape(scenario),
            seed,
            trace.len(),
            hex(final_hash),
            replay.len()
        );
    } else {
        println!(
            "battle won scenario={} seed={} controller=baseline commands={} hash={} replay_bytes={}",
            scenario,
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
    if swarm_disaster::is_replay(&bytes) {
        return swarm_disaster::verify_replay(&bytes, json).map_err(CliError::SwarmDisaster);
    }
    if gold_gears::is_replay(&bytes) {
        return gold_gears::verify_replay(&bytes, json).map_err(CliError::GoldAndGears);
    }
    if universe::is_universe_replay(&bytes) {
        return universe::verify_replay(&bytes, json).map_err(CliError::Universe);
    }
    let decoded = decode_replay(&bytes).map_err(BattleReplayError::from)?;
    let seed = decoded.header().master_seed();
    let synthetic_components = battle_components(
        SYNTHETIC_STANDARD_CONFIG_DIGEST,
        "synthetic-baseline-controller",
        controller_digest(),
    )?;
    let synthetic = matches!(
        decoded.header().entry(),
        ReplayEntry::Battle {
            definition_id: 1, ..
        } if decoded.header().components() == &synthetic_components
            && decoded.header().environment().game_version() == "synthetic"
    );
    let (battle, expected_components) = if synthetic {
        (
            SyntheticStandardProfile
                .instantiate(seed)
                .create_battle()
                .map_err(|_| CliError::Simulation("synthetic replay battle construction failed"))?,
            synthetic_components,
        )
    } else {
        let (definition_id, spec_digest) = match decoded.header().entry() {
            ReplayEntry::Battle {
                definition_id,
                spec_digest,
            } => (*definition_id, *spec_digest),
            _ => return Err(CliError::UnknownScenario),
        };
        let scenario = standard::SCENARIOS
            .iter()
            .find(|(_, _, encounter)| *encounter == definition_id)
            .map(|(scenario, _, _)| *scenario)
            .ok_or(CliError::UnknownScenario)?;
        let components = battle_components(
            standard::CONFIG_DIGEST,
            "standard-baseline-controller",
            standard_controller_digest(),
        )?;
        let valid_identity = decoded.header().components() == &components
            && decoded.header().environment().game_version() == "4.4";
        if !valid_identity {
            return Err(CliError::UnknownScenario);
        }
        let instantiated =
            standard::instantiate(scenario, Some(seed)).map_err(|_| CliError::UnknownScenario)?;
        if EntrySpecDigest::new(instantiated.assembly_digest().bytes()) != spec_digest {
            return Err(CliError::UnknownScenario);
        }
        (instantiated.into_battle(), components)
    };
    let report = verify_battle_replay(&bytes, battle, &expected_components)?;
    if json {
        println!(
            "{{\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":{},\"phase\":\"{}\",\"state_hash\":\"{}\"}}",
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
    let environment = ReplayEnvironment::new("synthetic")?;
    let components = battle_components(
        scenario.config_digest(),
        "synthetic-baseline-controller",
        controller_digest(),
    )?;
    let entry = ReplayEntry::Battle {
        definition_id: scenario.encounter().get(),
        spec_digest: EntrySpecDigest::new(scenario.assembly_digest().bytes()),
    };
    ReplayHeader::new(
        environment,
        components,
        scenario.master_seed(),
        entry,
        battle_record_count(command_count)?,
    )
    .map_err(Into::into)
}

fn standard_replay_header(
    (encounter, spec_digest, master_seed): (
        starclock_combat::EncounterId,
        starclock_combat::AssemblyDigest,
        u64,
    ),
    command_count: usize,
) -> Result<ReplayHeader, CliError> {
    let environment = ReplayEnvironment::new("4.4")?;
    let components = battle_components(
        standard::CONFIG_DIGEST,
        "standard-baseline-controller",
        standard_controller_digest(),
    )?;
    ReplayHeader::new(
        environment,
        components,
        master_seed,
        ReplayEntry::Battle {
            definition_id: encounter.get(),
            spec_digest: EntrySpecDigest::new(spec_digest.bytes()),
        },
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
        BattlePhase::ReadyToAdvance => "ready-to-advance",
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

fn controller_digest() -> [u8; 32] {
    let mut digest = Sha256Sink::new();
    digest.write(CONTROLLER_DESCRIPTOR);
    digest.finalize().bytes()
}

fn standard_controller_digest() -> [u8; 32] {
    let mut digest = Sha256Sink::new();
    digest.write(STANDARD_CONTROLLER_DESCRIPTOR);
    digest.finalize().bytes()
}

fn battle_components(
    catalog_digest: [u8; 32],
    controller_id: &str,
    controller_digest: [u8; 32],
) -> Result<ConfigurationComponentSet, CliError> {
    ConfigurationComponentSet::new(vec![
        ConfigurationComponentIdentity::new(
            ConfigurationComponentKind::CombatCatalog,
            "combat-catalog",
            ComponentDigest::new(catalog_digest),
        )
        .map_err(|_| CliError::Simulation("invalid combat replay component"))?,
        ConfigurationComponentIdentity::new(
            ConfigurationComponentKind::Controller,
            controller_id,
            ComponentDigest::new(controller_digest),
        )
        .map_err(|_| CliError::Simulation("invalid controller replay component"))?,
    ])
    .map_err(|_| CliError::Simulation("invalid replay component set"))
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
    Mcp(starclock_mcp::stdio::StdioServeError),
    McpHttp(starclock_mcp::http::HttpServeError),
    Universe(universe::UniverseCliError),
    GoldAndGears(gold_gears::GoldAndGearsCliError),
    SwarmDisaster(swarm_disaster::SwarmDisasterCliError),
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
            Self::Mcp(_) | Self::McpHttp(_) => 8,
            Self::Universe(error) => error.exit_code(),
            Self::GoldAndGears(error) => error.exit_code(),
            Self::SwarmDisaster(error) => error.exit_code(),
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

impl From<ReplayError> for CliError {
    fn from(value: ReplayError) -> Self {
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
            Self::Mcp(error) => write!(formatter, "MCP service error: {error}"),
            Self::McpHttp(error) => write!(formatter, "MCP service error: {error}"),
            Self::Universe(error) => error.fmt(formatter),
            Self::GoldAndGears(error) => error.fmt(formatter),
            Self::SwarmDisaster(error) => error.fmt(formatter),
        }
    }
}
