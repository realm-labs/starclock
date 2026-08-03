//! Atomic current-Activity assembly for one prepared Standard Universe battle.

use crate::battle_assembly::DEFAULT_BATTLE_ASSEMBLY_CACHE_CAPACITY;
use crate::battle_snapshot::StandardUniverseBattleSnapshot;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use starclock_activity::{ActivityBattleHandoff, BattleBinding};
use starclock_combat::catalog::CombatCatalog;

use crate::{
    battle_assembly::{
        BattleAssemblyCache, BattleAssemblyCacheError, BattleAssemblyCacheMetrics,
        BattleAssemblyKey,
    },
    battle_materialization::{
        UniverseBattleMaterialization, UniverseBattleMaterializationError,
        UniverseBattleMaterializer, UniverseBattleRoster,
        catalog_composition::UniverseBattleCatalogComposition,
    },
    battle_technique::UniverseBattleTechniqueDefinition,
    catalog::UniverseCatalog,
    runtime::{
        StandardUniverseActivity, StandardUniverseBattleContributionError,
        StandardUniverseBattleStartError,
    },
};

pub const DEFAULT_MAXIMUM_ASSEMBLY_RULE_BINDINGS: usize = 1_024;
pub const DEFAULT_MAXIMUM_ASSEMBLY_MODIFIERS: usize = 64;
pub const DEFAULT_MAXIMUM_ASSEMBLY_CARRY_ENTRIES: usize = 8;
pub const DEFAULT_MAXIMUM_ASSEMBLY_ENCOUNTERS: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleAssemblyBudget {
    maximum_rule_bindings: usize,
    maximum_modifiers: usize,
    maximum_carry_entries: usize,
    maximum_encounters: usize,
}

impl Default for BattleAssemblyBudget {
    fn default() -> Self {
        Self {
            maximum_rule_bindings: DEFAULT_MAXIMUM_ASSEMBLY_RULE_BINDINGS,
            maximum_modifiers: DEFAULT_MAXIMUM_ASSEMBLY_MODIFIERS,
            maximum_carry_entries: DEFAULT_MAXIMUM_ASSEMBLY_CARRY_ENTRIES,
            maximum_encounters: DEFAULT_MAXIMUM_ASSEMBLY_ENCOUNTERS,
        }
    }
}

impl BattleAssemblyBudget {
    #[must_use]
    pub const fn new(
        maximum_rule_bindings: usize,
        maximum_modifiers: usize,
        maximum_carry_entries: usize,
        maximum_encounters: usize,
    ) -> Self {
        Self {
            maximum_rule_bindings,
            maximum_modifiers,
            maximum_carry_entries,
            maximum_encounters,
        }
    }
}

pub struct StandardUniverseDynamicBattleStart {
    handoff: ActivityBattleHandoff,
    combat_catalog: Arc<CombatCatalog>,
    assembly_key: BattleAssemblyKey,
    cache_hit: bool,
}

impl StandardUniverseDynamicBattleStart {
    #[must_use]
    pub const fn handoff(&self) -> &ActivityBattleHandoff {
        &self.handoff
    }
    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
    #[must_use]
    pub const fn assembly_key(&self) -> BattleAssemblyKey {
        self.assembly_key
    }
    #[must_use]
    pub const fn cache_hit(&self) -> bool {
        self.cache_hit
    }
    #[must_use]
    pub fn into_parts(self) -> (ActivityBattleHandoff, Arc<CombatCatalog>) {
        (self.handoff, self.combat_catalog)
    }
}

pub struct StandardUniverseResolvedAssembly {
    materialization: Arc<UniverseBattleMaterialization>,
    assembly_key: BattleAssemblyKey,
    cache_hit: bool,
}

impl StandardUniverseResolvedAssembly {
    #[must_use]
    pub const fn materialization(&self) -> &Arc<UniverseBattleMaterialization> {
        &self.materialization
    }
    #[must_use]
    pub const fn assembly_key(&self) -> BattleAssemblyKey {
        self.assembly_key
    }
    #[must_use]
    pub const fn cache_hit(&self) -> bool {
        self.cache_hit
    }
}

pub struct StandardUniverseBattleAssembler {
    catalog: Arc<UniverseCatalog>,
    composition: Arc<UniverseBattleCatalogComposition>,
    roster: UniverseBattleRoster,
    template: Arc<UniverseBattleMaterialization>,
    budget: BattleAssemblyBudget,
    cache: Mutex<BattleAssemblyCache>,
}

impl StandardUniverseBattleAssembler {
    pub fn new(
        catalog: Arc<UniverseCatalog>,
        composition: Arc<UniverseBattleCatalogComposition>,
        roster: UniverseBattleRoster,
        template: Arc<UniverseBattleMaterialization>,
    ) -> Result<Self, StandardUniverseDynamicBattleError> {
        Self::new_with_policy(
            catalog,
            composition,
            roster,
            template,
            NonZeroUsize::new(DEFAULT_BATTLE_ASSEMBLY_CACHE_CAPACITY)
                .expect("default cache capacity is non-zero"),
            BattleAssemblyBudget::default(),
        )
    }

    pub fn new_with_policy(
        catalog: Arc<UniverseCatalog>,
        composition: Arc<UniverseBattleCatalogComposition>,
        roster: UniverseBattleRoster,
        template: Arc<UniverseBattleMaterialization>,
        cache_capacity: NonZeroUsize,
        budget: BattleAssemblyBudget,
    ) -> Result<Self, StandardUniverseDynamicBattleError> {
        if roster.participant_lock() != template.assembly_key().participant_lock()
            || composition.digest() != template.assembly_key().catalog_composition()
        {
            return Err(StandardUniverseDynamicBattleError::TemplateMismatch);
        }
        let mut cache = BattleAssemblyCache::new(cache_capacity);
        cache
            .insert(template.assembly_key(), Arc::clone(&template))
            .map_err(StandardUniverseDynamicBattleError::Cache)?;
        Ok(Self {
            catalog,
            composition,
            roster,
            template,
            budget,
            cache: Mutex::new(cache),
        })
    }

    pub fn fork_with_policy(
        &self,
        cache_capacity: NonZeroUsize,
        budget: BattleAssemblyBudget,
    ) -> Result<Self, StandardUniverseDynamicBattleError> {
        Self::new_with_policy(
            Arc::clone(&self.catalog),
            Arc::clone(&self.composition),
            self.roster.clone(),
            Arc::clone(&self.template),
            cache_capacity,
            budget,
        )
    }

    pub fn start_pending_battle(
        &self,
        activity: &mut StandardUniverseActivity,
    ) -> Result<StandardUniverseDynamicBattleStart, StandardUniverseDynamicBattleError> {
        if activity.view().pending_battle().is_none() {
            return Err(StandardUniverseDynamicBattleError::MissingPendingBattle);
        }
        let snapshot = activity
            .battle_start_snapshot()
            .map_err(StandardUniverseDynamicBattleError::Snapshot)?;
        self.start_pending_battle_from_snapshot(activity, snapshot)
    }

    pub fn start_pending_battle_from_snapshot(
        &self,
        activity: &mut StandardUniverseActivity,
        snapshot: StandardUniverseBattleSnapshot,
    ) -> Result<StandardUniverseDynamicBattleStart, StandardUniverseDynamicBattleError> {
        let view = activity.view();
        let expected_state_hash = view.state_hash();
        let pending = view
            .pending_battle()
            .ok_or(StandardUniverseDynamicBattleError::MissingPendingBattle)?;
        let template_binding = self
            .template
            .overlay()
            .binding_for_spec(pending.assembly_digest().bytes())
            .ok_or(StandardUniverseDynamicBattleError::TemplateMismatch)?;
        let member = template_binding.member();
        let selected_technique =
            selected_technique(pending.techniques(), self.template.techniques())?;
        if snapshot.source_state_hash() != expected_state_hash {
            return Err(StandardUniverseDynamicBattleError::StaleSnapshot);
        }
        let resolved = self.resolve_snapshot(&snapshot, selected_technique)?;
        let materialization = resolved.materialization();
        let binding = materialization
            .overlay()
            .binding(member)
            .ok_or(StandardUniverseDynamicBattleError::MissingEncounter)?;
        let variant = binding
            .preparation()
            .variants()
            .iter()
            .find(|variant| variant.techniques() == pending.techniques())
            .ok_or(StandardUniverseDynamicBattleError::MissingVariant)?;
        let dynamic_binding = clone_binding(variant.battle_binding())
            .map_err(|_| StandardUniverseDynamicBattleError::InvalidBinding)?;
        let handoff = activity
            .start_assembled_pending_battle(
                expected_state_hash,
                dynamic_binding,
                variant.contribution_digest(),
                Arc::clone(binding.contract()),
            )
            .map_err(StandardUniverseDynamicBattleError::Activity)?;
        Ok(StandardUniverseDynamicBattleStart {
            handoff,
            combat_catalog: Arc::clone(materialization.combat_catalog()),
            assembly_key: resolved.assembly_key(),
            cache_hit: resolved.cache_hit(),
        })
    }

    pub fn resolve_snapshot(
        &self,
        snapshot: &StandardUniverseBattleSnapshot,
        technique: Option<UniverseBattleTechniqueDefinition>,
    ) -> Result<StandardUniverseResolvedAssembly, StandardUniverseDynamicBattleError> {
        self.validate_budget(snapshot)?;
        if technique.is_some_and(|candidate| !self.template.techniques().contains(&candidate)) {
            return Err(StandardUniverseDynamicBattleError::MissingTechnique);
        }
        let key = UniverseBattleMaterializer
            .snapshot_assembly_key(&self.composition, &self.roster, snapshot, technique)
            .map_err(StandardUniverseDynamicBattleError::Materialization)?;
        let (materialization, cache_hit) = self.resolve(key, snapshot, technique)?;
        Ok(StandardUniverseResolvedAssembly {
            materialization,
            assembly_key: key,
            cache_hit,
        })
    }

    #[must_use]
    pub fn cache_metrics(&self) -> BattleAssemblyCacheMetrics {
        match self.cache.lock() {
            Ok(cache) => cache.metrics(),
            Err(poisoned) => poisoned.into_inner().metrics(),
        }
    }

    #[must_use]
    pub fn cache_entry_count(&self) -> usize {
        match self.cache.lock() {
            Ok(cache) => cache.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    fn resolve(
        &self,
        key: BattleAssemblyKey,
        snapshot: &StandardUniverseBattleSnapshot,
        technique: Option<UniverseBattleTechniqueDefinition>,
    ) -> Result<(Arc<UniverseBattleMaterialization>, bool), StandardUniverseDynamicBattleError>
    {
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|_| StandardUniverseDynamicBattleError::CachePoisoned)?;
            if let Some(value) = cache.get(key) {
                return Ok((value, true));
            }
        }
        let materialization = match technique {
            Some(technique) => UniverseBattleMaterializer
                .compile_snapshot_from_composition_with_technique(
                    &self.catalog,
                    &self.composition,
                    &self.roster,
                    snapshot,
                    technique,
                ),
            None => UniverseBattleMaterializer.compile_snapshot_from_composition(
                &self.catalog,
                &self.composition,
                &self.roster,
                snapshot,
            ),
        }
        .map_err(StandardUniverseDynamicBattleError::Materialization)?;
        if materialization.assembly_key() != key {
            return Err(StandardUniverseDynamicBattleError::KeyMismatch);
        }
        let materialization = Arc::new(materialization);
        self.cache
            .lock()
            .map_err(|_| StandardUniverseDynamicBattleError::CachePoisoned)?
            .insert(key, Arc::clone(&materialization))
            .map_err(StandardUniverseDynamicBattleError::Cache)?;
        Ok((materialization, false))
    }

    fn validate_budget(
        &self,
        snapshot: &StandardUniverseBattleSnapshot,
    ) -> Result<(), StandardUniverseDynamicBattleError> {
        if snapshot.contributions().rules().len() > self.budget.maximum_rule_bindings
            || snapshot.contributions().modifiers().len() > self.budget.maximum_modifiers
            || snapshot.participant_carry().len() > self.budget.maximum_carry_entries
            || self.template.overlay().bindings().len() > self.budget.maximum_encounters
        {
            return Err(StandardUniverseDynamicBattleError::BudgetExceeded);
        }
        Ok(())
    }
}

fn selected_technique(
    selected: &[starclock_activity::ActivityOptionId],
    available: &[UniverseBattleTechniqueDefinition],
) -> Result<Option<UniverseBattleTechniqueDefinition>, StandardUniverseDynamicBattleError> {
    match selected {
        [] => Ok(None),
        [option] => available
            .iter()
            .copied()
            .find(|technique| technique.option() == *option)
            .map(Some)
            .ok_or(StandardUniverseDynamicBattleError::MissingTechnique),
        _ => Err(StandardUniverseDynamicBattleError::UnsupportedTechniqueSequence),
    }
}

fn clone_binding(
    binding: &BattleBinding,
) -> Result<BattleBinding, starclock_activity::BattleBindingError> {
    BattleBinding::new(
        binding.battle_spec().clone(),
        binding.seed_stream_label(),
        binding.participant_lock_digest(),
    )
}

#[derive(Debug)]
pub enum StandardUniverseDynamicBattleError {
    MissingPendingBattle,
    TemplateMismatch,
    MissingEncounter,
    MissingVariant,
    MissingTechnique,
    UnsupportedTechniqueSequence,
    StaleSnapshot,
    KeyMismatch,
    InvalidBinding,
    BudgetExceeded,
    CachePoisoned,
    Snapshot(StandardUniverseBattleContributionError),
    Materialization(UniverseBattleMaterializationError),
    Cache(BattleAssemblyCacheError),
    Activity(StandardUniverseBattleStartError),
}

impl core::fmt::Display for StandardUniverseDynamicBattleError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Standard Universe dynamic battle error: {self:?}"
        )
    }
}

impl std::error::Error for StandardUniverseDynamicBattleError {}

#[cfg(test)]
mod transition_tests;
