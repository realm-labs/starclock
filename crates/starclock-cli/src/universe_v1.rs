use std::{fmt, fs, path::PathBuf, sync::Arc};

use starclock_activity::{
    ActivityInstanceId, ActivityMasterSeed, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, UnitDefinitionId, UnitLevel, catalog::action::AbilityKind,
};
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    baseline_runner::{StandardUniverseBaselinePolicy, StandardUniverseBaselineRunner},
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{UniverseBattleMaterializer, UniverseBattleRoster},
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    id::WorldId,
    nested_battle_executor::{
        UNIVERSE_NESTED_BATTLE_EXECUTOR_REVISION, UniverseNestedBattleExecutor,
    },
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
    universe_replay_v2::{
        encode_standard_universe_trace_v2, record_baseline_run_v2, standard_universe_component_set,
        standard_universe_header_v2, verify_standard_universe_replay_v2,
    },
};
use starclock_replay::{
    codec::CanonicalSink, digest::Sha256Sink, format::ReplayEntry, format_v2::ReplayCompatibilityV2,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const PROFILE_PREFIX: &str = "standard-universe-v1/world-";
const CLI_REVISION: &str = "starclock-cli-universe-v2";
const BATTLE_EXECUTOR: &str = "starclock-combat-nested-v1";

pub fn config_validate(args: &[String]) -> Result<(), UniverseCliError> {
    let json = json_only(args)?;
    let catalog = catalog()?;
    let digest = universe_bundle_digest();
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-config-validation\",\"valid\":true,\"bundle_sha256\":\"{}\",\"worlds\":{},\"difficulties\":{},\"paths\":{},\"blessings\":{},\"curios\":{}}}",
            hex(digest),
            catalog.worlds().len(),
            catalog.difficulties().len(),
            catalog.paths().len(),
            catalog.blessings().len(),
            catalog.curios().len(),
        );
    } else {
        println!(
            "universe config valid bundle_sha256={} worlds={} difficulties={} paths={} blessings={} curios={}",
            hex(digest),
            catalog.worlds().len(),
            catalog.difficulties().len(),
            catalog.paths().len(),
            catalog.blessings().len(),
            catalog.curios().len(),
        );
    }
    Ok(())
}

pub fn coverage(args: &[String]) -> Result<(), UniverseCliError> {
    let json = json_only(args)?;
    let catalog = catalog()?;
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-coverage\",\"goal_id\":\"standard-universe-runtime-v1\",\"content_records\":2201,\"rule_bindings\":786,\"fixtures\":78,\"worlds\":{},\"difficulties\":{},\"paths\":{},\"encounter_groups\":{}}}",
            catalog.worlds().len(),
            catalog.difficulties().len(),
            catalog.paths().len(),
            catalog.encounter_groups().len(),
        );
    } else {
        println!(
            "universe coverage goal=standard-universe-runtime-v1 content=2201 rules=786 fixtures=78 worlds={} difficulties={} paths={} encounter_groups={}",
            catalog.worlds().len(),
            catalog.difficulties().len(),
            catalog.paths().len(),
            catalog.encounter_groups().len(),
        );
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), UniverseCliError> {
    let options = RunOptions::parse(args)?;
    let context = context(options.world, options.difficulty_index, options.seed)?;
    let mut activity = context.activity;
    let header = standard_universe_header_v2(
        context.compatibility.clone(),
        context.components.clone(),
        options.seed,
        &activity,
        &context.profile_id,
    )
    .map_err(|_| UniverseCliError::Replay)?;
    let mut executor =
        UniverseNestedBattleExecutor::new(Arc::clone(&context.materialized_combat_catalog));
    let recorded = record_baseline_run_v2(
        &mut activity,
        &StandardUniverseBaselinePolicy::default(),
        &mut executor,
    )
    .map_err(|_| UniverseCliError::Simulation)?;
    let replay = encode_standard_universe_trace_v2(&header, &recorded)
        .map_err(|_| UniverseCliError::Replay)?;
    if let Some(path) = &options.replay_out {
        fs::write(path, &replay).map_err(UniverseCliError::Io)?;
    }
    let report = recorded.report();
    if options.json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-run\",\"world\":{},\"difficulty_index\":{},\"seed\":{},\"controller\":\"baseline\",\"battle_executor\":\"{BATTLE_EXECUTOR}\",\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\",\"replay_bytes\":{}}}",
            options.world,
            options.difficulty_index,
            options.seed,
            report.steps().len(),
            recorded.battles().len(),
            recorded
                .battles()
                .iter()
                .map(|battle| battle.trace().len())
                .sum::<usize>(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
        );
    } else {
        println!(
            "universe completed world={} difficulty_index={} seed={} controller=baseline battle_executor={BATTLE_EXECUTOR} actions={} nested_battles={} battle_commands={} hash={} replay_bytes={}",
            options.world,
            options.difficulty_index,
            options.seed,
            report.steps().len(),
            recorded.battles().len(),
            recorded
                .battles()
                .iter()
                .map(|battle| battle.trace().len())
                .sum::<usize>(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
        );
    }
    Ok(())
}

pub fn is_universe_replay_v2(bytes: &[u8]) -> bool {
    starclock_replay::format_v2::decode_replay_v2(bytes)
        .is_ok_and(|replay| is_universe_entry(replay.header().entry()))
}

fn is_universe_entry(entry: &ReplayEntry) -> bool {
    matches!(entry, ReplayEntry::Activity { profile_id, .. } if profile_id.starts_with(PROFILE_PREFIX))
}

pub fn verify_replay(bytes: &[u8], json: bool) -> Result<(), UniverseCliError> {
    let decoded = starclock_replay::format_v2::decode_replay_v2(bytes)
        .map_err(|_| UniverseCliError::Replay)?;
    let (world, difficulty_index, profile_id) = parse_profile(decoded.header().entry())?;
    let seed = decoded.header().master_seed();
    let context = context(world, difficulty_index, seed)?;
    if context.profile_id != profile_id {
        return Err(UniverseCliError::Replay);
    }
    let report = verify_standard_universe_replay_v2(
        bytes,
        context.activity,
        context.materialized_combat_catalog,
        &context.components,
        &context.compatibility,
        &profile_id,
    )
    .map_err(|_| UniverseCliError::Replay)?;
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"replay-verify\",\"entry\":\"standard-universe\",\"actions\":{},\"nested_battles\":{},\"battle_commands\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\"}}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    } else {
        println!(
            "universe replay verified actions={} nested_battles={} battle_commands={} terminal=completed hash={}",
            report.action_count(),
            report.battle_count(),
            report.battle_command_count(),
            hex(report.final_state_hash().bytes()),
        );
    }
    Ok(())
}

struct RunOptions {
    world: u32,
    difficulty_index: usize,
    seed: u64,
    replay_out: Option<PathBuf>,
    json: bool,
}

impl RunOptions {
    fn parse(args: &[String]) -> Result<Self, UniverseCliError> {
        let mut world = None;
        let mut difficulty_index = None;
        let mut seed = None;
        let mut replay_out = None;
        let mut json = false;
        let mut index = 0;
        while index < args.len() {
            let value = |offset: usize| args.get(index + offset).map(String::as_str);
            match args[index].as_str() {
                "--world" if world.is_none() => {
                    world = Some(parse(value(1), "--world")?);
                    index += 1;
                }
                "--difficulty-index" if difficulty_index.is_none() => {
                    difficulty_index = Some(parse(value(1), "--difficulty-index")?);
                    index += 1;
                }
                "--seed" if seed.is_none() => {
                    seed = Some(parse(value(1), "--seed")?);
                    index += 1;
                }
                "--replay-out" if replay_out.is_none() => {
                    replay_out = Some(PathBuf::from(value(1).ok_or(UniverseCliError::Usage)?));
                    index += 1;
                }
                "--controller" if value(1) == Some("baseline") => index += 1,
                "--json" if !json => json = true,
                _ => return Err(UniverseCliError::Usage),
            }
            index += 1;
        }
        Ok(Self {
            world: world.ok_or(UniverseCliError::Usage)?,
            difficulty_index: difficulty_index.ok_or(UniverseCliError::Usage)?,
            seed: seed.ok_or(UniverseCliError::Usage)?,
            replay_out,
            json,
        })
    }
}

fn parse<T: core::str::FromStr>(value: Option<&str>, _name: &str) -> Result<T, UniverseCliError> {
    value
        .ok_or(UniverseCliError::Usage)?
        .parse()
        .map_err(|_| UniverseCliError::Usage)
}

struct RunContext {
    profile_id: String,
    activity: starclock_mode_universe::runtime::StandardUniverseActivity,
    materialized_combat_catalog: Arc<starclock_combat::catalog::CombatCatalog>,
    components: starclock_replay::component::ConfigurationComponentSet,
    compatibility: ReplayCompatibilityV2,
}

fn context(
    world_raw: u32,
    difficulty_index: usize,
    seed: u64,
) -> Result<RunContext, UniverseCliError> {
    let catalog = catalog()?;
    let world_id = WorldId::new(world_raw).ok_or(UniverseCliError::UnknownEntry)?;
    let world = catalog
        .world(world_id)
        .ok_or(UniverseCliError::UnknownEntry)?;
    let difficulty = *world
        .difficulties()
        .get(difficulty_index)
        .ok_or(UniverseCliError::UnknownEntry)?;
    let (roster, lock) = participants(&catalog)?;
    let contributions = initial_contributions(&catalog)?;
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions)
        .map_err(|_| UniverseCliError::Configuration)?;
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world_id, difficulty, lock, vec![])
                .with_encounter_overlay(materialized.overlay().clone()),
        )
        .map_err(|_| UniverseCliError::Configuration)?;
    let controller_digest = controller_digest();
    let components = standard_universe_component_set(
        &catalog,
        &compiled,
        &materialized,
        "baseline-controller",
        StandardUniverseBaselineRunner::REVISION,
        controller_digest,
    )
    .map_err(|_| UniverseCliError::Configuration)?;
    let compatibility = ReplayCompatibilityV2::new(
        catalog.identity().game_version(),
        starclock_combat::NUMERIC_POLICY_REVISION,
        starclock_combat::rng::RNG_ALGORITHM_REVISION,
        starclock_activity::ACTIVITY_STATE_HASH_REVISION,
    )
    .map_err(|_| UniverseCliError::Configuration)?;
    let instance = ActivityInstanceId::new(seed.checked_add(1).ok_or(UniverseCliError::Usage)?)
        .ok_or(UniverseCliError::Usage)?;
    let activity = compiled
        .start_standard(instance, ActivityMasterSeed::from_u64(seed))
        .map_err(|_| UniverseCliError::Simulation)?
        .into_activity();
    Ok(RunContext {
        profile_id: format!("{PROFILE_PREFIX}{world_raw}/difficulty-{difficulty_index}"),
        activity,
        materialized_combat_catalog: Arc::clone(materialized.combat_catalog()),
        components,
        compatibility,
    })
}

fn catalog() -> Result<Arc<UniverseCatalog>, UniverseCliError> {
    let core =
        starclock_data::catalog::load(CORE_BUNDLE).map_err(|_| UniverseCliError::Configuration)?;
    UniverseCatalog::load(UNIVERSE_BUNDLE, core).map_err(|_| UniverseCliError::Configuration)
}

fn participants(
    catalog: &UniverseCatalog,
) -> Result<(UniverseBattleRoster, ParticipantLock), UniverseCliError> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        starclock_activity::ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .ok_or(UniverseCliError::Configuration)?;
    let mut lock_entries = Vec::new();
    let mut combatants = Vec::new();
    for index in 0_u8..4 {
        let form =
            UnitDefinitionId::new(u32::from(index) + 1).ok_or(UniverseCliError::Configuration)?;
        let unit = catalog
            .simulation_catalog()
            .combat_catalog()
            .unit(form)
            .ok_or(UniverseCliError::Configuration)?;
        let basic = unit
            .abilities()
            .iter()
            .copied()
            .find(|ability| {
                catalog
                    .simulation_catalog()
                    .combat_catalog()
                    .ability(*ability)
                    .and_then(|definition| definition.action())
                    .is_some_and(|action| action.kind() == AbilityKind::Basic)
            })
            .ok_or(UniverseCliError::Configuration)?;
        let combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).ok_or(UniverseCliError::Configuration)?,
            Hp::new(100_000).map_err(|_| UniverseCliError::Configuration)?,
            Speed::from_scaled(200_000_000).map_err(|_| UniverseCliError::Configuration)?,
            ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new())
                .map_err(|_| UniverseCliError::Configuration)?,
            CombatantSpecDigest::new([index + 1; 32]).ok_or(UniverseCliError::Configuration)?,
        )
        .map_err(|_| UniverseCliError::Configuration)?
        .with_base_attack_defense(
            StatValue::from_scaled(1_000_000_000).map_err(|_| UniverseCliError::Configuration)?,
            StatValue::from_scaled(1_000_000_000).map_err(|_| UniverseCliError::Configuration)?,
        )
        .with_energy(
            Energy::ZERO,
            Energy::from_scaled(100_000_000).map_err(|_| UniverseCliError::Configuration)?,
        )
        .map_err(|_| UniverseCliError::Configuration)?;
        let participant =
            ParticipantId::new(u32::from(index) + 1).ok_or(UniverseCliError::Configuration)?;
        lock_entries.push(
            ParticipantLockEntry::new(
                participant,
                0,
                index,
                form,
                OpaqueParticipantBuild::new(
                    combatant.digest(),
                    BuildDigest::new([index + 17; 32]).ok_or(UniverseCliError::Configuration)?,
                    "universe-cli-resolved-build-v1",
                    ParticipantSourceKind::FixedResolved,
                )
                .map_err(|_| UniverseCliError::Configuration)?,
            )
            .map_err(|_| UniverseCliError::Configuration)?,
        );
        combatants.push((participant, combatant));
    }
    let lock =
        ParticipantLock::seal(policy, lock_entries).map_err(|_| UniverseCliError::Configuration)?;
    let roster = UniverseBattleRoster::new(&lock, combatants)
        .map_err(|_| UniverseCliError::Configuration)?;
    Ok((roster, lock))
}

fn initial_contributions(
    catalog: &Arc<UniverseCatalog>,
) -> Result<UniverseBattleContributionSet, UniverseCliError> {
    let path_definition = catalog
        .paths()
        .first()
        .ok_or(UniverseCliError::Configuration)?;
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .map_err(|_| UniverseCliError::Configuration)?
        .contributions_from_owned(&[])
        .map_err(|_| UniverseCliError::Configuration)?;
    let path = PathRuntimeCatalog::compile(catalog)
        .map_err(|_| UniverseCliError::Configuration)?
        .contributions(path_definition.id(), &blessings, &[])
        .map_err(|_| UniverseCliError::Configuration)?;
    let curios = CurioRuntimeCatalog::compile(catalog)
        .map_err(|_| UniverseCliError::Configuration)?
        .contributions_from_owned(&[], &[], &[])
        .map_err(|_| UniverseCliError::Configuration)?;
    let abilities = RunRuntimeCatalog::compile(catalog)
        .map_err(|_| UniverseCliError::Configuration)?
        .ability_contributions(&[])
        .map_err(|_| UniverseCliError::Configuration)?;
    let projection = AbilityRuntimeCatalog::compile(catalog)
        .map_err(|_| UniverseCliError::Configuration)?
        .project(
            &[],
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::BattleStart,
                0,
                false,
            ),
        )
        .map_err(|_| UniverseCliError::Configuration)?;
    UniverseBattleContributionCompiler::compile(Arc::clone(catalog))
        .map_err(|_| UniverseCliError::Configuration)?
        .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
        .map_err(|_| UniverseCliError::Configuration)
}

fn parse_profile(entry: &ReplayEntry) -> Result<(u32, usize, String), UniverseCliError> {
    let ReplayEntry::Activity { profile_id, .. } = entry else {
        return Err(UniverseCliError::Replay);
    };
    let suffix = profile_id
        .strip_prefix(PROFILE_PREFIX)
        .ok_or(UniverseCliError::Replay)?;
    let (world, difficulty) = suffix
        .split_once("/difficulty-")
        .ok_or(UniverseCliError::Replay)?;
    Ok((
        parse(Some(world), "world")?,
        parse(Some(difficulty), "difficulty")?,
        profile_id.to_string(),
    ))
}

fn json_only(args: &[String]) -> Result<bool, UniverseCliError> {
    match args {
        [] => Ok(false),
        [flag] if flag == "--json" => Ok(true),
        _ => Err(UniverseCliError::Usage),
    }
}

fn universe_bundle_digest() -> [u8; 32] {
    let mut value = Sha256Sink::new();
    value.write(UNIVERSE_BUNDLE);
    value.finalize().bytes()
}
fn controller_digest() -> [u8; 32] {
    let mut value = Sha256Sink::new();
    value.write(StandardUniverseBaselineRunner::REVISION.as_bytes());
    value.write(&[0]);
    value.write(UNIVERSE_NESTED_BATTLE_EXECUTOR_REVISION.as_bytes());
    value.finalize().bytes()
}
fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug)]
pub enum UniverseCliError {
    Usage,
    Configuration,
    UnknownEntry,
    Simulation,
    Replay,
    Io(std::io::Error),
}
impl UniverseCliError {
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Usage => 2,
            Self::Configuration => 3,
            Self::Replay => 4,
            Self::UnknownEntry => 5,
            Self::Simulation => 6,
            Self::Io(_) => 7,
        }
    }
}
impl fmt::Display for UniverseCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => f.write_str("universe usage error"),
            Self::Configuration => f.write_str("universe configuration error"),
            Self::UnknownEntry => f.write_str("unknown universe world or difficulty"),
            Self::Simulation => f.write_str("universe simulation error"),
            Self::Replay => f.write_str("universe replay error"),
            Self::Io(error) => write!(f, "universe I/O error: {error}"),
        }
    }
}
