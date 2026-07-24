//! Shared production assembly for the engine-free Standard Universe facade.

use std::sync::Arc;

use starclock_activity::{
    ActivityInstanceId, ActivityMasterSeed, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, UnitDefinitionId, UnitLevel,
    catalog::{CombatCatalog, action::AbilityKind},
};
use starclock_replay::{component::ConfigurationComponentSet, format_v2::ReplayCompatibilityV2};

use crate::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{
        UniverseBattleMaterialization, UniverseBattleMaterializer, UniverseBattleRoster,
    },
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    id::WorldId,
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
    runtime::StandardUniverseActivity,
    universe_replay_v2::standard_universe_component_set,
};

pub const STANDARD_UNIVERSE_PROFILE_PREFIX: &str = "standard-universe-v1/world-";
pub const STANDARD_UNIVERSE_DEFAULT_BUILD_REVISION: &str =
    "standard-universe-default-resolved-build-v1";

/// Immutable bundles, roster and initial battle materialization shared by CLI
/// and protocol-neutral agent sessions.
#[derive(Clone)]
pub struct StandardUniverseRuntimeFactory {
    catalog: Arc<UniverseCatalog>,
    participants: ParticipantLock,
    materialization: Arc<UniverseBattleMaterialization>,
}

impl StandardUniverseRuntimeFactory {
    pub fn load(
        core_bundle: &[u8],
        universe_bundle: &[u8],
    ) -> Result<Self, StandardUniverseRuntimeFactoryError> {
        let core = starclock_data::catalog::load(core_bundle)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let catalog = UniverseCatalog::load(universe_bundle, core)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let (roster, participants) = default_roster(&catalog)?;
        let contributions = initial_contributions(&catalog)?;
        let materialization = UniverseBattleMaterializer
            .compile(&catalog, &roster, &contributions)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        Ok(Self {
            catalog,
            participants,
            materialization: Arc::new(materialization),
        })
    }

    pub fn start(
        &self,
        world_raw: u32,
        difficulty_index: usize,
        seed: u64,
        controller: StandardUniverseControllerIdentity<'_>,
    ) -> Result<StandardUniverseRuntimeInstance, StandardUniverseRuntimeFactoryError> {
        let world_id =
            WorldId::new(world_raw).ok_or(StandardUniverseRuntimeFactoryError::UnknownEntry)?;
        let world = self
            .catalog
            .world(world_id)
            .ok_or(StandardUniverseRuntimeFactoryError::UnknownEntry)?;
        let difficulty = *world
            .difficulties()
            .get(difficulty_index)
            .ok_or(StandardUniverseRuntimeFactoryError::UnknownEntry)?;
        let compiled = StandardUniverseProfile::new(Arc::clone(&self.catalog))
            .compile(
                StandardUniverseEntry::new(world_id, difficulty, self.participants.clone(), vec![])
                    .with_encounter_overlay(self.materialization.overlay().clone()),
            )
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let components = standard_universe_component_set(
            &self.catalog,
            &compiled,
            &self.materialization,
            controller.id,
            controller.revision,
            controller.digest,
        )
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let compatibility = ReplayCompatibilityV2::new(
            self.catalog.identity().game_version(),
            starclock_combat::NUMERIC_POLICY_REVISION,
            starclock_combat::rng::RNG_ALGORITHM_REVISION,
            starclock_activity::ACTIVITY_STATE_HASH_REVISION,
        )
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let instance = ActivityInstanceId::new(
            seed.checked_add(1)
                .ok_or(StandardUniverseRuntimeFactoryError::InvalidSeed)?,
        )
        .ok_or(StandardUniverseRuntimeFactoryError::InvalidSeed)?;
        let activity = compiled
            .start_standard(instance, ActivityMasterSeed::from_u64(seed))
            .map_err(|_| StandardUniverseRuntimeFactoryError::Start)?
            .into_activity();
        Ok(StandardUniverseRuntimeInstance {
            profile_id: format!(
                "{STANDARD_UNIVERSE_PROFILE_PREFIX}{world_raw}/difficulty-{difficulty_index}"
            )
            .into_boxed_str(),
            activity,
            combat_catalog: Arc::clone(self.materialization.combat_catalog()),
            components,
            compatibility,
        })
    }

    #[must_use]
    pub const fn catalog(&self) -> &Arc<UniverseCatalog> {
        &self.catalog
    }

    #[must_use]
    pub const fn materialization(&self) -> &Arc<UniverseBattleMaterialization> {
        &self.materialization
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardUniverseControllerIdentity<'a> {
    pub id: &'a str,
    pub revision: &'a str,
    pub digest: [u8; 32],
}

pub struct StandardUniverseRuntimeInstance {
    profile_id: Box<str>,
    activity: StandardUniverseActivity,
    combat_catalog: Arc<CombatCatalog>,
    components: ConfigurationComponentSet,
    compatibility: ReplayCompatibilityV2,
}

impl StandardUniverseRuntimeInstance {
    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }
    #[must_use]
    pub const fn activity(&self) -> &StandardUniverseActivity {
        &self.activity
    }
    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
    #[must_use]
    pub const fn components(&self) -> &ConfigurationComponentSet {
        &self.components
    }
    #[must_use]
    pub const fn compatibility(&self) -> &ReplayCompatibilityV2 {
        &self.compatibility
    }
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Box<str>,
        StandardUniverseActivity,
        Arc<CombatCatalog>,
        ConfigurationComponentSet,
        ReplayCompatibilityV2,
    ) {
        (
            self.profile_id,
            self.activity,
            self.combat_catalog,
            self.components,
            self.compatibility,
        )
    }
}

fn default_roster(
    catalog: &UniverseCatalog,
) -> Result<(UniverseBattleRoster, ParticipantLock), StandardUniverseRuntimeFactoryError> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
    let mut lock_entries = Vec::new();
    let mut combatants = Vec::new();
    for index in 0_u8..4 {
        let form = UnitDefinitionId::new(u32::from(index) + 1)
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
        let unit = catalog
            .simulation_catalog()
            .combat_catalog()
            .unit(form)
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
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
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
        let combatant = default_combatant(form, basic, index)?;
        let participant = ParticipantId::new(u32::from(index) + 1)
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
        lock_entries.push(
            ParticipantLockEntry::new(
                participant,
                0,
                index,
                form,
                OpaqueParticipantBuild::new(
                    combatant.digest(),
                    BuildDigest::new([index + 17; 32])
                        .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?,
                    STANDARD_UNIVERSE_DEFAULT_BUILD_REVISION,
                    ParticipantSourceKind::FixedResolved,
                )
                .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
            )
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        );
        combatants.push((participant, combatant));
    }
    let lock = ParticipantLock::seal(policy, lock_entries)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    let roster = UniverseBattleRoster::new(&lock, combatants)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    Ok((roster, lock))
}

fn default_combatant(
    form: UnitDefinitionId,
    basic: starclock_combat::AbilityId,
    index: u8,
) -> Result<ResolvedCombatantSpec, StandardUniverseRuntimeFactoryError> {
    ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).ok_or(StandardUniverseRuntimeFactoryError::Configuration)?,
        Hp::new(100_000).map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        Speed::from_scaled(200_000_000)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new())
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        CombatantSpecDigest::new([index + 1; 32])
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?,
    )
    .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
    .with_base_attack_defense(
        StatValue::from_scaled(1_000_000_000)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        StatValue::from_scaled(1_000_000_000)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
    )
    .with_energy(
        Energy::ZERO,
        Energy::from_scaled(100_000_000)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
    )
    .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)
}

fn initial_contributions(
    catalog: &Arc<UniverseCatalog>,
) -> Result<UniverseBattleContributionSet, StandardUniverseRuntimeFactoryError> {
    let path_definition = catalog
        .paths()
        .first()
        .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
        .contributions_from_owned(&[])
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    let path = PathRuntimeCatalog::compile(catalog)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
        .contributions(path_definition.id(), &blessings, &[])
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    let curios = CurioRuntimeCatalog::compile(catalog)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
        .contributions_from_owned(&[], &[], &[])
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    let abilities = RunRuntimeCatalog::compile(catalog)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
        .ability_contributions(&[])
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    let projection = AbilityRuntimeCatalog::compile(catalog)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
        .project(
            &[],
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::BattleStart,
                0,
                false,
            ),
        )
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    UniverseBattleContributionCompiler::compile(Arc::clone(catalog))
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
        .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseRuntimeFactoryError {
    Configuration,
    UnknownEntry,
    InvalidSeed,
    Start,
}
