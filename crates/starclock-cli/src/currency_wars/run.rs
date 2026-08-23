use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
};
use starclock_ai::{
    CurrencyWarsBaselineController, CurrencyWarsBaselineRunReport, CurrencyWarsReplayIdentity,
};
use starclock_data::{
    currency_wars::load_currency_wars_catalog_candidate, load_currency_wars_battle_resources,
};
use starclock_mode_currency_wars::{
    CurrencyWarsBattleAssembler, CurrencyWarsDeployment, CurrencyWarsEntryState,
    CurrencyWarsGambit, CurrencyWarsPosition, CurrencyWarsPositionKind, CurrencyWarsRoleId,
    CurrencyWarsRoleState, CurrencyWarsRoster, CurrencyWarsRun, CurrencyWarsRunDefinition,
    CurrencyWarsRunSetup,
};

use super::{CurrencyWarsCliError, io, replay, simulation, usage};

const DEFINITION_ID: u32 = 31;
const ACTIVITY_INSTANCE: u64 = 1;
const INITIAL_ROLES: [u32; 4] = [1301, 1306, 1014, 1015];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum CliGambit {
    Standard = 1,
    Overclock = 2,
}

impl CliGambit {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Overclock => "overclock",
        }
    }

    const fn domain(self) -> CurrencyWarsGambit {
        match self {
            Self::Standard => CurrencyWarsGambit::Standard,
            Self::Overclock => CurrencyWarsGambit::Overclock,
        }
    }

    fn parse(value: &str) -> Result<Self, CurrencyWarsCliError> {
        match value {
            "standard" => Ok(Self::Standard),
            "overclock" => Ok(Self::Overclock),
            _ => Err(usage("--gambit requires standard or overclock")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RunOptions {
    pub(super) route: u32,
    pub(super) difficulty: u32,
    pub(super) gambit: CliGambit,
    pub(super) seed: u64,
    pub(super) replay_out: Option<PathBuf>,
    pub(super) json: bool,
}

impl RunOptions {
    pub(super) fn parse(args: &[String]) -> Result<Self, CurrencyWarsCliError> {
        let mut route = None;
        let mut difficulty = None;
        let mut gambit = None;
        let mut seed = None;
        let mut replay_out = None;
        let mut json = false;
        let mut index = 0;
        while index < args.len() {
            let next = || args.get(index + 1).map(String::as_str);
            match args[index].as_str() {
                "--route" if route.is_none() => {
                    route = Some(parse_u32(next(), "--route requires an unsigned integer")?);
                    index += 1;
                }
                "--difficulty" if difficulty.is_none() => {
                    difficulty = Some(parse_u32(
                        next(),
                        "--difficulty requires an unsigned integer",
                    )?);
                    index += 1;
                }
                "--gambit" if gambit.is_none() => {
                    gambit = Some(CliGambit::parse(
                        next().ok_or_else(|| usage("--gambit requires a value"))?,
                    )?);
                    index += 1;
                }
                "--seed" if seed.is_none() => {
                    seed = Some(parse_u64(next(), "--seed requires an unsigned integer")?);
                    index += 1;
                }
                "--controller" if next() == Some("baseline") => index += 1,
                "--replay-out" if replay_out.is_none() => {
                    replay_out = Some(PathBuf::from(
                        next().ok_or_else(|| usage("--replay-out requires a path"))?,
                    ));
                    index += 1;
                }
                "--json" if !json => json = true,
                _ => return Err(usage("unknown or duplicate currency-wars run option")),
            }
            index += 1;
        }
        Ok(Self {
            route: route.ok_or_else(|| usage("run requires --route ID"))?,
            difficulty: difficulty.ok_or_else(|| usage("run requires --difficulty ID"))?,
            gambit: gambit.ok_or_else(|| usage("run requires --gambit NAME"))?,
            seed: seed.ok_or_else(|| usage("run requires --seed U64"))?,
            replay_out,
            json,
        })
    }

    pub(super) fn from_replay(route: u32, difficulty: u32, gambit: CliGambit, seed: u64) -> Self {
        Self {
            route,
            difficulty,
            gambit,
            seed,
            replay_out: None,
            json: false,
        }
    }
}

pub(super) struct Execution {
    pub(super) report: CurrencyWarsBaselineRunReport,
    pub(super) replay_identity: CurrencyWarsReplayIdentity,
}

pub(super) fn command(args: &[String]) -> Result<(), CurrencyWarsCliError> {
    let options = RunOptions::parse(args)?;
    let execution = execute(&options)?;
    let bytes = replay::encode(&options, &execution)?;
    if let Some(path) = &options.replay_out {
        std::fs::write(path, &bytes).map_err(io)?;
    }
    let report = &execution.report;
    let battle_commands = report
        .battles()
        .iter()
        .map(|battle| battle.trace().len())
        .sum::<usize>();
    if options.json {
        println!(
            "{{\"kind\":\"currency-wars-run\",\"route\":{},\"difficulty\":{},\"gambit\":\"{}\",\"seed\":{},\"controller\":\"baseline\",\"activity_actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\",\"replay_bytes\":{}}}",
            options.route,
            options.difficulty,
            options.gambit.name(),
            options.seed,
            report.activity_steps(),
            report.battles().len(),
            battle_commands,
            super::hex(report.final_state_hash().bytes()),
            bytes.len(),
        );
    } else {
        println!(
            "currency-wars completed route={} difficulty={} gambit={} seed={} controller=baseline activity_actions={} nested_battles={} battle_commands={} hash={} replay_bytes={}",
            options.route,
            options.difficulty,
            options.gambit.name(),
            options.seed,
            report.activity_steps(),
            report.battles().len(),
            battle_commands,
            super::hex(report.final_state_hash().bytes()),
            bytes.len(),
        );
    }
    Ok(())
}

pub(super) fn execute(options: &RunOptions) -> Result<Execution, CurrencyWarsCliError> {
    let candidate = load_currency_wars_catalog_candidate().map_err(super::configuration)?;
    let catalog_identity = candidate.identity().clone();
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(DEFINITION_ID).expect("static definition ID is non-zero"),
        ActivityDefinitionDigest::new(catalog_identity.content_digest().bytes())
            .expect("validated content digest is non-zero"),
        ActivityConfigDigest::new(catalog_identity.configuration_digest().bytes())
            .expect("validated configuration digest is non-zero"),
    );
    let catalog = Arc::new(candidate.into_catalog());
    let route = catalog
        .routes()
        .iter()
        .find(|candidate| candidate.id.get() == options.route)
        .ok_or_else(|| usage("unknown Currency Wars route"))?;
    let difficulty = catalog
        .difficulties()
        .iter()
        .find(|candidate| candidate.source_id == options.difficulty)
        .ok_or_else(|| usage("unknown Currency Wars difficulty"))?;
    let roles = INITIAL_ROLES.map(|raw| {
        CurrencyWarsRoleState::new(
            CurrencyWarsRoleId::new(raw).expect("static role ID is non-zero"),
            1,
        )
        .expect("released one-star role state is valid")
    });
    let roster = CurrencyWarsRoster::new(&catalog, roles.into_iter().map(|role| (role, 1)))
        .map_err(|_| simulation("baseline roster is unavailable for this configuration"))?;
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        4,
        (1_u8..=4).map(|index| {
            (
                CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, index)
                    .expect("front position is valid"),
                roles[usize::from(index - 1)],
            )
        }),
    )
    .map_err(|_| simulation("baseline deployment is unavailable for this configuration"))?;
    let definition = Arc::new(
        CurrencyWarsRunDefinition::new(
            identity,
            Arc::clone(&catalog),
            route.id,
            difficulty.source_id,
            options.gambit.domain(),
            CurrencyWarsEntryState::new(21, true, 9),
            CurrencyWarsRunSetup {
                initial_gold: 0,
                initial_team_level: 4,
                initial_experience: 0,
                roster,
                deployment,
                enemy_affix_ids: Box::new([]),
                owned_builds: BTreeMap::new(),
            },
        )
        .map_err(|_| simulation("Currency Wars run definition could not be compiled"))?,
    );
    let mut run = CurrencyWarsRun::start(
        definition,
        ActivityInstanceId::new(ACTIVITY_INSTANCE).expect("static instance ID is non-zero"),
        ActivityMasterSeed::from_u64(options.seed),
    )
    .map_err(|_| simulation("Currency Wars run could not be started"))?;
    let resources = Arc::new(
        load_currency_wars_battle_resources(&catalog)
            .map_err(|_| simulation("Currency Wars battle resources could not be loaded"))?,
    );
    let replay_identity = CurrencyWarsReplayIdentity::new(
        catalog_identity.schema_digest().bytes(),
        catalog_identity.configuration_digest().bytes(),
        catalog_identity.content_digest().bytes(),
        resources.digest(),
        resources.combat().digest().bytes(),
    );
    let mut assembler = CurrencyWarsBattleAssembler::new(resources, 16)
        .map_err(|_| simulation("Currency Wars battle assembler could not be created"))?;
    let report = CurrencyWarsBaselineController::default()
        .run_to_terminal(&mut run, &mut assembler)
        .map_err(|_| simulation("Currency Wars baseline run failed"))?;
    if report.activity_steps() as usize != report.activity_trace().len() {
        return Err(simulation("Currency Wars activity trace is incomplete"));
    }
    Ok(Execution {
        report,
        replay_identity,
    })
}

fn parse_u32(value: Option<&str>, message: &str) -> Result<u32, CurrencyWarsCliError> {
    value
        .ok_or_else(|| usage(message))?
        .parse()
        .map_err(|_| usage(message))
}

fn parse_u64(value: Option<&str>, message: &str) -> Result<u64, CurrencyWarsCliError> {
    value
        .ok_or_else(|| usage(message))?
        .parse()
        .map_err(|_| usage(message))
}
