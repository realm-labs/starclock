//! Private reference composition for the Goal 04 external encounter-provider seam.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleResultContract, ActivityInstanceId, ActivityMasterSeed, ActivityOptionId,
    ActivityParticipantCarryDefinition, BattleBinding, BattleOutcome, BattleResult, BuildDigest,
    EnergyCarryPolicy, EventDigest, HpCarryPolicy, LifeCarryPolicy, LoadoutLockScope,
    OpaqueParticipantBuild, ParticipantBattleState, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    PresenceCarryPolicy, ProjectedValue, ProjectionField, ProjectionId,
    TechniqueContributionDigest,
};
use starclock_combat::{
    AbilityId, BattleSpec, BattleSpecDigest, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, LifeState, ParticipantSource,
    ParticipantSpec, PresenceState, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
};
use starclock_mode_universe::{
    battle_overlay::{UniverseEncounterBattleBinding, UniverseEncounterOverlay},
    catalog::UniverseCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    id::WorldId,
    runtime::StandardUniverseActivity,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
pub(crate) const PROFILE_PREFIX: &str = "standard-universe-v1/world-";
pub(crate) const BATTLE_EXECUTOR_REVISION: &str = "verified-reference-projection-v1";

#[derive(Clone)]
pub(crate) struct ActivityReferenceFactory {
    catalog: Arc<UniverseCatalog>,
    participants: ParticipantLock,
    overlay: UniverseEncounterOverlay,
}

impl ActivityReferenceFactory {
    pub(crate) fn load() -> Result<Self, ActivityReferenceError> {
        let core = starclock_data::catalog::load(CORE_BUNDLE)
            .map_err(|_| ActivityReferenceError::Configuration)?;
        let catalog = UniverseCatalog::load(UNIVERSE_BUNDLE, core)
            .map_err(|_| ActivityReferenceError::Configuration)?;
        let participants = participants()?;
        let overlay = overlay(&catalog, &participants)?;
        Ok(Self {
            catalog,
            participants,
            overlay,
        })
    }

    pub(crate) fn start(
        &self,
        world_raw: u32,
        difficulty_index: usize,
        seed: u64,
    ) -> Result<(String, StandardUniverseActivity), ActivityReferenceError> {
        let world_id = WorldId::new(world_raw).ok_or(ActivityReferenceError::UnknownEntry)?;
        let world = self
            .catalog
            .world(world_id)
            .ok_or(ActivityReferenceError::UnknownEntry)?;
        let difficulty = *world
            .difficulties()
            .get(difficulty_index)
            .ok_or(ActivityReferenceError::UnknownEntry)?;
        let compiled = StandardUniverseProfile::new(Arc::clone(&self.catalog))
            .compile(
                StandardUniverseEntry::new(world_id, difficulty, self.participants.clone(), vec![])
                    .with_encounter_overlay(self.overlay.clone()),
            )
            .map_err(|_| ActivityReferenceError::Configuration)?;
        let instance = ActivityInstanceId::new(
            seed.checked_add(1)
                .ok_or(ActivityReferenceError::InvalidSeed)?,
        )
        .ok_or(ActivityReferenceError::InvalidSeed)?;
        let activity = compiled
            .start_standard(instance, ActivityMasterSeed::from_u64(seed))
            .map_err(|_| ActivityReferenceError::Start)?
            .into_activity();
        Ok((
            format!("{PROFILE_PREFIX}{world_raw}/difficulty-{difficulty_index}"),
            activity,
        ))
    }

    pub(crate) fn catalog(&self) -> &UniverseCatalog {
        &self.catalog
    }
}

pub(crate) fn reference_won_result(
    identity: starclock_activity::BattleResultIdentity,
) -> BattleResult {
    let mut values = vec![
        ProjectedValue::Outcome(BattleOutcome::Won),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
        ProjectedValue::EventDigest(
            EventDigest::new([0x72; 32]).expect("static digest is non-zero"),
        ),
        ProjectedValue::TerminalFault(None),
    ];
    values.extend((1_u32..=4).map(|raw| {
        ProjectedValue::ParticipantState(
            ParticipantBattleState::new(
                ParticipantId::new(raw).expect("static participant ID is non-zero"),
                Hp::new(900).expect("static HP is valid"),
                Hp::new(1_000).expect("static HP is valid"),
                Energy::from_scaled(50_000_000).expect("static energy is valid"),
                Energy::from_scaled(100_000_000).expect("static energy is valid"),
                LifeState::Alive,
                PresenceState::Present,
            )
            .expect("static participant carry is valid"),
        )
    }));
    BattleResult::seal(identity, values)
}

fn participants() -> Result<ParticipantLock, ActivityReferenceError> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .ok_or(ActivityReferenceError::Configuration)?;
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let participant =
                ParticipantId::new(u32::from(byte)).ok_or(ActivityReferenceError::Configuration)?;
            let form = UnitDefinitionId::new(20_001 + u32::from(index))
                .ok_or(ActivityReferenceError::Configuration)?;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32])
                    .ok_or(ActivityReferenceError::Configuration)?,
                BuildDigest::new([byte + 32; 32]).ok_or(ActivityReferenceError::Configuration)?,
                "universe-agent-reference-build-v1",
                ParticipantSourceKind::CompiledBuild,
            )
            .map_err(|_| ActivityReferenceError::Configuration)?;
            ParticipantLockEntry::new(participant, 0, index, form, build)
                .map_err(|_| ActivityReferenceError::Configuration)
        })
        .collect::<Result<Vec<_>, ActivityReferenceError>>()?;
    ParticipantLock::seal(policy, entries).map_err(|_| ActivityReferenceError::Configuration)
}

fn overlay(
    catalog: &UniverseCatalog,
    lock: &ParticipantLock,
) -> Result<UniverseEncounterOverlay, ActivityReferenceError> {
    let contract = Arc::new(
        ActivityBattleResultContract::new(
            Arc::new(
                starclock_activity::BattleResultProjection::new(
                    ProjectionId::new(1).expect("static projection ID is non-zero"),
                    vec![
                        ProjectionField::Outcome,
                        ProjectionField::FinalStateHash,
                        ProjectionField::EventDigest,
                        ProjectionField::TerminalFault,
                        ProjectionField::ParticipantState(
                            ParticipantId::new(1).expect("static participant ID is non-zero"),
                        ),
                        ProjectionField::ParticipantState(
                            ParticipantId::new(2).expect("static participant ID is non-zero"),
                        ),
                        ProjectionField::ParticipantState(
                            ParticipantId::new(3).expect("static participant ID is non-zero"),
                        ),
                        ProjectionField::ParticipantState(
                            ParticipantId::new(4).expect("static participant ID is non-zero"),
                        ),
                    ],
                )
                .map_err(|_| ActivityReferenceError::Configuration)?,
            ),
            (1..=4)
                .map(|raw| {
                    ActivityParticipantCarryDefinition::new(
                        ParticipantId::new(raw).expect("static participant ID is non-zero"),
                        HpCarryPolicy::CarryExact,
                        EnergyCarryPolicy::CarryExact,
                        LifeCarryPolicy::CarryExact,
                        PresenceCarryPolicy::CarryExact,
                    )
                })
                .collect(),
            vec![],
        )
        .map_err(|_| ActivityReferenceError::Configuration)?,
    );
    let bindings = catalog
        .encounter_groups()
        .iter()
        .flat_map(|group| group.members())
        .map(|member| {
            let preparation = Arc::new(
                starclock_activity::EncounterPreparationDefinition::new(
                    ActivityOptionId::new(10).expect("static option ID is non-zero"),
                    starclock_activity::EncounterInitiativePolicy::PlayerControlled,
                    lock.digest(),
                    0,
                    vec![],
                    vec![starclock_activity::PreparedBattleVariant::new(
                        vec![],
                        TechniqueContributionDigest::new([0x44; 32])
                            .expect("static digest is non-zero"),
                        BattleBinding::new(
                            battle_spec(member.id().get())?,
                            "universe-agent-reference",
                            "universe-battle-spec-v1",
                            lock.digest(),
                        )
                        .map_err(|_| ActivityReferenceError::Configuration)?,
                    )],
                )
                .map_err(|_| ActivityReferenceError::Configuration)?,
            );
            Ok(UniverseEncounterBattleBinding::new(
                member.id(),
                preparation,
                Arc::clone(&contract),
            ))
        })
        .collect::<Result<Vec<_>, ActivityReferenceError>>()?;
    UniverseEncounterOverlay::new(bindings).map_err(|_| ActivityReferenceError::Configuration)
}

fn battle_spec(member: u32) -> Result<BattleSpec, ActivityReferenceError> {
    let mut entries = (0_u8..4)
        .map(|index| {
            Ok(ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(index).ok_or(ActivityReferenceError::Configuration)?,
                ParticipantSource::Player,
                combatant(20_001 + u32::from(index), index + 1)?,
            ))
        })
        .collect::<Result<Vec<_>, ActivityReferenceError>>()?;
    let enemy = 30_000_u32
        .checked_add(member)
        .ok_or(ActivityReferenceError::Configuration)?;
    let digest = u8::try_from(member).map_err(|_| ActivityReferenceError::Configuration)?;
    entries.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).ok_or(ActivityReferenceError::Configuration)?,
        ParticipantSource::EncounterEnemy(
            EnemyDefinitionId::new(enemy).ok_or(ActivityReferenceError::Configuration)?,
        ),
        combatant(enemy, digest)?,
    ));
    BattleSpec::new(
        "universe-agent-reference-rules-v1",
        BattleSpecDigest::new([digest; 32]).ok_or(ActivityReferenceError::Configuration)?,
        EncounterId::new(member).ok_or(ActivityReferenceError::Configuration)?,
        entries,
        TeamResourceSpec::new(3, 5).ok_or(ActivityReferenceError::Configuration)?,
        TeamResourceSpec::new(0, 0).ok_or(ActivityReferenceError::Configuration)?,
        ConcedePolicy::Allowed,
    )
    .map_err(|_| ActivityReferenceError::Configuration)
}

fn combatant(form: u32, digest: u8) -> Result<ResolvedCombatantSpec, ActivityReferenceError> {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).ok_or(ActivityReferenceError::Configuration)?,
        UnitLevel::new(80).ok_or(ActivityReferenceError::Configuration)?,
        Hp::new(1_000).map_err(|_| ActivityReferenceError::Configuration)?,
        Speed::from_scaled(100_000_000).map_err(|_| ActivityReferenceError::Configuration)?,
        ResolvedDefinitionBindings::new(
            vec![AbilityId::new(form).ok_or(ActivityReferenceError::Configuration)?],
            vec![],
            vec![],
        )
        .map_err(|_| ActivityReferenceError::Configuration)?,
        CombatantSpecDigest::new([digest; 32]).ok_or(ActivityReferenceError::Configuration)?,
    )
    .map_err(|_| ActivityReferenceError::Configuration)?
    .with_energy(
        Energy::ZERO,
        Energy::from_scaled(100_000_000).map_err(|_| ActivityReferenceError::Configuration)?,
    )
    .map_err(|_| ActivityReferenceError::Configuration)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActivityReferenceError {
    Configuration,
    UnknownEntry,
    InvalidSeed,
    Start,
}
