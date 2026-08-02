//! Shared production assembly for the engine-free Standard Universe facade.

use std::sync::Arc;

use starclock_activity::{
    ActivityInstanceId, ActivityMasterSeed, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_build::{
    ability::AbilityInvestment,
    compiler::LoadoutCompiler,
    spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
};
use starclock_combat::{
    CombatantSpecDigest, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, StatValue,
    UnitDefinitionId, UnitLevel, catalog::CombatCatalog,
};
use starclock_replay::{component::ConfigurationComponentSet, format::ReplayEnvironment};

use crate::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    battle_assembly::BattleAssemblyCacheMetrics,
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{
        UniverseBattleMaterialization, UniverseBattleMaterializationCoverage,
        UniverseBattleMaterializer, UniverseBattleRoster,
        catalog_composition::UniverseBattleCatalogComposition,
    },
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    dynamic_battle_assembler::StandardUniverseBattleAssembler,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    id::WorldId,
    path_runtime::PathRuntimeCatalog,
    replay_execution::standard_universe_component_set,
    run_runtime::RunRuntimeCatalog,
    runtime::StandardUniverseActivity,
};

pub const STANDARD_UNIVERSE_PROFILE_PREFIX: &str = "standard-universe/world-";

/// Immutable bundles and catalog composition shared by CLI and
/// protocol-neutral agent sessions.
#[derive(Clone)]
pub struct StandardUniverseRuntimeFactory {
    catalog: Arc<UniverseCatalog>,
    participants: ParticipantLock,
    catalog_composition: Arc<UniverseBattleCatalogComposition>,
    battle_assembler: Arc<StandardUniverseBattleAssembler>,
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
        let catalog_composition = Arc::new(
            UniverseBattleCatalogComposition::compile(&catalog)
                .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        );
        let materialization = UniverseBattleMaterializer
            .compile_from_composition(&catalog, &catalog_composition, &roster, &contributions)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let materialization = Arc::new(materialization);
        let battle_assembler = Arc::new(
            StandardUniverseBattleAssembler::new(
                Arc::clone(&catalog),
                Arc::clone(&catalog_composition),
                roster,
                Arc::clone(&materialization),
            )
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        );
        Ok(Self {
            catalog,
            participants,
            catalog_composition,
            battle_assembler,
            materialization,
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
            controller.digest,
        )
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let environment = ReplayEnvironment::new(self.catalog.identity().game_version())
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
            battle_assembler: Arc::clone(&self.battle_assembler),
            combat_catalog: Arc::clone(self.materialization.combat_catalog()),
            components,
            environment,
        })
    }

    #[must_use]
    pub const fn catalog(&self) -> &Arc<UniverseCatalog> {
        &self.catalog
    }

    #[must_use]
    pub fn baseline_materialization_coverage(&self) -> &UniverseBattleMaterializationCoverage {
        self.materialization.coverage()
    }

    #[must_use]
    pub const fn catalog_composition(&self) -> &Arc<UniverseBattleCatalogComposition> {
        &self.catalog_composition
    }

    #[must_use]
    pub fn assembly_cache_metrics(&self) -> BattleAssemblyCacheMetrics {
        self.battle_assembler.cache_metrics()
    }

    #[must_use]
    pub fn assembly_cache_entry_count(&self) -> usize {
        self.battle_assembler.cache_entry_count()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardUniverseControllerIdentity<'a> {
    pub id: &'a str,
    pub digest: [u8; 32],
}

pub struct StandardUniverseRuntimeInstance {
    profile_id: Box<str>,
    activity: StandardUniverseActivity,
    battle_assembler: Arc<StandardUniverseBattleAssembler>,
    combat_catalog: Arc<CombatCatalog>,
    components: ConfigurationComponentSet,
    environment: ReplayEnvironment,
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
    pub const fn battle_assembler(&self) -> &Arc<StandardUniverseBattleAssembler> {
        &self.battle_assembler
    }
    #[must_use]
    pub const fn components(&self) -> &ConfigurationComponentSet {
        &self.components
    }
    #[must_use]
    pub const fn environment(&self) -> &ReplayEnvironment {
        &self.environment
    }
    /// Decomposes the instance for historical replay-v2 verification only.
    ///
    /// New execution must use [`Self::into_dynamic_parts`].
    #[must_use]
    pub fn into_replay_parts(
        self,
    ) -> (
        Box<str>,
        StandardUniverseActivity,
        Arc<CombatCatalog>,
        ConfigurationComponentSet,
        ReplayEnvironment,
    ) {
        (
            self.profile_id,
            self.activity,
            self.combat_catalog,
            self.components,
            self.environment,
        )
    }

    #[must_use]
    pub fn into_dynamic_parts(
        self,
    ) -> (
        Box<str>,
        StandardUniverseActivity,
        Arc<StandardUniverseBattleAssembler>,
        ConfigurationComponentSet,
        ReplayEnvironment,
    ) {
        (
            self.profile_id,
            self.activity,
            self.battle_assembler,
            self.components,
            self.environment,
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
    let core = catalog.simulation_catalog();
    let mut lock_entries = Vec::new();
    let mut builds = Vec::new();
    for index in 0_u8..4 {
        let form = UnitDefinitionId::new(u32::from(index) + 1)
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
        let character = core
            .build_catalog()
            .character(form)
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
        let investments = character
            .ability_levels()
            .iter()
            .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
            .collect::<Vec<_>>();
        let build = CombatantBuildSpec::new(
            form,
            UnitLevel::new(80).ok_or(StandardUniverseRuntimeFactoryError::Configuration)?,
            PromotionStage::new(6).ok_or(StandardUniverseRuntimeFactoryError::Configuration)?,
        )
        .with_ability_levels(investments)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
        .with_eidolon(
            EidolonLevel::new(0).ok_or(StandardUniverseRuntimeFactoryError::Configuration)?,
        );
        let compiled = LoadoutCompiler
            .compile(core.build_catalog(), core.combat_catalog(), &build)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
        let runtime = default_runtime_combatant(compiled.combatant(), index)?;
        let participant = ParticipantId::new(u32::from(index) + 1)
            .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?;
        lock_entries.push(
            ParticipantLockEntry::new(
                participant,
                0,
                index,
                form,
                OpaqueParticipantBuild::new(
                    runtime.digest(),
                    BuildDigest::new(compiled.build_digest().bytes())
                        .ok_or(StandardUniverseRuntimeFactoryError::Configuration)?,
                    ParticipantSourceKind::CompiledBuild,
                )
                .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
            )
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        );
        builds.push((participant, build, compiled.combatant().clone(), runtime));
    }
    let lock = ParticipantLock::seal(policy, lock_entries)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    let roster = UniverseBattleRoster::new_with_build_specs_and_runtime_stats(&lock, builds)
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    Ok((roster, lock))
}

fn default_runtime_combatant(
    compiled: &ResolvedCombatantSpec,
    index: u8,
) -> Result<ResolvedCombatantSpec, StandardUniverseRuntimeFactoryError> {
    let mut runtime = ResolvedCombatantSpec::new(
        compiled.form(),
        compiled.level(),
        Hp::new(100_000).map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        Speed::from_scaled(200_000_000)
            .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?,
        ResolvedDefinitionBindings::new(
            compiled.abilities().to_vec(),
            compiled.rule_bundles().to_vec(),
            compiled.modifiers().to_vec(),
        )
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
    .with_base_effect_stats(
        compiled.base_effect_hit_rate(),
        compiled.base_effect_resistance(),
    )
    .with_energy(compiled.current_energy(), compiled.maximum_energy())
    .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?
    .with_toughness(
        compiled.rank(),
        compiled.weaknesses().to_vec(),
        compiled.toughness_layers().to_vec(),
    )
    .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    runtime = runtime
        .with_sources(compiled.sources().to_vec())
        .map_err(|_| StandardUniverseRuntimeFactoryError::Configuration)?;
    runtime
        .with_modifier_bindings(compiled.modifier_bindings().to_vec())
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
