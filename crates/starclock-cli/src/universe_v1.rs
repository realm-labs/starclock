use std::{fmt, fs, path::PathBuf, sync::Arc};

use starclock_activity::{
    ActivityBattleResultContract, ActivityInstanceId, ActivityMasterSeed,
    ActivityParticipantCarryDefinition, BattleBinding, BattleOutcome, BattleResult, BuildDigest,
    EnergyCarryPolicy, EventDigest, HpCarryPolicy, LifeCarryPolicy, LoadoutLockScope,
    OpaqueParticipantBuild, ParticipantBattleState, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, PresenceCarryPolicy,
    ProjectedValue, ProjectionField, ProjectionId, TechniqueContributionDigest,
};
use starclock_combat::{
    AbilityId, BattleSpec, BattleSpecDigest, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, LifeState, ParticipantSource,
    ParticipantSpec, PresenceState, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
};
use starclock_mode_universe::{
    baseline_runner::{StandardUniverseBaselinePolicy, StandardUniverseBaselineRunner},
    battle_overlay::{UniverseEncounterBattleBinding, UniverseEncounterOverlay},
    catalog::UniverseCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    id::WorldId,
    universe_replay::{
        encode_standard_universe_trace, record_baseline_run, replay_entry_for,
        verify_standard_universe_replay,
    },
};
use starclock_replay::{
    codec::CanonicalSink,
    digest::{ConfigBundleDigest, ControllerDigest, Sha256Sink},
    format::{ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const PROFILE_PREFIX: &str = "standard-universe-v1/world-";
const CLI_REVISION: &str = "starclock-cli-universe-v1";
const RULES_REVISION: &str = "standard-universe-rules-v1";
const DATA_REVISION: &str = "standard-universe-data-v4.4";

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
    let header = replay_header(&activity, &context.profile_id, options.seed)?;
    let mut executor = |handoff: &starclock_activity::ActivityBattleHandoff| {
        reference_won_result(handoff.identity())
    };
    let recorded = record_baseline_run(
        &mut activity,
        &StandardUniverseBaselinePolicy::default(),
        &mut executor,
    )
    .map_err(|_| UniverseCliError::Simulation)?;
    let replay = encode_standard_universe_trace(&header, recorded.trace())
        .map_err(|_| UniverseCliError::Replay)?;
    if let Some(path) = &options.replay_out {
        fs::write(path, &replay).map_err(UniverseCliError::Io)?;
    }
    let report = recorded.report();
    if options.json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"universe-run\",\"world\":{},\"difficulty_index\":{},\"seed\":{},\"controller\":\"baseline\",\"battle_executor\":\"verified-reference-projection-v1\",\"actions\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\",\"replay_bytes\":{}}}",
            options.world,
            options.difficulty_index,
            options.seed,
            report.steps().len(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
        );
    } else {
        println!(
            "universe completed world={} difficulty_index={} seed={} controller=baseline battle_executor=verified-reference-projection-v1 actions={} hash={} replay_bytes={}",
            options.world,
            options.difficulty_index,
            options.seed,
            report.steps().len(),
            hex(report.final_state_hash().bytes()),
            replay.len(),
        );
    }
    Ok(())
}

pub fn is_universe_replay(header: &ReplayHeader) -> bool {
    matches!(header.entry(), ReplayEntry::Activity { profile_id, .. } if profile_id.starts_with(PROFILE_PREFIX))
}

pub fn verify_replay(bytes: &[u8], json: bool) -> Result<(), UniverseCliError> {
    let decoded =
        starclock_replay::format::decode_replay(bytes).map_err(|_| UniverseCliError::Replay)?;
    let (world, difficulty_index, profile_id) = parse_profile(decoded.header())?;
    let seed = decoded.header().master_seed();
    let context = context(world, difficulty_index, seed)?;
    if context.profile_id != profile_id {
        return Err(UniverseCliError::Replay);
    }
    let report = verify_standard_universe_replay(bytes, context.activity, &profile_id)
        .map_err(|_| UniverseCliError::Replay)?;
    if json {
        println!(
            "{{\"schema_revision\":\"{CLI_REVISION}\",\"kind\":\"replay-verify\",\"entry\":\"standard-universe\",\"actions\":{},\"nested_battles\":{},\"terminal\":\"completed\",\"state_hash\":\"{}\"}}",
            report.action_count(),
            report.nested_battle_count(),
            hex(report.final_state_hash().bytes()),
        );
    } else {
        println!(
            "universe replay verified actions={} nested_battles={} terminal=completed hash={}",
            report.action_count(),
            report.nested_battle_count(),
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
    let lock = participants()?;
    let overlay = overlay(&catalog, &lock)?;
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world_id, difficulty, lock, vec![])
                .with_encounter_overlay(overlay),
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
    })
}

fn catalog() -> Result<Arc<UniverseCatalog>, UniverseCliError> {
    let core =
        starclock_data::catalog::load(CORE_BUNDLE).map_err(|_| UniverseCliError::Configuration)?;
    UniverseCatalog::load(UNIVERSE_BUNDLE, core).map_err(|_| UniverseCliError::Configuration)
}

fn participants() -> Result<ParticipantLock, UniverseCliError> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        starclock_activity::ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .ok_or(UniverseCliError::Configuration)?;
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(byte)).unwrap(),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).unwrap(),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([byte; 32]).unwrap(),
                    BuildDigest::new([byte + 32; 32]).unwrap(),
                    "universe-cli-reference-build-v1",
                    ParticipantSourceKind::CompiledBuild,
                )
                .unwrap(),
            )
            .unwrap()
        })
        .collect();
    ParticipantLock::seal(policy, entries).map_err(|_| UniverseCliError::Configuration)
}

fn overlay(
    catalog: &UniverseCatalog,
    lock: &ParticipantLock,
) -> Result<UniverseEncounterOverlay, UniverseCliError> {
    let contract = Arc::new(
        ActivityBattleResultContract::new(
            Arc::new(
                starclock_activity::BattleResultProjection::new(
                    ProjectionId::new(1).unwrap(),
                    vec![
                        ProjectionField::Outcome,
                        ProjectionField::FinalStateHash,
                        ProjectionField::EventDigest,
                        ProjectionField::TerminalFault,
                        ProjectionField::ParticipantState(ParticipantId::new(1).unwrap()),
                        ProjectionField::ParticipantState(ParticipantId::new(2).unwrap()),
                        ProjectionField::ParticipantState(ParticipantId::new(3).unwrap()),
                        ProjectionField::ParticipantState(ParticipantId::new(4).unwrap()),
                    ],
                )
                .unwrap(),
            ),
            (1..=4)
                .map(|raw| {
                    ActivityParticipantCarryDefinition::new(
                        ParticipantId::new(raw).unwrap(),
                        HpCarryPolicy::CarryExact,
                        EnergyCarryPolicy::CarryExact,
                        LifeCarryPolicy::CarryExact,
                        PresenceCarryPolicy::CarryExact,
                    )
                })
                .collect(),
            vec![],
        )
        .unwrap(),
    );
    let bindings = catalog
        .encounter_groups()
        .iter()
        .flat_map(|group| group.members())
        .map(|member| {
            let preparation = Arc::new(
                starclock_activity::EncounterPreparationDefinition::new(
                    starclock_activity::ActivityOptionId::new(10).unwrap(),
                    starclock_activity::EncounterInitiativePolicy::PlayerControlled,
                    lock.digest(),
                    0,
                    vec![],
                    vec![starclock_activity::PreparedBattleVariant::new(
                        vec![],
                        TechniqueContributionDigest::new([0x44; 32]).unwrap(),
                        BattleBinding::new(
                            battle_spec(member.id().get()),
                            "universe-cli-reference",
                            "universe-battle-spec-v1",
                            lock.digest(),
                        )
                        .unwrap(),
                    )],
                )
                .unwrap(),
            );
            UniverseEncounterBattleBinding::new(member.id(), preparation, Arc::clone(&contract))
        })
        .collect();
    UniverseEncounterOverlay::new(bindings).map_err(|_| UniverseCliError::Configuration)
}

fn battle_spec(member: u32) -> BattleSpec {
    let mut entries = (0_u8..4)
        .map(|index| {
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(index).unwrap(),
                ParticipantSource::Player,
                combatant(20_001 + u32::from(index), index + 1),
            )
        })
        .collect::<Vec<_>>();
    let enemy = 30_000 + member;
    entries.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(enemy).unwrap()),
        combatant(enemy, u8::try_from(member).unwrap()),
    ));
    BattleSpec::new(
        "universe-cli-reference-rules-v1",
        BattleSpecDigest::new([u8::try_from(member).unwrap(); 32]).unwrap(),
        EncounterId::new(member).unwrap(),
        entries,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).unwrap(),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![AbilityId::new(form).unwrap()], vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
    .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
    .unwrap()
}

fn reference_won_result(identity: starclock_activity::BattleResultIdentity) -> BattleResult {
    let mut values = vec![
        ProjectedValue::Outcome(BattleOutcome::Won),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
        ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
        ProjectedValue::TerminalFault(None),
    ];
    values.extend((1_u32..=4).map(|raw| {
        ProjectedValue::ParticipantState(
            ParticipantBattleState::new(
                ParticipantId::new(raw).unwrap(),
                Hp::new(900).unwrap(),
                Hp::new(1_000).unwrap(),
                Energy::from_scaled(50_000_000).unwrap(),
                Energy::from_scaled(100_000_000).unwrap(),
                LifeState::Alive,
                PresenceState::Present,
            )
            .unwrap(),
        )
    }));
    BattleResult::seal(identity, values)
}

fn replay_header(
    activity: &starclock_mode_universe::runtime::StandardUniverseActivity,
    profile: &str,
    seed: u64,
) -> Result<ReplayHeader, UniverseCliError> {
    ReplayHeader::new(
        ReplayIdentity::new(
            "4.4",
            RULES_REVISION,
            DATA_REVISION,
            ConfigBundleDigest::new(
                activity
                    .graph()
                    .definition()
                    .identity()
                    .config_digest()
                    .bytes(),
            ),
            starclock_combat::NUMERIC_POLICY_REVISION,
            starclock_combat::rng::RNG_ALGORITHM_REVISION,
            starclock_activity::ACTIVITY_STATE_HASH_REVISION,
        )
        .map_err(|_| UniverseCliError::Replay)?,
        ControllerIdentity::new(
            StandardUniverseBaselineRunner::REVISION,
            controller_digest(),
        )
        .map_err(|_| UniverseCliError::Replay)?,
        seed,
        replay_entry_for(activity, profile),
        0,
    )
    .map_err(|_| UniverseCliError::Replay)
}

fn parse_profile(header: &ReplayHeader) -> Result<(u32, usize, String), UniverseCliError> {
    let ReplayEntry::Activity { profile_id, .. } = header.entry() else {
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
fn controller_digest() -> ControllerDigest {
    let mut value = Sha256Sink::new();
    value.write(b"standard-universe-baseline-runner-v1\0verified-reference-projection-v1");
    ControllerDigest::new(value.finalize().bytes())
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
